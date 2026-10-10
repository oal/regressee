// SPDX-License-Identifier: MPL-2.0
// Owned loopback browser evidence. Synthetic images are not user-accuracy data.
import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import http from 'node:http';
import path from 'node:path';
import zlib from 'node:zlib';
import {launch,capture,validateProfile} from '../src/capture.mjs';
const crc=bytes=>{let c=0xffffffff;for(const b of bytes){c^=b;for(let i=0;i<8;i++)c=(c>>>1)^((c&1)?0xedb88320:0);}const out=Buffer.alloc(4);out.writeUInt32BE((c^0xffffffff)>>>0);return out;};
const chunk=(name,data)=>{const n=Buffer.from(name),l=Buffer.alloc(4);l.writeUInt32BE(data.length);return Buffer.concat([l,n,data,crc(Buffer.concat([n,data]))]);};
const png=color=>{const header=Buffer.alloc(13);header.writeUInt32BE(80,0);header.writeUInt32BE(40,4);header[8]=8;header[9]=2;const row=Buffer.concat([Buffer.from([0]),Buffer.from(Array(80).fill(color).flat())]);return Buffer.concat([Buffer.from('89504e470d0a1a0a','hex'),chunk('IHDR',header),chunk('IDAT',zlib.deflateSync(Buffer.concat(Array(40).fill(row)))),chunk('IEND',Buffer.alloc(0))]);};
test('actual replacement capture: repeated changed raster/picture/background, source drift and unsupported media',async()=>{
  const output=process.env.REGRESSEE_PHASE5_WORKER_OUTPUT;
  assert.ok(output,'fresh owned evidence directory required');await fs.mkdir(output,{recursive:false});
  const profile=JSON.parse((await fs.readFile(new URL('fixtures/session-requests.ndjson',import.meta.url),'utf8')).trim().split('\n')[1]).command.payload.profile;
  profile.width=600;profile.height=300;profile.extent='VisibleScreen';
  let variant=0;
  const server=http.createServer((req,res)=>{
    if(req.url==='/a.png'||req.url==='/b.png'){res.writeHead(200,{'Content-Type':'image/png'});res.end(png(variant?[20,70,220]:[220,70,20]));return;}
    if(req.url==='/missing.png'){res.writeHead(404);res.end('deliberate missing image');return;}
    const src=variant?'b.png':'a.png';
    const source=req.url==='/source';const unsupported=req.url==='/unsupported';const missing=req.url==='/missing';
    res.writeHead(200,{'Content-Type':'text/html'});
    res.end(`<!doctype html><style>body{margin:0;background:white}img{width:120px;height:80px;object-fit:cover;object-position:20% 60%;border:2px solid black}.bg{width:150px;height:70px;background-image:url('/${src}');background-size:cover}${source?'img[src*="a.png"]{margin-left:12px}':''}</style><img src="/${missing?'missing.png':src}"><picture><source srcset="/${src}"><img src="/${src}"></picture><div class="bg"></div><svg width="12" height="12"><rect width="12" height="12" fill="red"/></svg>${unsupported?'<canvas width="10" height="10"></canvas><video></video><div style="background:linear-gradient(red,blue);height:10px"></div>':''}`);
  });
  await new Promise(resolve=>server.listen(0,'127.0.0.1',resolve));
  const origin=`http://127.0.0.1:${server.address().port}`;
  const browser=await launch();const results=[];
  try{
    const run=async(route,replace,label)=>{
      const p=structuredClone(profile);p.image_mode=replace?{ReplaceRaster:{algorithm:'neutral-raster-v2'}}:'Normal';validateProfile(p,p.runtime);
      const warnings=[];const result=await capture(browser,{address:origin+route,profile:p,key:{page:'11111111-1111-4111-8111-111111111111',profile:p.id},session:null},{identity:p.runtime,scope:'loopback',output,onContext:()=>{},progress:()=>{},warning:c=>warnings.push(c)});
      const meta=JSON.parse(await fs.readFile(path.join(output,result.output_token+'.json'),'utf8'));
      assert.equal(meta.profile.image_mode==='Normal',!replace);assert.equal(meta.stable,true);assert.equal(result.width,600);assert.equal(result.height,300);
      results.push({label,result,metadata:meta});return {result,meta,warnings};
    };
    const normal=await run('/',false,'normal-before');const before=await run('/',true,'replacement-before');
    assert.equal(before.meta.replacement.count,3);assert.equal(before.meta.replacement.geometryPreserved,true);assert.deepEqual(before.warnings,[]);
    assert.notEqual(normal.result.artifact.id,before.result.artifact.id);
    variant=1;const changed=await run('/',false,'normal-after');const after=await run('/',true,'replacement-after');const repeated=await run('/',true,'replacement-repeat');
    assert.notEqual(normal.result.artifact.id,changed.result.artifact.id);assert.equal(before.result.artifact.id,after.result.artifact.id);assert.equal(after.result.artifact.id,repeated.result.artifact.id);
    assert.deepEqual(before.meta.replacement.before,before.meta.replacement.after);assert.equal(after.meta.replacement.geometryPreserved,true);
    variant=0;const source=await run('/source',true,'source-selector');assert.equal(source.meta.replacement.geometryPreserved,false);assert.ok(source.warnings.includes('ReplacementSourceDependent'));assert.ok(source.warnings.includes('ReplacementGeometryChanged'));
    const unsupported=await run('/unsupported',true,'unsupported');assert.ok(unsupported.warnings.includes('ReplacementUnsupportedMedia'));
    const missing=await run('/missing',true,'readiness-failure');assert.ok(missing.warnings.includes('ImagesIncomplete'));assert.ok(missing.warnings.includes('ResourceFailed'));assert.equal(missing.meta.replacement.skipped.length,1);
    const bad=structuredClone(profile);bad.image_mode={ReplaceRaster:{algorithm:'neutral-raster-v3'}};assert.throws(()=>validateProfile(bad,bad.runtime));
    await fs.writeFile(path.join(output,'report.json'),JSON.stringify({qualification:'owned sandboxed Chromium synthetic replacement scope only; no human/native/public accuracy approval',results},null,2));
  }finally{await browser.close();await new Promise(resolve=>server.close(resolve));}
});
