#!/usr/bin/env python3
"""Finish generated masters and derive consistent runtime sprites (no v1 pixels)."""
from pathlib import Path
import argparse, hashlib, json, sys
import numpy as np
from PIL import Image, ImageDraw, ImageFilter
from build import components, spread, state_image, save_png, pack, STATES
from validate import nine_slice

ROOT=Path(__file__).resolve().parents[2]
PAD=8
SCALE=4
COLORS={'blue':(35,135,255),'cyan':(30,220,245),'purple':(177,55,245),'green':(55,195,30),'red':(195,22,18),'orange':(255,100,12),'amber':(245,173,30),'silver':(180,191,202)}


def finish_master(im):
    """Repair generated near-opaque interiors and remove faint alpha debris only."""
    a=np.asarray(im.convert('RGBA')).copy();alpha=a[:,:,3]
    alpha[alpha<8]=0
    alpha[alpha>=248]=255
    threshold=max(8,int((alpha>0).sum()*.00003))
    for points,_ in components(alpha>0):
        if len(points)<threshold:
            yy,xx=zip(*points);alpha[yy,xx]=0
    bbox=Image.fromarray(alpha).getbbox()
    if bbox is None:raise ValueError('Empty generated image')
    return Image.fromarray(a).crop(bbox)


def tint(im, colour, border_only=False):
    if not colour:return im
    a=np.asarray(im).copy();rgb=a[:,:,:3].astype(float)
    if colour=='bright':rgb=np.minimum(rgb*1.18+4,255)
    else:
        lum=rgb@np.array([.2126,.7152,.0722])
        target=np.array(COLORS[colour],float)
        # Retain lighting while recolouring painted content; protect gold rims
        # when recolouring blue backgrounds, except rarity-border variants.
        converted=np.minimum(target[None,None,:]*(.2+lum[:,:,None]/150),255)
        if border_only:
            h,w=lum.shape;yy,xx=np.indices((h,w))
            edge=((xx<w*.14)|(xx>w*.86)|(yy<h*.14)|(yy>h*.86)).astype(float)
            # Recolour painted gold on the rim, not a rectangular band of the
            # navy inset. Preserve neutral highlights and the item itself.
            amount=edge*np.clip((rgb[:,:,0]-rgb[:,:,2])/40,0,1)
        else:
            amount=np.clip((rgb[:,:,2]-rgb[:,:,0]+24)/48,0,1)
            if colour=='silver':amount=np.ones(lum.shape)
        rgb=rgb*(1-amount[:,:,None])+converted*amount[:,:,None]
    a[:,:,:3]=np.rint(rgb).astype('uint8');return Image.fromarray(a)


