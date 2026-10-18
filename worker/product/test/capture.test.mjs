// SPDX-License-Identifier: MPL-2.0
import test from 'node:test';
import assert from 'node:assert/strict';
// Import the actual adapter without invoking capture. The browser suite below
// exercises it from the signed private pack; this source test checks its boundary.
import fs from 'node:fs';
import fsp from 'node:fs/promises';
import path from 'node:path';
import os from 'node:os';
import crypto,{randomUUID} from 'node:crypto';
import {validateProfile,literalLoopback,navigationBoundary,capture} from '../src/capture.mjs';
import {collectDomInPage} from '../src/dom.mjs';
const profile=JSON.parse(fs.readFileSync(new URL('fixtures/session-requests.ndjson',import.meta.url),'utf8').trim().split('\n')[1]).command.payload.profile;
test('actual adapter repeats immutable desktop/profile/runtime bounds',()=>{
  validateProfile(profile,profile.runtime);
  validateProfile({...profile,mobile:true,touch:true,width:390,height:844},profile.runtime);
  for (const [key,value] of [['width',0],['width',4097],['dpr',2],['mobile','yes'],['image_mode',{ReplaceRaster:{algorithm:'unqualified'}}],['reduced_motion',false],['extra','unsafe']]) {
    const changed=structuredClone(profile);changed[key]=value;
    assert.throws(()=>validateProfile(changed,profile.runtime),{code:'InvalidRequest'});
  }
  const changed=structuredClone(profile);changed.stabilization.readiness_ms=NaN;
  assert.throws(()=>validateProfile(changed,profile.runtime),{code:'InvalidRequest'});
  assert.throws(()=>validateProfile(profile,{Pinned:{...profile.runtime.Pinned,browser:'other'}}),{code:'InvalidRequest'});
});
test('development scope accepts literal loopback, not localhost/DNS/external hosts',()=>{
  for(const url of ['http://127.0.0.1/','http://127.0.0.2/','http://[::1]/'])assert.equal(literalLoopback(url),true);
  for(const url of ['http://localhost/','http://example.com/','http://127.0.0.1.example.com/'])assert.equal(literalLoopback(url),false);
});
test('native document response interception never releases a redirect',async()=>{
   for(const status of [301,302,303,307,308]) {
     let routeHandler,paused;const commands=[];
     const cdp={on:(_,handler)=>{paused=handler;},send:async(method,params)=>{commands.push({method,params});if(method==='Page.getFrameTree')return {frameTree:{frame:{id:'main'}}};}};
     const context={on:()=>{},route:async(_,handler)=>{routeHandler=handler;},routeWebSocket:async()=>{},newCDPSession:async()=>cdp};
     const frame={page:()=>page};const page={mainFrame:()=>frame};
     const state=await navigationBoundary(context,'http://127.0.0.1:1234/scope','loopback',1000,'/scope',page);
     const request={url:()=> 'http://127.0.0.1:1234/scope',method:()=> 'GET',frame:()=>frame,isNavigationRequest:()=>true};
     let continued=0,aborts=0;
     const route={request:()=>request,abort:async()=>{aborts++;},continue:async()=>{continued++;},fetch:()=>assert.fail('must retain native network response'),fulfill:()=>assert.fail('must retain native network response')};
     await routeHandler(route);
     paused({requestId:'document',frameId:'main',request:{url:request.url()},resourceType:'Document',responseStatusCode:status,responseHeaders:[{name:'Location',value:'http://127.0.0.1:5678/identity-provider'}]});
     await new Promise(resolve=>setImmediate(resolve));
     assert.equal(state.redirect,'http://127.0.0.1:5678/identity-provider');
     assert.equal(continued,1);assert.equal(commands.at(-1).method,'Fetch.failRequest');
     await routeHandler(route); // even same-address second navigation is not admitted
     assert.equal(continued,1);assert.equal(aborts,1);assert.equal(state.blocked,true);
  }
});
test('forms, child documents and off-section initial navigations are blocked before fetching',async()=>{
  for(const kind of ['POST','frame','section','origin']) {
     let routeHandler;
      const context={on:()=>{},route:async(_,h)=>{routeHandler=h;},routeWebSocket:async()=>{},newCDPSession:async()=>({on:()=>{},send:async(method)=>method==='Page.getFrameTree'?{frameTree:{frame:{id:'main'}}}:undefined})};
     const frame={page:()=>page};const page={mainFrame:()=>frame};
     const state=await navigationBoundary(context,'http://127.0.0.1:1234/outside','loopback',1000,kind==='section'?'/scope':null,page);
    let aborted=false;
    await routeHandler({request:()=>({url:()=>kind==='origin'?'http://127.0.0.1:5678/outside':'http://127.0.0.1:1234/outside',method:()=>kind==='POST'?'POST':'GET',frame:()=>kind==='frame'?{page:()=>page}:frame,isNavigationRequest:()=>true}),abort:async()=>{aborted=true;},fetch:()=>assert.fail('forbidden request fetched'),continue:()=>assert.fail('forbidden request continued')});
     assert.equal(aborted,true);assert.equal(state.blocked,kind!=='frame');assert.equal(state.embeddedBlocked,kind==='frame');
  }
});

