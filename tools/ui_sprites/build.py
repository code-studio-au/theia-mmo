#!/usr/bin/env python3
"""Reproducible, source-preserving Theia sprite finishing and atlas export.

The user explicitly authorised deterministic image processing. Never consumes the
rejected Theia_UI_Kit_v1.zip or contact_sheet.png. Recipes pin the approved ZIP.
"""
from __future__ import annotations
import argparse
from collections import deque
import csv
import hashlib
import io
import json
from pathlib import Path
import shutil
import zipfile
import numpy as np
from PIL import Image, ImageDraw, ImageFilter

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]
PAD = 4
GUTTER = 4
ATLAS_SIZE = 2048
STATES = ('default', 'hover', 'pressed', 'selected', 'focused', 'disabled')


def sha(data):
    return hashlib.sha256(data).hexdigest()


def components(mask):
    """Four-connected components; used for conservative alpha cleanup only."""
    unseen = mask.copy()
    h, w = mask.shape
    for y, x in zip(*np.nonzero(mask)):
        if not unseen[y, x]:
            continue
        unseen[y, x] = False
        todo = deque([(int(y), int(x))])
        points = []
        edge = False
        while todo:
            yy, xx = todo.popleft()
            points.append((yy, xx))
            edge |= yy == 0 or xx == 0 or yy == h-1 or xx == w-1
            for ny, nx in ((yy-1, xx), (yy+1, xx), (yy, xx-1), (yy, xx+1)):
                if 0 <= ny < h and 0 <= nx < w and unseen[ny, nx]:
                    unseen[ny, nx] = False
                    todo.append((ny, nx))
        yield points, edge


def spread(rgb, seeds, steps=6):
    """Bleed nearby solid RGB without premultiplying or modifying alpha."""
    result = rgb.copy()
    known = seeds.copy()
    for _ in range(steps):
        sums = np.zeros((*known.shape, 3), np.float32)
        counts = np.zeros(known.shape, np.float32)
        for dy, dx in ((-1, 0), (1, 0), (0, -1), (0, 1)):
            values = np.roll(result, (dy, dx), (0, 1))
            valid = np.roll(known, (dy, dx), (0, 1))
            if dy == -1: valid[-1] = False
            if dy == 1: valid[0] = False
            if dx == -1: valid[:, -1] = False
            if dx == 1: valid[:, 0] = False
            sums += values * valid[:, :, None]
            counts += valid
        new = (~known) & (counts > 0)
        if not new.any():
            break
        result[new] = np.rint(sums[new] / counts[new, None]).astype(np.uint8)
        known |= new
    return result, known


def gold_hull(rgb, alpha):
    """Convex control outlines only; never applied to free-form ornaments/icons."""
    gold=(alpha>128)&(rgb[:,:,0]>100)&(rgb[:,:,1]>55)&(rgb[:,:,0]>rgb[:,:,1]*1.04)&(rgb[:,:,1]>rgb[:,:,2]*1.4)
    ys,xs=np.nonzero(gold)
    if len(xs)<12 or np.ptp(xs)<rgb.shape[1]*.65 or np.ptp(ys)<rgb.shape[0]*.65:return None
    points=sorted(set(zip(xs.tolist(),ys.tolist())))
    def cross(o,a,b):return (a[0]-o[0])*(b[1]-o[1])-(a[1]-o[1])*(b[0]-o[0])
    lower=[]
    for p in points:
        while len(lower)>=2 and cross(lower[-2],lower[-1],p)<=0:lower.pop()
        lower.append(p)
    upper=[]
    for p in reversed(points):
        while len(upper)>=2 and cross(upper[-2],upper[-1],p)<=0:upper.pop()
        upper.append(p)
    hull=lower[:-1]+upper[:-1]
    mask=Image.new('L',(rgb.shape[1]*4,rgb.shape[0]*4))
    draw=ImageDraw.Draw(mask);poly=[(int(x*4+2),int(y*4+2)) for x,y in hull]
    draw.polygon(poly,fill=255);draw.line(poly+[poly[0]],fill=255,width=3)
    return mask.resize((rgb.shape[1],rgb.shape[0]),Image.Resampling.LANCZOS)


