import test from 'node:test';
import assert from 'node:assert/strict';
import http from 'node:http';
import fs from 'node:fs/promises';
import path from 'node:path';
import os from 'node:os';
import {launch,replay,capture,replaceImages,IMAGE_ALGORITHM} from '../src/capture.mjs';
import {solidRaster,decodeScreenshot} from './png-helpers.mjs';

// Optional fresh evidence only: refuse to overwrite a previous verifier directory.
const evidence=process.env.REGRESSEE_IMAGE_EVIDENCE;
if(evidence)await fs.mkdir(evidence,{mode:0o700});
async function record(name,result,pixels) {
  if(!evidence)return;
  await fs.writeFile(path.join(evidence,name+'.png'),result.image,{flag:'wx'});
  await fs.writeFile(path.join(evidence,name+'.json'),JSON.stringify({metadata:result.metadata,pixels},null,2),{flag:'wx'});
}
async function fixture(run) {
  const tall=solidRaster(96,1800),wide=solidRaster(1800,96);
  const server=http.createServer((req,res)=>{
    const url=new URL(req.url,'http://fixture.invalid');
    if(url.pathname==='/asset.png'||url.pathname==='/wide.png'){res.setHeader('Content-Type','image/png');res.end(url.pathname==='/wide.png'?wide:tall);return;}
    const fit=url.searchParams.get('fit')||'fill',position=url.searchParams.get('position')||'50% 50%';
    res.setHeader('Content-Type','text/html');
    res.end(`<!doctype html><style>body{margin:24px;background:white}img{display:block;width:${url.pathname==='/selector'?'80':'160'}px;height:40px;object-fit:${fit};object-position:${position}}${url.pathname==='/selector'?'img[src="/asset.png"]{width:160px}':''}</style><img src="${url.searchParams.has('wide')?'/wide.png':'/asset.png'}">`);
  });
  await new Promise(r=>server.listen(0,'127.0.0.1',r));
  // Linux Chromium needs a short socket path, independent of the test runner.
  const scratch=await fs.mkdtemp(path.join(process.platform==='linux'?'/tmp':os.tmpdir(),'regressee-image-')),keys=['TMPDIR','TMP','TEMP'],previous=keys.map(k=>process.env[k]);
  for(const k of keys)process.env[k]=scratch;
  let browser,context;
  try{browser=await launch(false);context=await replay(browser);await run(context,'http://127.0.0.1:'+server.address().port);}
  finally{
    await context?.close();await browser?.close();await new Promise(r=>server.close(r));
    for(let i=0;i<keys.length;i++)if(previous[i]===undefined)delete process.env[keys[i]];else process.env[keys[i]]=previous[i];
    assert.deepEqual(await fs.readdir(scratch),[]);await fs.rmdir(scratch);
  }
}

test('source-selector geometry drift is a warning even with stable pixels',async()=>{
  await fixture(async(context,origin)=>{
    const normal=await capture(context,origin+'/selector',{fullPage:false});await record('selector-normal',normal);
    for(let i=0;i<3;i++){
      const result=await capture(context,origin+'/selector',{replace:true,fullPage:false}),m=result.metadata,r=m.replacement;
      assert.equal(r.count,1);assert(r.geometryMeasuredElements>0);
      assert.deepEqual(r.before.at(-1).slice(3),[160,40]);assert.deepEqual(r.after.at(-1).slice(3),[80,40]);
      assert.equal(r.geometryPreserved,false);assert.equal(m.stable,true);
      assert.deepEqual(m.warnings,['ReplacementGeometryChanged']);assert(r.limitations.some(x=>/source-dependent/.test(x)));
      assert.notEqual(m.sha256,normal.metadata.sha256);await record('selector-replacement-'+i,result);
    }
  });
});

