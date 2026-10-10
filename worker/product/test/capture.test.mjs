// SPDX-License-Identifier: MPL-2.0
import test from 'node:test';
import assert from 'node:assert/strict';
// Import the actual adapter without invoking capture. The browser suite below
// exercises it from the signed private pack; this source test checks its boundary.
import fs from 'node:fs';
import {validateProfile,literalLoopback,navigationBoundary} from '../src/capture.mjs';
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
     const cdp={on:(_,handler)=>{paused=handler;},send:async(method,params)=>{commands.push({method,params});}};
     const context={on:()=>{},route:async(_,handler)=>{routeHandler=handler;},routeWebSocket:async()=>{},newCDPSession:async()=>cdp};
     const frame={page:()=>page};const page={mainFrame:()=>frame};
     const state=await navigationBoundary(context,'http://127.0.0.1:1234/scope','loopback',1000,'/scope',page);
     const request={url:()=> 'http://127.0.0.1:1234/scope',method:()=> 'GET',frame:()=>frame,isNavigationRequest:()=>true};
     let continued=0,aborts=0;
     const route={request:()=>request,abort:async()=>{aborts++;},continue:async()=>{continued++;},fetch:()=>assert.fail('must retain native network response'),fulfill:()=>assert.fail('must retain native network response')};
     await routeHandler(route);
     paused({requestId:'document',request:{url:request.url()},resourceType:'Document',responseStatusCode:status,responseHeaders:[{name:'Location',value:'http://127.0.0.1:5678/identity-provider'}]});
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
     const context={on:()=>{},route:async(_,h)=>{routeHandler=h;},routeWebSocket:async()=>{},newCDPSession:async()=>({on:()=>{},send:async()=>{}})};
     const frame={page:()=>page};const page={mainFrame:()=>frame};
     const state=await navigationBoundary(context,'http://127.0.0.1:1234/outside','loopback',1000,kind==='section'?'/scope':null,page);
    let aborted=false;
    await routeHandler({request:()=>({url:()=>kind==='origin'?'http://127.0.0.1:5678/outside':'http://127.0.0.1:1234/outside',method:()=>kind==='POST'?'POST':'GET',frame:()=>kind==='frame'?{page:()=>page}:frame,isNavigationRequest:()=>true}),abort:async()=>{aborted=true;},fetch:()=>assert.fail('forbidden request fetched'),continue:()=>assert.fail('forbidden request continued')});
    assert.equal(aborted,true);assert.equal(state.blocked,true);
  }
});
