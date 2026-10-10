// SPDX-License-Identifier: MPL-2.0
import { chromium } from 'playwright';
import crypto from 'node:crypto';
import os from 'node:os';
export const PROFILE = Object.freeze({version:1, viewport:{width:1440,height:900}, screen:{width:1440,height:900}, deviceScaleFactor:1, isMobile:false, hasTouch:false, locale:'en-US', timezoneId:'UTC', colorScheme:'light', reducedMotion:'reduce', serviceWorkers:'block'});
export const launch = headed => chromium.launch({channel:'chromium',headless:!headed,chromiumSandbox:true});
export const IMAGE_ALGORITHM = 'neutral-raster-v2';
export const fingerprint = browser => ({os:os.platform(),release:os.release(),arch:os.arch(),browser:browser.version(),node:process.version,worker:1,profile:PROFILE,mode:'chromium-new-headless',imageAlgorithm:IMAGE_ALGORITHM});
export function comparable(a,b) {
  if(JSON.stringify(a.profile)!==JSON.stringify(b.profile)||a.browser!==b.browser||a.mode!==b.mode||a.imageAlgorithm!==b.imageAlgorithm)return 'blocked-profile-mismatch';
  if(a.os!==b.os||a.arch!==b.arch||a.release!==b.release||JSON.stringify(a.fonts)!==JSON.stringify(b.fonts))return 'caution-rendering-environment';
  return 'same-recorded-environment';
}
export async function replay(browser, session) {
  const context = await browser.newContext({...PROFILE, ...(session ? {storageState:session.state} : {})});
  if(session) await context.addInitScript(({origin,values})=>{if(location.origin===origin)for(const[k,v]of Object.entries(values))sessionStorage.setItem(k,v);},session.sessionStorage);
  return context;
}
export async function saveSession(context,page) {
  return {state:await context.storageState({indexedDB:true}),sessionStorage:{origin:new URL(page.url()).origin,values:await page.evaluate(()=>Object.fromEntries(Object.entries(sessionStorage)))}};
}
export async function replaceImages(page) {
  return page.evaluate(async()=>{
    const elements=[...document.querySelectorAll('*')];
    const measured=elements.length<=2048?elements:[];
    const rectangles=()=>measured.map((e,i)=>[i,...['x','y','width','height'].map(k=>e.getBoundingClientRect()[k])]);
    const before=rectangles();
    let count=0;
    const skipped=[];
    const imageDiagnostics=[...document.images].map(i=>({id:i.id,width:i.naturalWidth,height:i.naturalHeight,complete:i.complete}));
    // Keep source intrinsic dimensions/aspect for the img's object-fit/position.
    // The SVG's own viewBox must stretch, not apply a second (default meet) fit.
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
      // Only single, known raster URL backgrounds; leave complex layers intact.
      if(/^url\(["']?.*\.(png|jpe?g|webp|gif)([?#][^)]*)?["']?\)$/i.test(bg)) {el.style.backgroundImage=`url("${neutral(16,16)}")`;count++;}
    }
    const after=rectangles();
    return {count,before,after,imageDiagnostics,skipped,geometryMeasuredElements:measured.length,geometryScope:measured.length?'light-DOM element rectangles':'NOT MEASURED: element budget exceeded',geometryPreserved:measured.length?JSON.stringify(before)===JSON.stringify(after):null,limitations:['canvas','video','frames','shadow DOM','pseudo-elements','complex backgrounds','extensionless raster URLs','source-dependent CSS or script (src/srcset/picture attributes change)','geometry excludes shadow/frame contents and visual styling']};
  });
}
export async function capture(context,url,{replace=false,fullPage=true}={}) {
  const page=await context.newPage();
  const resources={failedRequests:0,httpErrors:0};
  page.on('requestfailed',()=>resources.failedRequests++);
  page.on('response',response=>{if(response.status()>=400)resources.httpErrors++;});
  const bounded=async(promise,ms,code)=>{let timer;try{return await Promise.race([promise,new Promise((_,reject)=>{timer=setTimeout(()=>reject(Object.assign(new Error(),{code})),ms);})]);}finally{clearTimeout(timer);}};
  try {
    const response=await page.goto(url,{waitUntil:'domcontentloaded',timeout:30000});
    const deadline=Date.now()+15000;
    if(!response||response.status()>=400)throw Object.assign(new Error(),{code:'HttpError'});
    if(new URL(page.url()).pathname==='/login')throw Object.assign(new Error(),{code:'RequiresLogin'});
    if(new URL(page.url()).origin!==new URL(url).origin)throw Object.assign(new Error(),{code:'UnexpectedRedirect'});
    if(new URL(url).pathname==='/private') {
      await page.waitForFunction(()=>document.documentElement.dataset.auth,null,{timeout:5000});
      if(await page.getAttribute('html','data-auth')!=='ok')throw Object.assign(new Error(),{code:'RequiresLogin'});
    }
    await page.addStyleTag({content:'*,*::before,*::after{animation:none!important;transition:none!important;caret-color:transparent!important}'});
    const warnings=[];
    await bounded(page.evaluate(()=>document.fonts.ready),5000,'FontsTimeout').catch(()=>warnings.push('FontsTimeout'));
    const fontFailures=await page.evaluate(()=>[...document.fonts].filter(f=>f.status==='error').length);
    if(fontFailures)warnings.push('FontLoadFailed');
    const start=Date.now();
    for(let y=0,iterations=0; fullPage && y<await page.evaluate(()=>document.documentElement.scrollHeight);y+=800){
      if(++iterations>42||Date.now()-start>10000||Date.now()>deadline)throw Object.assign(new Error(),{code:'PageTooTallOrInfinite'});
      await page.evaluate(y=>scrollTo(0,y),y);await page.waitForTimeout(20);
    }
    await page.evaluate(()=>scrollTo(0,0));
    // Native lazy-load scheduling can lag a scroll on a short document. Request
    // eager loading only after layout/scroll readiness, then wait for dimensions.
    await page.evaluate(()=>{for(const img of document.images)img.loading='eager';});
    await page.waitForFunction(()=>[...document.images].every(i=>i.complete),null,{timeout:Math.min(5000,Math.max(1,deadline-Date.now()))}).catch(()=>warnings.push('ImagesIncomplete'));
    const imageFailures=await bounded(page.evaluate(async()=>{
      const failures=[];
      await Promise.all([...document.images].map(async(i,index)=>{try{await i.decode();}catch{failures.push({index,reason:'ImageDecodeFailed',complete:i.complete,width:i.naturalWidth,height:i.naturalHeight});}}));
      return failures;
    }),5000,'ImagesTimeout').catch(()=>{warnings.push('ImagesTimeout');return [];});
    if(imageFailures.length)warnings.push('ImageDecodeFailed');
    const dimensions=await page.evaluate(()=>({width:Math.max(innerWidth,document.documentElement.scrollWidth),height:document.documentElement.scrollHeight}));
    if(dimensions.width>4096||dimensions.height>32768||dimensions.width*dimensions.height>64000000)throw Object.assign(new Error(),{code:'ImageBudgetExceeded'});
    const replacement=replace?await replaceImages(page):null;
    // Screenshot stability is not proof of valid replacement geometry.
    if(replacement?.geometryPreserved===false)warnings.push('ReplacementGeometryChanged');
    if(replacement?.geometryPreserved===null)warnings.push('ReplacementGeometryNotMeasured');
    let previous, image, stable=false;
    for(let attempt=0;attempt<3;attempt++){
      if(Date.now()>deadline)throw Object.assign(new Error(),{code:'StabilizationTimeout'});
      image=await page.screenshot({fullPage,animations:'disabled',caret:'hide',timeout:Math.max(1,deadline-Date.now())});
      if(previous?.equals(image)){stable=true;break;}previous=image;await page.waitForTimeout(100);
    }
    if(!stable)warnings.push('UnstablePage');
    if(resources.failedRequests)warnings.push('ResourceRequestFailed');
    if(resources.httpErrors)warnings.push('ResourceHttpError');
    const fonts=await page.evaluate(()=>({families:[...new Set([...document.querySelectorAll('*')].map(e=>getComputedStyle(e).fontFamily))].sort(),arial:document.fonts.check('16px Arial')}));
    return {image,metadata:{dimensions,stable,warnings,readiness:{imageFailures,fontFailures,resources},replacement,profile:{...PROFILE,fullPage,replace},fonts,userAgent:await page.evaluate(()=>navigator.userAgent),sha256:crypto.createHash('sha256').update(image).digest('hex')}};
  }finally{await page.close().catch(()=>{});}
}
