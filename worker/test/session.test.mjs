import test from 'node:test';
import assert from 'node:assert/strict';
import {fixtureServer} from '../src/fixtures.mjs';
import {launch,replay,saveSession,capture} from '../src/capture.mjs';

test('saved fixture session works only on its original origin and never leaks into fresh context',async()=>{
  let a,z,browser,login,restored,empty;
  try {
    a=await fixtureServer();z=await fixtureServer();browser=await launch(false);login=await replay(browser);
    const page=await login.newPage();await page.goto(a.origin+'/login');await page.click('#login');await page.waitForFunction(()=>document.documentElement.dataset.auth==='ok');
    restored=await replay(browser,await saveSession(login,page));
    const positive=await capture(restored,a.origin+'/private');assert.equal(positive.metadata.stable,true);
    await assert.rejects(capture(restored,z.origin+'/private'),e=>e.code==='RequiresLogin');
    empty=await replay(browser);await assert.rejects(capture(empty,a.origin+'/private'),e=>e.code==='RequiresLogin');
  } finally {await empty?.close();await restored?.close();await login?.close();await browser?.close();await a?.close();await z?.close();}
});
