// SPDX-License-Identifier: MPL-2.0
import assert from 'node:assert/strict';
import {test} from 'node:test';
import {randomUUID} from 'node:crypto';
import {readFileSync} from 'node:fs';
import {parseRequest, encodeEvent, MAX_FRAME, COMMANDS,VERSION} from '../src/protocol.mjs';
const envelope = command => ({version: VERSION, request_id: 1, job: randomUUID(), command});
const frame = request => Buffer.from(JSON.stringify(request) + '\n');
test('shared Rust-emitted session frames require all three non-nil IDs', () => {
  const fixtures = readFileSync(new URL('./fixtures/session-requests.ndjson', import.meta.url), 'utf8');
  for (const line of fixtures.trimEnd().split('\n')) {
    const request = parseRequest(Buffer.from(line + '\n'));
    assert(['SetupLogin', 'CaptureTarget'].includes(request.command.type));
    for (const field of ['id', 'project', 'environment']) {
      const invalid = structuredClone(request);
      invalid.command.payload.session[field] = '00000000-0000-0000-0000-000000000000';
      assert.throws(() => parseRequest(frame(invalid)), {code:'InvalidRequest'});
      delete invalid.command.payload.session[field];
      assert.throws(() => parseRequest(frame(invalid)), {code:'InvalidRequest'});
    }
    const publicRequest = structuredClone(request);
    publicRequest.command.payload.session = null;
    if (request.command.type === 'CaptureTarget') {
      assert.equal(parseRequest(frame(publicRequest)).command.payload.session, null);
    } else {
      assert.throws(() => parseRequest(frame(publicRequest)), {code:'InvalidRequest'});
    }
  }
});
test('handshake and whitelist are product-only, not fixture actions', () => {
  assert.equal(parseRequest(frame(envelope({type:'Handshake'}))).command.type, 'Handshake');
  assert(!COMMANDS.includes('selfTest'));
  for (const command of ['selfTest','crashBrowser','eval']) assert.throws(() => parseRequest(frame(envelope({type:command}))), /InvalidRequest/);
});
test('bounded frames, version, correlation and exact fields', () => {
  assert.throws(() => parseRequest(Buffer.from('{}')), /PayloadLimit/);
  assert.throws(() => parseRequest(Buffer.alloc(MAX_FRAME+1)), /PayloadLimit/);
  assert.throws(() => parseRequest(Buffer.from([123,34,120,34,58,34,255,34,125,10])), /InvalidRequest/);
  assert.throws(() => parseRequest(frame({...envelope({type:'Handshake'}), version:1})), /UnsupportedVersion/);
  assert.throws(() => parseRequest(frame({...envelope({type:'Handshake'}), request_id:0})), /InvalidRequest/);
  assert.throws(() => parseRequest(frame(envelope({type:'Handshake', payload:{script:'x'}}))), /InvalidRequest/);
});
test('navigation rejects credentials and non-http schemes', () => {
  for (const address of ['file:///secret', 'https://me:cookie@example.test', 'javascript:alert(1)','http:example.test','https://example.test\\action','https://example.test/\nsecret']) assert.throws(() => parseRequest(frame(envelope({type:'DiscoverPage', payload:{address,section:null}}))), /InvalidRequest/);
  const value = parseRequest(frame(envelope({type:'DiscoverPage',payload:{address:'https://example.test/?q=1&q=2',section:null}}))); assert.equal(value.command.payload.address, 'https://example.test/?q=1&q=2');
});
test('outgoing frames include JSON escaping and newline in the private budget',()=>{
  const r=envelope({type:'Handshake'});
  assert.equal(JSON.parse(encodeEvent(r,'Completed')).event.type,'Completed');
  assert.throws(()=>encodeEvent(r,'SessionReady',{material:'\u0001'.repeat(512*1024)}),{code:'PayloadLimit'});
  assert.throws(()=>encodeEvent(r,'LinksFound',{addresses:['x'.repeat(MAX_FRAME)]}),{code:'PayloadLimit'});
});
test('v3 capture selection is exact, versioned and never silently defaults on the wire',()=>{
  const request=JSON.parse(readFileSync(new URL('./fixtures/session-requests.ndjson',import.meta.url),'utf8').trimEnd().split('\n')[1]);
  for(const mode of ['ScreenshotsOnly','ScreenshotsAndDomSnapshots','DomSnapshotsOnly']){
    const changed=structuredClone(request);changed.command.payload.plan={version:1,mode,dom_policy:mode==='ScreenshotsOnly'?null:{version:1}};
    assert.equal(parseRequest(frame(changed)).command.payload.plan.mode,mode);
  }
  for(const bad of [undefined,{}, {version:2,mode:'ScreenshotsOnly',dom_policy:null},{version:1,mode:'ScreenshotsOnly',dom_policy:{version:1}},{version:1,mode:'DomSnapshotsOnly',dom_policy:null},{version:1,mode:'DomSnapshotsOnly',dom_policy:{version:2}},{version:1,mode:'DomSnapshotsOnly',dom_policy:{version:1,secret:'x'}}]){
    const changed=structuredClone(request);changed.command.payload.plan=bad;assert.throws(()=>parseRequest(frame(changed)),{code:'InvalidRequest'});
  }
  assert.throws(()=>parseRequest(frame({...request,version:2})),{code:'UnsupportedVersion'});
});
test('shared Rust-validated channel frames have exact Node wire bytes and no snapshot body',()=>{
  for(const line of readFileSync(new URL('./fixtures/capture-events.ndjson',import.meta.url),'utf8').trimEnd().split('\n')){
    const event=JSON.parse(line);
    assert.equal(encodeEvent(event,event.event.type,event.event.payload),line+'\n');
    assert(Buffer.byteLength(line)<MAX_FRAME);assert(!line.includes('regressee-dom'));
  }
});
