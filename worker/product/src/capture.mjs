// SPDX-License-Identifier: MPL-2.0
import { chromium } from 'playwright';
import crypto from 'node:crypto';
import fs from 'node:fs/promises';
import path from 'node:path';
import {createRequire} from 'node:module';
import {replayContext,needsLogin} from './sessions.mjs';
export const PLAYWRIGHT_VERSION=createRequire(import.meta.url)('playwright/package.json').version;
export const WORKER_VERSION='5.0.1';
export const IMAGE_ALGORITHM='neutral-raster-v2';
// Same material transform as the qualified Phase 0 implementation. Changing
// placeholders/source/layout rules requires a NEW algorithm identity.
export async function replaceImages(page) {
  return page.evaluate(async()=>{
    const elements=[...document.querySelectorAll('*')];
    const measured=elements.length<=2048?elements:[];
    const rectangles=()=>measured.map((e,i)=>[i,...['x','y','width','height'].map(k=>e.getBoundingClientRect()[k])]);
    const before=rectangles();let count=0;const skipped=[];
    const imageDiagnostics=[...document.images].map(i=>({id:i.id,width:i.naturalWidth,height:i.naturalHeight,complete:i.complete}));
    const neutral=(w,h)=>`data:image/svg+xml,${encodeURIComponent(`<svg xmlns="http://www.w3.org/2000/svg" width="${w}" height="${h}" viewBox="0 0 ${w} ${h}" preserveAspectRatio="none"><path fill="#cbd5e1" d="M0 0h${w}v${h}H0z"/></svg>`)}`;
    for(const img of [...document.images]){
      const source=img.currentSrc||img.src;
      if(/\.svg(?:[?#]|$)|^data:image\/svg/i.test(source))continue;
      if(!img.naturalWidth||!img.naturalHeight){skipped.push({index:[...document.images].indexOf(img),reason:'ImageNotDecoded'});continue;}
      const replacement=neutral(img.naturalWidth,img.naturalHeight);
      if(img.parentElement?.tagName==='PICTURE')for(const source of img.parentElement.querySelectorAll('source'))source.removeAttribute('srcset');
      img.removeAttribute('srcset');img.src=replacement;await img.decode();count++;
    }
    for(const el of document.querySelectorAll('*')){
      const bg=getComputedStyle(el).backgroundImage;
      if(/^url\(["']?.*\.(png|jpe?g|webp|gif)([?#][^)]*)?["']?\)$/i.test(bg)){el.style.backgroundImage=`url("${neutral(16,16)}")`;count++;}
    }
    const after=rectangles();
    return {count,before,after,imageDiagnostics,skipped,geometryMeasuredElements:measured.length,geometryScope:measured.length?'light-DOM element rectangles':'NOT MEASURED: element budget exceeded',geometryPreserved:measured.length?JSON.stringify(before)===JSON.stringify(after):null,limitations:['canvas','video','frames','shadow DOM','pseudo-elements','complex backgrounds','extensionless raster URLs','source-dependent CSS or script (src/srcset/picture attributes change)','geometry excludes shadow/frame contents and visual styling']};
  });
}
const replacementMode=p=>p.image_mode==='Normal'||(p.image_mode&&Object.keys(p.image_mode).join(',')==='ReplaceRaster'&&p.image_mode.ReplaceRaster&&Object.keys(p.image_mode.ReplaceRaster).join(',')==='algorithm'&&p.image_mode.ReplaceRaster.algorithm===IMAGE_ALGORITHM);
export const fail = code => Object.assign(new Error(code), {code});
export const literalLoopback = address => {
  const u = new URL(address);
  return u.hostname === '[::1]' || /^127\.(?:\d{1,3}\.){2}\d{1,3}$/.test(u.hostname);
};
export const launch = (headless=true) => chromium.launch({channel:'chromium',headless,chromiumSandbox:true,...(process.env.REGRESSEE_PROXY?{proxy:{server:process.env.REGRESSEE_PROXY}}:{})});
// Keep the document on Chromium's real network path (including its address-space
// classification). Fetch/fulfill is NOT transparent for local-network resources.
// Playwright guards initial documents across all targets; CDP checks each resource
// redirect hop and stops document redirects at response headers, before following.
// Discovery returns the unvisited candidate to Rust; capture admits its frozen URL.
export async function navigationBoundary(context,address,scope,timeout,section=null,page) {
  if(!page)throw fail('InvalidRequest');
   const requested=new URL(address),prefix=section?.replace(/\/$/,'')||'';
   const state={redirect:null,blocked:false,embeddedBlocked:false,error:null};let started=false,closed=false;
   context.on('close',()=>{closed=true;});
   context.on('page',other=>{if(other!==page){state.blocked=true;void other.close().catch(()=>{});}});
   await context.route('**/*',async route=>{
     const req=route.request(),u=new URL(req.url());
     let frame;
     if(req.isNavigationRequest()) {
       // A popup's initial request can arrive before Playwright has a frame.
       // It is not an approved main-page request; fail closed before visiting.
       try {frame=req.frame();}catch {state.blocked=true;return route.abort('blockedbyclient');}
     }
     // Embedded documents are not part of the approved page inventory. Block
     // them before visiting, but do not misreport an iframe as a main-page
     // redirect (or invalidate an otherwise valid main-page screenshot).
     if(frame&&frame.page()===page&&frame!==page.mainFrame()) {
       state.embeddedBlocked=true;return route.abort('blockedbyclient');
     }
    if(scope==='loopback'&&['http:','https:'].includes(u.protocol)&&!literalLoopback(u.href)) {
      if(req.isNavigationRequest())state.blocked=true;
      return route.abort('blockedbyclient');
    }
    if(!req.isNavigationRequest())return route.continue();
    // Automatic forms, frames, popups and script/meta navigations are not crawl
    // evidence. Only the one explicit GET is admitted, before any network visit.
     if(started||req.method()!=='GET'||frame.page()!==page||frame!==page.mainFrame()||u.origin!==requested.origin||u.pathname!==requested.pathname||u.search!==requested.search||(prefix&&u.pathname!==prefix&&!u.pathname.startsWith(prefix+'/'))) {
      state.blocked=true;return route.abort('blockedbyclient');
    }
    started=true;
     return route.continue(); // The CDP response guard is already installed.
   });
    const cdp=await context.newCDPSession(page);
    const mainFrameId=(await cdp.send('Page.getFrameTree')).frameTree.frame.id;
    cdp.on('Fetch.requestPaused',event=>{void (async()=>{
      const {requestId,request,responseStatusCode,responseHeaders=[]}=event;
      if(event.resourceType==='Document'&&event.frameId!==mainFrameId) {
        state.embeddedBlocked=true;
        return cdp.send('Fetch.failRequest',{requestId,errorReason:'BlockedByClient'});
      }
     if(responseStatusCode!==undefined||event.responseErrorReason) {
       const header=name=>responseHeaders.find(h=>h.name.toLowerCase()===name)?.value;
       const location=header('location');
       if([301,302,303,307,308].includes(responseStatusCode)&&location) {
         try {state.redirect=new URL(location,request.url).href;}catch {state.blocked=true;}
         return cdp.send('Fetch.failRequest',{requestId,errorReason:'BlockedByClient'});
       }
       if(header('refresh')) {
         state.blocked=true;return cdp.send('Fetch.failRequest',{requestId,errorReason:'BlockedByClient'});
       }
       return cdp.send('Fetch.continueResponse',{requestId});
     }
     const u=new URL(request.url);
     if(scope==='loopback'&&['http:','https:'].includes(u.protocol)&&!literalLoopback(u.href)) {
       if(event.resourceType==='Document')state.blocked=true;
       return cdp.send('Fetch.failRequest',{requestId,errorReason:'BlockedByClient'});
     }
     return cdp.send('Fetch.continueRequest',{requestId});
   })().catch(async e=>{
     // Cancellation destroys the session and releases paused requests. Any other
     // interception error is fatal, not a silently continued/ignored request.
     if(!closed){state.error=e;await context.close().catch(()=>{});}
   });});
   await cdp.send('Fetch.enable',{patterns:[{urlPattern:'*',requestStage:'Request'},{urlPattern:'*',resourceType:'Document',requestStage:'Response'}]});
  if(scope==='loopback')await context.routeWebSocket('**/*',socket=>socket.close());
  return state;
}
const bounded = async (promise, ms) => {
  let timer;
  try { return await Promise.race([promise, new Promise((_,reject)=>{timer=setTimeout(()=>reject(fail('Timeout')), Math.max(1,ms));})]); }
  finally {clearTimeout(timer);}
};
export function validateProfile(p, identity) {
  const keys = ['version','id','name','width','height','dpr','mobile','touch','user_agent','extent','image_mode','locale','timezone','dark','reduced_motion','stabilization','runtime'];
    if (!p || Object.keys(p).length !== keys.length || !keys.every(k=>Object.hasOwn(p,k)) || p.version!==1 || !Number.isInteger(p.width) || p.width<1 || p.width>4096 || !Number.isInteger(p.height) || p.height<1 || p.height>8192 || p.dpr!==1 || typeof p.mobile!=='boolean' || typeof p.touch!=='boolean' || !['FullPage','VisibleScreen'].includes(p.extent) || !replacementMode(p) || typeof p.user_agent!=='string' || !p.user_agent.length || p.user_agent.length>1024 || typeof p.locale!=='string' || p.locale.length>80 || typeof p.timezone!=='string' || p.timezone.length>120 || typeof p.dark!=='boolean' || p.reduced_motion!==true || JSON.stringify(p.runtime)!==JSON.stringify(identity)) throw fail('InvalidRequest');
  const s=p.stabilization;
  if (!s || Object.keys(s).sort().join(',')!=='max_height,navigation_ms,readiness_ms,version' || s.version!==1 || !Number.isInteger(s.navigation_ms) || s.navigation_ms<1 || s.navigation_ms>120000 || !Number.isInteger(s.readiness_ms) || s.readiness_ms<1 || s.readiness_ms>120000 || !Number.isInteger(s.max_height) || s.max_height<1 || s.max_height>100000) throw fail('InvalidRequest');
}
export async function capture(browser, payload, {identity,scope,output,onContext,progress,warning,sessions,diagnostics=()=>{}}) {
  const {profile:p,address,key,session}=payload;
  validateProfile(p,identity);
   const state=session===null?null:sessions?.get(session,address);
   if(session!==null&&!state)throw fail('InvalidRequest');
  if (scope==='loopback' && !literalLoopback(address)) throw fail('RedirectOutsideScope');
  const started=Date.now();
   const context=await replayContext(browser,{viewport:{width:p.width,height:p.height},screen:{width:p.width,height:p.height},deviceScaleFactor:1,isMobile:p.mobile,hasTouch:p.touch,userAgent:p.user_agent,locale:p.locale,timezoneId:p.timezone,colorScheme:p.dark?'dark':'light',reducedMotion:'reduce',serviceWorkers:'block',acceptDownloads:false},state);
   onContext(context);
  const watchdog=setTimeout(()=>void context.close().catch(()=>{}),p.stabilization.navigation_ms+p.stabilization.readiness_ms+5000);
  try {
     const page=await context.newPage();
      const navigation=await navigationBoundary(context,address,scope,p.stabilization.navigation_ms,null,page);
      let resourceFailures=0,cancelledSourceReloads=0,replacing=false;
       const replacementUrls=new Set(),replacementRequests=new WeakSet();
       const requestIds=new WeakMap();let nextRequest=0,traceCount=0;
       const trace=(event,r,extra={})=>{
         if(traceCount++>=128)return;
         if(!requestIds.has(r))requestIds.set(r,++nextRequest);
         const kind=['document','stylesheet','image','font','script','xhr','fetch'].includes(r.resourceType())?r.resourceType():'other';
         diagnostics({event,request:requestIds.get(r),kind,elapsed_ms:Date.now()-started,replacing,known_replacement_source:replacementUrls.has(r.url()),...extra});
       };
       page.on('request',r=>{trace('request',r);if(replacing&&r.resourceType()==='image'&&replacementUrls.has(r.url()))replacementRequests.add(r);});
    page.on('requestfailed',r=>{
      // Attribute replacement can initiate and immediately cancel a picture's
      // fallback source reload. Only exact known image URLs, begun DURING the
      // transform, and Chromium's ERR_ABORTED are accounted separately. Earlier
      // readiness failures, scope denials, HTTP errors and other resources remain
      // failures. This count is not a claim that those reloads completed.
       if(replacementRequests.has(r)&&r.failure()?.errorText==='net::ERR_ABORTED')cancelledSourceReloads++;
       else resourceFailures++;
       trace('failed',r,{failure:r.failure()?.errorText==='net::ERR_ABORTED'?'Aborted':'Other',attributed_replacement:replacementRequests.has(r),request_start_ms:Math.max(-1,r.timing().startTime-started)});
    });
     page.on('response',r=>{if(r.status()>=400)resourceFailures++;trace('response',r.request(),{status:r.status()});});
    progress(0,1);
     const response=await page.goto(address,{waitUntil:'domcontentloaded',timeout:p.stabilization.navigation_ms}).catch(e=>{if(navigation.error)throw navigation.error;if(navigation.blocked||navigation.redirect)throw fail(state?'NeedsLogin':'RedirectOutsideScope');throw e;});
     if(navigation.blocked||navigation.redirect)throw fail(state?'NeedsLogin':'RedirectOutsideScope');
    if (!response) throw fail('CaptureFailed');
     if ([401,403].includes(response.status())) throw fail(state?'NeedsLogin':'AccessDenied');
    if ([404,410].includes(response.status())) throw fail('NotFound');
    if (response.status()===429) throw fail('RateLimited');
    if (response.status()>=500) throw fail('ServerError');
     if (response.status()>=400) throw fail('CaptureFailed');
     if(await needsLogin(page))throw fail('NeedsLogin');
    const final=new URL(page.url()),requested=new URL(address);
    if (final.origin!==requested.origin || final.pathname!==requested.pathname || final.search!==requested.search || final.hash!==requested.hash) throw fail('RedirectOutsideScope');
    const warnings=new Set();const warn=code=>{if(!warnings.has(code)){warnings.add(code);warning(code);}};
    const deadline=Date.now()+p.stabilization.readiness_ms;const left=()=>Math.max(1,deadline-Date.now());
     await page.addStyleTag({content:'*,*::before,*::after{animation:none!important;transition:none!important;caret-color:transparent!important}'});
     await page.mouse.move(-1,-1);
     await bounded(page.waitForFunction(()=>document.readyState==='complete'),Math.min(3000,left())).catch(()=>warn('ReadinessBudget'));
    await bounded(page.evaluate(()=>document.fonts.ready.then(()=>true)),Math.min(5000,left())).catch(()=>warn('FontsIncomplete'));
    const fontFailures=await page.evaluate(()=>[...document.fonts].filter(f=>f.status==='error').length);
    if(fontFailures)warn('FontsIncomplete');
    const full=p.extent==='FullPage';
    for(let y=0,i=0;full;y+=Math.max(1,p.height-100)) {
      const height=await page.evaluate(()=>Math.max(document.documentElement.scrollHeight,document.body?.scrollHeight||0));
      if(height>Math.min(32768,p.stabilization.max_height) || height*p.width>64000000)throw fail('ImageLimit');
      if(y>=height)break;
       if(++i>64)throw fail('ImageLimit');
       if(Date.now()>=deadline)throw fail('Timeout');
      await page.evaluate(y=>scrollTo(0,y),y);await page.waitForTimeout(20);
    }
    await page.evaluate(()=>{scrollTo(0,0);for(const img of document.images)img.loading='eager';});
    const imageFailures=await bounded(page.evaluate(async()=>{
      if(document.images.length>512)return -1;
      const decoded=await Promise.all([...document.images].map(i=>i.decode().then(()=>true,()=>false)));
      return decoded.filter(ok=>!ok).length;
    }),Math.min(5000,left())).catch(()=>-1);
    if(imageFailures)warn('ImagesIncomplete');
    let replacement;
    if(p.image_mode!=='Normal') {
      const unsupported=await page.evaluate(()=>{
        let source=false,media=!!document.querySelector('canvas,video,iframe');
        for(const sheet of document.styleSheets){try{source ||= [...sheet.cssRules].some(r=>/\[(?:src|srcset)/i.test(r.cssText));}catch{source=true;}}
        for(const el of [...document.querySelectorAll('*')].slice(0,2049)){
          media ||= !!el.shadowRoot;
          const bg=getComputedStyle(el).backgroundImage;
          if(bg!=='none'&&!/^url\(["']?.*\.(png|jpe?g|webp|gif)([?#][^)]*)?["']?\)$/i.test(bg))media=true;
          for(const pseudo of ['::before','::after'])if(getComputedStyle(el,pseudo).backgroundImage!=='none')media=true;
        }
        return {source,media};
      });
      if(unsupported.source)warn('ReplacementSourceDependent');
      if(unsupported.media)warn('ReplacementUnsupportedMedia');
      const sources=await page.evaluate(()=>[...document.images].filter(i=>i.naturalWidth&&i.naturalHeight&&!/\.svg(?:[?#]|$)|^data:image\/svg/i.test(i.currentSrc||i.src)).flatMap(i=>[i.currentSrc,i.src]));
      for(const source of sources)replacementUrls.add(source);
       replacing=true;
       diagnostics({event:'replacement-start',elapsed_ms:Date.now()-started});
       try{replacement=await bounded(replaceImages(page),left());}finally{replacing=false;}
       diagnostics({event:'replacement-end',elapsed_ms:Date.now()-started});
      if(replacement.geometryPreserved===false)warn('ReplacementGeometryChanged');
      if(replacement.geometryPreserved===null)warn('ReplacementGeometryNotMeasured');
      if(replacement.skipped.length)warn('ImagesIncomplete');
    }
    const geometry=await page.evaluate(()=>({width:Math.max(innerWidth,document.documentElement.scrollWidth),height:Math.max(innerHeight,document.documentElement.scrollHeight,document.body?.scrollHeight||0)}));
    const width=geometry.width,height=full?geometry.height:p.height;
    if(width>4096 || height>Math.min(32768,p.stabilization.max_height) || width*height>64000000)throw fail('ImageLimit');
    let previous,image,stable=false;
    for(let i=0;i<3;i++) {
      if(Date.now()>=deadline)throw fail('Timeout');
      image=await page.screenshot({fullPage:full,animations:'disabled',caret:'hide',timeout:left()});
      if(previous?.equals(image)){stable=true;break;}previous=image;await page.waitForTimeout(Math.min(100,left()));
    }
     if(!stable)warn('Unstable');if(resourceFailures)warn('ResourceFailed');
     if(await needsLogin(page))throw fail('NeedsLogin');
     if(navigation.blocked||navigation.redirect||page.url()!==address)throw fail(state?'NeedsLogin':'RedirectOutsideScope');
     if(navigation.embeddedBlocked)warn('EmbeddedContentBlocked');
    const actualWidth=image.readUInt32BE(16),actualHeight=image.readUInt32BE(20);
    if(actualWidth!==width || actualHeight!==height || image.length>128*1024*1024)throw fail('ImageLimit');
    const token=crypto.randomUUID(),hash=crypto.createHash('sha256').update(image).digest('hex');
    const metadata={version:1,key,requested:address,final_url:page.url(),runtime:identity,profile:p,width:actualWidth,height:actualHeight,sha256:hash,bytes:image.length,warnings:[...warnings],stable,duration_ms:Date.now()-started,readiness:{font_failures:fontFailures,image_failures:imageFailures,resource_failures:resourceFailures},capture_mode:'chromium-new-headless'};
    if(replacement)metadata.replacement={...replacement,cancelledSourceReloads};
    // No worker-supplied arbitrary path crosses the protocol boundary.
    await fs.writeFile(path.join(output,`${token}.png`),image,{flag:'wx',mode:0o600});
    await fs.writeFile(path.join(output,`${token}.json`),JSON.stringify(metadata),{flag:'wx',mode:0o600});
    progress(1,1);
    return {key,output_token:token,artifact:{id:hash,bytes:image.length,kind:'Screenshot'},width:actualWidth,height:actualHeight};
  } finally {clearTimeout(watchdog);await context.close().catch(()=>{});onContext(null);}
}
