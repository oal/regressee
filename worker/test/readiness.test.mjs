import test from 'node:test';
import assert from 'node:assert/strict';
import http from 'node:http';
import {launch,replay,capture} from '../src/capture.mjs';
import {fixtureServer} from '../src/fixtures.mjs';

test('broken images/fonts/resources warn even when pixels stabilize; non-fixture geometry is measured',async()=>{
  let browser,context,fixture,raster;
  const server=http.createServer((req,res)=>{
    if(req.url==='/good.png'){res.setHeader('Content-Type','image/png');res.end(raster);}
    else if(req.url==='/plain'){res.setHeader('Content-Type','text/html');res.end('<!doctype html><p>Ordinary non-fixture page</p><div><img src="/good.png" style="width:120px;height:60px;object-fit:contain;border:3px solid black"><div style="width:120px;height:60px;background-image:url(/good.png)"></div></div>');}
    else if(req.url==='/geometry-budget'){res.setHeader('Content-Type','text/html');res.end('<!doctype html><img src="/good.png">'+('<div style="display:none"></div>'.repeat(2100)));}
    else if(req.url==='/bad.png'){res.setHeader('Content-Type','image/png');res.end('broken synthetic raster');}
    else if(req.url==='/bad.woff'){res.setHeader('Content-Type','font/woff');res.end('broken synthetic font');}
    else if(req.url==='/missing.css'){res.writeHead(404);res.end();}
    else if(req.url==='/wide'){res.end('<!doctype html><div style="width:4100px">wide</div>');}
    else {res.setHeader('Content-Type','text/html');res.end('<!doctype html><link rel="stylesheet" href="/missing.css"><style>@font-face{font-family:broken;src:url(/bad.woff)}body{font-family:broken,sans-serif}</style><p>Broken resources</p><img src="/bad.png" width="100" height="50">');}
  });
  await new Promise(r=>server.listen(0,'127.0.0.1',r));
  const origin='http://127.0.0.1:'+server.address().port;
  try {
    browser=await launch(false);context=await replay(browser);
    const {metadata:m}=await capture(context,origin,{replace:true});
    assert.equal(m.stable,true);assert(m.warnings.includes('ImageDecodeFailed'));assert(m.warnings.includes('FontLoadFailed'));assert(m.warnings.includes('ResourceHttpError'));
    assert.equal(m.readiness.imageFailures.length,1);assert.equal(m.readiness.imageFailures[0].reason,'ImageDecodeFailed');assert.equal(m.readiness.fontFailures,1);
    assert.equal(m.replacement.count,0);assert.equal(m.replacement.skipped[0].reason,'ImageNotDecoded');
    assert(m.replacement.geometryMeasuredElements>=5);assert.equal(m.replacement.before.length,m.replacement.geometryMeasuredElements);assert.deepEqual(m.replacement.before,m.replacement.after);
    await assert.rejects(capture(context,origin+'/wide'),e=>e.code==='ImageBudgetExceeded');
    fixture=await fixtureServer();
    raster=Buffer.from(await (await fetch(fixture.origin+'/raster.png')).arrayBuffer());
    const normal=await capture(context,origin+'/plain');
    const plain=await capture(context,origin+'/plain',{replace:true});
    assert.equal(plain.metadata.replacement.count,2);assert(plain.metadata.replacement.geometryMeasuredElements>=7);assert(plain.metadata.replacement.before.some(rect=>rect[3]>100&&rect[4]>50));
    assert.deepEqual(plain.metadata.replacement.before,plain.metadata.replacement.after);assert.equal(plain.metadata.replacement.geometryPreserved,true);assert.notEqual(plain.metadata.sha256,normal.metadata.sha256);assert.deepEqual(plain.metadata.warnings,[]);
    const large=await capture(context,origin+'/geometry-budget',{replace:true});assert.equal(large.metadata.replacement.count,1);assert.equal(large.metadata.replacement.geometryMeasuredElements,0);assert.equal(large.metadata.replacement.geometryPreserved,null);assert.match(large.metadata.replacement.geometryScope,/NOT MEASURED/);assert(large.metadata.warnings.includes('ReplacementGeometryNotMeasured'));
    for(let i=0;i<3;i++) {const r=await capture(context,fixture.origin+'/public',{replace:true});assert.equal(r.metadata.replacement.count,4);assert(r.metadata.replacement.geometryMeasuredElements>4);assert.equal(r.metadata.replacement.geometryPreserved,true);assert.deepEqual(r.metadata.warnings,[]);}
  } finally {await context?.close();await browser?.close();await fixture?.close();await new Promise(r=>server.close(r));}
});
