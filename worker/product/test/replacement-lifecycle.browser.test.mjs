// SPDX-License-Identifier: MPL-2.0
import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import http from 'node:http';
import path from 'node:path';
import {launch,capture,PLAYWRIGHT_VERSION,WORKER_VERSION} from '../src/capture.mjs';

test('bounded independent replacement request/lifecycle trace and genuine failure controls',async()=>{
  const output=process.env.REGRESSEE_REPLACEMENT_TRACE_OUTPUT;
  assert.ok(output,'fresh private evidence path required');await fs.mkdir(output,{recursive:false});
  const png=Buffer.from('iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mP8/x8AAwMCAO+aZ1cAAAAASUVORK5CYII=','base64');
  const hits=[];
  const server=http.createServer((req,res)=>{
    const url=new URL(req.url,'http://fixture');
    const label=url.pathname;hits.push({label,at:Date.now()});
    if(label.endsWith('.png')){
      if(label==='/missing.png'){res.writeHead(404);res.end();return;}
      res.writeHead(200,{'Content-Type':'image/png','Cache-Control':'no-store'});
      const timer=setTimeout(()=>res.end(png),label==='/fallback.png'?350:40);
      res.on('close',()=>clearTimeout(timer));return;
    }
    if(label==='/missing.js'){res.writeHead(500);res.end();return;}
    res.writeHead(200,{'Content-Type':'text/html','Cache-Control':'no-store'});
    const negative=label==='/negative';
    if(label==='/same') {
      res.end("<!doctype html><style>body{margin:0;background:white}img{width:120px;height:80px;object-fit:cover;object-position:20% 60%;border:2px solid black}.bg{width:150px;height:70px;background-image:url('/image.png');background-size:cover}</style><img src='/image.png'><picture><source srcset='/image.png'><img src='/image.png'></picture><div class=bg></div><svg width=12 height=12><rect width=12 height=12 fill=red /></svg>");return;
    }
    res.write(`<!doctype html><style>body{margin:0;background:white}.bg{width:80px;height:40px;background-image:url('/background.png')}img{width:80px;height:40px}</style><img src='/plain.png'><picture><source srcset='/selected.png'><img src='/fallback.png'></picture><img src='/fallback.png' srcset='/selected.png 1x'><div class=bg></div>${negative?"<img src='/missing.png'><script src='/missing.js'></script><img src='http://example.invalid/forbidden.png'><script>const c=new AbortController();fetch('/selected.png?genuine=1',{signal:c.signal}).catch(()=>{});setTimeout(()=>c.abort(),5);</script>":''}`);
    setTimeout(()=>res.end('<p>Slow HTML end</p>'),50);
  });
  await new Promise(resolve=>server.listen(0,'127.0.0.1',resolve));
  const browser=await launch();const reports=[];
  try {
    const p=JSON.parse((await fs.readFile(new URL('fixtures/session-requests.ndjson',import.meta.url),'utf8')).trim().split('\n')[1]).command.payload.profile;
    p.width=600;p.height=300;p.extent='VisibleScreen';p.image_mode={ReplaceRaster:{algorithm:'neutral-raster-v2'}};
    p.runtime={Pinned:{node:process.version.slice(1),worker:WORKER_VERSION,playwright:PLAYWRIGHT_VERSION,browser:browser.version()}};
    for(const route of ['/clean','/same','/negative']) {
      const trace=[],warnings=[];
      const result=await capture(browser,{address:`http://127.0.0.1:${server.address().port}${route}`,profile:p,key:{page:'11111111-1111-4111-8111-111111111111',profile:p.id},session:null},{identity:p.runtime,scope:'loopback',output,onContext:()=>{},progress:()=>{},warning:code=>warnings.push(code),diagnostics:event=>trace.push(event)});
      const metadata=JSON.parse(await fs.readFile(path.join(output,result.output_token+'.json'),'utf8'));
      reports.push({route,trace,warnings,metadata});
      if(route==='/negative'){assert.ok(warnings.includes('ResourceFailed'));assert.ok(warnings.includes('ImagesIncomplete'));assert.ok(trace.some(e=>e.event==='failed'&&e.kind==='fetch'&&e.failure==='Aborted'&&!e.attributed_replacement),'unrelated ERR_ABORTED must remain a genuine failure');}
      else {assert.deepEqual(warnings,[]);}
      const attributed=trace.filter(e=>e.event==='failed'&&e.failure==='Aborted'&&e.attributed_replacement);
      assert.equal(metadata.replacement.cancelledSourceReloads,attributed.length,'every independently traced intentional reload cancellation must be accounted');
      for(const failure of attributed)assert.ok(trace.some(e=>e.event==='request'&&e.request===failure.request&&e.replacing&&e.known_replacement_source&&e.kind==='image'));
    }
    await fs.writeFile(path.join(output,'trace.json'),JSON.stringify({reports,hits},null,2),{flag:'wx',mode:0o600});
  } finally {await browser.close();server.closeAllConnections();await new Promise(resolve=>server.close(resolve));}
});
