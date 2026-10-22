"""Bounded Unity YAML inspection; inventories dependencies, never executes scripts.

This is a serialization/dependency audit, not an instantiated Unity scene exporter.
Model-prefab internal fileIDs require additional verified model-import information.
"""
import argparse
from collections import Counter
import hashlib
import json
from pathlib import Path
import re

import yaml
from unity_package import asset_path

HEADER = re.compile(r'^--- !u!(\d+) &(-?\d+)( stripped)?[ \t]*$', re.M)
GUID = re.compile(r'[0-9a-f]{32}')
BaseLoader = getattr(yaml, 'CSafeLoader', yaml.SafeLoader)


class UnityLoader(BaseLoader):
    pass


def unique_mapping(loader, node, deep=False):
    result = {}
    for key_node, value_node in node.value:
        key = loader.construct_object(key_node, deep=deep)
        if not isinstance(key, (str, int)) or key in result:
            raise ValueError('Duplicate/unsupported Unity YAML key')
        result[key] = loader.construct_object(value_node, deep=deep)
    return result


UnityLoader.add_constructor(yaml.resolver.BaseResolver.DEFAULT_MAPPING_TAG, unique_mapping)


def load_yaml(text):
    # Unity serialization does not need aliases or object constructors. Reject
    # aliases before loading, including recursive and exponentially expanded data.
    for token in yaml.scan(text, Loader=UnityLoader):
        if isinstance(token, (yaml.tokens.AliasToken, yaml.tokens.AnchorToken)):
            raise ValueError('Unity YAML aliases/anchors are unsupported')
    return yaml.load(text, Loader=UnityLoader)


def records(path):
    path = Path(path)
    if not 1 <= path.stat().st_size <= 128 * 1024 * 1024:
        raise ValueError('Unity document exceeds byte budget')
    text = path.read_text(encoding='utf-8-sig')
    headers = list(HEADER.finditer(text))
    if not headers or len(headers) > 131072:
        raise ValueError('Expected bounded serialized Unity YAML records')
    prefix = text[:headers[0].start()]
    if any(line.strip() and not line.startswith(('%YAML ', '%TAG ', '#')) for line in prefix.splitlines()):
        raise ValueError('Unexpected Unity document prefix')
    result = {}
    for n, header in enumerate(headers):
        body = text[header.end():headers[n+1].start() if n+1 < len(headers) else len(text)]
        value = load_yaml(body)
        if not isinstance(value, dict) or len(value) != 1:
            raise ValueError('Unsupported Unity record body')
        name, fields = next(iter(value.items()))
        identifier = int(header[2])
        if identifier in result or not isinstance(fields, dict):
            raise ValueError('Duplicate/malformed Unity record')
        result[identifier] = {'kind': int(header[1]), 'name': name, 'fields': fields, 'stripped': bool(header[3])}
    return result


class Sources:
    """Hash-bound, contained extracted sources; metadata GUIDs checked on access."""
    def __init__(self, root):
        self.root = Path(root).resolve(strict=True)
        self.manifest = json.loads((self.root / 'extraction.json').read_text())
        self.by_guid = {}
        self.verified = set()
        for name, binding in self.manifest['selected'].items():
            if asset_path(name.encode()) != name or not GUID.fullmatch(binding['guid']):
                raise ValueError('Invalid extracted source binding')
            if binding['guid'] in self.by_guid:
                raise ValueError('Duplicate extracted source GUID')
            self.by_guid[binding['guid']] = name

    def checked(self, name, metadata=False):
        if name not in self.manifest['selected']:
            raise ValueError('Unbound extracted source')
        binding = self.manifest['metadata' if metadata else 'selected'].get(name)
        if binding is None:
            raise ValueError('Missing source metadata binding')
        path = self.root / (name + ('.meta' if metadata else ''))
        if not path.resolve(strict=True).is_relative_to(self.root):
            raise ValueError('Extracted source escapes root')
        if (name, metadata) not in self.verified:
            if path.stat().st_size != binding['bytes'] or hashlib.sha256(path.read_bytes()).hexdigest() != binding['sha256']:
                raise ValueError('Extracted source hash/size mismatch')
            self.verified.add((name, metadata))
        return path

    def metadata(self, name):
        path = self.checked(name, True)
        if path.stat().st_size > 8 * 1024 * 1024:
            raise ValueError('Source metadata exceeds budget')
        value = load_yaml(path.read_text())
        if value.get('guid') != self.manifest['selected'][name]['guid']:
            raise ValueError('Source metadata GUID mismatch')
        return value

    def lfs_pointers(self):
        result = []
        pattern = re.compile(rb'version https://git-lfs.github.com/spec/v1\r?\n'
                             rb'oid sha256:([0-9a-f]{64})\r?\nsize (\d+)\r?\n?')
        for name, binding in self.manifest['selected'].items():
            if binding['bytes'] > 1024:
                continue
            data = self.checked(name).read_bytes()
            match = pattern.fullmatch(data)
            if match:
                result.append({'path': name, 'guid': binding['guid'],
                               'pointer_sha256': binding['sha256'],
                               'oid': match[1].decode(), 'missing_bytes': int(match[2])})
            elif data.startswith(b'version https://git-lfs.github.com/spec/v1'):
                raise ValueError('Malformed source Git LFS pointer')
        return result


