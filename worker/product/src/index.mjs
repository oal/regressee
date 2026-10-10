// SPDX-License-Identifier: MPL-2.0
import {parseRequest,encodeEvent,MAX_FRAME} from './protocol.mjs';
import {launch,capture,fail,PLAYWRIGHT_VERSION,WORKER_VERSION} from './capture.mjs';
import {discover,robots} from './discover.mjs';
import {Sessions,openLogin,confirmLogin} from './sessions.mjs';
import fs from 'node:fs/promises';
import path from 'node:path';
const identity=JSON.parse(process.env.REGRESSEE_RUNTIME||'null');
const output=process.env.REGRESSEE_OUTPUT,scope=process.env.REGRESSEE_SCOPE;
let browser,context,active=null,buffer=Buffer.alloc(0),closing=false;
let login=null,loginTimer;const sessions=new Sessions();
const send=(r,type,payload)=>process.stdout.write(encodeEvent(r,type,payload));
const failure=(r,code)=>send(r,'Failure',{code,retryable:['Timeout','WorkerStopped','RateLimited','ServerError'].includes(code)});
async function run(r) {
  send(r,'Acknowledged');
  const type=r.command.type;
  if(type==='Cancel') {
    if(active && active.job===r.job){active.cancelled=true;await context?.close().catch(()=>{});}
    send(r,'Cancelled');return;
  }
  if(type==='Shutdown') {
    closing=true;if(active)active.cancelled=true;
    await context?.close().catch(()=>{});await login?.browser.close().catch(()=>{});await browser?.close().catch(()=>{});sessions.clear();
    send(r,'Completed');process.stdin.pause();return;
  }
  if(active || closing) {failure(r,'InvalidRequest');return;}
  active={job:r.job,cancelled:false};const task=active;
  try {
    if(type==='Handshake') {
      if(!identity?.Pinned || !output || !['public','loopback'].includes(scope))throw fail('InvalidRequest');
      browser=await launch();
       if(browser.version()!==identity.Pinned.browser || process.version.slice(1)!==identity.Pinned.node || PLAYWRIGHT_VERSION!==identity.Pinned.playwright || identity.Pinned.worker!==WORKER_VERSION)throw fail('InvalidRequest');
      const smoke=await browser.newContext();context=smoke;
      try {const page=await smoke.newPage();await page.setContent('<html><body>Regressee runtime smoke</body></html>');const png=await page.screenshot({timeout:10000});if(png.length<32)throw fail('CaptureFailed');}finally{await smoke.close();context=null;}
       send(r,'Ready',{worker_version:WORKER_VERSION,browser_version:browser.version()});
    } else if(type==='OpenLogin') {
      if(!browser||login)throw fail('InvalidRequest');
      const headed=await launch(false);
      try {login=await openLogin(headed,r.command.payload,{scope,onContext:c=>{context=c;}});}catch(e){await headed.close();throw e;}
      loginTimer=setTimeout(()=>{const expired=login;login=null;void expired?.browser.close();},15*60*1000);loginTimer.unref();
      send(r,'LoginOpened');
    } else if(type==='ConfirmLogin') {
      if(!login||JSON.stringify(login.payload.session)!==JSON.stringify(r.command.payload.session))throw fail('InvalidRequest');
      const material=await confirmLogin(login,{scope,sessions,onContext:c=>{context=c;}});
      clearTimeout(loginTimer);
      await login.browser.close();login=null;context=null;
      send(r,'SessionReady',{material});
    } else if(type==='InstallSession') {
      if(!browser)throw fail('InvalidRequest');sessions.install(r.command.payload);send(r,'Completed');
     } else if(type==='CaptureTarget') {
       if(!browser)throw fail('InvalidRequest');
       const trace=process.env.REGRESSEE_PRIVATE_CAPTURE_DIAGNOSTICS==='1'?[]:null;
       const result=await capture(browser,r.command.payload,{identity,scope,output,sessions,onContext:c=>{context=c;},progress:(completed,total)=>send(r,'Progress',{completed,total}),warning:code=>send(r,'Warning',{code}),diagnostics:e=>{if(trace&&trace.length<128)trace.push(e);}});
       if(trace)await fs.writeFile(path.join(output,`${result.output_token}.trace.json`),JSON.stringify(trace),{flag:'wx',mode:0o600});
      if(task.cancelled)send(r,'Cancelled');else send(r,'TargetCaptured',result);
    } else if(type==='DiscoverPage'||type==='DiscoverAuthenticated') {
      if(!browser)throw fail('InvalidRequest');
      const result=await discover(browser,r.command.payload,{scope,sessions,onContext:c=>{context=c;}});
      if(task.cancelled)send(r,'Cancelled');else if(result.redirect)send(r,'RedirectFound',{address:result.redirect});else send(r,'LinksFound',result);
    } else if(type==='FetchRobots') {
      const result=await robots(r.command.payload.address,scope);
      if(task.cancelled)send(r,'Cancelled');else send(r,'RobotsFound',result);
    } else throw fail('InvalidRequest');
  } catch(e) {
    if(task.cancelled)send(r,'Cancelled');else failure(r,['NeedsLogin','AccessDenied','NotFound','RedirectOutsideScope','ImageLimit','InvalidRequest','Timeout','RateLimited','ServerError','PayloadLimit'].includes(e.code)?e.code:e.name==='TimeoutError'?'Timeout':'CaptureFailed');
  } finally {active=null;}
}
process.stdin.on('data',chunk=>{
  buffer=Buffer.concat([buffer,chunk]);
  if(buffer.length>MAX_FRAME && buffer.indexOf(10)<0){process.exitCode=65;process.stdin.destroy();return;}
  for(let newline;(newline=buffer.indexOf(10))>=0;) {
    const frame=buffer.subarray(0,newline+1);buffer=buffer.subarray(newline+1);
    try {const r=parseRequest(frame);void run(r);} catch {process.exitCode=65;process.stdin.destroy();return;}
  }
  if(buffer.length>MAX_FRAME){process.exitCode=65;process.stdin.destroy();}
});
process.stdin.on('end',()=>{void (async()=>{await context?.close().catch(()=>{});await login?.browser.close().catch(()=>{});await browser?.close().catch(()=>{});sessions.clear();})();});
for(const signal of ['SIGTERM','SIGINT'])process.on(signal,()=>{void (async()=>{await context?.close().catch(()=>{});await login?.browser.close().catch(()=>{});await browser?.close().catch(()=>{});sessions.clear();process.exit(0);})();});
