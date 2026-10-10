// SPDX-License-Identifier: MPL-2.0
// Synthetic read-only fixtures; no external network or real credentials.
import http from 'node:http';
import {deflateSync,crc32} from 'node:zlib';
const pngChunk=(type,data)=>{const name=Buffer.from(type);const length=Buffer.alloc(4);length.writeUInt32BE(data.length);const crc=Buffer.alloc(4);crc.writeUInt32BE(crc32(Buffer.concat([name,data])));return Buffer.concat([length,name,data,crc]);};
function raster(width,height){const header=Buffer.alloc(13);header.writeUInt32BE(width,0);header.writeUInt32BE(height,4);header[8]=8;header[9]=6;const pixels=Buffer.alloc(height*(1+width*4));for(let y=0;y<height;y++)for(let x=0;x<width;x++){const i=y*(1+width*4)+1+x*4;pixels[i]=x<width/2?255:0;pixels[i+1]=x>=width/2?255:0;pixels[i+3]=255;}return Buffer.concat([Buffer.from([137,80,78,71,13,10,26,10]),pngChunk('IHDR',header),pngChunk('IDAT',deflateSync(pixels)),pngChunk('IEND',Buffer.alloc(0))]);}
const pixel=raster(2,1),pixel2x=raster(4,2);
const shell = body => `<!doctype html><meta charset="utf-8"><title>Regressee fixture</title><style>body{margin:24px;font:16px Arial;background:white;color:#18232d}h1{font-size:28px}img{margin:8px;border:2px solid #334155}header{position:sticky;top:0;background:#e2e8f0;padding:8px}.background{width:160px;height:80px;background-image:url('/raster.png');background-size:cover}</style>${body}`;
export async function fixtureServer() {
  const sockets = new Set();
  const server = http.createServer((req, res) => {
    const url = new URL(req.url, 'http://127.0.0.1');
    if (url.pathname === '/slow') return; // supervised cancellation fixture
    if (url.pathname === '/raster.png') { res.writeHead(200, {'Content-Type':'image/png'}); return res.end(pixel); }
    if (url.pathname === '/raster2x.png') { res.writeHead(200, {'Content-Type':'image/png'}); return res.end(pixel2x); }
    if (url.pathname === '/vector.svg') { res.writeHead(200, {'Content-Type':'image/svg+xml'}); return res.end('<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24"><circle cx="12" cy="12" r="10" fill="blue"/></svg>'); }
    if (url.pathname === '/robots.txt') return res.end('User-agent: *\nDisallow: /private\n');
    if (url.pathname === '/alias') { res.writeHead(302, {Location:'/public'}); return res.end(); }
    if (url.pathname === '/error') { res.writeHead(500); return res.end('Fixture error'); }
    if (url.pathname === '/expired') { res.writeHead(302, {Location:'/login'}); return res.end(); }
    let body;
    if (url.pathname === '/login') body = `<h1>Fixture sign in</h1><p>This establishes a synthetic read-only session.</p><button id="login">Sign in</button><script>
      login.onclick=async()=>{document.cookie='fixture=ok; SameSite=Lax; path=/';localStorage.setItem('fixture','ok');sessionStorage.setItem('fixture','ok');const r=indexedDB.open('fixture',1);r.onupgradeneeded=()=>r.result.createObjectStore('tokens');r.onsuccess=()=>{const t=r.result.transaction('tokens','readwrite');t.objectStore('tokens').put('ok','token');t.oncomplete=()=>location.href='/private';};};</script>`;
    else if (url.pathname === '/private') body = `<h1 id="status">Needs login</h1><script>const r=indexedDB.open('fixture',1);r.onupgradeneeded=()=>r.result.createObjectStore('tokens');r.onsuccess=()=>{const q=r.result.transaction('tokens').objectStore('tokens').get('token');q.onsuccess=()=>{if(document.cookie.includes('fixture=ok')&&localStorage.getItem('fixture')==='ok'&&sessionStorage.getItem('fixture')==='ok'&&q.result==='ok'){document.getElementById('status').textContent='Signed in';document.documentElement.dataset.auth='ok';}else{document.documentElement.dataset.auth='missing';}};};</script>`;
    else body = `<header>Deterministic public fixture</header><h1>Reference page</h1><a href="/private">Account</a> <a href="/public?meaningful=1&utm_source=test#part">Query link</a><div id="geometry">
      <img id="intrinsic" src="/raster.png"><img id="fit" src="/raster.png" style="width:180px;height:100px;object-fit:contain;object-position:left" loading="lazy">
      <picture><source srcset="/raster.png 1x, /raster2x.png 2x"><img id="picture" src="/raster.png" style="width:120px;height:80px"></picture>
      <img id="vector" src="/vector.svg"><div id="background" class="background"></div></div>
      ${url.pathname === '/tall' ? Array.from({length:100}, (_,i)=>`<section style="height:200px;border-bottom:1px solid #ddd">Generated band ${i}</section>`).join('') : ''}<footer>End of fixture</footer>`;
    res.writeHead(200, {'Content-Type':'text/html', 'Cache-Control':'no-store'}); res.end(shell(body));
  });
  server.on('connection', socket => { sockets.add(socket); socket.on('close', ()=>sockets.delete(socket)); });
  await new Promise(resolve => server.listen(0, '127.0.0.1', resolve));
  return {origin:`http://127.0.0.1:${server.address().port}`, close:async()=>{for(const s of sockets)s.destroy();await new Promise(resolve=>server.close(resolve));}};
}
