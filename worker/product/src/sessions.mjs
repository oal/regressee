// SPDX-License-Identifier: MPL-2.0
// Private bounded in-memory state; no storage-state files, diagnostics or exports.
import {fail,literalLoopback,navigationBoundary} from './capture.mjs';
const MAX_STATE=512*1024;
const reference=s=>JSON.stringify([s.project,s.environment,s.id]);
export class Sessions {
  #states=new Map();
  install({session,origins,material}) {
    if(typeof material!=='string'||Buffer.byteLength(material)>MAX_STATE||!origins.length||origins.length>16)throw fail('InvalidRequest');
    let state;try {state=JSON.parse(material);}catch{throw fail('InvalidRequest');}
    const approved=new Set(origins.map(o=>{const u=new URL(o);if(u.href!==u.origin+'/')throw fail('InvalidRequest');return u.origin;}));
    // Bound material must agree with install authority. Cookie hosts alone cannot
    // bind a port/scheme. Never upgrade an old cookie-only payload here.
    if(!state?.binding||state.binding.version!==1||!Array.isArray(state.binding.origins)||JSON.stringify(state.binding.origins)!==JSON.stringify([...approved].sort().map(o=>o+'/'))||Object.keys(state.binding).sort().join(',')!=='origins,version')throw fail('NeedsLogin');
    const previous=this.#states.get(reference(session));
    if(previous&&JSON.stringify([...previous.approved].sort())!==JSON.stringify([...approved].sort()))throw fail('NeedsLogin');
    if(Object.keys(state).sort().join(',')!=='binding,session_storage,storage'||!Array.isArray(state.storage?.cookies)||!Array.isArray(state.storage?.origins)||!state.session_storage||typeof state.session_storage!=='object')throw fail('InvalidRequest');
    const hosts=[...approved].map(o=>new URL(o).hostname);
    for(const c of state.storage.cookies){if(typeof c.domain!=='string'||!hosts.includes(c.domain.replace(/^\./,'')))throw fail('InvalidRequest');}
    for(const o of state.storage.origins){if(!approved.has(o.origin))throw fail('InvalidRequest');}
    for(const [o,entries] of Object.entries(state.session_storage)) {
      if(!approved.has(o)||!Array.isArray(entries)||entries.length>512||entries.some(e=>!Array.isArray(e)||e.length!==2||e.some(v=>typeof v!=='string')))throw fail('InvalidRequest');
    }
    if(this.#states.size>=16&&!this.#states.has(reference(session)))throw fail('PayloadLimit');
    this.#states.set(reference(session),{state,approved});
  }
  get(session,address) {
    if(session===null)return null;
    const saved=this.#states.get(reference(session));
    if(!saved||!saved.approved.has(new URL(address).origin))throw fail('InvalidRequest');
    return saved.state;
  }
  clear(){this.#states.clear();}
}
export async function replayContext(browser,options,state) {
  const context=await browser.newContext({...options,...(state?{storageState:state.storage}:{})});
  if(state)await context.addInitScript(values=>{
    const entries=values[location.origin];if(entries)for(const [key,value] of entries)sessionStorage.setItem(key,value);
  },state.session_storage);
  return context;
}
export async function needsLogin(page) {
  return page.evaluate(()=>!!document.querySelector('input[type="password"], [data-regressee-needs-login]'));
}
export async function openLogin(browser,payload,{scope,onContext}) {
  const {address,origins,identity_origins}=payload;
  const allowed=new Set([...origins,...identity_origins].map(o=>new URL(o).origin));
  if(!allowed.has(new URL(address).origin)||!origins.length)throw fail('InvalidRequest');
  for(const o of allowed)if(scope==='loopback'&&!literalLoopback(o))throw fail('RedirectOutsideScope');
  const context=await browser.newContext({serviceWorkers:'block',acceptDownloads:false});onContext(context);
  try {
    await context.route('**/*',route=>{
      const r=route.request(),u=new URL(r.url());
      if((scope==='loopback'&&['http:','https:'].includes(u.protocol)&&!literalLoopback(u.href))||(r.isNavigationRequest()&&!allowed.has(u.origin)))return route.abort('blockedbyclient');
      return route.continue();
    });
    if(scope==='loopback')await context.routeWebSocket('**/*',s=>s.close());
    const guards=new WeakMap();let closed=false;context.on('close',()=>{closed=true;});
    const guard=p=>{
      if(guards.has(p))return guards.get(p);
      const task=(async()=>{
        const cdp=await context.newCDPSession(p);
        cdp.on('Fetch.requestPaused',event=>{void(async()=>{
          const {requestId,request,responseStatusCode,responseHeaders=[]}=event;
          let u=new URL(request.url);
          if(responseStatusCode!==undefined||event.responseErrorReason){
            const location=responseHeaders.find(h=>h.name.toLowerCase()==='location')?.value;
            if(location&&[301,302,303,307,308].includes(responseStatusCode))u=new URL(location,u);
            if((scope==='loopback'&&!literalLoopback(u.href))||(event.resourceType==='Document'&&!allowed.has(u.origin)))return cdp.send('Fetch.failRequest',{requestId,errorReason:'BlockedByClient'});
            return cdp.send('Fetch.continueResponse',{requestId});
          }
          if((scope==='loopback'&&['http:','https:'].includes(u.protocol)&&!literalLoopback(u.href))||(event.resourceType==='Document'&&!allowed.has(u.origin)))return cdp.send('Fetch.failRequest',{requestId,errorReason:'BlockedByClient'});
          return cdp.send('Fetch.continueRequest',{requestId});
        })().catch(()=>{if(!closed)void context.close();});});
        await cdp.send('Fetch.enable',{patterns:[{urlPattern:'*',requestStage:'Request'},{urlPattern:'*',resourceType:'Document',requestStage:'Response'}]});
      })();guards.set(p,task);return task;
    };
    context.on('page',p=>{if(context.pages().length>8)void p.close();else void guard(p).catch(()=>context.close());});
    const page=await context.newPage();await guard(page);await page.goto(address,{waitUntil:'domcontentloaded',timeout:30000});
    return {context,payload,browser};
  }catch(e){await context.close();throw e;}
}
export async function confirmLogin(login,{scope,sessions,onContext}) {
  const {context,payload}=login;
  const origins=new Set(payload.origins.map(o=>new URL(o).origin));
  let storage=await context.storageState({indexedDB:true});
  // Identity-provider state is never replayed into capture/crawl contexts.
  const hosts=[...origins].map(o=>new URL(o).hostname);
  storage={cookies:storage.cookies.filter(c=>hosts.includes(c.domain.replace(/^\./,''))),origins:storage.origins.filter(o=>origins.has(o.origin))};
  const session_storage={};
  for(const page of context.pages())if(origins.has(new URL(page.url()).origin)) {
    const origin=new URL(page.url()).origin;
    const entries=await page.evaluate(()=>Object.entries(sessionStorage));
    if(session_storage[origin]&&JSON.stringify(session_storage[origin])!==JSON.stringify(entries))throw fail('InvalidRequest');
    session_storage[origin]=entries;
  }
  const binding={version:1,origins:[...origins].sort().map(o=>o+'/')};
  const material=JSON.stringify({binding,storage,session_storage});
  sessions.install({session:payload.session,origins:payload.origins,material});
  // Independent fresh context: the headed page alone is not replay qualification.
  const fresh=await replayContext(login.browser,{serviceWorkers:'block',acceptDownloads:false},sessions.get(payload.session,payload.representative));onContext(fresh);
  try {
    const page=await fresh.newPage();const guard=await navigationBoundary(fresh,payload.representative,scope,30000,null,page);
    const r=await page.goto(payload.representative,{waitUntil:'domcontentloaded',timeout:30000}).catch(()=>null);
    if(!r||guard.redirect||guard.blocked||[401,403].includes(r.status())||r.status()>=400||await needsLogin(page))throw fail('NeedsLogin');
    await page.waitForTimeout(150);
    if(guard.redirect||guard.blocked||page.url()!==payload.representative||await needsLogin(page))throw fail('NeedsLogin');
    return material;
  }finally{await fresh.close();onContext(context);}
}
