import test from 'node:test';
import assert from 'node:assert/strict';
import {comparable,PROFILE,IMAGE_ALGORITHM,fingerprint} from '../src/capture.mjs';
test('site origin alone is not a rendering mismatch',()=>{
  const base={profile:PROFILE,browser:'141.0.7390.37',mode:'chromium-new-headless',imageAlgorithm:IMAGE_ALGORITHM,os:'linux',arch:'x64',release:'fixture',fonts:['Arial'],origin:'http://production.invalid'};
  assert.equal(IMAGE_ALGORITHM,'neutral-raster-v2');assert.equal(fingerprint({version:()=>base.browser}).imageAlgorithm,IMAGE_ALGORITHM);
  assert.equal(comparable(base,{...base,imageAlgorithm:'neutral-raster-v1'}),'blocked-profile-mismatch');
  assert.equal(comparable(base,{...base,origin:'http://staging.invalid'}),'same-recorded-environment');
  assert.equal(comparable(base,{...base,os:'darwin'}),'caution-rendering-environment');
  assert.equal(comparable(base,{...base,profile:{...PROFILE,deviceScaleFactor:2}}),'blocked-profile-mismatch');
  assert.equal(comparable(base,{...base,imageAlgorithm:'changed'}),'blocked-profile-mismatch');
  assert.equal(comparable(base,{...base,profile:{...PROFILE,replace:true}}),'blocked-profile-mismatch');
});
