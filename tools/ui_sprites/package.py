#!/usr/bin/env python3
"""Validate and package runtime sprites with review evidence; no source ZIP/binaries."""
from pathlib import Path
import hashlib
import json
import shutil
import zipfile
from validate import validate

ROOT = Path(__file__).resolve().parents[2]
content = ROOT/'content/ui/sprites-v1'
evidence = ROOT/'build/ui-sprites-production'
_, _, result = validate(content)
shutil.copyfile(Path(__file__).with_name('README.template.md'), content/'README.md')
(content/'validation.json').write_text(json.dumps(result, indent=2)+'\n')
report = json.loads((evidence/'native/report.json').read_text())
assert report['completed'] and report['pages'] == 10
shutil.copyfile(evidence/'native/report.json', content/'native-report.json')
output = evidence/'Theia_UI_Production_Sprites_v1.zip'
entries = [(p, 'Theia_UI_Production_Sprites_v1/'+p.relative_to(content).as_posix()) for p in content.rglob('*') if p.is_file()]
entries += [(p, 'Theia_UI_Production_Sprites_v1/previews/'+p.name) for p in (evidence/'qa').glob('*.png')]
entries += [(p, 'Theia_UI_Production_Sprites_v1/previews/native-'+p.name) for p in (evidence/'native').glob('page-*.png')]
with zipfile.ZipFile(output, 'w', compression=zipfile.ZIP_DEFLATED, compresslevel=9) as archive:
    for path, name in sorted(entries, key=lambda e:e[1]):
        info = zipfile.ZipInfo(name, date_time=(2026, 11, 22, 0, 0, 0))
        info.compress_type = zipfile.ZIP_DEFLATED
        info.external_attr = 0o100644 << 16
        archive.writestr(info, path.read_bytes())
with zipfile.ZipFile(output) as archive:
    assert archive.testzip() is None
checksum = hashlib.sha256(output.read_bytes()).hexdigest()
output.with_suffix('.zip.sha256').write_text(checksum+'  '+output.name+'\n')
print(json.dumps({'archive':str(output),'bytes':output.stat().st_size,'files':len(entries),'sha256':checksum},indent=2))
