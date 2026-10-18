// SPDX-License-Identifier: MPL-2.0
import test from 'node:test';
import assert from 'node:assert/strict';
import {collectDomInPage,DOM_LIMITS,sampleDom,DOM_QUIET_MS,encodeDom as encode} from '../src/dom.mjs';
import {readFileSync} from 'node:fs';

const text=data=>({nodeType:3,data});
const children=(parent,nodes)=>{parent.firstChild=nodes[0]||null;nodes.forEach((node,i)=>{node.nextSibling=nodes[i+1]||null;});return parent;};
const element=(localName,attrs={},nodes=[],namespaceURI='http://www.w3.org/1999/xhtml')=>children({nodeType:1,localName,namespaceURI,prefix:null,attributes:{length:Object.keys(attrs).length,item:i=>({localName:Object.keys(attrs)[i],namespaceURI:null,prefix:null,get value(){return Object.values(attrs)[i];}})}},nodes);
const collect=(nodes,limits={})=>{
  globalThis.document=children({nodeType:9},nodes);
  try{return collectDomInPage({...DOM_LIMITS,...limits});}finally{delete globalThis.document;}
};

test('collector excludes code/form state before transfer but preserves ordinary private content and whitespace',()=>{
  const secret=()=>assert.fail('excluded body read');
  const excluded={nodeType:3,get data(){return secret();}};
  const script=element('script',{type:'application/json'},[excluded]);
  const textarea=element('textarea',{value:'PRIVATE',name:'message'},[{nodeType:3,get data(){return secret();}}]);
  const password=element('input',{type:'password',value:'PRIVATE',checked:'PRIVATE',name:'password',nonce:'PRIVATE',OnClick:'PRIVATE','data-private':'ordinary private attribute'});
  Object.defineProperty(password,'value',{get:secret});Object.defineProperty(password,'checked',{get:secret});
  const result=collect([script,textarea,password,element('iframe',{srcdoc:'PRIVATE',src:'/frame'}),element('a',{href:' \tjaVa\nscript:PRIVATE',id:'unchanged',class:'site-owned',onclick:'PRIVATE'},[text(' hidden\r\n  ordinary private text ')]),element('a',{href:'/ordinary?q=1'})]);
  const body=encode(result.records);
  assert(!body.includes('PRIVATE'));assert(body.includes('ordinary private attribute'));assert(body.includes(' hidden\\r\\n  ordinary private text '));
  assert(body.includes('/ordinary?q=1'));assert(body.includes('site-owned'));assert.equal(result.iframe_hosts,1);
  assert.equal(result.visited_nodes,10); // Document + hosts + excluded descendants + text.
});

test('nested open roots and template content use explicit boundaries without slot duplication',()=>{
  const inner=element('nested-host',{},[text('light')]);inner.shadowRoot=children({nodeType:11},[element('slot'),text('shadow')]);
  const template=element('template');template.content=children({nodeType:11},[inner,element('script',{},[text('PRIVATE')])]);
  const host=element('outer-host',{},[template]);host.shadowRoot=children({nodeType:11},[text('outer')]);
  const result=collect([host]);const body=encode(result.records);
  assert.equal(result.open_shadow_roots,2);assert(!body.includes('PRIVATE'));
  assert.equal(result.records.filter(r=>r[0]==='shadow').length,2);assert.equal(result.records.filter(r=>r[0]==='template').length,1);
  assert.equal(result.records.filter(r=>r[0]==='text'&&r[2]==='light').length,1);assert.equal(result.records.filter(r=>r[0]==='text'&&r[2]==='shadow').length,1);
  assert(result.records.findIndex(r=>r[0]==='text'&&r[2]==='light')<result.records.findIndex(r=>r[0]==='text'&&r[2]==='shadow'));
});

test('attributes sort independently of source order and UTF-16 is made well formed, comments omitted',()=>{
  const a=collect([element('p',{z:'last',a:'first'},[text('a\ud800b'),{nodeType:8,data:'comment'}])]);
  const b=collect([element('p',{a:'first',z:'last'},[text('a\ufffdb')])]);
  assert.equal(encode(a.records),encode(b.records));assert.equal(a.visited_nodes,b.visited_nodes+1);
});

test('hard walk bounds fail explicitly, including excluded descendants and escaped allocation',()=>{
  assert.throws(()=>collect([element('script',{},[text('not read')])],{nodes:2}),/DomLimit/);
  assert.throws(()=>collect([element('div',{},[text('too deep')])],{depth:1}),/DomLimit/);
  assert.throws(()=>collect([element('div',{a:'x',b:'y'})],{attributes:1}),/DomLimit/);
  assert.throws(()=>collect([text('abcdef')],{stringBytes:5}),/DomLimit/);
  assert.throws(()=>collect([text('\u0001'.repeat(20))],{bytes:100}),/DomLimit/);
});

test('excluded script/textarea subtrees still count all template and open-root nodes at actual hard limits',()=>{
  for(const tag of ['textarea','script']){
    const template=element('template');
    template.content=children({nodeType:11},Array.from({length:100001},()=>({nodeType:8})));
    assert.throws(()=>collect([element(tag,{},[template])]),/DomLimit/);

    let nested=text('excluded');
    for(let i=0;i<600;i++)nested=element('div',{},[nested]);
    template.content=children({nodeType:11},[nested]);
    assert.throws(()=>collect([element(tag,{},[template])]),/DomLimit/);

    const host=element('div');host.shadowRoot=children({nodeType:11},Array.from({length:100001},()=>({nodeType:8})));
    assert.throws(()=>collect([element(tag,{},[host])]),/DomLimit/);
  }
});

