// SPDX-License-Identifier: MPL-2.0
// Owned concurrent numeric-loopback fixtures, NOT public-site qualification.
// Run from a fresh signed pack via actual_phase3_resource_corrections.
import test from 'node:test';
import assert from 'node:assert/strict';
import http from 'node:http';
import fs from 'node:fs/promises';
import path from 'node:path';
import crypto from 'node:crypto';
import {spawn} from 'node:child_process';
import {fileURLToPath} from 'node:url';
import {launch,navigationBoundary,capture,literalLoopback,PLAYWRIGHT_VERSION} from '../src/capture.mjs';
import {discover} from '../src/discover.mjs';

const profile=JSON.parse((await fs.readFile(new URL('fixtures/session-requests.ndjson',import.meta.url),'utf8')).trim().split('\n')[1]).command.payload.profile;
const listen=s=>new Promise((resolve,reject)=>{s.once('error',reject);s.listen(0,'127.0.0.1',resolve);});
const stop=async s=>{s.closeAllConnections();await new Promise(resolve=>s.close(resolve));};
const png=Buffer.from('iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mP8/x8AAwMCAO+aZ1cAAAAASUVORK5CYII=','base64');
const ids=['same-direct','cross-direct','same-redirect','cross-redirect','chain'];
const kinds=['js','css','png'];

