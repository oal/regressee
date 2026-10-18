// SPDX-License-Identifier: MPL-2.0
// The collector runs wholly in the page. Excluded data is never returned to Node,
// even transiently. `records` is an internal, flat representation; the canonical
// encoder is deliberately separate from DOM traversal.
export const DOM_LIMITS = Object.freeze({bytes:8*1024*1024,nodes:100000,depth:512,attributes:256,stringBytes:64*1024,attributeBytes:256*1024,recordBytes:1024*1024,nameBytes:1024,namespaceBytes:4096});
export const DOM_QUIET_MS = 100;
export const DOM_POLICY = Object.freeze({version:1});
export const DOM_HEADER = '["regressee-dom",1]\n';
export const encodeDom = records => DOM_HEADER+records.map(record=>JSON.stringify(record)).join('\n')+'\n';

export function collectDomInPage(limits) {
  const encoder=new TextEncoder(),records=[];
  let visited=0,bytes=20,iframes=0,openRoots=0,templates=0,allocated=0,maxDepth=0;
  const coverage={iframe_hosts:0,open_shadow_roots:0,template_contents:0};
  const fail=()=>{throw new Error('DomLimit');};
  const string=(value,bound=limits.stringBytes)=>{
    // Bound before normalization/encoding allocation too. Do not normalize text
    // whitespace or CR/LF. Replace ill-formed UTF-16, never emit invalid UTF-8.
    if(value.length>bound)fail();
    const result=value.toWellFormed();
    const size=encoder.encode(result).length;
    if(size>bound||(allocated+=size)+bytes>limits.bytes)fail();
    return result;
  };
  const emit=record=>{
    // Bound the internal transfer as well as the final encoding. JSON escaping
    // can expand a short raw string by six times, so count escaped UTF-8 bytes.
    const size=encoder.encode(JSON.stringify(record)).length;
    if(size>limits.recordBytes||bytes+size+1>limits.bytes)fail();
    bytes+=size+1;maxDepth=Math.max(maxDepth,record[1]);records.push(record);
  };
  const compare=(a,b)=>{
    // UTF-8 lexicographic order equals Unicode scalar order, not UTF-16 order.
    let i=0,j=0;
    while(i<a.length&&j<b.length){
      const ac=a.codePointAt(i),bc=b.codePointAt(j),difference=ac-bc;
      if(difference)return difference;
      i+=ac>0xffff?2:1;j+=bc>0xffff?2:1;
    }
    return a.length-i-(b.length-j);
  };
  const urlNames=new Set(['href','src','action','formaction','poster','background','cite','data','longdesc','usemap','manifest','codebase','profile','icon','base']);
  const isJavascript=value=>/^[\u0000-\u0020]*javascript:/i.test(value.replace(/[\t\n\r]/g,''));
  const stack=[{node:document,depth:0,omit:false,entered:false}];
  while(stack.length){
    const item=stack.at(-1),node=item.node;
    if(!item.entered){
      if(++visited>limits.nodes||item.depth>limits.depth)fail();
      allocated=0;
      item.entered=true;
      let omit=item.omit;
      // Exclusion suppresses serialization, not traversal/accounting. Website
      // script may construct templates/open roots inside an excluded subtree.
      // Discover those fragments even when no names, attributes or text from
      // their descendants will be emitted.
      let local,ns,lower;
      if(node.nodeType===1){
        local=node.localName;ns=node.namespaceURI||'';
        if(local.length>limits.stringBytes||ns.length>limits.stringBytes||node.attributes.length>limits.attributes)fail();
        lower=local.toLowerCase();
        item.shadow=node.shadowRoot;
        if(ns==='http://www.w3.org/1999/xhtml'&&lower==='template')item.template=node.content;
        if(lower==='iframe')iframes++;
      }
      if(!omit){
        if(item.boundary){
          emit([item.boundary,item.depth]);
          if(item.boundary==='shadow')coverage.open_shadow_roots++;
          else coverage.template_contents++;
        }
        else if(node.nodeType===9)emit(['document',0]);
        else if(node.nodeType===10)emit(['doctype',item.depth,string(node.name,limits.nameBytes),string(node.publicId),string(node.systemId)]);
        else if(node.nodeType===1){
          local=string(local,limits.nameBytes);ns=string(ns,limits.namespaceBytes);
          const attrs=[];let attributeBytes=0;
          for(let i=0;i<node.attributes.length;i++){
            const attr=node.attributes.item(i),low=attr.localName.toLowerCase();
            if(low.startsWith('on')||low==='nonce'||(low==='value'&&['input','select','option','button','textarea'].includes(lower))||(low==='checked'&&lower==='input')||(low==='selected'&&lower==='option')||(low==='srcdoc'&&lower==='iframe'))continue;
            // Inspect only the attribute string, never value/checked/selection
            // properties. Javascript code is excluded before it crosses the page.
            const value=string(attr.value);
            if(urlNames.has(low)&&isJavascript(value))continue;
            const name=string(attr.name??((attr.prefix?attr.prefix+':':'')+attr.localName),limits.nameBytes),namespace=string(attr.namespaceURI||'',limits.namespaceBytes);
            attributeBytes+=encoder.encode(name).length+encoder.encode(namespace).length+encoder.encode(value).length;
            if(attributeBytes>limits.attributeBytes)fail();
            attrs.push([namespace,name,value]);
          }
          attrs.sort((a,b)=>compare(a[0],b[0])||compare(a[1],b[1]));
          const record=['element',item.depth,ns,local,attrs];
          if(node.prefix!==null&&node.prefix!==undefined){
            const prefix=string(node.prefix,limits.nameBytes);
            if(!prefix)throw new Error('DomSerialization');
            record.push(prefix);
          }
          emit(record);
          if(lower==='iframe')coverage.iframe_hosts++;
          // All namespaces, including SVG and JSON/hydration script elements.
          omit=lower==='script'||lower==='textarea'||lower==='iframe';
        }else if(node.nodeType===3||node.nodeType===4)emit(['text',item.depth,string(node.data)]);
      }
      item.omitChildren=omit;
      item.next=node.firstChild;
    }
    if(item.template){
      const fragment=item.template;item.template=null;
      const omit=item.omitChildren;
      templates++;item.templateHost=true;
      stack.push({node:fragment,depth:item.depth+1,omit,entered:false,boundary:'template'});continue;
    }
    if(item.next){
      const child=item.next;item.next=child.nextSibling;
      stack.push({node:child,depth:item.depth+1,omit:item.omitChildren||item.templateHost,entered:false});continue;
    }
    // Core v1: light children precede a terminal shadow boundary. Excluded roots
    // are still counted, but never represented as serialized coverage.
    if(item.shadow){
      const root=item.shadow;item.shadow=null;openRoots++;
      stack.push({node:root,depth:item.depth+1,omit:item.omitChildren||item.templateHost,entered:false,boundary:'shadow'});continue;
    }
    stack.pop();
  }
  return {records,visited_nodes:visited,node_count:records.length,max_depth:maxDepth,coverage,iframe_hosts:iframes,open_shadow_roots:openRoots,template_contents:templates};
}

