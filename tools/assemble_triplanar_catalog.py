"""Combine qualified placed profiles and Standard assets into a bounded catalog.

Only material profile IDs and identity provenance parents change. Ordinary glTF
geometry/materials/images retain their source bytes; external roles share typed,
content-addressed PNG paths. This is an opt-in origin-zero inspector asset.
"""
import argparse
import hashlib
import json
from pathlib import Path
import shutil
import re
from merge_static_glb import read, merge_documents
from add_terrain_house import write_glb


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def build(profiles, standards, output, shared_profiles=False):
    output = Path(output).resolve()
    if output.exists() or not output.is_relative_to(Path(__file__).resolve().parents[1] / 'build'):
        raise ValueError('Use a fresh catalog under ignored build/')
    identifiers = {v[0] for v in profiles}
    if not 1 <= len(profiles) <= 64 or len(identifiers) > 16 or (not shared_profiles and len(identifiers) != len(profiles)) or any(not 1 <= v[0] <= 16 for v in profiles):
        raise ValueError('Use 1..16 distinct positive profile IDs')
    if len({Path(v[1]).resolve() for v in profiles}) != len(profiles):
        raise ValueError('Duplicate profile source stage')
    if len(standards) > 16:
        raise ValueError('Standard input count exceeds inspection budget')
    output.mkdir(parents=True)
    try:
        descriptors = []
        records = []
        document = None
        packed = bytearray()
        images = set()
        decoded = 0
        inputs = [(identifier, Path(path).resolve(strict=True)) for identifier, path in profiles]
        inputs.extend((None, Path(path).resolve(strict=True)) for path in standards)
        for index, (identifier, source) in enumerate(inputs):
            asset = source / 'preview.glb' if identifier is not None else source
            if output.is_relative_to(source) or source.is_relative_to(output):
                raise ValueError('Catalog output overlaps inputs')
            doc, binary = read(asset, allow_external_images=True)
            if doc.get('animations') or doc.get('skins') or len(doc['scenes']) != 1:
                raise ValueError('Catalog inputs must be supported static single scenes')
            if len(packed) + len(binary) > 256 * 1024 * 1024:
                raise ValueError('Catalog geometry exceeds bounded inspection profile')
            record = {'asset': str(asset), 'asset_sha256': sha(asset), 'profile_id': identifier}
            external = []
            for image in doc.get('images', []):
                if 'uri' not in image:
                    continue
                uri = image['uri']
                if 'bufferView' in image or not re.fullmatch(r'shared/[A-Za-z0-9_.-]+\.png', uri):
                    raise ValueError('Unqualified external image path')
                original = (asset.parent / uri).resolve(strict=True)
                if not original.is_relative_to(asset.parent) or original.stat().st_size > 64 * 1024 * 1024:
                    raise ValueError('External image escapes input or exceeds budget')
                destination = f'shared/catalog-{index}-{Path(uri).name}'
                (output / 'shared').mkdir(exist_ok=True)
                shutil.copyfile(original, output / destination)
                external.append({'source_uri': uri, 'uri': destination, 'sha256': sha(original)})
                image['uri'] = destination
            if external:
                record['external_images'] = external
            if identifier is not None:
                metadata = source / 'triplanar.json'
                if metadata.stat().st_size > 65536:
                    raise ValueError('Profile metadata exceeds budget')
                value = json.loads(metadata.read_text())
                if value['schema'] != 1 or value['profile'] != 'synty-triplanar-world-v1' or value['geometry_sha256'] != sha(asset):
                    raise ValueError('Input profile/geometry binding differs')
                proof = json.loads((source / 'independent-verification.json').read_text())
                if not proof['passed'] or proof['asset_sha256'] != sha(asset):
                    raise ValueError('Input source qualification differs')
                if len(doc['materials']) != 1 or doc['materials'][0].get('extras', {}).get('theia_triplanar') != 1:
                    raise ValueError('Input must have one marked material')
                doc['materials'][0]['extras']['theia_triplanar'] = identifier
                descriptor = {'id': identifier, 'profile': value['profile'], 'tiling': value['tiling'], 'falloff': value['falloff'], 'textures': {}}
                for role in ['top', 'side', 'top_normal', 'side_normal']:
                    texture = value['textures'][role]
                    if texture['file'] != f'triplanar-{role}.png':
                        raise ValueError('Unqualified source image path')
                    original = source / texture['file']
                    if original.stat().st_size > 64 * 1024 * 1024 or sha(original) != texture['sha256']:
                        raise ValueError('Input texture hash/size differs')
                    header = original.read_bytes()[:29]
                    if len(header) < 29 or header[:8] != b'\x89PNG\r\n\x1a\n' or header[12:16] != b'IHDR' or header[24] != 8:
                        raise ValueError('Catalog requires bounded eight-bit PNG images')
                    width, height = int.from_bytes(header[16:20], 'big'), int.from_bytes(header[20:24], 'big')
                    if not 1 <= width <= 4096 or not 1 <= height <= 4096:
                        raise ValueError('Catalog image dimensions exceed profile')
                    kind = 'normal' if role.endswith('normal') else 'color'
                    filename = f'triplanar-{kind}-{texture["sha256"]}.png'
                    if filename not in images:
                        decoded += width * height * 4
                        images.add(filename)
                        if len(images) > 16 or decoded > 256 * 1024 * 1024:
                            raise ValueError('Catalog image count/decoded budget exceeded')
                        shutil.copyfile(original, output / filename)
                    descriptor['textures'][role] = dict(texture, file=filename)
                previous = next((v for v in descriptors if v['id'] == identifier), None)
                if previous is not None and previous != descriptor:
                    raise ValueError('Shared profile ID has different material settings')
                if previous is None:
                    descriptors.append(descriptor)
                record.update(metadata=str(metadata), metadata_sha256=sha(metadata), source_proof_sha256=sha(source / 'independent-verification.json'))
            elif any('theia_triplanar' in m.get('extras', {}) for m in doc.get('materials', [])):
                raise ValueError('Standard input unexpectedly contains a profile mark')
            root = len(doc['nodes'])
            doc['nodes'].append({'name': f'Catalog input {index}', 'extras': {'theia_catalog_input': index}, 'children': doc['scenes'][0]['nodes']})
            doc['scenes'][0]['nodes'] = [root]
            document = doc if document is None else merge_documents(document, doc, len(packed), allow_external_images=True)
            packed.extend(binary)
            records.append(record)
        document['buffers'][0]['byteLength'] = len(packed)
        write_glb(document, packed, output / 'preview.glb')
        metadata = {'schema': 2, 'profile': 'synty-triplanar-catalog-v1', 'geometry_sha256': sha(output / 'preview.glb'),
                    'profiles': sorted(descriptors, key=lambda p: p['id']), 'scope': 'bounded origin-zero material catalog inspection; no rebase/runtime adoption'}
        (output / 'triplanar.json').write_text(json.dumps(metadata, indent=2) + '\n')
        receipt = {'schema': 1, 'asset_sha256': metadata['geometry_sha256'], 'metadata_sha256': sha(output / 'triplanar.json'),
                   'inputs': records, 'profiles': len(descriptors), 'unique_images': len(images), 'decoded_bytes': decoded}
        (output / 'catalog-assembly.json').write_text(json.dumps(receipt, indent=2) + '\n')
        return receipt
    except BaseException:
        shutil.rmtree(output)
        raise


if __name__ == '__main__':
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('output', type=Path); p.add_argument('--profile', action='append', nargs=2, metavar=('ID', 'STAGE'), required=True)
    p.add_argument('--standard', action='append', type=Path, default=[])
    p.add_argument('--shared-profiles', action='store_true', help='Permit identical profiles across distinct qualified stages')
    a = p.parse_args(); print(json.dumps(build([(int(n), Path(path)) for n, path in a.profile], a.standard, a.output, a.shared_profiles)))
