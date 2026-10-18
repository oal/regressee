// SPDX-License-Identifier: MPL-2.0
// Explicit opt-in, owned loopback fixtures and already-installed pinned binaries.
// No install/download, user state, external origin, closed-root probing or iframe
// document traversal is authorized by this suite.
import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import http from 'node:http';
import path from 'node:path';
import crypto from 'node:crypto';
import {spawn} from 'node:child_process';
import {fileURLToPath} from 'node:url';
import {chromium} from 'playwright';
import {collectDomInPage,DOM_LIMITS,sampleDom,encodeDom as encode} from '../src/dom.mjs';
import {navigationBoundary,capture,WORKER_VERSION} from '../src/capture.mjs';
import {VERSION,MAX_FRAME} from '../src/protocol.mjs';

const root=process.env.REGRESSEE_DOM_TEST_RUNTIME;
const parent=process.env.REGRESSEE_DOM_TEST_OUTPUT;
test('safe live collector: owned loopback, open roots/templates, exclusions, bounds and stability',{skip:!root||!parent},async t=>{
  const catalog=JSON.parse(await fs.readFile(new URL('../../../crates/runtime/assets/official/linux-x86_64.json',import.meta.url),'utf8'));
  const executable='browsers/chromium-1248/chrome-linux64/chrome';
  for(const name of [executable,'node/bin/node']){
    const file=path.join(root,name),stat=await fs.lstat(file),expected=catalog.manifest.files[name];
    assert(stat.isFile()&&!stat.isSymbolicLink());assert.equal(stat.size,expected.bytes);
    assert.equal(crypto.createHash('sha256').update(await fs.readFile(file)).digest('hex'),expected.sha256);
  }
  assert.equal(process.version.slice(1),catalog.manifest.identity.Pinned.node);
  const output=await fs.mkdtemp(path.join(parent,'dom-browser-'));
  const report={runtime:catalog.manifest.identity,cases:[]};let browser,frameHits=0,heldFontRequests=0,origin;
  const server=http.createServer((req,res)=>{
    res.setHeader('Content-Type','text/html; charset=utf-8');
    if(req.url==='/embedded'){frameHits++;res.end('<body>FRAME_PRIVATE</body>');return;}
    if(req.url==='/notfound'){res.statusCode=404;res.end('Not found');return;}
    if(req.url==='/held.woff2'){heldFontRequests++;return;}
    if(req.url==='/stalled-font'){res.end('<!doctype html><style>@font-face{font-family:held;src:url(/held.woff2)}body{font-family:held}</style><body>Owned delayed-font timeout fixture</body>');return;}
    if(req.url==='/tall'){res.end('<!doctype html><style>body{height:100001px}</style><body>Tall ordinary DOM, too tall for screenshots</body>');return;}
    if(req.url==='/stable'){res.end('<!doctype html><body><p hidden aria-label="ordinary"> stable  ordinary\n text </p><p>Stable visible pixels</p></body>');return;}
    if(req.url==='/html-names'){res.end('<!doctype 123><a:b @click="ordinary" 123="ordinary" [binding]="ordinary" :class="ordinary" a:foo="one" b:foo="two" a:b:c="ordinary"></a:b><script>for(const name of ["a:b","c:b","b"])document.body.append(document.createElementNS("urn:fixture",name));</script>');return;}
    if(req.url==='/mutations'){res.end('<!doctype html><style id="site-owned-style">body{color:black}</style><body><img id="site-owned-image" loading="lazy" width="1" height="1" src="data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mP8/x8AAwMCAO+aZ1cAAAAASUVORK5CYII="/><p>Untransformed snapshot</p></body>');return;}
    if(req.url==='/dom-limit'){res.end('<!doctype html><body><p '+Array.from({length:257},(_,i)=>`data-a${i}="ordinary"`).join(' ')+'>Image remains capturable</p></body>');return;}
    res.end(`<!doctype html><html><body><div id="private" hidden aria-label="kept">  private ordinary text\n  </div>
      <script type="application/json">{"token":"JSON_PRIVATE"}</script>
      <svg><script type="application/json">SVG_PRIVATE</script><a href="javascript:SVG_URL_PRIVATE">link</a></svg>
      <input type="password" name="password" value="INPUT_PRIVATE" checked onclick="EVENT_PRIVATE" nonce="NONCE_PRIVATE">
      <select value="SELECT_PRIVATE"><option value="OPTION_PRIVATE" selected>Kept option</option></select>
      <textarea value="TEXTAREA_ATTR_PRIVATE">TEXTAREA_PRIVATE</textarea><button value="BUTTON_PRIVATE">Kept button</button>
      <a id="ordinary-link" class="owned-class" href="/ordinary?q=kept" data-private="ordinary private attribute">Link</a>
      <a href="javascript:URL_PRIVATE">No code</a><iframe src="/embedded" srcdoc="SRCDOC_PRIVATE"></iframe>
      <div id="host">light slot text</div><div id="closed-host"></div>
      <template id="outer-template"><input value="TEMPLATE_PRIVATE"><textarea>TEMPLATE_AREA_PRIVATE</textarea><template><script>DEEP_SCRIPT_PRIVATE</script><b>Deep inert ordinary</b></template></template>
      <script>
        document.getElementById('private').append(' hydrated ordinary');
        const root=document.getElementById('host').attachShadow({mode:'open'});
        root.innerHTML='<slot></slot><template><input value="SHADOW_TEMPLATE_PRIVATE"><b>Shadow inert ordinary</b></template><span id="nested"></span>';
        const nested=root.getElementById('nested').attachShadow({mode:'open'});
        nested.innerHTML='<script type="application/json">SHADOW_JSON_PRIVATE</scr'+'ipt><textarea>SHADOW_AREA_PRIVATE</textarea><i>Nested shadow ordinary</i>';
        document.getElementById('closed-host').attachShadow({mode:'closed'}).innerHTML='<b>CLOSED_PRIVATE</b>';
        document.querySelector('input').value='LIVE_PRIVATE';document.querySelector('textarea').value='LIVE_AREA_PRIVATE';
      </script></body></html>`);
  });
  await new Promise((resolve,reject)=>{server.once('error',reject);server.listen(0,'127.0.0.1',resolve);});
  origin=`http://127.0.0.1:${server.address().port}`;
  try{
    browser=await chromium.launch({executablePath:path.join(root,executable),headless:true,chromiumSandbox:true});
    assert.equal(browser.version(),catalog.manifest.identity.Pinned.browser);
    const context=await browser.newContext({serviceWorkers:'block',acceptDownloads:false});
    try{
      const page=await context.newPage();const boundary=await navigationBoundary(context,origin+'/fixture','loopback',5000,null,page);
      await page.goto(origin+'/fixture',{waitUntil:'domcontentloaded',timeout:5000});
      await t.test('live JS state and exclusions apply recursively before transfer',async()=>{
        const sample=await sampleDom(page,{deadline:Date.now()+3000,encode});
        assert.equal(sample.stable,true);assert.equal(sample.samples,2);assert.equal(sample.open_shadow_roots,2);assert.equal(sample.iframe_hosts,1);
        assert(!sample.body.includes('_PRIVATE'));assert(sample.body.includes('private ordinary text\\n  '));assert(sample.body.includes('hydrated ordinary'));
        for(const kept of ['owned-class','/ordinary?q=kept','aria-label','ordinary private attribute','Deep inert ordinary','Shadow inert ordinary','Nested shadow ordinary','Kept button','Kept option'])assert(sample.body.includes(kept));
        assert.equal(sample.body.match(/light slot text/g).length,1);assert.equal(frameHits,0);assert.equal(boundary.blocked,false);
        await fs.writeFile(path.join(output,'sanitized-transfer.txt'),sample.body,{mode:0o600,flag:'wx'});
        report.cases.push({name:'live-and-recursive-exclusions',stable:sample.stable,nodes:sample.visited_nodes,iframe_hosts:sample.iframe_hosts,open_shadow_roots:sample.open_shadow_roots});
      });
      await t.test('attribute order/comments are neutral; whitespace and hidden text remain material',async()=>{
        const read=async()=>encode((await page.evaluate(collectDomInPage,DOM_LIMITS)).records);
        const first=await read();
        await page.evaluate(()=>{const e=document.getElementById('ordinary-link'),v=e.getAttribute('class');e.removeAttribute('class');e.setAttribute('class',v);e.append(document.createComment('neutral'));});
        assert.equal(await read(),first);
        await page.evaluate(()=>document.getElementById('private').firstChild.data+=' ');assert.notEqual(await read(),first);
        report.cases.push({name:'canonical-order-whitespace',passed:true});
      });
      await t.test('all hard bounds include shadow/template/excluded nodes',async()=>{
        for(const limits of [{nodes:5},{depth:2},{attributes:1},{stringBytes:5},{bytes:80}])await assert.rejects(()=>page.evaluate(collectDomInPage,{...DOM_LIMITS,...limits}),/DomLimit/);
        report.cases.push({name:'hard-bounds',passed:true});
      });
      await t.test('actual limits count templates and open roots below excluded ancestors',async()=>{
        try{
          await page.evaluate(()=>{
            const excluded=document.createElement('textarea'),template=document.createElement('template');
            excluded.id='excluded-accounting-fixture';document.body.append(excluded);excluded.append(template);
            for(let i=0;i<100001;i++)template.content.append(document.createComment('excluded'));
          });
          await assert.rejects(()=>page.evaluate(collectDomInPage,DOM_LIMITS),/DomLimit/);
          await page.evaluate(()=>{
            const template=document.getElementById('excluded-accounting-fixture').firstChild;template.content.replaceChildren();
            let parent=template.content;for(let i=0;i<600;i++){const child=document.createElement('div');parent.append(child);parent=child;}
          });
          await assert.rejects(()=>page.evaluate(collectDomInPage,DOM_LIMITS),/DomLimit/);
          await page.evaluate(()=>{
            const excluded=document.getElementById('excluded-accounting-fixture');excluded.replaceChildren();
            const host=document.createElement('div');excluded.append(host);const root=host.attachShadow({mode:'open'});
            for(let i=0;i<100001;i++)root.append(document.createComment('excluded'));
          });
          await assert.rejects(()=>page.evaluate(collectDomInPage,DOM_LIMITS),/DomLimit/);
          report.cases.push({name:'excluded-template-shadow-actual-limits',passed:true});
        }finally{await page.evaluate(()=>document.getElementById('excluded-accounting-fixture')?.remove());}
      });
      await t.test('continuous live mutation is not stable in three samples',async()=>{
        await page.evaluate(()=>{window.fixtureTimer=setInterval(()=>document.getElementById('private').textContent=String(performance.now()),10);});
        const sample=await sampleDom(page,{deadline:Date.now()+3000,encode});assert.equal(sample.stable,false);assert.equal(sample.samples,3);
        await page.evaluate(()=>clearInterval(window.fixtureTimer));report.cases.push({name:'unstable-three-samples',passed:true});
      });
    }finally{await context.close();}
    const baseProfile=JSON.parse((await fs.readFile(new URL('fixtures/session-requests.ndjson',import.meta.url),'utf8')).trim().split('\n')[1]).command.payload.profile;
    baseProfile.runtime=catalog.manifest.identity;baseProfile.width=640;baseProfile.height=480;
    const runCapture=async(pathname,kinds,{guardScreenshot=false,replace=false,onChannel=true,readinessMs,extent}={})=>{
      const profile=structuredClone(baseProfile),results=[],warnings=[];
      if(replace)profile.image_mode={ReplaceRaster:{algorithm:'neutral-raster-v2'}};
      if(readinessMs!==undefined)profile.stabilization.readiness_ms=readinessMs;
      if(extent!==undefined)profile.extent=extent;
      const adapter=guardScreenshot?{newContext:async options=>{
        const context=await browser.newContext(options),newPage=context.newPage.bind(context);
        context.newPage=async()=>{const page=await newPage();page.screenshot=()=>assert.fail('DOM-only screenshot called');page.addStyleTag=()=>assert.fail('DOM-only screenshot style injected');page.mouse.move=()=>assert.fail('DOM-only screenshot mouse moved');return page;};return context;
      }}:browser;
      const options={identity:profile.runtime,scope:'loopback',output,run:crypto.randomUUID(),onContext:()=>{},progress:()=>{},warning:(code,kind)=>warnings.push({code,kind}),kinds,encodeDom:encode,domPolicy:{version:1}};
      if(onChannel)options.onChannel=async channel=>{results.push(channel);if(channel.artifact)assert.equal((await fs.stat(path.join(output,channel.output_token+(channel.kind==='Screenshot'?'.png':'.dom.jsonl')))).size,channel.artifact.bytes);};
      const result=await capture(adapter,{profile,address:origin+pathname,key:{page:crypto.randomUUID(),profile:profile.id},session:null},options);
      if(!onChannel)results.push(...result.results);
      assert.equal(result.results.length,kinds.length);return {results,warnings};
    };
    await t.test('DOM-only bypasses screenshot transforms and image geometry gates',async()=>{
      const {results}=await runCapture('/tall',['DomSnapshot'],{guardScreenshot:true});
      assert.equal(results.length,1);assert.equal(results[0].kind,'DomSnapshot');assert.ok(results[0].artifact);
      const body=await fs.readFile(path.join(output,results[0].output_token+'.dom.jsonl'),'utf8');assert(body.includes('100001px'));
      report.cases.push({name:'dom-only-no-pixels-large-height',passed:true});
    });
    await t.test('canonical collector preserves real Chromium HTML names without XML assumptions',async()=>{
      const {results}=await runCapture('/html-names',['DomSnapshot']);assert.ok(results[0].artifact);
      const body=await fs.readFile(path.join(output,results[0].output_token+'.dom.jsonl'),'utf8'),records=body.trimEnd().split('\n').map(JSON.parse);
      assert(records.some(r=>r[0]==='doctype'&&r[2]==='123'));
      const element=records.find(r=>r[0]==='element'&&r[3]==='a:b');assert.ok(element);
      for(const name of ['@click','123','[binding]',':class','a:foo','b:foo','a:b:c'])assert(element[4].some(attr=>attr[0]===''&&attr[1]===name));
      const prefixed=records.filter(r=>r[0]==='element'&&r[2]==='urn:fixture');assert.deepEqual(prefixed.map(r=>r[5]??null),['a','c',null]);
      assert(body.endsWith('\n'));report.cases.push({name:'real-html-name-identity',passed:true});
    });
    await t.test('both-mode snapshots publish before every screenshot-owned mutation',async()=>{
      const {results}=await runCapture('/mutations',['Screenshot','DomSnapshot'],{replace:true});
      assert.deepEqual(results.map(r=>r.kind),['DomSnapshot','Screenshot']);assert(results.every(r=>r.artifact));
      const body=await fs.readFile(path.join(output,results[0].output_token+'.dom.jsonl'),'utf8');
      assert(body.includes('site-owned-style'));assert(body.includes('site-owned-image'));assert(body.includes('"loading","lazy"'));assert(!body.includes('animation:none'));assert(!body.includes('"eager"'));assert(!body.includes('cbd5e1'));
      report.cases.push({name:'both-pre-mutation-publication',passed:true});
    });
    await t.test('independent failures preserve the successful sibling and scope warnings',async()=>{
      const imageFailure=await runCapture('/tall',['Screenshot','DomSnapshot']);
      assert.ok(imageFailure.results[0].artifact);assert.equal(imageFailure.results[1].failure,'ImageLimit');
      const domFailure=await runCapture('/dom-limit',['Screenshot','DomSnapshot']);
      assert.equal(domFailure.results[0].failure,'DomLimit');assert.ok(domFailure.results[1].artifact);
      report.cases.push({name:'independent-dom-and-image-failures',passed:true});
    });
    await t.test('both-mode returns independent results even without a publication callback',async()=>{
      const domFailure=await runCapture('/dom-limit',['Screenshot','DomSnapshot'],{onChannel:false});
      assert.equal(domFailure.results[0].failure,'DomLimit');assert.ok(domFailure.results[1].artifact);
      const imageFailure=await runCapture('/tall',['Screenshot','DomSnapshot'],{onChannel:false});
      assert.ok(imageFailure.results[0].artifact);assert.equal(imageFailure.results[1].failure,'ImageLimit');
      report.cases.push({name:'independent-results-without-callback',passed:true});
    });
    await t.test('short screenshot stage budgets are not spent by preceding DOM samples',async()=>{
      const pairs=[];
      for(const readinessMs of [180,220,260]){
        const alone=await runCapture('/stable',['Screenshot'],{readinessMs,extent:'VisibleScreen'});
        const both=await runCapture('/stable',['Screenshot','DomSnapshot'],{readinessMs,extent:'VisibleScreen'});
        assert.ok(alone.results[0].artifact,`screenshot-only ${readinessMs}ms fixture must succeed`);
        assert.ok(both.results[0].artifact,`DOM ${readinessMs}ms fixture must succeed`);
        assert.ok(both.results[1].artifact,`preceding DOM must not spend the ${readinessMs}ms screenshot budget`);
        pairs.push({readiness_ms:readinessMs,screenshot_only:'captured',both:['captured','captured']});
      }
      report.cases.push({name:'independent-short-stage-budgets',pairs});
    });
    await t.test('actual v3 worker publishes scoped channels then completes, and fans out shared failures',async()=>{
      const ipcOutput=await fs.mkdtemp(path.join(output,'ipc-'));
      const identity={Pinned:{...catalog.manifest.identity.Pinned,worker:WORKER_VERSION}};
      const env={HOME:process.env.HOME,TMPDIR:process.env.TMPDIR,XDG_CONFIG_HOME:process.env.HOME,XDG_CACHE_HOME:process.env.HOME,LANG:'C.UTF-8',PLAYWRIGHT_BROWSERS_PATH:path.join(root,'browsers'),REGRESSEE_RUNTIME:JSON.stringify(identity),REGRESSEE_SCOPE:'loopback',REGRESSEE_OUTPUT:ipcOutput};
      const runWorker=async({pixelsForbidden=false}={},body)=>{
        const args=[];
        if(pixelsForbidden){
          const preload=path.join(ipcOutput,'forbid-pixels.mjs');
          await fs.writeFile(preload,`import {chromium} from ${JSON.stringify(new URL('../node_modules/playwright/index.mjs',import.meta.url).href)};
const launch=chromium.launch.bind(chromium);chromium.launch=async options=>{const browser=await launch(options),create=browser.newContext.bind(browser);browser.newContext=async options=>{const context=await create(options),createPage=context.newPage.bind(context);context.newPage=async()=>{const page=await createPage();page.screenshot=async()=>{throw new Error('test-only pixel path forbidden');};return page;};return context;};return browser;};\n`,{mode:0o600,flag:'wx'});
          args.push('--import',preload);
        }
        args.push(fileURLToPath(new URL('../src/index.mjs',import.meta.url)));
        const child=spawn(process.execPath,args,{env,stdio:['pipe','pipe','pipe']});
        const frames=[],waiters=[];let pending='',next=0,stderrBytes=0;
        const exit=new Promise((resolve,reject)=>{child.once('error',reject);child.once('exit',(code,signal)=>resolve({code,signal}));});
        child.stderr.on('data',chunk=>{stderrBytes+=chunk.length;});
        child.stdout.on('data',chunk=>{
          pending+=chunk.toString('utf8');let end;
          while((end=pending.indexOf('\n'))>=0){const line=pending.slice(0,end);pending=pending.slice(end+1);assert(Buffer.byteLength(line)<MAX_FRAME);const event=JSON.parse(line);frames.push(event);for(const waiter of [...waiters])if(waiter.match(event)){waiters.splice(waiters.indexOf(waiter),1);waiter.resolve(event);}}
          assert(Buffer.byteLength(pending)<MAX_FRAME);
        });
        const wait=(id,types)=>new Promise((resolve,reject)=>{
          const match=event=>event.request_id===id&&types.includes(event.event.type),found=frames.find(match);
          if(found){resolve(found);return;}
          const timer=setTimeout(()=>reject(new Error('owned worker event timeout')),10000);
          waiters.push({match,resolve:event=>{clearTimeout(timer);resolve(event);}});
        });
        const command=async(type,payload)=>{
          const id=++next,job=crypto.randomUUID();
          child.stdin.write(JSON.stringify({version:VERSION,request_id:id,job,command:payload===undefined?{type}:{type,payload}})+'\n');
          await wait(id,type==='CaptureTarget'?['CaptureCompleted','Failure','Cancelled']:['Ready','Completed','Failure','Cancelled']);
          if(type==='CaptureTarget')await new Promise(resolve=>setTimeout(resolve,80));
          return {job,events:frames.filter(event=>event.request_id===id).map(event=>event.event)};
        };
        try{await body(command);await command('Shutdown');assert.deepEqual(await exit,{code:0,signal:null});}
        finally{
          child.stdin.end();const timer=setTimeout(()=>child.kill('SIGTERM'),2000);
          try{await exit;}finally{clearTimeout(timer);report.cases.push({name:'owned-worker-ipc',pixels_forbidden:pixelsForbidden,frames,stderr_bytes:stderrBytes});}
        }
      };
      const target=(pathname,mode)=>{const profile=structuredClone(baseProfile);profile.runtime=identity;return {address:origin+pathname,profile,key:{page:crypto.randomUUID(),profile:profile.id},session:null,plan:{version:1,mode,dom_policy:mode==='ScreenshotsOnly'?null:{version:1}}};};
      await runWorker({pixelsForbidden:true},async command=>{
        assert.equal((await command('Handshake')).events.at(-1).type,'Ready');
        const request=target('/tall','DomSnapshotsOnly'),captured=await command('CaptureTarget',request);
        assert.equal(captured.events.at(-1).type,'CaptureCompleted');
        const publications=captured.events.filter(e=>e.type==='ChannelCaptured');assert.equal(publications.length,1);
        const capture=publications[0].payload.capture;assert.equal(capture.kind,'DomSnapshot');
        const metadata=JSON.parse(await fs.readFile(path.join(ipcOutput,capture.output_token+'.dom.json'),'utf8'));
        const canonical=await fs.readFile(path.join(ipcOutput,capture.output_token+'.dom.jsonl'),'utf8');
        assert.equal(metadata.run,captured.job);assert.equal(metadata.bytes,capture.artifact.bytes);assert.equal(metadata.node_count,canonical.trimEnd().split('\n').length-1);
        assert.equal((await command('InstallSmoke')).events.at(-1).type,'Failure');
      });
      await runWorker({},async command=>{
        assert.equal((await command('Handshake')).events.at(-1).type,'Ready');
        for(const [pathname,successful,failed,code] of [['/tall','DomSnapshot','Screenshot','ImageLimit'],['/dom-limit','Screenshot','DomSnapshot','DomLimit']]){
          const result=await command('CaptureTarget',target(pathname,'ScreenshotsAndDomSnapshots'));
          assert.equal(result.events.at(-1).type,'CaptureCompleted');
          assert.equal(result.events.filter(e=>e.type==='ChannelCaptured').length,1);
          assert.equal(result.events.find(e=>e.type==='ChannelCaptured').payload.capture.kind,successful);
          assert.deepEqual(result.events.filter(e=>e.type==='ChannelFailed').map(e=>[e.payload.kind,e.payload.code]),[[failed,code]]);
          assert(result.events.filter(e=>e.type==='ChannelWarning').every(e=>['Screenshot','DomSnapshot'].includes(e.payload.kind)));
        }
        const shared=await command('CaptureTarget',target('/notfound','ScreenshotsAndDomSnapshots'));
        assert.deepEqual(shared.events.filter(e=>e.type==='ChannelFailed').map(e=>[e.payload.kind,e.payload.code]),[['Screenshot','NotFound'],['DomSnapshot','NotFound']]);
        assert.equal(shared.events.at(-1).type,'CaptureCompleted');
        for(const mode of ['ScreenshotsOnly','ScreenshotsAndDomSnapshots']){
          for(const ms of mode==='ScreenshotsOnly'?[100,180,300]:[300])for(let repeat=0;repeat<3;repeat++){
            const request=target('/stalled-font',mode);request.profile.stabilization.readiness_ms=ms;
            const {events}=await command('CaptureTarget',request);
            const published=new Set();let complete=false;
            for(const event of events){
              assert(!complete,'expired work emitted an event after CaptureCompleted');
              if(event.type==='ChannelWarning')assert(!published.has(event.payload.kind),'expired work emitted a warning after its channel publication');
              if(event.type==='ChannelCaptured'||event.type==='ChannelFailed'){
                const kind=event.type==='ChannelCaptured'?event.payload.capture.kind:event.payload.kind;
                assert(!published.has(kind));published.add(kind);
              }
              if(event.type==='CaptureCompleted'){assert.equal(published.size,mode==='ScreenshotsOnly'?1:2);complete=true;}
            }
            assert(complete);assert.equal(events.find(e=>e.type==='ChannelFailed'&&e.payload.kind==='Screenshot')?.payload.code,'Timeout');
            if(mode==='ScreenshotsAndDomSnapshots')assert(events.some(e=>e.type==='ChannelCaptured'&&e.payload.capture.kind==='DomSnapshot'));
            report.cases.push({name:'expired-font-stage-frame-order',mode,readiness_ms:ms,repeat,events});
          }
        }
        assert(heldFontRequests>0,'owned delayed-font resource must actually have been requested');
      });
    });
  }finally{
    await browser?.close();server.closeAllConnections();await new Promise(resolve=>server.close(resolve));
    report.cleanup={browser_connected:browser?.isConnected()??false,server_listening:server.listening,frame_hits:frameHits,held_font_requests:heldFontRequests};
    await fs.writeFile(path.join(output,'report.json'),JSON.stringify(report,null,2),{mode:0o600});
  }
});