def fitted(im,size):
    out=Image.new('RGBA',size)
    im=im.copy();im.thumbnail(size,Image.Resampling.LANCZOS)
    out.alpha_composite(im,((size[0]-im.width)//2,(size[1]-im.height)//2))
    return out


def resized(im,size,sliceable=False):
    if not sliceable:return fitted(im,size)
    # Downsample uniformly first, then extend only the centre/rails.
    scale=min(size[0]/im.width,size[1]/im.height)
    w=max(12,round(im.width*scale));h=max(12,round(im.height*scale))
    im=im.resize((w,h),Image.Resampling.LANCZOS)
    l=r=max(4,round(w*.18));t=b=max(4,round(h*.23))
    if min(size[0]-l-r,size[1]-t-b)>0:return nine_slice(im,[l,t,r,b],size)
    return im.resize(size,Image.Resampling.LANCZOS)


def safe_border(im):
    out=Image.new('RGBA',(im.width+PAD*2,im.height+PAD*2));out.paste(im,(PAD,PAD))
    a=np.asarray(out).copy();rgb,known=spread(a[:,:,:3],a[:,:,3]>=128,PAD)
    mask=(a[:,:,3]<32)&known;a[:,:,:3][mask]=rgb[mask]
    return Image.fromarray(a)


def derive(name,rule,masters,size,sliceable,state="default"):
    im=masters[rule['master']].copy();op=rule['operation']
    if op=='corner':
        im=im.crop((0,0,round(im.width*.13),round(im.height*.22)))
        # Retain gold ornamental chrome only; navy fill becomes transparent.
        a=np.asarray(im).copy();rgb=a[:,:,:3].astype(float)
        gold=(rgb[:,:,0]>rgb[:,:,2]*1.4)&(rgb[:,:,0]>80)
        expanded=np.asarray(Image.fromarray(gold.astype('uint8')*255).filter(ImageFilter.MaxFilter(3)))
        a[:,:,3]=np.minimum(a[:,:,3],expanded);im=Image.fromarray(a)
        if rule.get('mirror'):im=im.transpose(Image.Transpose.FLIP_LEFT_RIGHT)
    elif op=='divider_plain':
        # Reuse the regenerated painted rail, excluding the central jewel.
        im=im.crop((round(im.width*.16),round(im.height*.4),round(im.width*.35),round(im.height*.6)))
        im=im.resize(size,Image.Resampling.LANCZOS)
    elif op=='fill':
        # Extract a small clean, shaded interior strip; no old source pixels.
        im=im.crop((round(im.width*.3),round(im.height*.35),round(im.width*.7),round(im.height*.65)))
        im=im.resize(size,Image.Resampling.LANCZOS)
        a=np.asarray(im).copy();rgb=a[:,:,:3].astype(float);lum=rgb@np.array([.2126,.7152,.0722])
        lum=np.repeat(np.median(lum,axis=1)[:,None],lum.shape[1],axis=1)
        lo,hi=np.percentile(lum,[3,97]);v=.45+.55*np.clip((lum-lo)/max(hi-lo,1),0,1)
        a[:,:,:3]=np.minimum(v[:,:,None]*np.array(COLORS[rule['tint']]),255).astype('uint8');a[:,:,3]=255
        return Image.fromarray(a)
    im=resized(im,size,sliceable)
    im=tint(im,rule.get('tint'),rule.get('border_only',False))
    im=state_image(im,state)
    if op in ('button_glyph','circle_glyph','notice','hotbar'):
        glyph=masters[rule['glyph']].copy()
        if rule.get('unframe'):
            w,h=glyph.size;glyph=glyph.crop((round(w*.15),round(h*.15),round(w*.85),round(h*.85)))
        if state=='disabled':glyph=state_image(glyph,'disabled')
        if rule.get('rotation'):glyph=glyph.rotate(rule['rotation'],expand=True)
        if op=='notice':
            badge=resized(masters['radio'],(size[1],size[1]),False)
            badge=tint(badge,rule.get('badge_tint'))
            glyph=fitted(glyph,(round(size[1]*.55),round(size[1]*.55)))
            badge.alpha_composite(glyph,((badge.width-glyph.width)//2,(badge.height-glyph.height)//2));im.alpha_composite(badge,(0,0))
        elif op=='hotbar':
            glyph=fitted(glyph,(round(size[0]*.7),round(size[1]*.7)))
            im.alpha_composite(glyph,((size[0]-glyph.width)//2,(size[1]-glyph.height)//2))
            if rule.get('mirror'):im=im.transpose(Image.Transpose.FLIP_LEFT_RIGHT)
        elif rule.get('double'):
            glyph=fitted(glyph,(round(size[0]*.3),round(size[1]*.58)))
            im.alpha_composite(glyph,(round(size[0]*.2),(size[1]-glyph.height)//2));im.alpha_composite(glyph,(round(size[0]*.46),(size[1]-glyph.height)//2))
        else:
            ratio=.34 if rule.get('small') else .56
            glyph=fitted(glyph,(round(size[0]*ratio),round(size[1]*ratio)))
            pos=(round(size[0]*.56),round(size[1]*.56)) if rule.get('small') else ((size[0]-glyph.width)//2,(size[1]-glyph.height)//2)
            im.alpha_composite(glyph,pos)
    return im


def slice_insets(name, rule, size):
    insets=[round(size[0]*.22)+PAD,round(size[1]*.28)+PAD]*2
    if name.startswith('panel.'):
        insets=[round(size[0]*.18)+PAD,round(size[1]*.22)+PAD]*2
    if rule['operation']=='notice':
        insets[0]=size[1]+PAD+4
    return insets


def build(source,output,partial=False):
    mapping=json.loads((source/'mapping.json').read_text())
    old=json.loads((ROOT/'content/ui/sprites-v1/manifest.json').read_text())
    recipes=json.loads((ROOT/'tools/ui_sprites/recipes.json').read_text())['assets']
    needed={v['master'] for v in mapping.values()}|{v['glyph'] for v in mapping.values() if 'glyph' in v}
    missing=sorted(n for n in needed if not (source/'masters'/f'{n}.png').exists())
    if missing and not partial:raise ValueError('Missing generated masters: '+', '.join(missing))
    masters={n:finish_master(Image.open(source/'masters'/f'{n}.png')) for n in sorted(needed) if n not in missing}
    output.mkdir(parents=True,exist_ok=True)
    records={};images={};families={}
    for recipe in recipes:
        name=recipe['name'];rule=mapping[name]
        if rule['master'] not in masters or rule.get('glyph',rule['master']) not in masters:continue
        previous=old['assets'][name+'.default' if recipe.get('states') else name]
        # Source dimensions serve only as a logical layout contract, never pixels.
        logical=[max(12,x-8) for x in previous['size']]
        size=tuple(min(1600,max(48,x*SCALE)) for x in logical)
        sliceable=bool(previous['nine_slice'])
        im=derive(name,rule,masters,size,sliceable)
        clean=safe_border(im)
        inset=None
        if sliceable:
            # Keep the complete corner treatment out of the stretchable centre.
            inset=slice_insets(name,rule,size)
        family={}
        for state in STATES if recipe.get('states') else ['default']:
            key=name+'.'+state if recipe.get('states') else name
            variant=clean if state=='default' else safe_border(derive(name,rule,masters,size,sliceable,state))
            rel='sprites/'+key.replace('.','/')+'.png'
            records[key]={'file':rel,'size':list(variant.size),'logical_size':[v/SCALE for v in variant.size],
                'pixels_per_ui_unit':SCALE,'padding':PAD,'state':state,'kind':recipe['kind'],'nine_slice':inset,
                'min_size':[inset[0]+inset[2]+1,inset[1]+inset[3]+1] if inset else list(variant.size),
                'resize_axis':'horizontal' if rule['operation']=='notice' else ('both' if sliceable else 'uniform'),
                'pivot':[.5,.5],'master':rule['master'],'derivation':rule,'sha256':save_png(variant,output/rel)}
            images[key]=variant;family[state]=key
        if recipe.get('states'):families[name]=family
    atlases=pack(images,output,records)
    manifest={'schema':2,'name':'Theia regenerated UI sprites','version':'3.0.0','alpha':'straight','colour_space':'sRGB','filter':'linear','mipmaps':False,
        'source_count':408,'canonical_count':len(recipes),'regenerated_master_count':len(masters),'pending_masters':missing,
        'assets':records,'families':families,'atlases':atlases,'layers':old['layers'],'pixels_per_ui_unit':SCALE,
        'generation':'Individually generated masters using built-in image_gen; states and variants derived deterministically.'}
    (output/'manifest.json').write_text(json.dumps(manifest,indent=2)+'\n')
    print(json.dumps({'masters':len(masters),'sprites':len(records),'atlases':len(atlases),'missing':len(missing)}))

if __name__=='__main__':
    p=argparse.ArgumentParser();p.add_argument('--source',type=Path,default=ROOT/'build/ui-sprites-regenerated-v3');p.add_argument('--output',type=Path,default=ROOT/'content/ui/sprites-v3');p.add_argument('--partial',action='store_true');a=p.parse_args();build(a.source,a.output,a.partial)
