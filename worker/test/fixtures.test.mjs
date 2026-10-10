import test from 'node:test';
import assert from 'node:assert/strict';
import {crc32,inflateSync} from 'node:zlib';
import {fixtureServer} from '../src/fixtures.mjs';
test('generated raster PNG CRCs and responsive 1x/2x dimensions are valid',async()=>{
  const server=await fixtureServer();
  try{
    for(const [route,width,height] of [['/raster.png',2,1],['/raster2x.png',4,2]]){
      const bytes=Buffer.from(await (await fetch(server.origin+route)).arrayBuffer());
      assert.deepEqual([...bytes.subarray(0,8)],[137,80,78,71,13,10,26,10]);
      assert.equal(bytes.readUInt32BE(16),width);assert.equal(bytes.readUInt32BE(20),height);
      let at=8;const compressed=[];
      while(at<bytes.length){const size=bytes.readUInt32BE(at);const chunk=bytes.subarray(at+4,at+8+size);assert.equal(crc32(chunk),bytes.readUInt32BE(at+8+size));if(chunk.subarray(0,4).toString()==='IDAT')compressed.push(chunk.subarray(4));at+=size+12;}
      assert.equal(inflateSync(Buffer.concat(compressed)).length,height*(1+width*4));
    }
    assert.match(await (await fetch(server.origin+'/public')).text(),/raster\.png 1x, \/raster2x\.png 2x/);
  }finally{await server.close();}
});
