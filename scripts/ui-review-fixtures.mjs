// SPDX-License-Identifier: MPL-2.0
// Local, synthetic website pixels for the opt-in native UI critique matrix.
// No requests, cookies, user projects, or production application settings.
import { createRequire } from 'node:module';
import { mkdir, writeFile } from 'node:fs/promises';
import path from 'node:path';

const root = process.env.REGRESSEE_RENDERER_CAPTURE_DIR;
if (!root || !path.isAbsolute(root)) throw new Error('Explicit capture directory required');
const require = createRequire(path.resolve(process.env.REGRESSEE_REVIEW_WORKER || 'worker', 'package.json'));
const { chromium } = require('playwright');
await mkdir(path.join(root, 'website-fixtures'), { recursive: true, mode: 0o700 });
const browser = await chromium.launch({ headless: true, channel: 'chromium' });
try {
  for (const [profile, width, height] of [['desktop', 1440, 900], ['mobile', 390, 844]]) {
    for (const revision of ['before', 'after']) {
      const page = await browser.newPage({ viewport: { width, height }, deviceScaleFactor: 1 });
      await page.route('**/*', route => route.abort());
      const changed = revision === 'after';
      await page.setContent(`<!doctype html><html lang="en"><head><meta charset="utf-8"><title>Northline — synthetic screenshot fixture</title><style>
      *{box-sizing:border-box}body{margin:0;color:#233b32;background:#fcfaf5;font:16px/1.6 Arial,sans-serif}header{height:88px;border-bottom:1px solid #d6dccf;display:flex;align-items:center;justify-content:space-between;padding:0 6vw}b.brand{font-size:27px;letter-spacing:3px}nav{display:flex;gap:32px}main{max-width:1180px;margin:auto;padding:52px 32px}.eyebrow{font-size:12px;letter-spacing:2px;font-weight:bold}h1{font:64px/1.05 Georgia,serif;max-width:620px;margin:20px 0}p{max-width:520px;color:#566358}.hero{display:grid;grid-template-columns:1fr 1fr;gap:44px;align-items:center}.art{height:420px;background:#dde5d7;position:relative;overflow:hidden}.art:before{content:'';position:absolute;background:#789581;width:65%;height:90%;border-radius:45% 45% 0 0;left:18%;top:30%}.art:after{content:'';position:absolute;background:#bac9af;width:30%;height:62%;border-radius:55% 55% 0 0;left:57%;top:8%;transform:rotate(-18deg)}.button{display:inline-block;background:${changed ? '#a45131' : '#233b32'};color:white;padding:13px 24px;margin-top:12px;font-weight:bold}section{margin-top:56px;border-top:1px solid #cfd7cb;padding-top:28px}h2{font:32px Georgia,serif}.grid{display:grid;grid-template-columns:repeat(3,1fr);gap:28px}.product{background:#eeeae0;height:220px;position:relative}.product:before{content:'';position:absolute;width:44%;height:75%;left:28%;top:12%;border-radius:14px 14px 2px 2px;background:#708274}.grid article:nth-child(2) .product:before{background:#b69570}.grid article:nth-child(3) .product:before{background:#4c6471}h3{margin-bottom:0}.price{margin-top:2px}.story{display:grid;grid-template-columns:1fr 1fr;gap:48px}.story p{margin-bottom:40px}footer{padding:40px 6vw;border-top:1px solid #cfd7cb;color:#566358;font-size:13px}.mock{font-size:11px;letter-spacing:1px}@media(max-width:600px){header{height:70px;padding:0 24px}nav{gap:12px;font-size:12px}nav span:nth-child(2){display:none}b.brand{font-size:20px}main{padding:28px 24px}h1{font-size:44px}.hero{grid-template-columns:1fr;gap:28px}.art{height:300px}.grid{grid-template-columns:1fr}.product{height:260px}.story{grid-template-columns:1fr;gap:0}section{margin-top:36px}}
      </style></head><body><header><b class="brand">NORTHLINE</b><nav><span>Shop</span><span>Our story</span><span>Journal</span><span>Bag (0)</span></nav></header><main><div class="hero"><div><div class="eyebrow">AUTUMN / 2026</div><h1>${changed ? 'Made for the long way home.' : 'For the places you go.'}</h1><p>Considered essentials for everyday adventures. Natural materials, practical details, and a little room to roam.</p><span class="button">${changed ? 'Explore the autumn collection' : 'Discover the collection'}</span></div><div class="art" aria-label="Synthetic illustration"></div></div><section><h2>Everyday favourites</h2><div class="grid">${[['Field jacket','1,490 kr'],['Weekend tote','690 kr'],['Coastal overshirt','990 kr']].map(([name,price]) => `<article><div class="product"></div><h3>${name}</h3><p class="price">${price}</p></article>`).join('')}</div></section><section class="story"><h2>Fewer things.<br>Better stories.</h2><div><p>We make useful pieces that stay with you. Explore our materials, meet the people behind our products, and learn how to care for what you own.</p><p>Designed in Bergen. Ready for wherever the day takes you.</p></div></section></main><footer>Shipping & returns · Care guide · Contact<br><span class="mock">SYNTHETIC WEBSITE — UI REVIEW FIXTURE, NOT A CUSTOMER SITE</span></footer></body></html>`);
      await page.screenshot({ path: path.join(root, 'website-fixtures', `${profile}-${revision}.png`), fullPage: true });
      await page.close();
    }
  }
  await writeFile(path.join(root, 'website-fixtures', 'provenance.json'), JSON.stringify({ synthetic: true, websiteVisited: false, externalRequestsAllowed: false, source: 'scripts/ui-review-fixtures.mjs', profiles: ['1440×900', '390×844'], revisions: ['before', 'after'], imagery: 'CSS-drawn synthetic illustrations', purpose: 'Native app critique only' }, null, 2));
} finally { await browser.close(); }
