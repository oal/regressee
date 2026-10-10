#!/usr/bin/env python3
"""Validate actual fixture capture bytes, CRCs, metadata and optional freshness."""
import argparse, hashlib, json, pathlib, struct, time, zlib
ROOT=pathlib.Path(__file__).resolve().parents[1]

def png_dimensions(path):
    data=path.read_bytes();assert data[:8]==b'\x89PNG\r\n\x1a\n',f'{path}: not PNG'
    p=8;dimensions=None;ended=False
    while p<len(data):
        assert p+12<=len(data),f'{path}: truncated chunk'
        n=struct.unpack('>I',data[p:p+4])[0];kind=data[p+4:p+8];payload=data[p+8:p+8+n]
        assert p+12+n<=len(data),f'{path}: truncated payload'
        crc=struct.unpack('>I',data[p+8+n:p+12+n])[0]
        assert zlib.crc32(kind+payload)&0xffffffff==crc,f'{path}: bad {kind!r} CRC'
        if kind==b'IHDR':dimensions=list(struct.unpack('>II',payload[:8]))
        p+=12+n
        if kind==b'IEND':assert n==0 and p==len(data),f'{path}: invalid final chunk';ended=True;break
    assert ended and dimensions,f'{path}: missing IEND/IHDR'
    return dimensions,hashlib.sha256(data).hexdigest()

def inspect_captures(since=None):
    results={}
    for name in ['public-before','public-after','replacement','authenticated','tall']:
        image=ROOT/'artifacts'/f'{name}.png';metadata=image.with_suffix('.json')
        if since is not None:assert image.stat().st_mtime>=since and metadata.stat().st_mtime>=since,f'{name}: stale capture evidence'
        dimensions,digest=png_dimensions(image);m=json.loads(metadata.read_text())
        assert dimensions==[m['dimensions']['width'],m['dimensions']['height']],f'{name}: metadata dimensions disagree'
        assert digest==m['sha256'],f'{name}: metadata hash disagrees'
        results[name]={'dimensions':dimensions,'sha256':digest,'warnings':m['warnings']}
    assert results['public-before']['sha256']==results['public-after']['sha256'],'public captures differ'
    assert (ROOT/'artifacts/public-before.png').read_bytes()==(ROOT/'artifacts/public-after.png').read_bytes()
    replacement=json.loads((ROOT/'artifacts/replacement.json').read_text())['replacement']
    assert replacement['count']==4 and replacement['geometryMeasuredElements']>4 and replacement['before']==replacement['after']
    assert results['tall']['dimensions']==[1440,20492]
    return results

if __name__=='__main__':
    parser=argparse.ArgumentParser();parser.add_argument('--tiles',type=pathlib.Path);args=parser.parse_args()
    result={'generated_at_unix':time.time(),'captures':inspect_captures()}
    if args.tiles:
        t=json.loads(args.tiles.read_text());heights=[]
        for path in t['files']:
            dimensions,_=png_dimensions(pathlib.Path(path));assert dimensions[0]==t['width'] and 0<dimensions[1]<=512;heights.append(dimensions[1])
        assert sum(heights)==t['height'] and len(heights)==41 and t['height']==20492
        result['tiles']={'count':len(heights),'height_sum':sum(heights),'maximum_height':max(heights),'generation':str(pathlib.Path(t['files'][0]).parent)}
    (ROOT/'artifacts/corrections-image-evidence.json').write_text(json.dumps(result,indent=2));print(json.dumps(result,indent=2))