test('accounting excluded roots never emits boundary markers, attributes or descendant text',()=>{
  const secret={nodeType:3,get data(){assert.fail('excluded data read');}};
  const host=element('div',{title:'PRIVATE'},[secret]);
  host.shadowRoot=children({nodeType:11},[{nodeType:3,get data(){assert.fail('excluded shadow data read');}}]);
  const template=element('template');template.content=children({nodeType:11},[host]);
  const result=collect([element('textarea',{},[template])]);
  assert.equal(result.visited_nodes,8);assert.equal(result.open_shadow_roots,1);
  assert.deepEqual(result.records,[['document',0],['element',1,'http://www.w3.org/1999/xhtml','textarea',[]]]);
  assert.equal(result.coverage.open_shadow_roots,0);
});

test('actual collector exactly reproduces the shared Rust canonical golden bytes',()=>{
  for(const fixture of ['dom-v1.ndjson','dom-v1-html-names.ndjson']){
  const golden=readFileSync(new URL('../../../crates/core/tests/fixtures/'+fixture,import.meta.url),'utf8');
  const stack=[];let root;
  const append=(parent,node)=>{if(parent.lastChild)parent.lastChild.nextSibling=node;else parent.firstChild=node;parent.lastChild=node;};
  for(const [kind,depth,...fields] of golden.trimEnd().split('\n').map(JSON.parse).slice(1)){
    let node;
    if(kind==='document'){node={nodeType:9};root=node;}
    else if(kind==='element'){
      const [ns,name,attrs]=fields;node=element(name,{},[],ns);
      node.prefix=fields[3]??null;
      node.attributes={length:attrs.length,item:i=>({namespaceURI:attrs[i][0],name:attrs[i][1],localName:attrs[i][0]?attrs[i][1].split(':').at(-1):attrs[i][1],value:attrs[i][2]})};
    }else if(kind==='text')node=text(fields[0]);
    else if(kind==='doctype')node={nodeType:10,name:fields[0],publicId:fields[1],systemId:fields[2]};
    else node={nodeType:11};
    if(depth){const parent=stack[depth-1];if(kind==='shadow')parent.shadowRoot=node;else if(kind==='template')parent.content=node;else append(parent,node);}
    stack[depth]=node;stack.length=depth+1;
  }
  globalThis.document=root;
  try{
    const result=collectDomInPage(DOM_LIMITS);
    assert.equal(encode(result.records),golden);
    assert.equal(result.node_count,golden.trimEnd().split('\n').length-1);
    assert.equal(result.max_depth,Math.max(...golden.trimEnd().split('\n').map(JSON.parse).slice(1).map(r=>r[1])));
    assert.deepEqual(result.coverage,fixture==='dom-v1.ndjson'?{iframe_hosts:1,open_shadow_roots:1,template_contents:2}:{iframe_hosts:0,open_shadow_roots:0,template_contents:0});
  }finally{delete globalThis.document;}
  }
});

test('v1 string, total-attribute and escaped-record limits are enforced during collection',()=>{
  assert.throws(()=>collect([text('x'.repeat(64*1024+1))]),/DomLimit/);
  assert.throws(()=>collect([element('p',Object.fromEntries(Array.from({length:5},(_,i)=>['a'+i,'x'.repeat(64*1024)])))]),/DomLimit/);
  assert.throws(()=>collect([element('p',Object.fromEntries(Array.from({length:4},(_,i)=>['a'+i,'\u0001'.repeat(64*1024-20)])))]),/DomLimit/);
});
test('real HTML local names and null-namespace colon attributes are not misparsed as XML',()=>{
  const result=collect([element('a:b',{'@click':'ordinary','123':'ordinary','[binding]':'ordinary',':class':'ordinary','a:foo':'one','b:foo':'two','a:b:c':'ordinary','a:onclick':'ordinary inert attribute'})]);
  const record=result.records.find(record=>record[0]==='element');
  assert.equal(record[3],'a:b');
  for(const name of ['@click','123','[binding]',':class','a:foo','b:foo','a:b:c','a:onclick'])assert(record[4].some(attr=>attr[0]===''&&attr[1]===name));
});

test('sampling uses at most three samples and a full quiet interval, never PNG/geometry',async()=>{
  const run=async values=>{
    let calls=0;const waits=[];
    const page={evaluate:async()=>({records:[['text',values[calls++]]],visited_nodes:1,iframe_hosts:0,open_shadow_roots:0}),waitForTimeout:async ms=>{waits.push(ms);}};
    return {result:await sampleDom(page,{deadline:Date.now()+10000,encode}),calls,waits};
  };
  const stable=await run(['same','same']);assert.equal(stable.result.stable,true);assert.equal(stable.calls,2);assert.deepEqual(stable.waits,[DOM_QUIET_MS]);
  const changing=await run(['one','two','three']);assert.equal(changing.result.stable,false);assert.equal(changing.calls,3);assert.equal(changing.waits.length,2);
  const lateStable=await run(['one','two','two']);assert.equal(lateStable.result.stable,true);assert.equal(lateStable.calls,3);
  const returningStable=await run(['one','two','one']);assert.equal(returningStable.result.stable,true);assert.equal(returningStable.calls,3);
});

test('an unresponsive page evaluator cannot exceed the sampler deadline',async()=>{
  const start=Date.now();
  await assert.rejects(()=>sampleDom({evaluate:()=>new Promise(()=>{}),waitForTimeout:()=>assert.fail('must not retry')},{deadline:start+20,encode}),{code:'Timeout'});
  assert(Date.now()-start<1000);
});