def references(value, path=''):
    if isinstance(value, dict):
        if 'guid' in value and 'fileID' in value:
            yield value, path
        for key, item in value.items():
            yield from references(item, path + '.' + str(key))
    elif isinstance(value, list):
        for item in value:
            yield from references(item, path + '[]')


def builtin(guid):
    return guid.startswith('0' * 16)


def audit(root, scene):
    sources = Sources(root)
    if not scene.endswith('.unity'):
        raise ValueError('Select an explicit Unity scene')
    pending, visited, documents = [scene], set(), {}
    missing, builtins = Counter(), Counter()
    missing_contexts = {}
    prefab_edges, model_prefabs, changes, adjustments = [], [], Counter(), Counter()
    mesh_refs, materials = set(), set()
    while pending:
        name = pending.pop()
        if name in visited:
            continue
        visited.add(name)
        path = sources.checked(name)
        sources.metadata(name)
        if path.suffix.lower() not in ('.unity', '.prefab', '.mat'):
            continue
        rr = records(path)
        documents[name] = {'records': len(rr), 'kinds': dict(Counter(str(v['kind']) for v in rr.values()))}
        for identifier, record in rr.items():
            fields = record['fields']
            for ref, property_path in references(fields, record['name']):
                guid = str(ref['guid'])
                if not ref['fileID']:
                    continue
                if not GUID.fullmatch(guid):
                    raise ValueError('Malformed Unity GUID reference')
                if builtin(guid):
                    builtins[guid] += 1
                elif guid not in sources.by_guid:
                    missing[guid] += 1
                    missing_contexts.setdefault(guid, set()).add(property_path)
                else:
                    dependency = sources.by_guid[guid]
                    if dependency not in visited:
                        pending.append(dependency)
            if record['kind'] in (33, 137) and fields.get('m_Mesh', {}).get('fileID'):
                ref = fields['m_Mesh'];mesh_refs.add((ref.get('guid'), ref['fileID']))
            if record['kind'] in (23, 137):
                materials.update(r.get('guid') for r in fields.get('m_Materials', []) if r.get('fileID'))
            if record['kind'] != 1001:
                continue
            source = fields['m_SourcePrefab'];target = sources.by_guid.get(source['guid'])
            edge = {'document': name, 'instance': identifier, 'source': target, 'guid': source['guid']}
            prefab_edges.append(edge)
            modification = fields['m_Modification']
            changes.update(m['propertyPath'] for m in modification.get('m_Modifications', []))
            for key, value in modification.items():
                if key.startswith(('m_Removed', 'm_Added')) and value:
                    adjustments[key] += len(value)
            if target and target.lower().endswith('.fbx'):
                importer = sources.metadata(target).get('ModelImporter', {})
                model_prefabs.append(dict(edge, has_internal_name_table=bool(importer.get('internalIDToNameTable'))))
    scene_doc = documents[scene]
    return {'schema': 1, 'scope': 'hash-bound recursive dependency audit; no scene instantiation or rendering',
            'archive_sha256': sources.manifest['archive_sha256'], 'scene': scene,
            'scene_records': scene_doc['records'], 'scene_kinds': scene_doc['kinds'],
            'visited_assets': len(visited), 'documents': documents,
            'dependency_extensions': dict(Counter(Path(n).suffix.lower() for n in visited)),
            'prefab_edges': prefab_edges, 'model_prefab_edges': model_prefabs,
            'modifications': dict(changes), 'structural_adjustments': dict(adjustments),
            'mesh_references': [{'guid': g, 'fileID': i} for g, i in sorted(mesh_refs, key=str)],
            'material_guids': sorted(materials, key=str),
            'missing_external_guids': dict(missing), 'builtin_guids': dict(builtins),
            'missing_reference_contexts': {g: sorted(paths) for g, paths in missing_contexts.items()},
            'source_lfs_pointers': [dict(p, referenced_by_scene_dependency_closure=p['path'] in visited)
                                    for p in sources.lfs_pointers()],
            'complete_scene_export': False,
            'remaining': ['Resolve model-prefab hashed fileIDs against verified imported model hierarchy',
                          'Instantiate nested prefabs, stripped references, overrides, additions/removals and LODs',
                          'Verify Unity-to-glTF coordinates/materials and qualify native assembled scene',
                          'Author and qualify collision/navigation separately']}


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('source', type=Path)
    parser.add_argument('--scene', required=True)
    parser.add_argument('--report', required=True, type=Path)
    args = parser.parse_args()
    result = audit(args.source, args.scene)
    with args.report.open('x') as output:
        json.dump(result, output, indent=2);output.write('\n')
    print(json.dumps({k: v for k, v in result.items() if k not in ('documents', 'prefab_edges', 'model_prefab_edges', 'mesh_references', 'material_guids')}, indent=2))