export async function sampleDom(page,{deadline,limits=DOM_LIMITS,quietMs=DOM_QUIET_MS,encode=encodeDom}) {
  const previous=new Set();let last,samples=0;
  for(let i=0;i<3;i++){
    if(Date.now()>=deadline)throw Object.assign(new Error('Timeout'),{code:'Timeout'});
    let collected,timer;
    try{
      collected=await Promise.race([
        page.evaluate(collectDomInPage,limits),
        new Promise((_,reject)=>{timer=setTimeout(()=>reject(Object.assign(new Error('Timeout'),{code:'Timeout'})),Math.max(1,deadline-Date.now()));}),
      ]);
      if(Date.now()>=deadline)throw Object.assign(new Error('Timeout'),{code:'Timeout'});
    }
    catch(error){
      if(error.message?.includes('DomLimit'))throw Object.assign(new Error('DomLimit'),{code:'DomLimit'});
      if(error.code==='Timeout')throw error;
      throw Object.assign(new Error('DomSerialization'),{code:'DomSerialization'});
    }finally{clearTimeout(timer);}
    let body;
    try{body=encode(collected.records);if(typeof body!=='string')throw new Error();}
    catch{throw Object.assign(new Error('DomSerialization'),{code:'DomSerialization'});}
    if(Buffer.byteLength(body)>limits.bytes)throw Object.assign(new Error('DomLimit'),{code:'DomLimit'});
    samples++;last={body,...collected,records:undefined,samples,stable:false};
    if(previous.has(body))return {...last,stable:true};
    previous.add(body);
    if(i<2){
      if(Date.now()+quietMs>=deadline)break;
      await page.waitForTimeout(quietMs);
    }
  }
  return last;
}
