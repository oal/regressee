import test from 'node:test';
import assert from 'node:assert/strict';
import {spawn} from 'node:child_process';
import {fileURLToPath} from 'node:url';

function probe(input, {end=false, stopAfter}={}) {
  return new Promise((resolve,reject)=>{
    const child=spawn(process.execPath,[fileURLToPath(new URL('../src/index.mjs',import.meta.url))],{env:{...process.env,REGRESSEE_ARTIFACTS:'/tmp/opencode/regressee-protocol-tests'}});
    let stdout='',stderr='';const events=[];let pending='';
    const timer=setTimeout(()=>{child.kill('SIGTERM');reject(new Error('protocol probe deadline'));},5000);
    child.stdout.on('data',data=>{stdout+=data;pending+=data;let n;while((n=pending.indexOf('\n'))>=0){events.push(JSON.parse(pending.slice(0,n)));pending=pending.slice(n+1);}if(stopAfter&&events.length>=stopAfter)child.stdin.end();});
    child.stderr.on('data',data=>stderr+=data);child.on('error',reject);
    child.on('close',code=>{clearTimeout(timer);resolve({code,events,stderr,stdout});});
    if(end)child.stdin.end(input);else child.stdin.write(input);
  });
}
test('schema rejects null, primitives, arrays and malformed payloads without crashing',async()=>{
  const envelope={v:1,id:1,job:'phase0',command:'handshake',payload:{}};
  for(const value of [null,true,1,'text',[],{}, {...envelope,v:99},{...envelope,payload:null},{...envelope,payload:[]},{...envelope,command:'capture',payload:{target:5}}]) {
    const result=await probe(JSON.stringify(value)+'\n',{stopAfter:1});
    assert.equal(result.code,0);assert.equal(result.stderr,'');assert.equal(result.events.length,1);assert.equal(result.events[0].code,'InvalidRequest');
  }
});
test('framing failures are typed and exit 64, never generic crashes',async()=>{
  for(const [input,code] of [['{bad}\n','MalformedJson'],['\xff\n','MalformedJson'],['x'.repeat(1024*1024+1),'FrameTooLarge'],['{}','IncompleteFrame']]) {
    const result=await probe(Buffer.from(input,'latin1'),{end:true});assert.equal(result.code,64);assert.equal(result.events[0].code,code);assert.equal(result.stderr,'');
  }
});
test('handshake then unknown command and deliberate crash have exact causes',async()=>{
  const request=command=>JSON.stringify({v:1,id:command==='handshake'?1:2,job:'phase0',command,payload:{}})+'\n';
  const unknown=await probe(request('handshake')+request('notAllowed'),{stopAfter:4});
  assert.equal(unknown.code,0);assert.equal(unknown.events.at(-1).code,'UnknownCommand');
  const crash=await probe(request('handshake')+request('crashWorker'));
  assert.equal(crash.code,86);assert.equal(crash.events.at(-1).type,'ack');assert.equal(crash.stderr,'');
});
test('malformed framing during browser launch awaits cleanup and retains exit 64',async()=>{
  const request=(id,command)=>JSON.stringify({v:1,id,job:'phase0',command,payload:{}})+'\n';
  const result=await probe(request(1,'handshake')+request(2,'openLogin')+'{bad}\n',{end:true});
  assert.equal(result.code,64);assert(result.events.some(e=>e.type==='failure'&&e.code==='MalformedJson'));
  assert(!result.stderr.includes('TypeError'));
});
