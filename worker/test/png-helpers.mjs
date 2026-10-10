// Dependency-free synthetic opaque PNGs and screenshot pixel decoding.
import assert from 'node:assert/strict';
import {crc32,deflateSync,inflateSync} from 'node:zlib';

const chunk=(type,data)=>{const name=Buffer.from(type),size=Buffer.alloc(4),crc=Buffer.alloc(4);size.writeUInt32BE(data.length);crc.writeUInt32BE(crc32(Buffer.concat([name,data])));return Buffer.concat([size,name,data,crc]);};
export function solidRaster(width,height) {
  const header=Buffer.alloc(13);header.writeUInt32BE(width,0);header.writeUInt32BE(height,4);header[8]=8;header[9]=2;
  const rows=Buffer.alloc(height*(1+width*3));
  for(let y=0;y<height;y++)for(let x=0;x<width;x++)rows[y*(1+width*3)+1+x*3]=255;
  return Buffer.concat([Buffer.from([137,80,78,71,13,10,26,10]),chunk('IHDR',header),chunk('IDAT',deflateSync(rows)),chunk('IEND',Buffer.alloc(0))]);
}
export function decodeScreenshot(bytes) {
  assert.deepEqual([...bytes.subarray(0,8)],[137,80,78,71,13,10,26,10]);
  let at=8,header,ended=false;const compressed=[];
  while(at<bytes.length){
    const size=bytes.readUInt32BE(at),body=bytes.subarray(at+4,at+8+size),name=body.subarray(0,4).toString();
    assert.equal(crc32(body),bytes.readUInt32BE(at+8+size));
    if(name==='IHDR')header=body.subarray(4);
    if(name==='IDAT')compressed.push(body.subarray(4));
    at+=size+12;
    if(name==='IEND'){ended=true;break;}
  }
  assert(ended);assert.equal(at,bytes.length);assert(header);
  const width=header.readUInt32BE(0),height=header.readUInt32BE(4),channels=header[9]===2?3:4;
  assert.equal(header[8],8);assert([2,6].includes(header[9]));assert.equal(header[12],0);
  const raw=inflateSync(Buffer.concat(compressed)),stride=width*channels,pixels=Buffer.alloc(height*stride);
  assert.equal(raw.length,height*(stride+1));
  const paeth=(a,b,c)=>{const p=a+b-c,pa=Math.abs(p-a),pb=Math.abs(p-b),pc=Math.abs(p-c);return pa<=pb&&pa<=pc?a:pb<=pc?b:c;};
  for(let y=0;y<height;y++){
    const filter=raw[y*(stride+1)];assert(filter<=4);
    for(let x=0;x<stride;x++){
      const i=y*stride+x,a=x>=channels?pixels[i-channels]:0,b=y?pixels[i-stride]:0,c=y&&x>=channels?pixels[i-stride-channels]:0;
      pixels[i]=(raw[y*(stride+1)+1+x]+[0,a,b,Math.floor((a+b)/2),paeth(a,b,c)][filter])&255;
    }
  }
  return {width,height,pixel:(x,y)=>{assert(x>=0&&x<width&&y>=0&&y<height);const i=(y*width+x)*channels;if(channels===4)assert.equal(pixels[i+3],255);return [...pixels.subarray(i,i+3)];}};
}
