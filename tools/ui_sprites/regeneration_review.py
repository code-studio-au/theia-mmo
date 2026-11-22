#!/usr/bin/env python3
"""Validate regenerated exports and create source comparisons at equal UI size."""
import argparse, base64, csv, hashlib, json
from pathlib import Path
from PIL import Image, ImageDraw, ImageFont
from validate import validate, nine_slice

ROOT = Path(__file__).resolve().parents[2]

def prepare(root, source):
    jobs = json.loads((source/'jobs.json').read_text())
    if (source/'generation.json').exists():
        # A delivered master archive is self-contained; no earlier build folder
        # is required to recover the three accepted comparison prompts.
        previous = json.loads((source/'generation.json').read_text())
        prompts = [j for j in previous['prompts'] if j.get('origin')=='accepted comparison master']
    else:
        accepted = json.loads((ROOT/'build/ui-sprites-regeneration-v2/prompts.json').read_text())
        aliases = {'panel':'panel.navy','button':'button.primary','party-icon':'icon.party'}
        prompts = [{'name':aliases[j['key']], 'prompt':j['prompt'], 'origin':'accepted comparison master'} for j in accepted]
    assert len(prompts)==3
    prompts += [{'name':j['name'], 'prompt':j['prompt'], 'reference':Path(j['reference']).name, 'style':Path(j['style']).name} for j in jobs]
    records = []
    for job in prompts:
        path = source/'masters'/f"{job['name']}.png"
        records.append(dict(name=job['name'], file='masters/'+path.name, size=list(Image.open(path).size), sha256=hashlib.sha256(path.read_bytes()).hexdigest()))
    (root/'generation.json').write_text(json.dumps(dict(tool='built-in image_gen', source_archive_sha256='7de9cfe741783121df90c37434597fd2f88abb9ce9d621c4bccc207c0206f90b', prompts=prompts, masters=records, finishing='Alpha below 8 removed, alpha >=248 made opaque; isolated alpha specks removed. Painted holes retained. Derived states, colour variants, resize and composites use generated masters only.'), indent=2)+'\n')
    with (ROOT/'content/ui/sprites-v1/source-audit.csv').open() as f:
        rows=list(csv.DictReader(f));fields=list(rows[0])
    for row in rows:
        if row['status']=='exported':row['reason']='Artwork reference only; regenerated replacement mapped by canonical name. No original pixels in v3 runtime sprite.'
    with (root/'source-audit.csv').open('w') as f:
        w=csv.DictWriter(f,fieldnames=fields);w.writeheader();w.writerows(rows)


def review(root, out):
    m, loaded, result = validate(root)
    assert not m['pending_masters']
    assert len(loaded)==416 and len(m['families'])==42
    assert m['pixels_per_ui_unit']==4
    out.mkdir(parents=True,exist_ok=True)
    font=ImageFont.truetype('/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf',13)
    old=json.loads((ROOT/'content/ui/sprites-v1/manifest.json').read_text())
    names=[n for n,a in m['assets'].items() if a['state']=='default']
    for page in range((len(names)+23)//24):
        canvas=Image.new('RGB',(1440,1280),'#132431');d=ImageDraw.Draw(canvas)
        d.text((12,10),'SOURCE / REGENERATED — equal display size; alternating light and dark backgrounds',font=font,fill='white')
        for i,name in enumerate(names[page*24:(page+1)*24]):
            x=i%4*360;y=40+i//4*204
            d.rectangle((x+4,y,x+355,y+170),fill='#e6e2d9' if i%2==0 else '#07101b')
            original=Image.open(ROOT/'content/ui/sprites-v1'/old['assets'][name]['file']).convert('RGBA')
            for col,im in enumerate([original,loaded[name]]):
                logical=m['assets'][name]['logical_size'];scale=min(168/logical[0],155/logical[1],1.0)
                target=tuple(max(1,round(v*scale)) for v in logical)
                im=im.resize(target,Image.Resampling.LANCZOS)
                canvas.paste(im,(x+col*180+(180-im.width)//2,y+(170-im.height)//2),im)
            d.text((x+8,y+177),name,font=font,fill='white')
        canvas.save(out/f'comparison-{page+1:02}.png')
    for page in range((len(m['families'])+11)//12):
        canvas=Image.new('RGB',(1440,1200),'#132431');d=ImageDraw.Draw(canvas)
        for row,family in enumerate(list(m['families'])[page*12:(page+1)*12]):
            d.text((6,row*100+2),family,font=font,fill='white')
            for col,(state,name) in enumerate(m['families'][family].items()):
                im=loaded[name].copy();im.thumbnail((220,65),Image.Resampling.LANCZOS)
                canvas.paste(im,(col*240+(240-im.width)//2,row*100+22),im)
                d.text((col*240+8,row*100+83),state,font=font,fill='#d8bd82')
        canvas.save(out/f'states-{page+1}.png')
    # Render slice borders in logical units; source textures use 4 pixels per unit.
    canvas=Image.new('RGB',(1440,1000),'#e6e2d9')
    for x,y,name,size in [(20,20,'panel.navy',(700,440)),(750,20,'panel.compact-navy',(650,200)),(750,240,'panel.parchment',(650,220)),(20,490,'panel.compact-danger',(400,440)),(440,490,'button.primary.default',(950,70)),(440,580,'button.primary.selected',(950,70)),(440,670,'title.ornate',(950,64)),(440,750,'notice.warning',(950,60)),(440,830,'notice.success',(950,60))]:
        a=m['assets'][name]
        if a.get('resize_axis')=='horizontal':size=(size[0],round(a['logical_size'][1]))
        im=loaded[name].resize(tuple(round(v/4) for v in loaded[name].size),Image.Resampling.LANCZOS)
        inset=[round(v/4) for v in a['nine_slice']]
        im=nine_slice(im,inset,size);canvas.paste(im,(x,y),im)
    canvas.save(out/'nine-slice.png')
    entries=[]
    def uri(path):return 'data:image/png;base64,'+base64.b64encode(path.read_bytes()).decode()
    for name,a in m['assets'].items():
        previous=old['assets'][name]
        entries.append(dict(name=name,original=uri(ROOT/'content/ui/sprites-v1'/previous['file']),regenerated=uri(root/a['file']),oldSize=previous['size'],size=a['size'],logical=a['logical_size'],resize=a['resize_axis']))
    template=Path(__file__).with_name('comparison.template.html').read_text()
    (out/'comparison.html').write_text(template.replace('__SPRITE_DATA__',json.dumps(entries)))
    result['scope']='Asset qualification only; no game UI integration or Windows execution.'
    (root/'validation.json').write_text(json.dumps(result,indent=2)+'\n')
    print(json.dumps(result,indent=2))

if __name__=='__main__':
    p=argparse.ArgumentParser();p.add_argument('--root',type=Path,default=ROOT/'content/ui/sprites-v3');p.add_argument('--source',type=Path,default=ROOT/'build/ui-sprites-regenerated-v3');a=p.parse_args();prepare(a.root,a.source);review(a.root,a.source/'qa')