def finish(image, solid=False, chrome=False, convex=False, preserve_rim=False):
    a = np.asarray(image.convert('RGBA')).copy()
    rgb, alpha = a[:, :, :3], a[:, :, 3]
    removed = restored = neutral = 0
    # Remove only white matte connected to the exterior, not enclosed painted
    # highlights. Source PNGs retained the white-sheet RGB after alpha keying.
    white = (rgb.min(axis=2) > 205) & (np.ptp(rgb, axis=2) < 35) & (not preserve_rim)
    matte = 0
    for points, edge in components((alpha < 16) | white):
        if edge:
            yy, xx = zip(*points)
            affected = white[yy, xx] & (alpha[yy, xx] > 0)
            matte += int(affected.sum())
            coords = np.asarray(points)[affected]
            if len(coords):
                alpha[coords[:, 0], coords[:, 1]] = 0
    if convex:
        envelope=gold_hull(rgb, alpha)
        if envelope is not None:
            alpha[:]=np.minimum(alpha,np.asarray(envelope))
    # Remove isolated extraction specks, never larger ornaments/letter strokes.
    for points, _ in components(alpha > 12):
        if len(points) <= 4:
            yy, xx = zip(*points)
            removed += len(points)
            alpha[yy, xx] = 0
    if solid:
        # White-keyed enclosed pinholes are recoverable from retained source RGB.
        # Do not bridge exterior gaps or paint across intentional open icon shapes.
        for points, edge in components(alpha < 16):
            if edge or len(points) > 128:
                continue
            yy, xx = zip(*points)
            bright = rgb[yy, xx].min(axis=1) > 215
            if bright.all():
                alpha[yy, xx] = 255
                restored += len(points)
    if chrome:
        # Only correct pale neutral matting along the outer silhouette. Interior
        # gold highlights, silver/portrait/item detail and dark shadows survive.
        outer = np.asarray(Image.fromarray((alpha > 32).astype('uint8')*255)
                           .filter(ImageFilter.MinFilter(3))) == 0
        neutral_edge = outer & (alpha > 0) & (rgb.min(axis=2) > 175) & (np.ptp(rgb, axis=2) < 26)
        seeds = (alpha > 192) & ~neutral_edge & ((np.ptp(rgb, axis=2) > 30) | (rgb.max(axis=2) < 120))
        colours, known = spread(rgb, seeds, 4)
        replace = neutral_edge & known
        rgb[replace] = colours[replace]
        neutral = int(replace.sum())
    # Correct transparent and translucent edge RGB for bilinear filtering. This
    # deliberately leaves alpha coverage unchanged, avoiding dark premult fringes.
    colours, known = spread(rgb, alpha >= 192, 6)
    replace = (alpha < 32) & known
    rgb[replace] = colours[replace]
    rgb[(alpha == 0) & ~known] = 0
    image = Image.fromarray(a)
    bounds = image.getchannel('A').getbbox()
    if bounds is None:
        raise ValueError('Empty sprite after cleanup')
    image = image.crop(bounds)
    result = Image.new('RGBA', (image.width+PAD*2, image.height+PAD*2))
    result.paste(image, (PAD, PAD))
    # Continue RGB into the transparent safety border for linear sampling.
    arr = np.asarray(result).copy()
    colours, known = spread(arr[:, :, :3], arr[:, :, 3] > 0, PAD)
    arr[:, :, :3][known & (arr[:, :, 3] == 0)] = colours[known & (arr[:, :, 3] == 0)]
    return Image.fromarray(arr), bounds, dict(exterior_matte_pixels_removed=matte, speck_pixels_removed=removed,
            enclosed_highlight_pixels_restored=restored, neutral_matte_pixels_recoloured=neutral)