test('each channel gets a fresh bounded stage budget, including common readiness and DOM quiet time',async t=>{
  // Deterministic timing regression for the verifier's real-browser 180/220/
  // 260ms counterexample: DOM's 100ms quiet interval is not PNG's budget.
  let now=0;t.mock.method(Date,'now',()=>now);
  const output=await fsp.mkdtemp(path.join(process.env.REGRESSEE_DOM_TEST_OUTPUT||os.tmpdir(),'regressee-capture-budget-'));
  const p=structuredClone(profile);p.extent='VisibleScreen';p.width=640;p.height=480;p.stabilization.readiness_ms=180;
  const address='http://127.0.0.1:1234/owned',screenshotTimeouts=[];
  const image=Buffer.alloc(32);image.writeUInt32BE(p.width,16);image.writeUInt32BE(p.height,20);
  const browser={newContext:async()=>{
    const page={on:()=>{},url:()=>address,goto:async()=>({status:()=>200}),waitForFunction:async()=>{now+=30;},addStyleTag:async()=>{},mouse:{move:async()=>{}},waitForTimeout:async ms=>{now+=ms;},screenshot:async options=>{screenshotTimeouts.push(options.timeout);now+=20;return image;},evaluate:async fn=>{
      now+=5;
      if(fn===collectDomInPage)return {records:[['document'],['text','ordinary']],visited_nodes:2,iframe_hosts:0,open_shadow_roots:0};
      const source=fn.toString();
      if(source.includes('document.querySelector'))return false;
      if(source.includes('width:Math.max'))return {width:p.width,height:p.height};
      return 0;
    }};
    page.mainFrame=()=>({});
    return {on:()=>{},route:async()=>{},routeWebSocket:async()=>{},newCDPSession:async()=>({on:()=>{},send:async method=>method==='Page.getFrameTree'?{frameTree:{frame:{id:'main'}}}:undefined}),newPage:async()=>page,close:async()=>{}};
  }};
  const run=async kinds=>{
    const result=await capture(browser,{profile:p,address,key:{page:randomUUID(),profile:p.id},session:null},{identity:p.runtime,scope:'loopback',output,onContext:()=>{},progress:()=>{},warning:()=>{},kinds,run:randomUUID(),encodeDom:records=>JSON.stringify(records),domPolicy:{version:1},onChannel:()=>{}});
    assert(result.results.every(r=>r.artifact));return result;
  };
  try{
    await run(['Screenshot']);const singleFirst=screenshotTimeouts[0];screenshotTimeouts.length=0;
    const both=await run(['Screenshot','DomSnapshot']);
    assert.deepEqual(both.results.map(r=>r.kind),['DomSnapshot','Screenshot']);
    assert.equal(screenshotTimeouts[0],singleFirst);assert.equal(singleFirst,180);
  }finally{await fsp.rm(output,{recursive:true,force:true});}
});

