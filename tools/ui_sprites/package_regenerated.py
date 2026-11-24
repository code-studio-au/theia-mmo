#!/usr/bin/env python3
"""Package regenerated runtime assets and raw generation masters separately."""
from pathlib import Path
import hashlib, json, shutil, zipfile
from validate import validate
ROOT=Path(__file__).resolve().parents[2]
content=ROOT/'content/ui/sprites-v3'
evidence=ROOT/'build/ui-sprites-regenerated-v3'
m,_,result=validate(content)
assert not m['pending_masters'] and result['sprites']==416
report=json.loads((evidence/'native/report.json').read_text())
assert report['completed'] and report['pages']==10
shutil.copyfile(Path(__file__).with_name('README.regenerated.md'),content/'README.md')
shutil.copyfile(evidence/'native/report.json',content/'native-report.json')
viewer=json.loads((evidence/'viewer-report.json').read_text())
assert viewer['assets']==416 and viewer['matchedDisplaySize'] and viewer['navigation']
shutil.copyfile(evidence/'viewer-report.json',content/'viewer-report.json')

def archive(name, entries):
    output=evidence/(name+'.zip')
    with zipfile.ZipFile(output,'w',compression=zipfile.ZIP_DEFLATED,compresslevel=6) as z:
        for path,rel in sorted(entries,key=lambda e:e[1]):
            info=zipfile.ZipInfo(name+'/'+rel,date_time=(2026,11,22,0,0,0));info.compress_type=zipfile.ZIP_DEFLATED;info.external_attr=0o100644<<16
            z.writestr(info,path.read_bytes())
    with zipfile.ZipFile(output) as z:assert z.testzip() is None
    digest=hashlib.sha256(output.read_bytes()).hexdigest()
    output.with_suffix('.zip.sha256').write_text(digest+'  '+output.name+'\n')
    return dict(path=str(output),bytes=output.stat().st_size,files=len(entries),sha256=digest)

runtime=[(p,p.relative_to(content).as_posix()) for p in content.rglob('*') if p.is_file()]
runtime += [(p,'previews/'+p.name) for p in (evidence/'qa').iterdir() if p.suffix in ('.png','.html')]
runtime += [(p,'previews/native-'+p.name) for p in (evidence/'native').glob('page-*.png')]
masters=[(p,'masters/'+p.name) for p in (evidence/'masters').glob('*.png')]
masters += [(content/'generation.json','generation.json'),(evidence/'mapping.json','mapping.json'),(evidence/'jobs.json','jobs.json')]
print(json.dumps([archive('Theia_UI_Regenerated_Sprites_v3',runtime),archive('Theia_UI_Regenerated_Masters_v3',masters)],indent=2))