def state_image(image, state):
    """Deterministic tint variants: exact same geometry, artwork and alpha."""
    arr = np.asarray(image).copy()
    rgb = arr[:, :, :3].astype(np.float32)
    lum = rgb @ np.array([.2126, .7152, .0722], np.float32)
    if state == 'disabled':
        rgb = np.repeat((lum*.58)[:, :, None], 3, 2)
    elif state in ('selected', 'pressed'):
        navy = np.clip((rgb[:,:,2]-rgb[:,:,0]+8)/24,0,1)[:,:,None]
        warmth = np.stack((lum*2.4+30, lum*1.45+16, lum*.35+3), 2)
        rgb = rgb*(1-navy)+warmth*navy
        if state == 'pressed': rgb *= .74
    elif state == 'hover':
        rgb = rgb*1.16 + np.array([4, 5, 6])
    elif state == 'focused':
        rgb = rgb*1.10 + np.array([7, 5, 1])
        # Strong inner gold edge, without an external rectangular focus box.
        gold = (rgb[:, :, 0] > rgb[:, :, 2]*1.5) & (rgb[:, :, 0] > 90)
        rgb[gold] = rgb[gold]*1.18 + np.array([6, 5, 0])
    arr[:, :, :3] = np.clip(np.rint(rgb), 0, 255).astype(np.uint8)
    return Image.fromarray(arr)


def custom_image(image, recipe):
    kind = recipe['kind']
    if 'crop' in recipe:
        image = image.crop(tuple(recipe['crop']))
    if recipe['name'] == 'checkbox.tick':
        arr = np.asarray(image).copy()
        rgb = arr[:, :, :3].astype(float)
        mark = ((rgb[:, :, 0] > 100) & (rgb[:, :, 1] > 70) &
                (rgb[:, :, 0] > rgb[:, :, 2]*1.2)) | (rgb.min(axis=2) > 200)
        mask = Image.fromarray(mark.astype('uint8')*255).filter(ImageFilter.MaxFilter(3))
        arr[:, :, 3] = np.minimum(arr[:, :, 3], np.asarray(mask))
        return Image.fromarray(arr)
    if kind == 'radio-dot':
        # The extracted centre has a keyed white specular highlight at its rim.
        # Restore its retained RGB inside the measured circular centre only.
        arr = np.asarray(image).copy()
        circle = Image.new('L', image.size)
        ImageDraw.Draw(circle).ellipse((0, 0, image.width-1, image.height-1), fill=255)
        mask = np.asarray(circle)
        bright = arr[:, :, :3].min(axis=2) > 215
        arr[:, :, 3] = np.where(bright & (mask > 0), 255, arr[:, :, 3])
        missing = (arr[:, :, 3] < 128) & (mask > 0)
        colours, _ = spread(arr[:, :, :3], arr[:, :, 3] >= 128, 8)
        arr[:, :, :3][missing] = colours[missing]
        arr[:, :, 3] = mask
        # Keep the circular highlight; exterior matte cleanup must not key it.
        return Image.fromarray(arr)
    if kind == 'slider-track':
        # Empty right-hand track; mirror its rounded end to form the left cap.
        right = image.crop((170, 9, 297, 25))
        result = Image.new('RGBA', (240, 16))
        strip = right.crop((12, 0, 13, 16)).resize((220, 16))
        result.paste(strip, (10, 0))
        result.paste(right.crop((117, 0, 127, 16)), (230, 0))
        result.paste(right.crop((117, 0, 127, 16)).transpose(Image.Transpose.FLIP_LEFT_RIGHT), (0, 0))
        return result
    if kind == 'slider-fill':
        return image.crop((20, 10, 100, 24))
    if kind == 'bar-frame':
        result = image.copy()
        alpha = result.getchannel('A')
        ImageDraw.Draw(alpha).polygon([(17, 8), (229, 8), (238, 18), (229, 28), (17, 28), (8, 18)], fill=0)
        result.putalpha(alpha)
        return result
    if kind == 'bar-fill':
        return image.crop((40, 9, 120, 27))
    if kind == 'portrait-frame':
        result = image.copy()
        alpha = result.getchannel('A')
        w, h = image.size
        ImageDraw.Draw(alpha).ellipse((11, 11, w-12, h-12), fill=0)
        result.putalpha(alpha)
        return result
    return image


def save_png(image, path):
    path.parent.mkdir(parents=True, exist_ok=True)
    image.save(path, optimize=False, compress_level=9)
    return sha(path.read_bytes())


