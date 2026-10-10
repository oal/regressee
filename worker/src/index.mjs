// SPDX-License-Identifier: MPL-2.0
import fs from 'node:fs/promises';
import path from 'node:path';
import assert from 'node:assert/strict';
import { fixtureServer } from './fixtures.mjs';
import { launch, PROFILE, IMAGE_ALGORITHM, fingerprint, comparable, replay, saveSession, capture } from './capture.mjs';

const MAX_LINE=1024*1024;
const fixture=await fixtureServer();
const artifacts=path.resolve(process.env.REGRESSEE_ARTIFACTS||'artifacts');
await fs.mkdir(artifacts,{recursive:true,mode:0o700});
let browser, loginBrowser, loginContext, loginPage, session, active=false, handshaken=false, closing=false, cleanupPromise;
const launches=new Set();
async function startBrowser(headed){if(closing)throw Object.assign(new Error(),{code:'ShuttingDown'});const pending=launch(headed);launches.add(pending);try{return await pending;}finally{launches.delete(pending);}}
const emit=(request,type,payload={},code)=>process.stdout.write(JSON.stringify({v:1,id:Number.isSafeInteger(request?.id)?request.id:null,job:'phase0',type,payload,...(code?{code}: {})})+'\n');
async function closeBrowsers(){const pending=await Promise.allSettled([...launches]);await Promise.allSettled([browser?.close(),loginBrowser?.close(),...pending.filter(x=>x.status==='fulfilled').map(x=>x.value.close())]);browser=loginBrowser=loginContext=loginPage=undefined;session=undefined;}
function cleanup(){if(cleanupPromise)return cleanupPromise;closing=true;session=undefined;cleanupPromise=(async()=>{await closeBrowsers();await fixture.close();})();return cleanupPromise;}
process.on('SIGTERM',()=>cleanup().finally(()=>process.exit(0)));
process.on('SIGINT',()=>cleanup().finally(()=>process.exit(0)));
async function managed(){if(!browser?.isConnected())browser=await startBrowser(false);return browser;}
async function storeCapture(context,url,name,options){
  const result=await capture(context,url,options);
  const output=path.join(artifacts,name+'.png');
  await fs.writeFile(output+'.partial',result.image,{mode:0o600});await fs.rename(output+'.partial',output);
  const metadata={...result.metadata,environment:{...fingerprint(browser),profile:result.metadata.profile,imageAlgorithm:result.metadata.profile.replace?IMAGE_ALGORITHM:'normal-v1',fonts:result.metadata.fonts},output};
  await fs.writeFile(path.join(artifacts,name+'.json'),JSON.stringify(metadata,null,2),{mode:0o600});
  return metadata;
}
async function selfTest(headed){
  const b=await managed();const context=await replay(b);
  let headedHeadlessByteEqual;
  try {
    const one=await storeCapture(context,fixture.origin+'/public','public-before');
    const two=await storeCapture(context,fixture.origin+'/public','public-after');
    assert.equal(one.sha256,two.sha256);assert.equal(one.stable,true);
    const replacement=await storeCapture(context,fixture.origin+'/public','replacement',{replace:true});
    assert(replacement.replacement.geometryMeasuredElements>4);assert(replacement.replacement.before.length>4);assert.equal(replacement.replacement.geometryPreserved,true);assert.equal(replacement.replacement.count,4,JSON.stringify(replacement.replacement.imageDiagnostics));
    const normal=await capture(context,fixture.origin+'/public');
    assert.notEqual(normal.metadata.sha256,replacement.sha256);
    await assert.rejects(capture(context,fixture.origin+'/private'),e=>e.code==='RequiresLogin');
    await assert.rejects(capture(context,fixture.origin+'/expired'),e=>e.code==='RequiresLogin');
    await assert.rejects(capture(context,fixture.origin+'/error'),e=>e.code==='HttpError');
    const login=await startBrowser(headed);const lc=await replay(login);const lp=await lc.newPage();
    try{
      await lp.goto(fixture.origin+'/login');await lp.click('#login');await lp.waitForFunction(()=>document.documentElement.dataset.auth==='ok');
      const saved=await saveSession(lc,lp);
      assert(saved.state.cookies.length);assert(saved.state.origins[0].localStorage.length);assert(saved.state.origins[0].indexedDB.length);assert.equal(saved.sessionStorage.values.fixture,'ok');
      const headedImage=await lp.screenshot({animations:'disabled'});
      const rc=await replay(b,saved);
      try{
        const auth=await storeCapture(rc,fixture.origin+'/private','authenticated');
        const rp=await rc.newPage();await rp.goto(fixture.origin+'/private');await rp.waitForFunction(()=>document.documentElement.dataset.auth==='ok');
        const headlessImage=await rp.screenshot({animations:'disabled'});
        headedHeadlessByteEqual=headedImage.equals(headlessImage);
        await fs.writeFile(path.join(artifacts,headed?'auth-headed.png':'auth-login-headless.png'),headedImage,{mode:0o600});
        await fs.writeFile(path.join(artifacts,'auth-replay-headless.png'),headlessImage,{mode:0o600});
        await rp.close();
        assert.equal(auth.stable,true);
      }finally{await rc.close();}
      // Each storage mechanism is independently necessary; negative replay tests.
      for(const missing of ['cookie','localStorage','indexedDB','sessionStorage']){
        const state=structuredClone(saved);
        if(missing==='cookie')state.state.cookies=[];
        if(missing==='localStorage')state.state.origins[0].localStorage=[];
        if(missing==='indexedDB')state.state.origins[0].indexedDB=[];
        if(missing==='sessionStorage')state.sessionStorage.values={};
        const negative=await replay(b,state);
        try{await assert.rejects(capture(negative,fixture.origin+'/private'),e=>e.code==='RequiresLogin');}finally{await negative.close();}
      }
    }finally{await login.close();}
    const tall=await storeCapture(context,fixture.origin+'/tall','tall');assert(tall.dimensions.height>20000);
    const env=one.environment;
    assert.equal(comparable(env,structuredClone(env)),'same-recorded-environment');
    assert.equal(comparable(env,{...env,os:'other'}),'caution-rendering-environment');
    assert.equal(comparable(env,{...env,browser:'other'}),'blocked-profile-mismatch');
    const outcome={passed:['public byte equality','screenshot stability','dimension-preserving img/picture/background','cookie + localStorage + IndexedDB + sessionStorage replay','four independent missing-state negatives','login expiry','HTTP failure','tall capture','comparability policy'],investigations:{headedHeadlessByteEqual,warning:headed&&!headedHeadlessByteEqual?'Headed/headless pixels differ. Login replay works; keep all evidence captures in pinned headless mode.':null},headed,environment:env,fixtureImageDimensions:tall.dimensions};
    await fs.writeFile(path.join(artifacts,headed?'self-test-headed.json':'self-test.json'),JSON.stringify(outcome,null,2),{mode:0o600});return outcome;
  }finally{await context.close();}
}
async function dispatch(r){
  if(!r||typeof r!=='object'||Array.isArray(r)||r.v!==1||!Number.isSafeInteger(r.id)||r.id<1||r.job!=='phase0'||typeof r.command!=='string'||!r.payload||typeof r.payload!=='object'||Array.isArray(r.payload)
    ||(r.command==='capture'&&(('target' in r.payload&&typeof r.payload.target!=='string')||('replace' in r.payload&&typeof r.payload.replace!=='boolean')))
    ||(r.command==='selfTest'&&'headed' in r.payload&&typeof r.payload.headed!=='boolean')) {emit(r,'failure',{},'InvalidRequest');return;}
  emit(r,'ack');
  if(r.command==='cancel'){await closeBrowsers();emit(r,'cancelled');emit(r,'result',{cancelled:true});return;}
  if(active){emit(r,'failure',{},'Busy');return;}
  active=true;
  try{
    if(r.command!=='handshake'&&!handshaken)throw Object.assign(new Error(),{code:'HandshakeRequired'});
    let result;
    switch(r.command){
      case 'handshake':handshaken=true;result={protocol:1,runtime:'node22.20.0-pw1.56.1-protocol1',fixtureOrigin:fixture.origin,sessionPersistence:'ephemeral',sandbox:true};break;
      case 'smoke':{const b=await managed();const c=await replay(b);try{const x=await capture(c,fixture.origin+'/public');result={browser:b.version(),bytes:x.image.length,sandbox:true};}finally{await c.close();}break;}
      case 'selfTest':result=await selfTest(r.payload.headed===true);break;
      case 'openLogin':await loginBrowser?.close();loginBrowser=await startBrowser(true);loginContext=await replay(loginBrowser);loginPage=await loginContext.newPage();await loginPage.goto(fixture.origin+'/login');result={instruction:'Sign in in the managed browser, then choose I’m ready.'};break;
      case 'finishLogin':if(!loginPage)throw Object.assign(new Error(),{code:'OpenLoginFirst'});await loginPage.waitForFunction(()=>document.documentElement.dataset.auth==='ok',null,{timeout:5000});session=await saveSession(loginContext,loginPage);await loginBrowser.close();loginBrowser=loginContext=loginPage=undefined;result={session:'ephemeral-ready'};break;
      case 'capture':{
        const target=r.payload.target||'/public';if(!['/public','/private','/tall'].includes(target))throw Object.assign(new Error(),{code:'FixtureScopeOnly'});
        const c=await replay(await managed(),session);try{emit(r,'progress',{stage:'Capturing fixture'});result=await storeCapture(c,fixture.origin+target,'ui-capture',{replace:r.payload.replace===true});}finally{await c.close();}break;
      }
      case 'crashBrowser':{const b=await managed();const cdp=await b.newBrowserCDPSession();await cdp.send('Browser.crash').catch(()=>{});await new Promise(resolve=>setTimeout(resolve,250));assert.equal(b.isConnected(),false);browser=undefined;const c=await replay(await managed());try{await capture(c,fixture.origin+'/public');result={recovered:true};}finally{await c.close();}break;}
      case 'hang':{const c=await replay(await managed());const p=await c.newPage();await p.goto(fixture.origin+'/slow',{timeout:0});result={};break;}
      case 'crashWorker':process.exit(86);break;
      case 'oversized':process.stdout.write('x'.repeat(MAX_LINE+1)+'\n');result={};break;
      case 'shutdown':await cleanup();result={stopped:true};break;
      default:throw Object.assign(new Error(),{code:'UnknownCommand'});
    }
    emit(r,'result',result);
    if(r.command==='shutdown')process.exit(0);
  }catch(error){
    // Never echo page URLs, request payloads, storage state, browser stderr or stack.
    const known=['RequiresLogin','HttpError','UnexpectedRedirect','ImageBudgetExceeded','PageTooTallOrInfinite','StabilizationTimeout','OpenLoginFirst','HandshakeRequired','UnknownCommand','FixtureScopeOnly','ShuttingDown'];
    const code=known.includes(error.code)?error.code:(error.name==='TimeoutError'?'Timeout':'OperationFailed');
    if(code==='RequiresLogin')emit(r,'requires-login');
    const diagnostic = r.command==='selfTest' && error.code==='ERR_ASSERTION' ? {fixtureAssertion:error.message.slice(0,256)} : {};
    process.stderr.write(`regressee-worker: ${code}\n`);emit(r,'failure',diagnostic,code);
  }finally{active=false;}
}
let pending=Buffer.alloc(0), poisoned=false;
function framingFailure(code){if(poisoned)return;poisoned=true;emit(null,'failure',{},code);cleanup().finally(()=>process.exit(64));}
process.stdin.on('data',chunk=>{
  if(poisoned)return;
  pending=Buffer.concat([pending,chunk]);
  for(;;){const n=pending.indexOf(10);if(n<0)break;if(n>MAX_LINE){framingFailure('FrameTooLarge');return;}
    const line=pending.subarray(0,n);pending=pending.subarray(n+1);
    let request;
    try{request=JSON.parse(new TextDecoder('utf-8',{fatal:true}).decode(line));}catch{framingFailure('MalformedJson');return;}
    void dispatch(request).catch(()=>{emit(request,'failure',{},'OperationFailed');poisoned=true;cleanup().finally(()=>process.exit(70));});}
  if(pending.length>MAX_LINE){framingFailure('FrameTooLarge');}
});
process.stdin.on('end',()=>{if(poisoned)return;if(pending.length){framingFailure('IncompleteFrame');return;}cleanup().finally(()=>process.exit(0));});
