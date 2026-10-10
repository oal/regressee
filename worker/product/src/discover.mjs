// SPDX-License-Identifier: MPL-2.0
import {fail,literalLoopback,navigationBoundary} from './capture.mjs';
import {replayContext,needsLogin} from './sessions.mjs';
export async function robots(address, scope) {
  if(process.env.REGRESSEE_PROXY)throw fail('CaptureFailed'); // Node 22 fetch has no qualified proxy transport; explicit authorized robots override is required, never bypass a configured proxy silently.
  const url=new URL('/robots.txt',address);
  if(scope==='loopback'&&!literalLoopback(url.href))throw fail('RedirectOutsideScope');
  const controller=new AbortController();const timer=setTimeout(()=>controller.abort(),10000);
  try {
    const response=await fetch(url,{redirect:'manual',signal:controller.signal,headers:{'User-Agent':'Regressee'}});
    if(response.status===404||response.status===410)return {text:'',status:response.status};
    if(response.status!==200){await response.body?.cancel();return {text:'',status:response.status};}
    let bytes=0;const parts=[];
    for await (const chunk of response.body){bytes+=chunk.length;if(bytes>256*1024)throw fail('PayloadLimit');parts.push(chunk);}
    return {text:Buffer.concat(parts).toString('utf8'),status:200};
  } finally {clearTimeout(timer);controller.abort();}
}
export async function discover(browser,{address,section,session=null},{scope,onContext,sessions}) {
  if(scope==='loopback'&&!literalLoopback(address))throw fail('RedirectOutsideScope');
  const state=session===null?null:sessions?.get(session,address);if(session!==null&&!state)throw fail('InvalidRequest');
  const context=await replayContext(browser,{viewport:{width:1440,height:900},serviceWorkers:'block',acceptDownloads:false,userAgent:'Regressee'},state);onContext(context);
  const watchdog=setTimeout(()=>void context.close().catch(()=>{}),35000);
  try {
    const page=await context.newPage();
    const navigation=await navigationBoundary(context,address,scope,30000,section,page);
    const response=await page.goto(address,{waitUntil:'domcontentloaded',timeout:30000}).catch(e=>{if(navigation.error)throw navigation.error;if(navigation.redirect)return null;if(navigation.blocked)throw fail('RedirectOutsideScope');throw e;});
    if(navigation.redirect){if(state)throw fail('NeedsLogin');return {redirect:navigation.redirect};} // Unvisited Rust-policy candidate.
    if(!response)throw fail('CaptureFailed');
    if(response.status()===429)throw fail('RateLimited');
    if(response.status()>=500)throw fail('ServerError');
    if([401,403].includes(response.status()))throw fail(state?'NeedsLogin':'AccessDenied');
    if([404,410].includes(response.status()))throw fail('NotFound');
    if(response.status()>=400)throw fail('CaptureFailed');
    await page.waitForTimeout(100);
    if(await needsLogin(page))throw fail('NeedsLogin');
    if(navigation.blocked||page.url()!==address)throw fail('RedirectOutsideScope');
    return await page.evaluate(()=>{
      const addresses=[];let truncated=false,bytes=0;
      for(const a of document.querySelectorAll('a[href],area[href]')){
        if(a.hasAttribute('download'))continue;
        try {const u=new URL(a.href,document.baseURI);if(!['http:','https:'].includes(u.protocol)||u.username||u.password||u.href.length>4096)continue;if(addresses.length===2000||bytes+u.href.length+3>512000){truncated=true;break;}bytes+=u.href.length+3;addresses.push(u.href);}catch{}
      }
      return {addresses,truncated,final_url:location.href};
    });
  } finally {clearTimeout(watchdog);await context.close().catch(()=>{});onContext(null);}
}
