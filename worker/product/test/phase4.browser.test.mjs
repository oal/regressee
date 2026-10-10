// SPDX-License-Identifier: MPL-2.0
// Run only from a freshly signed installed product pack. Fake local accounts.
import test from 'node:test';
import assert from 'node:assert/strict';
import http from 'node:http';
import fs from 'node:fs/promises';
import path from 'node:path';
import crypto from 'node:crypto';
import {launch,capture,PLAYWRIGHT_VERSION,WORKER_VERSION} from '../src/capture.mjs';
import {Sessions,openLogin,confirmLogin,replayContext} from '../src/sessions.mjs';
import {discover} from '../src/discover.mjs';
const reference=()=>({id:crypto.randomUUID(),project:crypto.randomUUID(),environment:crypto.randomUUID()});
const fixtureState=`async function writeState(){localStorage.setItem('fixture-local','ready');sessionStorage.setItem('fixture-tab','ready');await new Promise((resolve,reject)=>{const r=indexedDB.open('fixture-auth',1);r.onupgradeneeded=()=>r.result.createObjectStore('auth');r.onerror=reject;r.onsuccess=()=>{const db=r.result;const t=db.transaction('auth','readwrite');t.objectStore('auth').put('ready','token');t.oncomplete=()=>{db.close();resolve();};};});}`;
const fixtureCheck=`async function checkState(){const db=await new Promise(resolve=>{const r=indexedDB.open('fixture-auth',1);r.onupgradeneeded=()=>r.result.createObjectStore('auth');r.onsuccess=()=>resolve(r.result);});const token=await new Promise(resolve=>{const r=db.transaction('auth').objectStore('auth').get('token');r.onsuccess=()=>resolve(r.result);});db.close();if(localStorage.getItem('fixture-local')!=='ready'||localStorage.getItem('fixture-consent')!=='yes'||sessionStorage.getItem('fixture-tab')!=='ready'||token!=='ready'){document.body.innerHTML='<input type=password data-regressee-needs-login>';return;}document.body.dataset.ready='yes';}void checkState();`;
async function site(prefix) {
  const hits=new Map();let expired=false;
  const png=Buffer.from('iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mP8/x8AAwMCAO+aZ1cAAAAASUVORK5CYII=','base64');
  const server=http.createServer(async(req,res)=>{
    hits.set(req.url,(hits.get(req.url)||0)+1);
    const u=new URL(req.url,'http://fixture'),p=u.pathname;
    const html=body=>{res.setHeader('Content-Type','text/html');res.end(`<!doctype html><html><head><meta charset=utf-8><style>body{margin:0;font:20px sans-serif;background:white}h1{padding:24px}</style></head><body>${body}</body></html>`);};
    if(p==='/robots.txt'){res.end('User-agent: *\nDisallow: /logout');return;}
    if(p==='/login'){html(`<h1>Fixture login</h1><button id=signin>Sign in with read-only fixture account</button><button id=consent>Allow fixture consent</button><script>${fixtureState}document.querySelector('#consent').onclick=()=>localStorage.setItem('fixture-consent','yes');document.querySelector('#signin').onclick=async()=>{await fetch('/fixture-login',{method:'POST'});await writeState();location.href='${prefix}/home';};</script>`);return;}
    if(p==='/fixture-login'){res.setHeader('Set-Cookie','fixture-session=ready; HttpOnly; SameSite=Lax; Path=/');res.end('ok');return;}
    if(p.startsWith(prefix+'/')&&!p.includes('public')){
      if(expired){res.writeHead(302,{Location:'/login'});res.end();return;}
      if(!req.headers.cookie?.includes('fixture-session=ready')){res.statusCode=401;res.end('access denied');return;}
      html(`<h1>Read-only application fixture</h1><a href='${prefix}/details?b=2&a=1&a=0#/orders'>Details</a><script>${fixtureCheck}</script>`);return;
    }
    if(p==='/delayed-font'){await new Promise(r=>setTimeout(r,500));res.setHeader('Content-Type','font/ttf');res.end(await fs.readFile('/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf'));return;}
    if(p==='/delayed-image'){await new Promise(r=>setTimeout(r,450));res.setHeader('Content-Type','image/png');res.end(png);return;}
    if(p==='/public-delayed'){html(`<style>@font-face{font-family:Fixture;src:url('/delayed-font')}h1{font-family:Fixture}body:hover{outline:20px solid red}</style><h1>Delayed resources</h1><img src='/delayed-image' width=20 height=20><p id=profile></p><script>document.querySelector('#profile').textContent=[navigator.language,Intl.DateTimeFormat().resolvedOptions().timeZone,matchMedia('(prefers-color-scheme: dark)').matches,matchMedia('(prefers-reduced-motion: reduce)').matches].join(' / ')</script>`);return;}
    if(p==='/public-unstable'){html(`<h1 id=value>0</h1><script>let value=0;setInterval(()=>document.querySelector('#value').textContent=++value,10)</script>`);return;}
    if(p==='/public-incomplete'){html(`<style>@font-face{font-family:Missing;src:url('/missing-font')}h1{font-family:Missing}</style><h1>Incomplete resources</h1><img src='/missing-image'>`);return;}
    if(p==='/public-rate-limit'){res.writeHead(429);res.end('rate limited');return;}
    if(p==='/unexpected-login'){res.writeHead(302,{Location:u.searchParams.get('destination')});res.end();return;}
    if(p==='/public-tall'){html('<div style="height:40000px">Tall limit fixture</div>');return;}
    if(p==='/public-infinite'){html(`<div style="height:1000px"></div><script>addEventListener('scroll',()=>{if(scrollY+innerHeight>=document.documentElement.scrollHeight-150){const d=document.createElement('div');d.style.height='1000px';document.body.append(d)}})</script>`);return;}
    if(p==='/public-login-form'){html('<h1>Please sign in</h1><input type=password>');return;}
    res.statusCode=404;res.end('missing');
  });
  await new Promise(resolve=>server.listen(0,'127.0.0.1',resolve));
  const origin=`http://127.0.0.1:${server.address().port}`;
  return {origin,hits,expire:()=>{expired=true;},close:()=>new Promise(resolve=>server.close(resolve))};
}
function profile(browser){return {version:1,id:crypto.randomUUID(),name:'Fixture',width:600,height:300,dpr:1,mobile:false,touch:false,user_agent:'Regressee read-only fixture',extent:'VisibleScreen',image_mode:'Normal',locale:'nb-NO',timezone:'Europe/Oslo',dark:true,reduced_motion:true,stabilization:{version:1,navigation_ms:10000,readiness_ms:7000,max_height:32768},runtime:{Pinned:{node:process.version.slice(1),worker:WORKER_VERSION,playwright:PLAYWRIGHT_VERSION,browser:browser.version()}}};}
test('signed product Phase 4 auth, independent negative storage and readiness fixtures',async t=>{
  const output=process.env.REGRESSEE_PHASE4_WORKER_OUTPUT;assert.ok(output,'fresh owned evidence output required');await fs.mkdir(output,{recursive:false});
  const prod=await site('/scope'),stage=await site('/preview');const browser=await launch();const sessions=new Sessions();
  const p=profile(browser),key={page:crypto.randomUUID(),profile:p.id};
  const results=[];
  const take=async(address,session=null,changed={})=>{
    const warnings=[];
    const result=await capture(browser,{profile:{...p,...changed},address,key,session},{identity:p.runtime,scope:'loopback',output,onContext:()=>{},progress:()=>{},warning:w=>warnings.push(w),sessions});
    const meta=JSON.parse(await fs.readFile(path.join(output,result.output_token+'.json'),'utf8'));
    assert.ok(!Object.hasOwn(meta,'session')&&!JSON.stringify(meta).includes('fixture-local'));
    return {meta,warnings};
  };
  const login=async(s,scope,prefix)=>{
    // Explicit test boundary: production OpenLogin launches headed. This fixture
    // uses the exact signed login controller with an owned headless browser.
    const setup=await openLogin(browser,{session:s,address:scope.origin+'/login',representative:scope.origin+prefix+'/home',origins:[scope.origin+'/'],identity_origins:[]},{scope:'loopback',onContext:()=>{}});
    const page=setup.context.pages()[0];
    await assert.rejects(()=>confirmLogin(setup,{scope:'loopback',sessions,onContext:()=>{}}),{code:'NeedsLogin'});
    await page.locator('#consent').click();await page.locator('#signin').click();await page.waitForURL(scope.origin+prefix+'/home');await page.waitForSelector('body[data-ready=yes]');
    const material=await confirmLogin(setup,{scope:'loopback',sessions,onContext:()=>{}});await setup.context.close();return material;
  };
  try {
    const s=reference();let material;
    await t.test('login plus consent confirms fresh-context cookies/localStorage/IndexedDB/sessionStorage replay',async()=>{material=await login(s,prod,'/scope');const {meta,warnings}=await take(prod.origin+'/scope/home',s);assert.ok(meta.stable);assert.equal(warnings.length,0);results.push('all-storage replay and representative confirmation');});
    for(const kind of ['cookies','localStorage','indexedDB','sessionStorage'])await t.test(`independent removal of ${kind} requires login rather than screenshot`,async()=>{
      const state=JSON.parse(material);if(kind==='cookies')state.storage.cookies=[];if(kind==='localStorage')for(const o of state.storage.origins)o.localStorage=[];if(kind==='indexedDB')for(const o of state.storage.origins)o.indexedDB=[];if(kind==='sessionStorage')state.session_storage={};
      const other=reference();sessions.install({session:other,origins:[prod.origin+'/'],material:JSON.stringify(state)});
      const before=(await fs.readdir(output)).length;await assert.rejects(()=>take(prod.origin+'/scope/home',other),{code:'NeedsLogin'});assert.equal((await fs.readdir(output)).length,before);results.push(`independent ${kind} negative`);
    });
    await t.test('bad reference and staging replay scope fail before visits; separate staging session succeeds',async()=>{
      const count=stage.hits.get('/preview/home')||0;
      await assert.rejects(()=>take(stage.origin+'/preview/home',s),{code:'InvalidRequest'});await assert.rejects(()=>take(prod.origin+'/scope/home',reference()),{code:'InvalidRequest'});assert.equal(stage.hits.get('/preview/home')||0,count);
      const staging=reference();await login(staging,stage,'/preview');const {meta}=await take(stage.origin+'/preview/details?b=2&a=1&a=0#/orders',staging);assert.equal(meta.final_url,stage.origin+'/preview/details?b=2&a=1&a=0#/orders');results.push('separate staging scope, prefix, ordered queries and hash');
    });
    await t.test('cookie-only origin binding refuses reinstall, legacy, scheme and port drift before any visit',async()=>{
      const state=JSON.parse(material);state.storage.origins=[];state.session_storage={};
      const cookieOnly=JSON.stringify(state),fresh=reference();sessions.install({session:fresh,origins:[prod.origin+'/'],material:cookieOnly});
      const before=[...stage.hits];
      for(const origin of [stage.origin+'/',prod.origin.replace('http:','https:')+'/']) {
        assert.throws(()=>sessions.install({session:fresh,origins:[origin],material:cookieOnly}),{code:'NeedsLogin'});
        assert.throws(()=>sessions.install({session:reference(),origins:[origin],material:cookieOnly}),{code:'NeedsLogin'});
      }
      delete state.binding;assert.throws(()=>sessions.install({session:reference(),origins:[prod.origin+'/'],material:JSON.stringify(state)}),{code:'NeedsLogin'});
      await assert.rejects(()=>take(stage.origin+'/preview/home',fresh),{code:'InvalidRequest'});
      await assert.rejects(()=>discover(browser,{address:stage.origin+'/preview/home',section:'/preview',session:fresh},{scope:'loopback',onContext:()=>{},sessions}),{code:'InvalidRequest'});
      assert.deepEqual([...stage.hits],before);results.push('cookie-only immutable origins: legacy/reinstall/port/scheme rejected; zero destination visits');
    });
    await t.test('origin-qualified sessionStorage is not replayed on another origin',async()=>{
      const ctx=await replayContext(browser,{serviceWorkers:'block'},JSON.parse(material));try{const page=await ctx.newPage();await page.goto(stage.origin+'/public-login-form');assert.equal(await page.evaluate(()=>sessionStorage.getItem('fixture-tab')),null);}finally{await ctx.close();}results.push('sessionStorage origin negative');
    });
    await t.test('authenticated discovery uses replay and never adds login redirect to crawl',async()=>{
      const result=await discover(browser,{address:prod.origin+'/scope/home',section:'/scope',session:s},{scope:'loopback',onContext:()=>{},sessions});assert.ok(result.addresses.includes(prod.origin+'/scope/details?b=2&a=1&a=0#/orders'));
      prod.expire();const before=prod.hits.get('/login')||0;await assert.rejects(()=>take(prod.origin+'/scope/home',s),{code:'NeedsLogin'});await assert.rejects(()=>discover(browser,{address:prod.origin+'/scope/home',section:'/scope',session:s},{scope:'loopback',onContext:()=>{},sessions}),{code:'NeedsLogin'});assert.equal(prod.hits.get('/login')||0,before);results.push('expiry redirect produces NeedsLogin, zero login destination visits');
    });
    await t.test('profile-consistent delayed fonts/images settle within readiness budget',async()=>{
      const {meta,warnings}=await take(prod.origin+'/public-delayed');assert.equal(warnings.length,0);assert.ok(meta.stable&&meta.duration_ms>=450);assert.equal(meta.profile.locale,'nb-NO');assert.equal(meta.profile.timezone,'Europe/Oslo');results.push('delayed fonts/images positive, fixed profile');
    });
    await t.test('missing fonts/images emit unique visible warnings; rate limits retain no screenshot',async()=>{
      const {meta,warnings}=await take(prod.origin+'/public-incomplete');
      for(const code of ['FontsIncomplete','ImagesIncomplete','ResourceFailed'])assert.ok(warnings.includes(code));
      assert.equal(new Set(warnings).size,warnings.length);assert.deepEqual(meta.warnings,warnings);
      const before=(await fs.readdir(output)).length;await assert.rejects(()=>take(prod.origin+'/public-rate-limit'),{code:'RateLimited'});assert.equal((await fs.readdir(output)).length,before);results.push('incomplete fonts/images warning and 429 no-evidence failure');
    });
    await t.test('interactive login refuses an unapproved identity-provider redirect before visiting it',async()=>{
      const before=stage.hits.get('/login')||0;
      await assert.rejects(()=>openLogin(browser,{session:reference(),address:prod.origin+'/unexpected-login?destination='+encodeURIComponent(stage.origin+'/login'),representative:prod.origin+'/scope/home',origins:[prod.origin+'/'],identity_origins:[]},{scope:'loopback',onContext:()=>{}}));
      assert.equal(stage.hits.get('/login')||0,before);results.push('unapproved interactive IdP redirect: zero destination visits');
    });
    await t.test('unstable counters warn; tall/infinite pages fail without cropped full-page evidence',async()=>{
      const {warnings}=await take(prod.origin+'/public-unstable');assert.ok(warnings.includes('Unstable'));
      for(const route of ['/public-tall','/public-infinite']){const before=(await fs.readdir(output)).length;await assert.rejects(()=>take(prod.origin+route,null,{extent:'FullPage'}),{code:'ImageLimit'});assert.equal((await fs.readdir(output)).length,before);}results.push('instability warning and tall/infinite no-crop failures');
    });
    await t.test('even a public login form cannot become baseline evidence',async()=>{await assert.rejects(()=>take(prod.origin+'/public-login-form'),{code:'NeedsLogin'});results.push('public login-form negative');});
    await fs.writeFile(path.join(output,'report.json'),JSON.stringify({cases:results,scope:'owned literal-loopback signed browser fixtures',login_control:'same signed module with explicit headless test boundary; not native/human login qualification',sites:[prod.origin,stage.origin],state_persisted:false,outputs:(await fs.readdir(output)).filter(f=>f.endsWith('.png')).length},null,2));
  }finally{sessions.clear();await browser.close();await Promise.all([prod.close(),stage.close()]);}
});