def pack(images, output, records):
    """Stable shelf packing; 4 px gutter plus per-sprite 4 px alpha-safe padding."""
    sheets = []
    x = y = row_h = 0
    sheet = Image.new('RGBA', (ATLAS_SIZE, ATLAS_SIZE))
    for name, im in sorted(images.items(), key=lambda it: (-it[1].height, -it[1].width, it[0])):
        w, h = im.size
        if w+2*GUTTER > ATLAS_SIZE or h+2*GUTTER > ATLAS_SIZE:
            raise ValueError('Sprite exceeds atlas size: '+name)
        if x+w+2*GUTTER > ATLAS_SIZE:
            x = 0; y += row_h; row_h = 0
        if y+h+2*GUTTER > ATLAS_SIZE:
            sheets.append(sheet); sheet = Image.new('RGBA', (ATLAS_SIZE, ATLAS_SIZE))
            x = y = row_h = 0
        px, py = x+GUTTER, y+GUTTER
        sheet.paste(im, (px, py))
        # Extrude the outer padded texels 2px; alpha remains transparent.
        sheet.paste(im.crop((0, 0, w, 1)).resize((w, 2)), (px, py-2))
        sheet.paste(im.crop((0, h-1, w, h)).resize((w, 2)), (px, py+h))
        sheet.paste(im.crop((0, 0, 1, h)).resize((2, h)), (px-2, py))
        sheet.paste(im.crop((w-1, 0, w, h)).resize((2, h)), (px+w, py))
        records[name]['atlas'] = len(sheets)
        records[name]['rect'] = [px, py, px+w, py+h]
        x += w+2*GUTTER; row_h = max(row_h, h+2*GUTTER)
    sheets.append(sheet)
    return [dict(file=f'atlases/ui-{i:02}.png', size=list(s.size),
                 sha256=save_png(s, output/f'atlases/ui-{i:02}.png')) for i, s in enumerate(sheets)]