function lifecycleFixture({stall='fonts'}={}) {
  const events=[],lateOperations=[],releases=[];let screenshotFinished=false;
  const p=structuredClone(profile);p.extent='VisibleScreen';p.width=640;p.height=480;p.stabilization.readiness_ms=30;
  const address='http://127.0.0.1:1234/owned';
  const image=Buffer.alloc(32);image.writeUInt32BE(p.width,16);image.writeUInt32BE(p.height,20);
  const operation=name=>{if(screenshotFinished)lateOperations.push(name);};
  const pending=()=>new Promise(resolve=>releases.push(resolve));
  const browser={newContext:async()=>{
    const page={on:()=>{},url:()=>address,goto:async()=>({status:()=>200}),waitForFunction:async()=>{},addStyleTag:async()=>{operation('style');},mouse:{move:async()=>{operation('mouse');}},waitForTimeout:async()=>{},screenshot:async()=>{operation('PNG');return image;},evaluate:async fn=>{
      const source=fn.toString();operation('evaluate');
      if(fn===collectDomInPage)return {records:[['document',0],['element',1,'http://www.w3.org/1999/xhtml','html',[]],['text',2,'ordinary']],node_count:3,visited_nodes:3,max_depth:2,coverage:{iframe_hosts:0,open_shadow_roots:0,template_contents:0},iframe_hosts:0,open_shadow_roots:0};
      if(source.includes('document.querySelector'))return false;
      if(source.includes('document.fonts.ready'))return stall==='fonts'?pending():true;
      // If the font sub-budget wins by a clock tick, keep the following read
      // pending too. The whole preparation stage must expire deterministically;
      // a sub-budget warning followed by a successful capture is otherwise valid.
      if(source.includes("f.status==='error'"))return stall==='fonts'?pending():stall==='images'?1:0;
      if(source.includes('i.decode()'))return stall==='images'?pending():0;
      if(source.includes('width:Math.max'))return stall==='images'?pending():{width:p.width,height:p.height};
      return 0;
    }};
    page.mainFrame=()=>({});
    return {on:()=>{},route:async()=>{},routeWebSocket:async()=>{},newCDPSession:async()=>({on:()=>{},send:async method=>method==='Page.getFrameTree'?{frameTree:{frame:{id:'main'}}}:undefined}),newPage:async()=>page,close:async()=>{}};
  }};
  return {
    p,browser,address,events,lateOperations,
    release:()=>{for(const release of releases)release(0);},
    options:output=>({identity:p.runtime,scope:'loopback',output,run:randomUUID(),onContext:()=>{},progress:(completed,total)=>events.push({type:'Progress',completed,total}),warning:(code,kind)=>events.push({type:'Warning',kind,code}),onChannel:channel=>{events.push({type:'Published',...channel});if(channel.kind==='Screenshot')screenshotFinished=true;}}),
  };
}
const settle=()=>new Promise(resolve=>setTimeout(resolve,40));

test('expired font/image preparation cannot emit warnings or continue browser work after failure',async()=>{
  for(const stall of ['fonts','images'])for(const kinds of [['Screenshot'],['Screenshot','DomSnapshot']]){
    const fixture=lifecycleFixture({stall});
    const output=await fsp.mkdtemp(path.join(process.env.REGRESSEE_DOM_TEST_OUTPUT||os.tmpdir(),'regressee-expired-stage-'));
    try{
      const result=await capture(fixture.browser,{profile:fixture.p,address:fixture.address,key:{page:randomUUID(),profile:fixture.p.id},session:null},{...fixture.options(output),kinds});
      const failed=result.results.find(r=>r.kind==='Screenshot');assert.equal(failed.failure,'Timeout');
      const terminal=fixture.events.findIndex(e=>e.type==='Published'&&e.kind==='Screenshot');assert(terminal>=0);
      fixture.release();await settle();
      assert.deepEqual(fixture.lateOperations,[]);
      assert(!fixture.events.slice(terminal+1).some(e=>e.type==='Warning'&&e.kind==='Screenshot'));
      if(stall==='images')assert(fixture.events.slice(0,terminal).some(e=>e.type==='Warning'&&e.kind==='Screenshot'&&e.code==='FontsIncomplete'),'valid pre-failure warning must remain');
      const files=await fsp.readdir(output);assert(!files.some(name=>name.endsWith('.png')));
      if(kinds.includes('DomSnapshot'))assert.ok(result.results.find(r=>r.kind==='DomSnapshot').artifact);
    }finally{fixture.release();await fsp.rm(output,{recursive:true,force:true});}
  }
});