test('signed Chromium native scope boundary: real resources, zero forbidden visits and paused cancellation',async t=>{
  const output=process.env.REGRESSEE_RESOURCE_EVIDENCE;
  assert.ok(output,'fresh evidence directory required');
  const identity=JSON.parse(process.env.REGRESSEE_RESOURCE_RUNTIME);
  profile.runtime=identity;profile.user_agent='Regressee owned resource fixture';
  await fs.mkdir(output,{mode:0o700});
  const hits={},report={cases:[],hits};let mainOrigin,secondOrigin,deniedOrigin,browser;
  const count=(server,url)=>hits[server+' '+url]||0;
  const redirect=(res,address)=>{res.writeHead(302,{Location:address,'Access-Control-Allow-Origin':'*'});res.end();};
  const asset=(res,id,kind)=>{
    res.setHeader('Access-Control-Allow-Origin','*');
    res.setHeader('Content-Type',kind==='js'?'text/javascript':kind==='css'?'text/css':'image/png');
    res.end(kind==='js'?`window.executed=(window.executed||[]).concat(${JSON.stringify(id)});document.body.insertAdjacentHTML('beforeend','<a href="/scope/from-${id}">executed</a>');`:kind==='css'?`:root{--${id}:loaded}`:png);
  };
  const html=srcs=>'<!doctype html><body><h1>Owned resource fixture</h1>'+srcs;
  const resourceTag=(url,id,kind)=>kind==='js'?`<script src="${url}"></script>`:kind==='css'?`<link rel="stylesheet" href="${url}">`:`<img id="${id}" src="${url}" width="1" height="1">`;
  const handler=server=>(req,res)=>{
    hits[server+' '+req.url]=count(server,req.url)+1;
    const u=new URL(req.url,'http://127.0.0.1');
    if(u.pathname==='/scope/slow')return; // Deliberately pending until owned context cancellation.
    if(u.pathname==='/scope/unauthorized'||u.pathname==='/scope/forbidden') {res.writeHead(u.pathname.endsWith('unauthorized')?401:403);res.end('denied');return;}
    if(u.pathname==='/worker.js') {
      res.setHeader('Content-Type','text/javascript');
      res.end(`Promise.all([fetch('${secondOrigin}/r/cross-redirect.js').then(r=>r.ok?'loaded':'failed',()=> 'failed'),fetch('/deny-chain/js').then(()=> 'forbidden',()=> 'denied')]).then(results=>postMessage(results));`);return;
    }
    if(u.pathname==='/scope/worker') {
      res.setHeader('Content-Type','text/html');
      res.end(html(`<script>new Worker('/worker.js').onmessage=e=>window.workerResults=e.data;Promise.all([fetch('${secondOrigin}/r/cross-redirect.js').then(r=>r.ok?'loaded':'failed',()=> 'failed'),fetch('/deny-chain/js').then(()=> 'forbidden',()=> 'denied')]).then(results=>window.fetchResults=results);</script>`));return;
    }
    const match=u.pathname.match(/^\/r\/([a-z-]+)\.(js|css|png)$/);
    if(match){
      const [,id,kind]=match;
      if(id==='same-redirect')return redirect(res,mainOrigin+`/r/same-final.${kind}`);
      if(id==='cross-redirect')return redirect(res,secondOrigin+`/r/cross-final.${kind}`);
      if(id==='chain')return redirect(res,secondOrigin+`/r/chain-middle.${kind}`);
      if(id==='chain-middle')return redirect(res,mainOrigin+`/r/chain-final.${kind}`);
      return asset(res,id==='same-final'?'same-redirect':id==='cross-final'?'cross-redirect':id==='chain-final'?'chain':id,kind);
    }
    if(u.pathname.startsWith('/deny/'))return redirect(res,deniedOrigin+'/forbidden-resource/'+u.pathname.split('/').at(-1));
    if(u.pathname.startsWith('/deny-chain/'))return redirect(res,secondOrigin+'/deny/'+u.pathname.split('/').at(-1));
    if(u.pathname.startsWith('/forbidden-resource/'))return asset(res,'forbidden',u.pathname.split('/').at(-1));
    if(u.pathname==='/scope/doc-section')return redirect(res,mainOrigin+'/outside');
    if(u.pathname==='/scope/doc-cross')return redirect(res,secondOrigin+'/forbidden-document');
    if(u.pathname==='/scope/refresh'){res.writeHead(200,{Refresh:`0;url=${secondOrigin}/forbidden-document`});res.end(html('refresh'));return;}
    if(u.pathname==='/scope/client') {res.end(html(`<script>location.href='${secondOrigin}/forbidden-document'</script>`));return;}
     if(u.pathname==='/scope/frame') {res.end(html(`<iframe src="${secondOrigin}/forbidden-document"></iframe>`));return;}
     if(u.pathname==='/scope/frame-same') {res.end(html('<a href="/scope/linked">Linked page</a><iframe src="/scope/frame-content"></iframe>'));return;}
     if(u.pathname==='/scope/frame-redirect') {res.end(html('<iframe src="/scope/doc-cross"></iframe>'));return;}
    if(u.pathname==='/scope/popup') {res.end(html(`<script>window.open('${secondOrigin}/forbidden-document')</script>`));return;}
    if(u.pathname==='/scope/form') {res.end(html('<form action="/outside" method="post"></form><script>document.forms[0].submit()</script>'));return;}
    if(u.pathname==='/scope/reload') {res.end(html('<script>location.reload()</script>'));return;}
    if(u.pathname==='/scope/document-one')return redirect(res,mainOrigin+'/scope/document-two');
    if(u.pathname==='/scope/document-two')return redirect(res,secondOrigin+'/scope/resources');
    if(u.pathname==='/scope/denied') {
      res.setHeader('Content-Type','text/html');
      res.end(html(kinds.map(kind=>resourceTag(deniedOrigin+'/forbidden-resource/'+kind,'direct-'+kind,kind)+resourceTag(mainOrigin+'/deny/'+kind,'redirect-'+kind,kind)+resourceTag(mainOrigin+'/deny-chain/'+kind,'chain-'+kind,kind)).join('')));return;
    }
    if(u.pathname==='/scope/resources') {
      res.setHeader('Content-Type','text/html');
      res.end(html(ids.flatMap(id=>kinds.map(kind=>resourceTag((id==='cross-direct'?secondOrigin:mainOrigin)+`/r/${id}.${kind}`,id,kind))).join('')));return;
    }
    if(u.pathname.startsWith('/scope/variant/')) {
      const id=u.pathname.split('/').at(-1);
      res.setHeader('Content-Type','text/html');res.end(html(resourceTag((id==='cross-direct'?secondOrigin:mainOrigin)+`/r/${id}.js`,id,'js')));return;
    }
    res.setHeader('Content-Type','text/html');res.end(html('ordinary page'));
  };
  const main=http.createServer(handler('main')),second=http.createServer(handler('second'));
  try {
    await listen(main);await listen(second);
    mainOrigin=`http://127.0.0.1:${main.address().port}`;secondOrigin=`http://127.0.0.1:${second.address().port}`;
    // Numeric OS-loopback alias reaches our IPv4 server but is runtime-denied.
    deniedOrigin=`http://[::ffff:127.0.0.1]:${second.address().port}`;
    assert.equal(literalLoopback(deniedOrigin),false);
    Object.assign(report,{mainOrigin,secondOrigin,deniedOrigin});browser=await launch();
    assert.equal(browser.version(),identity.Pinned.browser);assert.equal(process.version.slice(1),identity.Pinned.node);assert.equal(PLAYWRIGHT_VERSION,identity.Pinned.playwright);
    const guarded=async(address,fn,section='/scope')=>{
      const context=await browser.newContext({serviceWorkers:'block',acceptDownloads:false});
      try {
        const page=await context.newPage();const events=[];
        const cdp=await context.newCDPSession(page);await cdp.send('Network.enable');
        cdp.on('Network.requestWillBeSentExtraInfo',e=>{if(e.clientSecurityState)events.push(e.clientSecurityState);});
        const state=await navigationBoundary(context,address,'loopback',5000,section,page);
        return await fn(page,state,events);
      }finally{await context.close();}
    };
    const runCapture=async address=>{
      const warnings=[];let current;
      const result=await capture(browser,{address,profile,key:{page:crypto.randomUUID(),profile:profile.id},session:null},{identity:profile.runtime,scope:'loopback',output,onContext:c=>{current=c;},progress:()=>{},warning:w=>warnings.push(w)});
      assert.equal(current,null);
      const metadata=JSON.parse(await fs.readFile(path.join(output,result.output_token+'.json'),'utf8'));
      assert.equal(metadata.requested,address);assert.equal(metadata.final_url,address);
      assert.equal(metadata.width,profile.width);assert.equal(metadata.height,profile.height);
      return {result,metadata,warnings};
    };
    const zero=()=>{
      assert.equal(count('main','/outside'),0);
      assert.equal(count('second','/forbidden-document'),0);
      for(const kind of kinds)assert.equal(count('second','/forbidden-resource/'+kind),0,`forbidden ${kind} requested`);
    };

    await t.test('raw and ordinary continue controls; guarded direct/redirect script variants',async()=>{
      for(const mode of ['raw','continue']) {
        const context=await browser.newContext({serviceWorkers:'block',acceptDownloads:false});
        try {
          if(mode==='continue')await context.route('**/*',r=>r.continue());
          const page=await context.newPage();await page.goto(mainOrigin+'/scope/variant/cross-redirect',{waitUntil:'domcontentloaded',timeout:5000});
          assert.deepEqual(await page.evaluate(()=>window.executed),['cross-redirect']);
        }finally{await context.close();}
      }
      for(const id of ids)await guarded(mainOrigin+'/scope/variant/'+id,async(page,state)=>{
        await page.goto(mainOrigin+'/scope/variant/'+id,{waitUntil:'domcontentloaded',timeout:5000});
        assert.deepEqual(await page.evaluate(()=>window.executed),[id]);assert.equal(state.blocked,false);assert.equal(state.error,null);
      });
      report.cases.push({name:'controls-and-five-script-variants',passed:true});
    });
    await t.test('scripts execute, CSS applies, images decode and real document retains Loopback address space',async()=>{
      const address=mainOrigin+'/scope/resources';
      await guarded(address,async(page,state,events)=>{
        await page.goto(address,{waitUntil:'load',timeout:5000});
        const actual=await page.evaluate(async()=>({executed:window.executed,css:['same-direct','cross-direct','same-redirect','cross-redirect','chain'].map(id=>getComputedStyle(document.documentElement).getPropertyValue('--'+id).trim()),images:await Promise.all([...document.images].map(async i=>{await i.decode();return i.naturalWidth;})),url:location.href}));
        assert.deepEqual(actual.executed,ids);assert.deepEqual(actual.css,ids.map(()=>'loaded'));assert.deepEqual(actual.images,ids.map(()=>1));assert.equal(actual.url,address);
        assert.ok(events.length>=15);assert.ok(events.every(e=>e.initiatorIPAddressSpace==='Loopback'));
        assert.equal(state.redirect,null);assert.equal(state.blocked,false);assert.equal(state.error,null);
        report.cases.push({name:'real-resource-matrix',actual,clientSecurityStates:events,passed:true});
      });
      for(const kind of kinds) {
        for(const id of ['same-direct','same-redirect','same-final','cross-redirect','chain','chain-final'])assert.ok(count('main',`/r/${id}.${kind}`)>0);
        for(const id of ['cross-direct','cross-final','chain-middle'])assert.ok(count('second',`/r/${id}.${kind}`)>0);
      }
      const result=await runCapture(address);
      assert.deepEqual(result.warnings,[]);assert.equal(result.metadata.readiness.resource_failures,0);assert.equal(result.metadata.readiness.image_failures,0);
      const found=await discover(browser,{address,section:'/scope'},{scope:'loopback',onContext:()=>{}});
      assert.equal(found.final_url,address);for(const id of ids)assert.ok(found.addresses.includes(mainOrigin+'/scope/from-'+id));
      report.cases.push({name:'actual-capture-and-rendered-discovery',...result,found,passed:true});
    });
    await t.test('direct and multi-hop runtime-denied script/CSS/image resources never reach destination',async()=>{
      const address=mainOrigin+'/scope/denied';
      await guarded(address,async(page,state)=>{
        await page.goto(address,{waitUntil:'load',timeout:5000});
        assert.equal(await page.evaluate(()=>window.executed),undefined);
        assert.ok((await page.evaluate(()=>[...document.images].map(i=>i.naturalWidth))).every(n=>n===0));
        assert.equal(state.blocked,false);assert.equal(state.error,null);zero();
      });
      const result=await runCapture(address);
      assert.ok(result.warnings.includes('ResourceFailed'));assert.ok(result.warnings.includes('ImagesIncomplete'));assert.ok(result.metadata.readiness.resource_failures>=9);zero();
      for(const kind of kinds){assert.ok(count('main','/deny/'+kind)>0);assert.ok(count('main','/deny-chain/'+kind)>0);assert.ok(count('second','/deny/'+kind)>0);}
      report.cases.push({name:'truthful-denied-resource-capture',...result,passed:true});
    });
    await t.test('document redirects are unvisited policy candidates, including finite allowed chain and capture provenance',async()=>{
      for(const [pathname,destination] of [['doc-section',mainOrigin+'/outside'],['doc-cross',secondOrigin+'/forbidden-document'],['document-one',mainOrigin+'/scope/document-two'],['document-two',secondOrigin+'/scope/resources']]) {
        const before=count(destination.startsWith(mainOrigin)?'main':'second',new URL(destination).pathname);
        const address=mainOrigin+'/scope/'+pathname;
        const result=await discover(browser,{address,section:'/scope'},{scope:'loopback',onContext:()=>{}});
        assert.deepEqual(result,{redirect:destination});
        assert.equal(count(destination.startsWith(mainOrigin)?'main':'second',new URL(destination).pathname),before);
        await assert.rejects(()=>runCapture(address),{code:'RedirectOutsideScope'});zero();
      }
      // Explicit next visits emulate already-approved candidates, not automatic
      // navigation. Rust's maintained policy test verifies actual policy/aliases.
      const final=secondOrigin+'/scope/resources';const result=await runCapture(final);
      assert.equal(result.metadata.requested,final);assert.equal(result.metadata.final_url,final);assert.deepEqual(result.warnings,[]);
      report.cases.push({name:'finite-document-chain-final-provenance',...result,passed:true});
    });
    await t.test('page and dedicated-worker fetch resource redirects retain allow/deny scope',async()=>{
      const address=mainOrigin+'/scope/worker';
      await guarded(address,async(page,state)=>{
        await page.goto(address,{waitUntil:'load',timeout:5000});
        await page.waitForFunction(()=>window.workerResults&&window.fetchResults,{},{timeout:5000});
        const actual=await page.evaluate(()=>({worker:window.workerResults,page:window.fetchResults}));
        assert.deepEqual(actual,{worker:['loaded','denied'],page:['loaded','denied']});assert.equal(state.error,null);zero();
        report.cases.push({name:'page-and-worker-redirect-scope',actual,passed:true});
      });
    });
    await t.test('actual signed worker protocol captures resources, reports redirects/access failures and cancels pending network',async()=>{
      const workerOutput=path.join(output,'ipc');await fs.mkdir(workerOutput,{mode:0o700});
      const child=spawn(process.execPath,[fileURLToPath(new URL('../src/index.mjs',import.meta.url))],{env:{...process.env,REGRESSEE_RUNTIME:JSON.stringify(identity),REGRESSEE_SCOPE:'loopback',REGRESSEE_OUTPUT:workerOutput},stdio:['pipe','pipe','pipe']});
      const frames=[],waiters=[];let buffer='',stderr='',requestId=0;
      const exit=new Promise((resolve,reject)=>{child.once('error',reject);child.once('exit',(code,signal)=>resolve({code,signal}));});
      child.stderr.on('data',chunk=>{stderr+=chunk;});
      child.stdout.on('data',chunk=>{
        buffer+=chunk;let end;
        while((end=buffer.indexOf('\n'))>=0){const frame=JSON.parse(buffer.slice(0,end));buffer=buffer.slice(end+1);frames.push(frame);for(const waiter of [...waiters])if(waiter.match(frame)){waiters.splice(waiters.indexOf(waiter),1);waiter.resolve(frame);}}
      });
      const wait=(id,types)=>new Promise((resolve,reject)=>{
        const match=f=>f.request_id===id&&types.includes(f.event.type),found=frames.find(match);
        if(found){resolve(found);return;}
        const timer=setTimeout(()=>{const index=waiters.indexOf(waiter);if(index>=0)waiters.splice(index,1);reject(new Error(`worker event timeout ${id}: ${types}`));},10000);
        const waiter={match,resolve:f=>{clearTimeout(timer);resolve(f);}};waiters.push(waiter);
      });
      const send=(type,payload,job=crypto.randomUUID())=>{
        const id=++requestId;child.stdin.write(JSON.stringify({version:2,request_id:id,job,command:payload===undefined?{type}:{type,payload}})+'\n');return id;
      };
      const target=(address,session=null)=>({address,session,profile,key:{page:crypto.randomUUID(),profile:profile.id}});
      try {
        assert.equal((await wait(send('Handshake'),['Ready','Failure'])).event.type,'Ready');
        const captureEvent=await wait(send('CaptureTarget',target(mainOrigin+'/scope/resources')),['TargetCaptured','Failure']);
        assert.equal(captureEvent.event.type,'TargetCaptured');
        const metadata=JSON.parse(await fs.readFile(path.join(workerOutput,captureEvent.event.payload.output_token+'.json'),'utf8'));
        assert.deepEqual(metadata.runtime,identity);assert.equal(metadata.requested,mainOrigin+'/scope/resources');assert.equal(metadata.final_url,metadata.requested);assert.deepEqual(metadata.warnings,[]);
        const redirectEvent=await wait(send('DiscoverPage',{address:mainOrigin+'/scope/doc-cross',section:'/scope'}),['RedirectFound','Failure']);
        assert.deepEqual(redirectEvent.event,{type:'RedirectFound',payload:{address:secondOrigin+'/forbidden-document'}});zero();
        for(const name of ['unauthorized','forbidden']) {
          const event=await wait(send('CaptureTarget',target(mainOrigin+'/scope/'+name)),['Failure','TargetCaptured']);
          assert.equal(event.event.type,'Failure');assert.equal(event.event.payload.code,'AccessDenied');
        }
        const before=count('main','/scope/resources');
        const session={id:crypto.randomUUID(),project:crypto.randomUUID(),environment:crypto.randomUUID()};
        const refused=await wait(send('CaptureTarget',target(mainOrigin+'/scope/resources',session)),['Failure','TargetCaptured']);
        assert.equal(refused.event.payload.code,'InvalidRequest');assert.equal(count('main','/scope/resources'),before);
        const job=crypto.randomUUID(),slowBefore=count('main','/scope/slow');
        const slowId=send('CaptureTarget',target(mainOrigin+'/scope/slow'),job);
        const deadline=Date.now()+5000;
        while(count('main','/scope/slow')===slowBefore&&Date.now()<deadline)await new Promise(resolve=>setTimeout(resolve,10));
        assert.equal(count('main','/scope/slow'),slowBefore+1);
        const started=Date.now(),cancelId=send('Cancel',undefined,job);
        assert.equal((await wait(cancelId,['Cancelled','Failure'])).event.type,'Cancelled');
        assert.equal((await wait(slowId,['Cancelled','Failure','TargetCaptured'])).event.type,'Cancelled');assert.ok(Date.now()-started<1500);
        assert.equal((await wait(send('Shutdown'),['Completed','Failure'])).event.type,'Completed');
        assert.deepEqual(await exit,{code:0,signal:null});
        report.cases.push({name:'actual-worker-ipc-resource-capture-cancel',metadata,cancel_wall_ms:Date.now()-started,passed:true});
      }finally {
        child.stdin.end();
        const timer=setTimeout(()=>child.kill('SIGTERM'),2000);
        try {await exit;}finally{clearTimeout(timer);await fs.writeFile(path.join(workerOutput,'events.json'),JSON.stringify({frames,stderr},null,2));}
      }
    });
    await t.test('blocked embedded frames do not masquerade as main-page redirects or poison capture/discovery',async()=>{
      const contentBefore=count('main','/scope/frame-content'),redirectBefore=count('main','/scope/doc-cross');
      for(const name of ['frame','frame-same','frame-redirect']) {
        const address=mainOrigin+'/scope/'+name;
        await guarded(address,async(page,state)=>{
          await page.goto(address,{waitUntil:'load',timeout:5000});
          assert.equal(page.url(),address);assert.equal(state.blocked,false);
          assert.equal(state.redirect,null);assert.equal(state.embeddedBlocked,true);assert.equal(state.error,null);
        });
        const result=await runCapture(address);
        assert.ok(result.warnings.includes('EmbeddedContentBlocked'));
        const found=await discover(browser,{address,section:'/scope'},{scope:'loopback',onContext:()=>{}});
        assert.equal(found.final_url,address);
        if(name==='frame-same')assert.ok(found.addresses.includes(mainOrigin+'/scope/linked'));
        zero();
      }
      assert.equal(count('main','/scope/frame-content'),contentBefore);
      assert.equal(count('main','/scope/doc-cross'),redirectBefore);
      report.cases.push({name:'blocked-embeds-valid-main-page',passed:true});
    });
    await t.test('exhausted full-page readiness is a timeout, not a false image-size limit',async()=>{
      const short=structuredClone(profile);short.stabilization.readiness_ms=1;
      await assert.rejects(()=>capture(browser,{address:mainOrigin+'/scope/frame-same',profile:short,key:{page:crypto.randomUUID(),profile:short.id},session:null},{identity:short.runtime,scope:'loopback',output,onContext:()=>{},progress:()=>{},warning:()=>{}}),{code:'Timeout'});
      zero();
      report.cases.push({name:'readiness-timeout-not-image-limit',passed:true});
    });
    await t.test('client navigation, popups, forms, reload and Refresh remain pre-network blocked',async()=>{
      for(const name of ['client','popup','form','reload','refresh']) {
        const address=mainOrigin+'/scope/'+name,before=count('main','/scope/'+name);
        await assert.rejects(()=>discover(browser,{address,section:'/scope'},{scope:'loopback',onContext:()=>{}}),{code:'RedirectOutsideScope'});
        assert.equal(count('main','/scope/'+name),before+1);zero();
      }
      await guarded(mainOrigin+'/outside',async(page,state)=>{
        await assert.rejects(()=>page.goto(mainOrigin+'/outside',{timeout:5000}));assert.equal(state.blocked,true);zero();
      });
      report.cases.push({name:'automatic-document-and-section-zero-visits',passed:true});
    });
    await t.test('closing context during native request/response pauses releases interception and browser remains usable',async()=>{
      for(const stage of ['Request','Response']) {
        const context=await browser.newContext({serviceWorkers:'block',acceptDownloads:false});
        let release,observed;const gate=new Promise(r=>{release=r;}),paused=new Promise(r=>{observed=r;});
        const original=context.newCDPSession.bind(context);
        context.newCDPSession=async page=>{
          const cdp=await original(page),send=cdp.send.bind(cdp);let continued=0,held=false;
          cdp.send=async(method,params)=>{
            if(method===`Fetch.continue${stage}`)continued++;
            // Request cancellation holds the cross-origin script, not the first
            // document. Response cancellation holds the real document headers.
            if(!held&&method===`Fetch.continue${stage}`&&continued===(stage==='Request'?2:1)){held=true;observed();await gate;}
            return send(method,params);
          };return cdp;
        };
        try {
          const address=mainOrigin+'/scope/variant/cross-direct',page=await context.newPage();
          const state=await navigationBoundary(context,address,'loopback',5000,'/scope',page);
          const navigation=page.goto(address,{waitUntil:'domcontentloaded',timeout:5000}).then(()=>assert.fail('paused navigation unexpectedly completed'),e=>e.message);
          await Promise.race([paused,new Promise((_,reject)=>{const timer=setTimeout(()=>reject(new Error('pause not observed')),5000);timer.unref();})]);
          const started=Date.now();await context.close();release();assert.match(await navigation,/closed|aborted|ERR/);assert.ok(Date.now()-started<1500);assert.equal(state.error,null);
          report.cases.push({name:'cancel-during-'+stage.toLowerCase()+'-pause',wall_ms:Date.now()-started,passed:true});
        }finally{release();await context.close();}
      }
      assert.equal(browser.contexts().length,0);await runCapture(mainOrigin+'/scope/resources');zero();
    });
    assert.equal(browser.contexts().length,0);zero();
  }finally {
    if(browser)await browser.close();await stop(main);await stop(second);
    report.cleanup={browser_connected:browser?.isConnected()??false,main_listening:main.listening,second_listening:second.listening};
    await fs.writeFile(path.join(output,'report.json'),JSON.stringify(report,null,2));
  }
});