def build(source_zip, output, recipes):
    source_bytes = source_zip.read_bytes()
    if sha(source_bytes) != recipes['source_archive_sha256']:
        raise ValueError('Input is not the approved Final Sprite Package; refusing substitution')
    if output.exists() and any(output.iterdir()):
        if not (output/'manifest.json').is_file() or json.loads((output/'manifest.json').read_text()).get('name') != 'Theia production sprites':
            raise ValueError('Refusing to replace an unrelated nonempty output directory')
        shutil.rmtree(output)
    output.mkdir(parents=True, exist_ok=True)
    with zipfile.ZipFile(io.BytesIO(source_bytes)) as archive:
        prefix = 'Theia_UI_Cleaned_V6/'
        original = json.loads(archive.read(prefix+'manifest.json'))
        sources = {a['id']:a for a in original['assets']}
        images = {}
        for sid, a in sources.items():
            im = Image.open(io.BytesIO(archive.read(prefix+a['file'])))
            im.load()
            if im.mode != 'RGBA' or list(im.size) != a['dimensions']:
                raise ValueError('Unexpected source format: '+sid)
            images[sid] = im
    exported = {}; records = {}; families = {}; used = {}
    for recipe in recipes['assets']:
        sid, base = recipe['source'], recipe['name']
        used.setdefault(sid, []).append(base)
        raw = custom_image(images[sid], recipe)
        chrome = recipe['kind'] not in ('item', 'portrait', 'ability', 'icon', 'crest')
        # Cut-out frames must remain open even if their source was solid.
        solid = recipe.get('solid', False) and recipe['kind'] not in ('bar-frame', 'portrait-frame')
        clean, trim, metrics = finish(raw, solid, chrome, recipe['kind'] in ('panel','button','slot','checkbox','radio'), recipe['kind'] == 'radio-dot')
        inset = recipe.get('slice')
        if recipe['kind'] == 'slider-track': inset = [10, 5, 10, 5]
        if recipe['kind'] in ('slider-fill', 'bar-fill'): inset = [1, 1, 1, 1]
        if recipe['kind'] == 'bar-frame': inset = [22, 12, 22, 12]
        if inset:
            l,t,r,b = inset; x0,y0,x1,y1 = trim
            inset = [max(PAD,l-x0+PAD),max(PAD,t-y0+PAD),max(PAD,r-(raw.width-x1)+PAD),max(PAD,b-(raw.height-y1)+PAD)]
            if inset[0]+inset[2]>=clean.width or inset[1]+inset[3]>=clean.height:
                raise ValueError('Invalid nine-slice insets: '+base)
        variants = STATES if recipe.get('states') else ('default',)
        family = {}
        for state in variants:
            name = base+'.'+state if recipe.get('states') else base
            im = state_image(clean, state)
            path = 'sprites/'+name.replace('.', '/')+'.png'
            records[name] = dict(file=path, source=sid, source_rect=recipe.get('crop'),
                source_size=list(images[sid].size), size=list(im.size), trim=list(trim), padding=PAD,
                kind=recipe['kind'], state=state, pivot=[.5,.5],
                nine_slice=inset, min_size=[inset[0]+inset[2]+1,inset[1]+inset[3]+1] if inset else list(im.size),
                resizing='nine_slice' if inset else 'uniform_only',
                state_derivation='canonical artwork with deterministic RGB-only tint; alpha and silhouette identical' if state!='default' else 'source-faithful cleanup',
                cleanup=metrics, sha256=save_png(im, output/path))
            exported[name]=im; family[state]=name
        if recipe.get('states'): families[base]=family
    atlases = pack(exported, output, records)
    audit=[]
    for sid, src in sources.items():
        sheet, number = src['sheet'], int(sid.split('-')[1])
        names=used.get(sid,[])
        if names:
            status='exported';reason='Named, cleaned asset(s); see recipe and per-asset metrics.'
        elif number in recipes['labels'][str(sheet)]:
            status='excluded-label';reason='Source-sheet caption/heading, not runtime artwork; use live text.'
        elif number in recipes['demos'][str(sheet)]:
            status='replaced-composition';reason='Merged controls, sample text/data or demonstration; compose named runtime assets and live text.'
        else:
            status='superseded-variant';reason='Alternate source crop/state; canonical family supplies aligned, consistent variants.'
        audit.append(dict(source_id=sid,source_file=src['file'],status=status,runtime_assets=';'.join(names),reason=reason))
    with (output/'source-audit.csv').open('w',newline='') as f:
        writer=csv.DictWriter(f,fieldnames=list(audit[0]));writer.writeheader();writer.writerows(audit)
    manifest=dict(schema=1,name='Theia production sprites',version='1.0.0',
        source_archive_sha256=sha(source_bytes),source_count=len(sources),
        alpha='straight',colour_space='sRGB',filter='linear',mipmaps=False,
        atlas_gutter=GUTTER,atlas_extrusion=2,atlases=atlases,
        families=families,assets=records,
        layers={'checkbox':['checkbox.default','checkbox.tick'],
                'radio':['radio.default','radio.dot'],
                'slider':['slider.track','slider.fill','slider.thumb'],
                'bar':['bar.fill-health','bar.frame']},
        constraints=['Live labels/values must be rendered separately.',
                     'Use nine-slice metadata; never stretch icon or portrait artwork independently on X/Y.',
                     'Native raster detail is preserved, not regenerated or artificially upscaled.',
                     'Character portraits are artwork choices, not account/player data.',
                     'Boolean overlays are centred within the same base control; disabled tint also applies to overlay.',
                     'Clip slider/bar fill dynamically to the value, then draw frame/thumb above it.'])
    (output/'manifest.json').write_text(json.dumps(manifest,indent=2)+'\n')
    (output/'recipes.json').write_text(json.dumps(recipes,indent=2)+'\n')
    return manifest,audit


def main():
    p=argparse.ArgumentParser();p.add_argument('--source',type=Path,required=True)
    p.add_argument('--output',type=Path,default=ROOT/'content/ui/sprites-v1')
    args=p.parse_args();recipes=json.loads((HERE/'recipes.json').read_text())
    manifest,audit=build(args.source,args.output,recipes)
    print(json.dumps(dict(sprites=len(manifest['assets']),families=len(manifest['families']),
        atlases=len(manifest['atlases']),source_crops_audited=len(audit)),indent=2))

if __name__=='__main__':main()