test('expired DOM I/O is aborted and cannot later write its metadata or publish success',async t=>{
  const fixture=lifecycleFixture({stall:null});
  const output=await fsp.mkdtemp(path.join(process.env.REGRESSEE_DOM_TEST_OUTPUT||os.tmpdir(),'regressee-expired-dom-'));
  const original=fsp.open.bind(fsp);let releaseBody,domMetadataWrites=0;
  t.mock.method(fsp,'open',async(file,...options)=>{
    if(String(file).endsWith('.dom.json'))domMetadataWrites++;
    const handle=await original(file,...options);
    if(String(file).endsWith('.dom.jsonl')){
      const write=handle.writeFile.bind(handle);
      handle.writeFile=async(body,options)=>{
        await new Promise((resolve,reject)=>{
          releaseBody=resolve;
          if(options?.signal?.aborted)reject(options.signal.reason);
          else options?.signal?.addEventListener('abort',()=>reject(options.signal.reason),{once:true});
        });
        return write(body,options);
      };
    }
    return handle;
  });
  try{
    const result=await capture(fixture.browser,{profile:fixture.p,address:fixture.address,key:{page:randomUUID(),profile:fixture.p.id},session:null},{...fixture.options(output),kinds:['Screenshot','DomSnapshot']});
    assert.equal(result.results[0].failure,'Timeout');assert.ok(result.results[1].artifact);
    assert.equal(typeof releaseBody,'function');releaseBody();await settle();
    assert.equal(domMetadataWrites,0);
    assert(!(await fsp.readdir(output)).some(file=>file.includes('.dom.')));
    assert.equal(fixture.events.filter(e=>e.type==='Published'&&e.kind==='DomSnapshot').length,1);
  }finally{releaseBody?.();await fsp.rm(output,{recursive:true,force:true});}
});

test('failed-stage cleanup never removes a pre-existing UUID file',async t=>{
  const fixture=lifecycleFixture({stall:null});
  const output=await fsp.mkdtemp(path.join(process.env.REGRESSEE_DOM_TEST_OUTPUT||os.tmpdir(),'regressee-stage-collision-'));
  const token='aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa',existing=path.join(output,token+'.dom.jsonl');
  await fsp.writeFile(existing,'retained earlier owned evidence',{flag:'wx',mode:0o600});
  t.mock.method(crypto,'randomUUID',()=>token);
  try{
    const result=await capture(fixture.browser,{profile:fixture.p,address:fixture.address,key:{page:randomUUID(),profile:fixture.p.id},session:null},{...fixture.options(output),kinds:['DomSnapshot']});
    assert.equal(result.results[0].failure,'CaptureFailed');
    assert.equal(await fsp.readFile(existing,'utf8'),'retained earlier owned evidence');
  }finally{await fsp.rm(output,{recursive:true,force:true});}
});

test('publication callback failures still reject the capture rather than being swallowed',async()=>{
  const fixture=lifecycleFixture({stall:null});
  const output=await fsp.mkdtemp(path.join(process.env.REGRESSEE_DOM_TEST_OUTPUT||os.tmpdir(),'regressee-publication-error-'));
  const rejected=new Error('owned event sink rejected publication');
  try{
    await assert.rejects(()=>capture(fixture.browser,{profile:fixture.p,address:fixture.address,key:{page:randomUUID(),profile:fixture.p.id},session:null},{...fixture.options(output),onChannel:()=>{throw rejected;}}),error=>error===rejected);
  }finally{await fsp.rm(output,{recursive:true,force:true});}
});