const colors={source:[255,0,0],neutral:[203,213,225],white:[255,255,255]};
function checkPixels(image,{fit,wide,position},replace) {
  const png=decodeScreenshot(image),w=wide?1800:96,h=wide?96:1800;
  const scale=fit==='contain'?Math.min(160/w,40/h):Math.max(160/w,40/h);
  const cw=fit==='fill'?160:w*scale,ch=fit==='fill'?40:h*scale;
  const left=(160-cw)*position[0],top=(40-ch)*position[1];
  let checked=0,content=0;const counts={source:0,neutral:0,white:0,other:0};
  for(let y=0;y<40;y++)for(let x=0;x<160;x++){
    const pixel=png.pixel(24+x,24+y),key=Object.keys(colors).find(k=>pixel.every((c,i)=>c===colors[k][i]))||'other';counts[key]++;
    // Ignore only subpixel contain edges, not the interior or letterbox positions.
    const internalEdge=(value,size,coord)=>value>0&&value<size&&Math.abs(coord-value)<0.5;
    if(internalEdge(left,160,x+0.5)||internalEdge(left+cw,160,x+0.5)||internalEdge(top,40,y+0.5)||internalEdge(top+ch,40,y+0.5))continue;
    const inside=x+0.5>=left&&x+0.5<left+cw&&y+0.5>=top&&y+0.5<top+ch;
    assert.deepEqual(pixel,inside?(replace?colors.neutral:colors.source):colors.white,JSON.stringify({fit,wide,position,replace,x,y}));
    checked++;if(inside)content++;
  }
  assert(checked>6000);assert(content>0);
  if(fit!=='contain')assert.equal(counts[replace?'neutral':'source'],6400);
  else assert(counts.white>0);
  return {box:[24,24,160,40],checked,content,counts};
}

test('neutral source retains intrinsic aspect and raster fill/contain/cover screenshot coverage',async()=>{
  await fixture(async(context,origin)=>{
    const cases=[
      {name:'tall-fill',fit:'fill',position:[0.5,0.5]},
      {name:'tall-contain-left',fit:'contain',position:[0,0.5]},
      {name:'tall-contain-center',fit:'contain',position:[0.5,0.5]},
      {name:'tall-contain-right',fit:'contain',position:[1,0.5]},
      {name:'wide-contain-top',fit:'contain',wide:true,position:[0.5,0]},
      {name:'wide-contain-bottom',fit:'contain',wide:true,position:[0.5,1]},
      {name:'tall-cover-bottom',fit:'cover',position:[0.5,1]},
      {name:'wide-cover-right',fit:'cover',wide:true,position:[1,0.5]}
    ];
    for(const c of cases){
      const url=origin+'/?'+new URLSearchParams({fit:c.fit,position:c.position.map(p=>p*100+'%').join(' '),...(c.wide?{wide:'1'}:{})});
      for(const replace of [false,true]){
        const result=await capture(context,url,{replace,fullPage:false}),m=result.metadata;
        assert.equal(m.stable,true);assert.deepEqual(m.warnings,[]);
        if(replace){assert.equal(m.replacement.count,1);assert.equal(m.replacement.geometryPreserved,true);assert.deepEqual(m.replacement.before.at(-1).slice(3),[160,40]);assert.deepEqual(m.replacement.before,m.replacement.after);}
        await record(c.name+'-'+(replace?'replacement':'normal'),result,checkPixels(result.image,c,replace));
      }
    }
    // Rectangles cannot prove retained intrinsic source aspect or computed fit.
    const page=await context.newPage();
    try{
      await page.goto(origin+'/?fit=contain&position=100%25+0%25');await page.locator('img').evaluate(i=>i.decode());
      const inspect=()=>page.locator('img').evaluate(i=>({width:i.naturalWidth,height:i.naturalHeight,fit:getComputedStyle(i).objectFit,position:getComputedStyle(i).objectPosition}));
      const before=await inspect();assert.deepEqual(before,{width:96,height:1800,fit:'contain',position:'100% 0%'});
      await replaceImages(page);assert.deepEqual(await inspect(),before);
      await page.locator('img').evaluate(i=>{i.style.width='auto';i.style.height='auto';});
      assert.deepEqual(await page.locator('img').evaluate(i=>[i.getBoundingClientRect().width,i.getBoundingClientRect().height]),[96,1800]);
      if(evidence)await fs.writeFile(path.join(evidence,'intrinsic.json'),JSON.stringify({before,after:await inspect(),imageAlgorithm:IMAGE_ALGORITHM},null,2),{flag:'wx'});
    }finally{await page.close();}
  });
});
