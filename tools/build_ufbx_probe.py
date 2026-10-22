"""Build the pinned, offline-only FBX geometry verifier under ignored build/."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess
import urllib.request

COMMIT='26a482ae66871d7de36eb722aa060bce95bce274'
HASHES={'ufbx.h':'f787be529af577efc04f3e3e735844a08556718dbb613d1211c2b313d72895dc',
        'ufbx.c':'4a956c26a708e40d82ecb0aeaee54da709d5ba7c144c9642174970059bda7e1d'}


def build(directory):
    root=Path(__file__).resolve().parents[1];directory=Path(directory).resolve()
    if not directory.is_relative_to(root/'build'):raise ValueError('Keep verifier dependencies/output under ignored build/')
    directory.mkdir(parents=True,exist_ok=True)
    for name,expected in HASHES.items():
        path=directory/name
        if not path.exists():
            with urllib.request.urlopen(f'https://raw.githubusercontent.com/ufbx/ufbx/{COMMIT}/{name}',timeout=30) as response:
                value=response.read(8*1024*1024+1)
            if len(value)>8*1024*1024 or hashlib.sha256(value).hexdigest()!=expected:raise ValueError('Pinned ufbx download hash mismatch')
            path.write_bytes(value)
        if hashlib.sha256(path.read_bytes()).hexdigest()!=expected:raise ValueError('Cached ufbx source hash mismatch')
    source=root/'tools/qualification/ufbx_geometry.c';output=directory/'ufbx_geometry'
    subprocess.run(['cc','-std=c99','-Wall','-Wextra','-Werror','-I',str(directory),'-c',str(source),'-o',str(directory/'probe.o')],check=True)
    subprocess.run(['cc','-std=c99','-O2','-I',str(directory),'-c',str(directory/'ufbx.c'),'-o',str(directory/'ufbx.o')],check=True)
    subprocess.run(['cc',str(directory/'probe.o'),str(directory/'ufbx.o'),'-lm','-o',str(output)],check=True)
    report={'version':'0.23.1','commit':COMMIT,'source_hashes':HASHES,
            'probe_source_sha256':hashlib.sha256(source.read_bytes()).hexdigest(),
            'compiler':subprocess.check_output(['cc','--version'],text=True).splitlines()[0],
            'binary_sha256':hashlib.sha256(output.read_bytes()).hexdigest(),
            'scope':'offline independent geometry verifier; no Rust/runtime dependency'}
    (directory/'build.json').write_text(json.dumps(report,indent=2)+'\n');return report


if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--directory',type=Path,default=Path(__file__).resolve().parents[1]/'build/ufbx-0.23.1')
    args=parser.parse_args();print(json.dumps(build(args.directory)))
