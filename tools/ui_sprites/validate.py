#!/usr/bin/env python3
"""Validate the exported manifest, alpha safety, state geometry and atlas pixels."""
import argparse,csv,hashlib,json
from pathlib import Path
import numpy as np
from PIL import Image,ImageDraw,ImageFont


def validate(root):
    m=json.loads((root/'manifest.json').read_text());assets=m['assets'];atlases=[]
    for sheet in m['atlases']:
        p=root/sheet['file'];assert hashlib.sha256(p.read_bytes()).hexdigest()==sheet['sha256'],p
        im=Image.open(p);assert im.mode=='RGBA' and list(im.size)==sheet['size'];atlases.append(np.asarray(im))
    occupied=[np.zeros(a.shape[:2],bool) for a in atlases];loaded={}
    for name,a in assets.items():
        p=root/a['file'];assert p.resolve().is_relative_to(root.resolve()),name
        assert hashlib.sha256(p.read_bytes()).hexdigest()==a['sha256'],name
        im=Image.open(p);assert im.mode=='RGBA' and list(im.size)==a['size'],name
        pixels=np.asarray(im);alpha=pixels[:,:,3];pad=a['padding']
        assert alpha.max()==255 and (alpha==0).any(),name
        assert not alpha[:pad].any() and not alpha[-pad:].any() and not alpha[:,:pad].any() and not alpha[:,-pad:].any(),name
        x0,y0,x1,y1=a['rect'];sheet=a['atlas'];atlas=atlases[sheet]
        assert 0<=x0<x1<=atlas.shape[1] and 0<=y0<y1<=atlas.shape[0],name
        assert (x1-x0,y1-y0)==im.size,name
        assert not occupied[sheet][y0:y1,x0:x1].any(),name
        occupied[sheet][y0:y1,x0:x1]=True
        assert np.array_equal(pixels,atlas[y0:y1,x0:x1]),name
        if a['nine_slice']:
            l,t,r,b=a['nine_slice'];assert min(l,t,r,b)>0 and l+r<im.width and t+b<im.height,name
            assert a['min_size']==[l+r+1,t+b+1],name
        loaded[name]=im
    for family,states in m['families'].items():
        assert set(states)=={'default','hover','pressed','selected','focused','disabled'},family
        base=loaded[states['default']]
        for state,name in states.items():
            other=loaded[name]
            assert other.size==base.size and other.getchannel('A').tobytes()==base.getchannel('A').tobytes(),(family,state)
            if state!='default':assert other.tobytes()!=base.tobytes(),(family,state)
    with (root/'source-audit.csv').open() as stream:
        audit=list(csv.DictReader(stream))
    assert len(audit)==m['source_count']==408 and len({a['source_id'] for a in audit})==408
    assert all(a['status'] in {'exported','excluded-label','replaced-composition','superseded-variant'} for a in audit)
    for name,parts in m['layers'].items():
        assert all(p in assets for p in parts),name
    return m,loaded,dict(sprites=len(assets),families=len(m['families']),atlases=len(atlases),audited_sources=len(audit),
        checks=['PNG decode/hash/dimensions','straight RGBA transparent safety border','atlas bounds/nonoverlap/pixel identity',
                'complete six-state families with identical alpha/dimensions','valid nine-slice centres/minimum sizes','all 408 input crops accounted for'])


def nine_slice(im,insets,size):
    l,t,r,b=insets;w,h=size
    assert w>l+r and h>t+b
    srcx=[0,l,im.width-r,im.width];srcy=[0,t,im.height-b,im.height]
    dstx=[0,l,w-r,w];dsty=[0,t,h-b,h]
    out=Image.new('RGBA',size)
    for yy in range(3):
        for xx in range(3):
            p=im.crop((srcx[xx],srcy[yy],srcx[xx+1],srcy[yy+1]))
            p=p.resize((dstx[xx+1]-dstx[xx],dsty[yy+1]-dsty[yy]),Image.Resampling.BILINEAR)
            out.paste(p,(dstx[xx],dsty[yy]))
    return out


def previews(root,out,m,loaded):
    out.mkdir(parents=True,exist_ok=True);font=ImageFont.truetype('/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf',12)
    names=[n for n,a in m['assets'].items() if a['state']=='default']
    for page in range((len(names)+47)//48):
        canvas=Image.new('RGB',(1440,1440),'#182430');d=ImageDraw.Draw(canvas)
        for i,name in enumerate(names[page*48:(page+1)*48]):
            x=i%6*240;y=i//6*180
            d.rectangle((x+4,y+4,x+235,y+147),fill='#e6e2d9' if i%2==0 else '#07101b')
            im=loaded[name].copy();im.thumbnail((224,136),Image.Resampling.LANCZOS)
            canvas.paste(im,(x+(240-im.width)//2,y+5+(138-im.height)//2),im)
            d.text((x+7,y+151),name,font=font,fill='white')
        canvas.save(out/f'finished-{page+1}.png')
    families=list(m['families'])
    for page in range((len(families)+11)//12):
        canvas=Image.new('RGB',(1440,1200),'#132431');d=ImageDraw.Draw(canvas)
        for row,family in enumerate(families[page*12:(page+1)*12]):
            d.text((6,row*100+2),family,font=font,fill='white')
            for col,(state,name) in enumerate(m['families'][family].items()):
                im=loaded[name].copy();im.thumbnail((220,65),Image.Resampling.LANCZOS)
                canvas.paste(im,(col*240+(240-im.width)//2,row*100+22),im)
                d.text((col*240+8,row*100+83),state,font=font,fill='#d8bd82')
        canvas.save(out/f'states-{page+1}.png')
    canvas=Image.new('RGB',(1440,1000),'#142334');d=ImageDraw.Draw(canvas)
    for x,y,name,size in [(20,20,'panel.navy',(700,440)),(750,20,'panel.compact-navy',(650,200)),(750,240,'panel.parchment',(650,220)),(20,490,'panel.compact-danger',(400,440)),(440,490,'button.primary.default',(950,70)),(440,580,'button.primary.selected',(950,70)),(440,670,'title.ornate',(950,64)),(440,750,'notice.warning',(950,60)),(440,830,'notice.success',(950,60))]:
        im=nine_slice(loaded[name],m['assets'][name]['nine_slice'],size);canvas.paste(im,(x,y),im)
        d.text((x+40,y+40 if size[1]>100 else y+20),name,font=font,fill='white' if 'parchment' not in name else 'black')
    canvas.save(out/'nine-slice.png')


def main():
    p=argparse.ArgumentParser();p.add_argument('root',type=Path);p.add_argument('--previews',type=Path);args=p.parse_args()
    m,loaded,result=validate(args.root)
    if args.previews:
        previews(args.root,args.previews,m,loaded)
        (args.previews/'validation.json').write_text(json.dumps(result,indent=2)+'\n')
    print(json.dumps(result,indent=2))
if __name__=='__main__':main()
