#!/usr/bin/env python3
"""Run an actual server and two rendered clients on the caller's display.

Use a private native compositor for headless qualification. Requires Pillow.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import struct
import time
import tempfile

from PIL import Image
from package_world import identity as package_identity
from collision_geometry import mesh_capsule_clearance


def verify_shared_bindings(bindings, directory, loaded):
    """Compare real rendered material handles/settings to bound glTF references."""
    from material_profiles import parse
    from merge_static_glb import read
    from shared_images import filename
    expected = {}
    material_flags = {}
    for x,z in loaded:
        package = directory/f'chunk_{x}_{z}'
        manifest = json.loads((package/'world.manifest.json').read_text())
        assert manifest['schema'] in (2,3)
        descriptors={'shared/'+filename(image):image for image in manifest['shared_images']}
        document,_=read(package/'terrain.glb',allow_external_images=True)
        used={primitive['material'] for mesh in document['meshes'] for primitive in mesh['primitives'] if 'material' in primitive}
        for n in used:
            material=document['materials'][n]
            if parse(material.get('extras')) is not None:
                # The bound extension replaces the glTF fallback material;
                # verify_projected_bindings checks its actual resources below.
                continue
            for owner,key,slot in [(material.get('pbrMetallicRoughness',{}),'baseColorTexture','base_color'),(material,'emissiveTexture','emissive'),(material.get('pbrMetallicRoughness',{}),'metallicRoughnessTexture','metallic_roughness'),(material,'normalTexture','normal'),(material,'occlusionTexture','occlusion')]:
                if key not in owner:continue
                texture=document['textures'][owner[key]['index']]
                path=document['images'][texture['source']]['uri'].removeprefix('../')
                assert path in descriptors
                key=(x,z,slot,path)
                expected[key]=descriptors[path]
                if document.get('extensionsUsed'):
                    extensions=material.get('extensions',{})
                    strength=extensions.get('KHR_materials_emissive_strength',{}).get('emissiveStrength',1)
                    material_flags.setdefault(key,[]).append(('KHR_materials_unlit' in extensions,[v*strength for v in material.get('emissiveFactor',[0,0,0])]))
    actual={(value['root'][0],value['root'][1],value['slot'],value['path']) for value in bindings}
    assert actual==set(expected) and len(actual)>0,'Actual scene material bindings must match bound image paths'
    by_path={}
    for value in bindings:
        key=(*value['root'],value['slot'],value['path']);descriptor=expected[key]
        if key in material_flags:
            assert any(value['unlit']==unlit and max(abs(a-b) for a,b in zip(value['emissive'],emissive))<1e-5 for unlit,emissive in material_flags[key]), 'Actual standard material extension properties differ'
        assert value['srgb']==(descriptor['color_space']=='srgb')
        with (directory/'shared'/filename(descriptor)).open('rb') as image_file:
            header=image_file.read(24)
        assert value['dimensions']==list(struct.unpack('>II',header[16:24])), 'Rendered image dimensions differ from bound PNG'
        by_path.setdefault(value['path'],set()).add(value['handle'])
        s,t,minimum,maximum=descriptor['sampler']
        for field,number in [('address_mode_u',s),('address_mode_v',t)]:
            mode={10497:'Repeat',33071:'ClampToEdge',33648:'MirrorRepeat'}[number]
            assert f'{field}: {mode}' in value['sampler']
        if minimum==9728:assert 'min_filter: Nearest' in value['sampler']
        if maximum==9728:assert 'mag_filter: Nearest' in value['sampler']
    assert all(len(handles)==1 for handles in by_path.values()),'Compatible scene uses must share their actual Image asset'
    assert len({next(iter(handles)) for handles in by_path.values()})==len(by_path),'Incompatible variant paths must retain distinct Image handles'


def verify_projected_bindings(evidence,directory,loaded,render_origin):
    """Actual material uniforms/handles against package-bound source profiles."""
    from collections import Counter
    from decimal import Decimal, localcontext
    from merge_static_glb import read
    from material_profiles import parse, ROLES
    from static_geometry import hierarchy
    expected=Counter();by_signature={};expected_flags=Counter()
    def signature(root,identifier,tiling,falloff,anchor,uris):
        return (tuple(root),identifier,tiling,falloff,tuple(anchor),tuple(uris))
    for owner in loaded:
        directory_owner=directory/f'chunk_{owner[0]}_{owner[1]}'
        document,_=read(directory_owner/'terrain.glb',allow_external_images=True)
        _,nodes=hierarchy(document)
        for node in nodes:
            if 'mesh' not in document['nodes'][node]:continue
            for primitive in document['meshes'][document['nodes'][node]['mesh']]['primitives']:
                material=document.get('materials',[])[primitive['material']] if 'material' in primitive else {}
                flags=material.get('extras',{}).get('theia_render_flags')
                if flags is not None and not all(flags.values()):
                    expected_flags[(flags['cast_shadows'],flags['receive_shadows'])]+=1
                profile=parse(material.get('extras'))
                if profile is None:continue
                key=signature(owner,profile['id'],profile['tiling'],profile['falloff'],profile['source_anchor'],[profile['textures'][role] for role,_ in ROLES])
                expected[key]+=1;by_signature[key]=profile
    if evidence is None:
        assert not expected,'Bound materials require actual renderer evidence'
        return
    actual=Counter();images={};materials={}
    actual_flags=Counter((row['cast_shadows'],row['receive_shadows']) for row in evidence.get('shadow_flags',[]))
    assert actual_flags==expected_flags,'Actual static-world shadow markers differ from source flags'
    assert evidence['render_origin']==render_origin
    for row in evidence['bindings']:
        key=signature(row['root'],row['source_id'],row['tiling'],row['falloff'],row['source_anchor'],[image['uri'] for image in row['textures']])
        assert key in expected,'Unbound rendered projection parameters'
        actual[key]+=1
        shared=key[2:]
        materials.setdefault(shared,set()).add(row['material_handle'])
        with localcontext() as context:
            context.prec=120
            for axis in range(3):
                offset=Decimal.from_float(float(render_origin[axis]))-Decimal.from_float(float(row['source_anchor'][axis]))
                cycles=offset*Decimal(-1 if axis==0 else 1)*Decimal.from_float(row['tiling'])
                phase=float(cycles%Decimal(1))%1
                error=abs(phase-row['phase'][axis]);assert min(error,abs(1-error))<2e-6,'Source texture phase lost global precision'
        assert row['phase'][3]==0 and len(row['textures'])==4
        for image,(_,color) in zip(row['textures'],ROLES):
            assert image['path']==image['uri'].removeprefix('../'),'Actual material image slot path differs'
            assert image['format']==('Rgba8UnormSrgb' if color=='srgb' else 'Rgba8Unorm') and image['mips']==1
            assert all(0<value<=4096 for value in image['dimensions'])
            for field in ['address_mode_u','address_mode_v']:assert f'{field}: Repeat' in image['sampler']
            for field in ['mag_filter','min_filter']:assert f'{field}: Linear' in image['sampler']
            images.setdefault(image['uri'],set()).add(image['handle'])
    assert actual==expected,'Rendered projected primitive coverage differs'
    assert all(len(handles)==1 for handles in images.values()),'Compatible projected uses must share their actual image handle'
    assert all(len(handles)==1 for handles in materials.values()),'Compatible bound profiles must share their actual material handle'
    assert evidence['profiles']==len(materials)


def verify_static_resource_inventory(evidence, directory, loaded):
    """Check actual shared image assets against the entire resident manifest set."""
    from shared_images import filename
    expected={}
    for x,z in loaded:
        manifest=json.loads((directory/f'chunk_{x}_{z}'/'world.manifest.json').read_text())
        for descriptor in manifest['shared_images']:
            path='shared/'+filename(descriptor)
            if path in expected:assert expected[path]==descriptor
            expected[path]=descriptor
    rows=evidence['shared_resources']
    assert len(rows)==len({r['path'] for r in rows})==len({r['handle'] for r in rows})
    assert {r['path'] for r in rows}==set(expected),'Resident shared image variants differ from package dependencies'
    total=0
    for row in rows:
        descriptor=expected[row['path']]
        with (directory/row['path']).open('rb') as image:
            header=image.read(24)
        dimensions=list(struct.unpack('>II',header[16:24]))
        assert row['dimensions']==dimensions and row['mips']==1
        assert row['format']==('Rgba8UnormSrgb' if descriptor['color_space']=='srgb' else 'Rgba8Unorm')
        assert row['cpu_bytes']==dimensions[0]*dimensions[1]*4
        total+=row['cpu_bytes']
    return dict(images=len(rows),base_rgba_cpu_bytes=total,mips=1,
                scope='Actual resident CPU image assets; excludes meshes, render targets, GPU allocation overhead and process memory')


def verify_bound_publication(events, directory, initial, final):
    """Static prop contributors need source-bound geometry evidence, not green-floor pixels."""
    from merge_static_glb import read
    from static_geometry import hierarchy
    from shared_images import filename
    phases = ['prepare_started', 'prepared', 'staged', 'published', 'settled']
    selected = [next(e for e in events if e['phase'] == phase) for phase in phases]
    assert len({e['generation'] for e in selected}) == 1
    assert all(a['elapsed_ms'] <= b['elapsed_ms'] for a,b in zip(selected,selected[1:]))
    assert all(not e['shared_images_truncated'] for e in selected)
    old, new = set(map(tuple,initial)), set(map(tuple,final))
    added, retained = new-old, new&old
    assert added, 'Geometry publication must actually add a package'
    roots = [{tuple(r['id']): r for r in e['roots']} for e in selected]
    assert all(len(r) == len(e['roots']) for r,e in zip(roots,selected))
    assert set(roots[0]) == set(roots[1]) == old
    assert set(roots[2]) == old|added and set(roots[3]) == set(roots[4]) == new
    for owner in retained:
        assert len({r[owner]['entity'] for r in roots}) == 1, 'Retained root entity changed'
    for owner in added:
        document,_ = read(directory/f'chunk_{owner[0]}_{owner[1]}'/'terrain.glb',allow_external_images=True)
        _,matrices = hierarchy(document)
        meshes = sum(len(document['meshes'][node['mesh']]['primitives']) for n,node in enumerate(document['nodes']) if n in matrices and 'mesh' in node)
        assert meshes > 0 and roots[2][owner]['meshes'] == roots[3][owner]['meshes'] == roots[4][owner]['meshes'] == meshes
        assert roots[2][owner]['visibility'] == 'Hidden'
        assert roots[3][owner]['visibility'] == roots[4][owner]['visibility'] == 'Inherited'
    retained_paths = set()
    for x,z in retained:
        manifest=json.loads((directory/f'chunk_{x}_{z}'/'world.manifest.json').read_text())
        retained_paths.update('shared/'+filename(image) for image in manifest.get('shared_images',[]))
    images=[{v['path']:v['handle'] for v in e['shared_images']} for e in selected]
    assert all(len(v)==len(e['shared_images']) for v,e in zip(images,selected))
    for path in retained_paths:
        assert all(path in image for image in images)
        assert len({image[path] for image in images}) == 1, 'Retained image handle changed'



def covered_cell(index, resident, point):
    """Independently derive complete-cell coverage from bound owner envelopes."""
    import math
    cell = tuple(math.floor(v / 128.) for v in point)
    entries = {tuple(entry['id']): entry for entry in index['chunks']}
    if cell not in entries:
        return False
    required = {cell}
    for owner, entry in entries.items():
        bounds = entry.get('bounds')
        if bounds is None:
            continue
        relative = [(owner[a]-cell[a])*128. for a in range(2)]
        def overlaps(axis):
            lo,hi = (relative[axis]+bounds[n][axis] for n in range(2))
            return 0. <= lo <= 128. if lo == hi else lo < 128. and hi > 0.
        if all(overlaps(a) for a in range(2)):
            required.add(owner)
    return required <= set(map(tuple,resident))


def sha256(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def run(output, binaries, references=None, terrain=None, player_assets=None, collision=False, interior=False, full_map=False, map_access=False, transport="renet", world_origin=None, replication_span=None, render_span=None, jump=False, chunk_world=None, client_chunk_world=None, expect_remote_pixels=True, chunk_direction="z", chunk_stream=False, chunk_delay_ms=0, chunk_asset_failure=False, chunk_bounds_failure=False, chunk_image_failure=False, chunk_shared_images=False, staging_root_limit=1, stream_probe="green_floor"):
    assert stream_probe in ('green_floor', 'bound_geometry')
    assert stream_probe == 'green_floor' or (chunk_stream and chunk_shared_images and not any((chunk_asset_failure, chunk_bounds_failure, chunk_image_failure)))
    assert type(staging_root_limit) is int and 1 <= staging_root_limit <= 32
    assert sum(map(bool,[chunk_asset_failure,chunk_bounds_failure,chunk_image_failure])) <= 1
    candidate_failure = chunk_asset_failure or chunk_bounds_failure or chunk_image_failure
    assert not candidate_failure or chunk_stream
    assert not chunk_shared_images or chunk_world
    assert not chunk_world or not any((terrain, references, collision, interior, full_map, map_access, jump, world_origin)), "Initial chunk qualifier uses its own world/poses"
    assert not (chunk_stream and player_assets), 'Human chunk qualification currently covers initial residency'
    assert not chunk_world or transport == "renet", "Chunk worlds require Renet readiness"
    assert expect_remote_pixels or chunk_world, "Offscreen remote checks require an explicit separated chunk fixture"
    assert not client_chunk_world or chunk_world, "A client chunk frame requires a server chunk world"
    assert chunk_direction in ("x", "z") and (chunk_world or chunk_direction == "z")
    assert not chunk_stream or (chunk_world and chunk_direction == "x")
    client_chunk_world = client_chunk_world or chunk_world
    chunk_index = json.loads((chunk_world / 'world.chunks.json').read_text()) if chunk_world else None
    chunk_session = json.loads((chunk_world / 'world.session.json').read_text()) if chunk_world else None
    assert not map_access or (terrain and player_assets and not full_map and not interior and not collision), "Map access needs its own terrain/human scene"
    assert not full_map or (terrain and not collision and not interior), "Full-map smoke needs its own terrain world"
    assert not (references and terrain), "Use separate reference or terrain scenes"
    assert not player_assets or ((terrain or chunk_world) and not references), "Player assets require a content world"
    assert not interior or (terrain and player_assets and not collision), "Interior smoke requires terrain and player assets"
    assert not collision or terrain, "Static collision smoke requires terrain mode"
    suffix = ".exe" if os.name == "nt" else ""
    server_binary = binaries / f"theia-server{suffix}"
    client_binary = binaries / f"theia-client{suffix}"
    output.mkdir(parents=True, exist_ok=True)
    proof = output / "verification.json"
    proof.unlink(missing_ok=True)
    certificate = output / "server-cert.der"
    certificate.unlink(missing_ok=True)
    gate = output / 'start'
    gate.unlink(missing_ok=True)
    capture_gate = output/'capture'
    finish_gate = output/'finish'
    capture_gate.unlink(missing_ok=True);finish_gate.unlink(missing_ok=True)
    client_scene = (["--reference-scene", str(references)] if references else
                    ["--terrain-world", str(terrain)] if terrain else ["--chunk-world", str(client_chunk_world)] if chunk_world else [])
    if player_assets:
        client_scene.extend(["--player-assets", str(player_assets)])
    server_scene = ["--terrain-world", str(terrain)] if terrain else ["--chunk-world", str(chunk_world)] if chunk_world else []
    tickets = tempfile.TemporaryDirectory(prefix="renet-", dir=output) if transport == "renet" else None
    if chunk_stream:
        server_scene.append("--chunk-stream")
    if world_origin is not None:
        server_scene.extend(['--world-origin', ','.join(str(v) for v in world_origin)])
    if replication_span is not None:
        server_scene.extend(['--replication-span', str(replication_span)])
    processes, logs = [], []
    try:
        log = (output / "server.log").open("w")
        logs.append(log)
        server = subprocess.Popen(
            [str(server_binary), "127.0.0.1:0", str(certificate), *server_scene,
             *(["--renet-tickets", tickets.name] if tickets else [])],
            stdout=log, stderr=subprocess.STDOUT,
        )
        processes.append(server)
        deadline = time.monotonic() + 10
        address = None
        while time.monotonic() < deadline:
            assert server.poll() is None, "Server exited before readiness"
            match = re.search(r"listening=(\S+) (?:public_certificate|local_tickets)=", (output / "server.log").read_text())
            if match and (tickets or certificate.exists()):
                address = match.group(1)
                break
            time.sleep(0.05)
        assert address, "Server did not become ready"
        clients = []
        for name, direction in [("a", "z-jump" if jump else "map-inside" if map_access else "z-map" if full_map else "inside" if interior else "house" if collision else "z-gaits" if player_assets else "z"), ("b", "x-jump" if jump else "map-outside" if map_access else "x-map" if full_map else "x-interior" if interior else "x-gaits" if player_assets else "x")]:
            if chunk_world:
                direction = "x-chunks" if chunk_stream else chunk_direction + ('-gaits' if player_assets else '')
            directory = output / name
            directory.mkdir(exist_ok=True)
            (directory / "network.json").unlink(missing_ok=True)
            (directory / 'ready').unlink(missing_ok=True)
            (directory / 'capture-ready').unlink(missing_ok=True)
            log = (directory / "run.log").open("w")
            logs.append(log)
            process = subprocess.Popen(
                [str(client_binary), *client_scene, "--renet-smoke" if tickets else "--network-smoke", address,
                 str(Path(tickets.name) / ("client-1.token" if name == "a" else "client-2.token")) if tickets else str(certificate), str(directory), direction],
                # Gait acceptance below requires application info logs regardless
                # of the caller's ambient logging filter.
                env=dict(os.environ,
                         THEIA_WINDOW_SIZE='1280x720',
                         RUST_LOG=os.environ.get('RUST_LOG', 'info') + ',theia_client=info',
                         THEIA_RENDER_SPAN=str(render_span if render_span is not None else 128),
                         THEIA_SMOKE_START_GATE=str(gate),
                         THEIA_SMOKE_CAPTURE_GATE=str(capture_gate),
                         THEIA_SMOKE_FINISH_GATE=str(finish_gate),
                         THEIA_CHUNK_PREPARE_DELAY_MS=str(chunk_delay_ms if name == "b" else 0)),
                stdout=log, stderr=subprocess.STDOUT,
            )
            clients.append(process)
            processes.append(process)
        # Start both diagnostic routes only after both independently loaded
        # clients have real scene/pipeline readiness. No gameplay protocol change.
        deadline = time.monotonic() + 150
        while not all((output/name/'ready').is_file() for name in ['a','b']):
            assert all(p.poll() is None for p in clients), 'Client failed before scene readiness'
            assert time.monotonic() < deadline, 'Two-client scene readiness timed out'
            time.sleep(.05)
        gate.write_text('start\n')
        deadline = time.monotonic()+180
        while not all((output/name/'capture-ready').is_file() for name in ['a','b']):
            assert all(p.poll() is None for p in clients), 'Client failed before capture readiness'
            assert time.monotonic()<deadline, 'Two-client capture readiness timed out'
            time.sleep(.05)
        capture_gate.write_text('capture\n')
        while not all((output/name/'network.json').is_file() for name in ['a','b']):
            assert all(p.poll() is None for p in clients), 'Client failed before capture completion'
            assert time.monotonic()<deadline, 'Two-client capture completion timed out'
            time.sleep(.05)
        finish_gate.write_text('finish\n')
        for client in clients:
            assert client.wait(timeout=30) == 0, "Client failed; inspect run.log"
        assert server.poll() is None, "Server exited during verification"
        reports = [json.loads((output / name / "network.json").read_text()) for name in ("a", "b")]
        ids = {report["id"] for report in reports}
        assert len(ids) == 2, "Expected distinct server-assigned identities"
        images = []
        ground_checks = []
        for name, report in zip(("a", "b"), reports):
            assert report["completed"] and not report["offline"]
            assert report['final']['version'] == 12
            coordinates = report['coordinates']
            expected_origin = world_origin if world_origin is not None else chunk_index["query_origin"] if chunk_world else [0., 0., 0.]
            assert coordinates['content_origin'] == expected_origin
            assert coordinates['epoch'] == report['final']['epoch'] != 0
            if replication_span is not None:
                assert coordinates['wire_rebases'] > 0
            for global_player in coordinates['global_players']:
                player = next(p for p in report['final']['players'] if p['id'] == global_player['id'])
                relative = [v-o for v,o in zip(global_player['position'], expected_origin)]
                assert max(abs(a-b) for a,b in zip(relative,[*player['xz'][:1],player['y'],player['xz'][1]])) < 1e-5
            assert report["transport"] == transport
            if transport == 'renet':
                metrics = report['transport_metrics']
                assert metrics['snapshots_received'] > 0
                assert 0 <= metrics['rtt_ms'] < 5000
                assert 0 < metrics['max_movement_queue_bytes'] <= 8192
                assert 0 <= metrics['max_event_queue_bytes'] <= 8192
            if jump:
                assert report['jump']['local_peak'] > 1.0 and report['jump']['remote_peak'] > 1.0, 'Actual local and remote rendered jump evidence missing'
            physics = report['physics_backend']
            assert physics['engine'] == 'avian'
            assert physics['revision'] == '1070759b2145dc078abedace91dc7fe78013a45c'
            assert physics['static_colliders'] > 0
            players = {player["id"]: player for player in report["final"]["players"]}
            assert set(players) == ids and report["max_players"] == 2
            assert report["final"]["tick"] > report["initial"]["tick"]
            render_origin = coordinates['render_origin']
            if render_span is not None:
                assert coordinates['render_rebases'] > 0
            assert physics['frame_revision'] == coordinates['render_rebases']
            assert physics['probes'] > 0 and physics['max_ground_error'] < .002
            def content_position(position):
                # Subtract global f64 origins before adding local f32 offsets.
                return [(r-c)+p for r,c,p in zip(render_origin, expected_origin, position)]
            local = content_position(report["local_render_position"])
            authoritative = players[report["id"]]["xz"]
            assert abs(local[1]-players[report["id"]]["y"]) < .01
            assert max(abs(local[0] - authoritative[0]), abs(local[2] - authoritative[1])) < .01
            remote, = report["remote_render_positions"]
            remote = dict(remote, position=content_position(remote["position"]))
            assert remote["id"] in ids - {report["id"]}
            authoritative = players[remote["id"]]["xz"]
            assert max(abs(remote["position"][0] - authoritative[0]),
                       abs(remote["position"][2] - authoritative[1])) < .05
            assert abs(remote['position'][1]-players[remote['id']]['y']) < .05
            pipelines=report['pipelines']
            assert pipelines['pending']==0 and pipelines['failed']==0 and pipelines['compiled']>0 and pipelines['clear_frames']>=3
            if chunk_world:
                entries = sorted(chunk_index['chunks'], key=lambda e: e['id'])
                assert chunk_index['schema'] in (1, 2)
                index_bytes = f"theia-chunk-index-v{chunk_index['schema']}\0".encode() + struct.pack('<IH', chunk_index['chunk_metres'], len(entries))
                for entry in entries:
                    index_bytes += struct.pack('<qq', *entry['id']) + bytes(entry['package'])
                    if chunk_index['schema'] == 2:
                        index_bytes += struct.pack('<dddd', *(0. if v == 0 else v for pair in entry['bounds'] for v in pair))
                assert chunk_session['schema'] in (1, 2)
                domain = b'theia-authoritative-chunk-world-v4\0' if chunk_session['schema'] == 2 else b'theia-authoritative-chunk-world-v3\0'
                world_bytes = domain + hashlib.sha256(index_bytes).digest() + struct.pack('<H', chunk_session['resident_budget'])
                if chunk_session['schema'] == 2:
                    world_bytes += struct.pack('<dddd', *(v for row in chunk_session['world_bounds'] for v in row))
                world_bytes += struct.pack('<H', len(chunk_session['spawns']))
                for spawn in chunk_session['spawns']:
                    world_bytes += struct.pack('<ddd', *spawn)
                world_bytes += struct.pack('<d', chunk_session['fall_plane'])
                initial_ids = sorted(chunk_session['initial_chunks'])
                world_bytes += struct.pack('<H', len(initial_ids))
                for chunk_id in initial_ids:
                    world_bytes += struct.pack('<qq', *chunk_id)
                assert report['transport_metrics']['chunk_ready_reports_sent'] > 0
                assert report['transport_metrics']['chunk_ready_acks_received'] > 0
                assert report['chunk_world']['authority_confirmed']
                sent = report['chunk_world']['report']
                ack = report['chunk_world']['ack']
                assert ack['version'] == 12 and ack['epoch'] == coordinates['epoch']
                assert ack['sequence'] == sent['sequence'] > 0
                expected_ids = report['chunk_world']['offer']['chunks'] if chunk_stream and not candidate_failure else initial_ids
                assert ack['chunks'] == sent['chunks'] == expected_ids
                assert 0 < ack['tick'] <= report['final']['tick']
                assert bytes(report['chunk_world']['identity']) == hashlib.sha256(world_bytes).digest()
                client_index = json.loads((client_chunk_world / 'world.chunks.json').read_text())
                assert coordinates['query_origin'] == client_index['query_origin']
                assert sorted(report['chunk_world']['loaded']) == expected_ids
                assert sorted(report['chunk_world']['roots']) == expected_ids
                if chunk_shared_images:
                    verify_shared_bindings(report['chunk_world']['shared_images'],client_chunk_world,expected_ids)
                    verify_projected_bindings(report['chunk_world'].get('material_profiles'),client_chunk_world,expected_ids,render_origin)
                if chunk_stream:
                    stream = report['chunk_world']['stream']
                    assert report['chunk_world']['offer']['generation'] >= 2 and stream['server_generation'] >= 2
                    assert stream['publications'] == (0 if candidate_failure else 1)
                    assert stream['rejections'] >= 1 if candidate_failure else stream['rejections'] == 0
                    assert not stream['pending'] and not stream['waiting_probe']
                    assert stream['max_staging_roots'] <= staging_root_limit and len(stream['records']) <= 4
                    assert physics['generation'] == physics['probed_generation'] == report['chunk_world']['generation']
                    assert physics['residency_probes'] > 0
                    if not candidate_failure:
                        record, = stream['records']
                        assert record['retained_preserved']
                        retained = sorted(set(map(tuple, initial_ids)) & set(map(tuple, expected_ids)))
                        assert record['ack']['chunks'] == [list(value) for value in retained]
                        assert record['ack']['tick'] <= record['snapshot']['tick']
                    if name == 'b' and chunk_delay_ms and not candidate_failure:
                        assert stream['hidden_remote_frames'] > 0, 'Delayed peer must withhold actors outside its old scene'
                        if chunk_delay_ms >= 2000:
                            snap = stream['records'][0]['snapshot']
                            boundary = (max(value[0] for value in chunk_session['initial_chunks']) + 1) * 128.
                            own = next(p for p in snap['players'] if p['id']==report['id'])
                            other = next(p for p in snap['players'] if p['id']!=report['id'])
                            if stream_probe == 'bound_geometry':
                                own_point = [snap['origin'][a*2]+own['xz'][a] for a in range(2)]
                                other_point = [snap['origin'][a*2]+other['xz'][a] for a in range(2)]
                                assert covered_cell(client_index, initial_ids, own_point)
                                assert not covered_cell(client_index, initial_ids, other_point), 'Ready peer must leave initial complete coverage'
                                assert covered_cell(client_index, expected_ids, other_point)
                            else:
                                assert snap['origin'][0]+own['xz'][0] < boundary
                                assert snap['origin'][0]+other['xz'][0] > boundary, 'Ready peer must progress before delayed publication'
                triangles = []
                shape_profiles = []
                for x, z in expected_ids:
                    profile = json.loads((chunk_world / f'chunk_{x}_{z}' / 'terrain.json').read_text())
                    offset = [x * 128. - expected_origin[0], -expected_origin[1], z * 128. - expected_origin[2]]
                    if profile['schema'] == 4:
                        shape_profiles.append((profile, offset))
                        continue
                    triangles.extend([[[v+a for v,a in zip(vertex,offset)] for vertex in triangle] for triangle in profile['triangles']])
                for player in players.values():
                    gap = mesh_capsule_clearance(triangles, player)
                    for profile, offset in shape_profiles:
                        from schema4_clearance import profile_capsule_clearance
                        local_player = dict(player, xz=[player['xz'][0]-offset[0], player['xz'][1]-offset[2]], y=player['y']-offset[1])
                        gap = min(gap, profile_capsule_clearance(profile, local_player))
                    assert -.001 <= gap <= .02, f'Chunk capsule mesh clearance: {gap}; {player}'
                    ground_checks.append({'client':name,'actor':player['id'],'clearance':gap})
            if terrain:
                profile_bytes=(terrain/'terrain.json').read_bytes()
                manifest_path = terrain / 'world.manifest.json'
                manifest = json.loads(manifest_path.read_text()) if manifest_path.exists() else None
                expected_identity = package_identity(manifest) if manifest else hashlib.sha256(profile_bytes).hexdigest()
                assert bytes(report['terrain_world']).hex() == expected_identity
                assert report['content_package'] == ({'content_id': manifest['content_id'], 'revision': manifest['revision']} if manifest else None)
                profile=json.loads(profile_bytes)
                for player in players.values():
                    if profile['schema'] == 4:
                        from schema4_clearance import profile_capsule_clearance
                        gap = profile_capsule_clearance(profile, player)
                    else:
                        gap = mesh_capsule_clearance(profile['triangles'] + profile.get('walk_surfaces', []), player)
                    assert -.001 <= gap <= .02, f'Capsule mesh clearance: {gap}; {player}'
                    ground_checks.append({'client': name, 'actor': player['id'], 'clearance': gap})
                    if full_map or map_access:
                        assert profile['schema'] in (2, 3, 4)
                        assert all(profile['bounds'][0][i]+.35 <= player['xz'][i] <= profile['bounds'][1][i]-.35 for i in range(2))
                        ground = player['y']-.9
                        for solid in profile['solids']:
                            center, size = solid['center'], solid['size']
                            # Independent capsule-vs-box clearance, including
                            # rounded corners/caps, with 1 mm contact tolerance.
                            dx = max(abs(player['xz'][0]-center[0])-size[0]/2, 0)
                            dz = max(abs(player['xz'][1]-center[2])-size[2]/2, 0)
                            dy = max(center[1]-size[1]/2-(ground+1.45),
                                     ground+.35-(center[1]+size[1]/2), 0)
                            assert dx*dx+dy*dy+dz*dz >= (.35-.001)**2, 'Player overlaps map collision'
                own=players[report['id']]
                initial=next(p for p in report['initial']['players'] if p['id']==report['id'])
                if not collision and not interior and not map_access and not jump and (profile['schema'] < 3 or full_map):
                    assert abs(own['y']-initial['y']) > .02, 'Server height must change on this terrain walk'
            png = output / name / "scene.png"
            image = Image.open(png).convert("RGB")
            assert image.size == (1280, 720)
            if chunk_stream:
                surface = report['chunk_world']['stream']['surface_probe']
                assert surface['present'] == (not candidate_failure)
                x,y = [round(v) for v in surface['viewport']]
                assert 2 <= x < image.width-2 and 2 <= y < image.height-2
                pixels = [image.getpixel((x+dx,y+dy)) for dx in [-1,0,1] for dy in [-1,0,1]]
                if stream_probe == 'green_floor':
                    green = sum(g>r+10 and g>b+10 for r,g,b in pixels)
                    assert green >= 7 if not candidate_failure else green <= 2, f'New floor render probe: {surface}; {pixels}'
                else:
                    verify_bound_publication(report['chunk_world']['stream']['events'],client_chunk_world,initial_ids,expected_ids)
            blue = sum(b > r * 2.5 and b > g * 1.8 for r, g, b in image.getdata())
            red = sum(r > g * 2 and r > b * 2 for r, g, b in image.getdata())
            assert report['local_visible'], 'Local character is hidden at capture'
            assert report['scenery_camera']['distance'] > 1., 'Camera collapsed into local character'
            if references or player_assets:
                expected_environment = int(not player_assets)
                assert report['reference_visuals'] == {'humans': 2, 'wolves': expected_environment, 'houses': expected_environment}
                assert len(image.getcolors(image.width * image.height)) > 256
            else:
                assert blue > 100 and (red > 100 or not expect_remote_pixels), f"Missing pawn pixels: {blue=}, {red=}"
                if not expect_remote_pixels:
                    a, b = players.values()
                    assert sum((x-y)**2 for x,y in zip(a['xz'],b['xz']))**.5 > 4096., "Offscreen exception requires actually separated authoritative actors"
            log = (output / name / "run.log").read_text()
            for marker in ("VUID-", "Validation Error", " ERROR ", "panicked at"):
                audited = log
                if marker == " ERROR " and chunk_stream and chunk_asset_failure:
                    new_ids = [value for value in report['chunk_world']['offer']['chunks'] if value not in initial_ids]
                    assert len(new_ids)==1
                    x,z = new_ids[0]
                    expected = f"ERROR bevy_asset::server: Failed to load asset 'chunk_{x}_{z}/terrain.glb' with asset loader 'bevy_gltf::loader::GltfLoader': invalid glTF file:"
                    allowed = [line for line in log.splitlines() if expected in line]
                    assert len(allowed)==1, 'Expected exactly one deliberate candidate glTF failure'
                    audited = '\n'.join(line for line in log.splitlines() if line not in allowed)
                assert marker not in audited, f"Diagnostic {marker}: inspect {name}/run.log"
            if (references or player_assets) and not (jump and terrain):
                walking = set(re.findall(r'Reference actor (\S+): Walk', log))
                assert len(walking) == 2, 'Both local and remote humans must enter walking playback'
            if chunk_bounds_failure:
                assert 'Chunk candidate rejected: Visual exceeds indexed contributor bounds; active scene retained' in log
            if chunk_image_failure:
                assert 'Chunk candidate rejected: Shared image ' in log and 'failed verification:' in log
            if player_assets:
                evidence = report['human_animation']
                assert len(evidence) == 2 and len({e['owner'] for e in evidence}) == 2
                count = 1 if jump and terrain else 2 if collision or interior or map_access else 3
                assert all(all(n > 0 for n in e['pose_changes'][:count]) for e in evidence), 'Humans need sampled locomotion motion'
                for gait in (['Stand'] if jump and terrain else ['Stand', 'Walk', 'Run']):
                    expected = 1 if (collision or interior or map_access) and gait == 'Run' else 2
                    assert len(set(re.findall(r'Reference actor (\S+): '+gait, log))) == expected
            if map_access and name == 'a':
                access = json.loads((terrain/'access.json').read_text())
                evidence = report['map_access']
                assert evidence['stage'] == 6 and len(evidence['samples']) == 6, 'Complete map entry, wall, exit and re-entry route'
                samples = evidence['samples']
                matrix = access['house_matrix']
                def source_position(player):
                    v = [player['xz'][0],player['y']-.9,player['xz'][1]]
                    return [sum(matrix[k][i]*(v[k]-matrix[k][3]) for k in range(3)) for i in range(3)]
                entered,wall,exited = [source_position(samples[i]) for i in [1,2,4]]
                if 'qualification' in access:
                    checks = access['qualification']
                    assert max(abs(a-b) for a,b in zip(entered,checks['inside'])) < .2
                    axis = checks['wall_axis']
                    assert checks['wall_range'][0] < wall[axis] < checks['wall_range'][1], f'Expected authored wall contact: {wall}'
                    other = 2 if axis == 0 else 0
                    assert abs(wall[other]-checks['inside'][other])<.3
                    assert exited[checks['exit_axis']] > checks['exit_min']
                else:
                    assert abs(entered[0]+1.445)<.2 and abs(entered[2]-2)<.2
                    assert abs(entered[1]-4.11)<.05
                    assert 3.5<wall[0]<5.6 and abs(wall[2]-2)<.3, f'Expected right-wall contact: {wall}'
                    assert exited[2]>17.8
                assert samples[1]['y']>samples[4]['y']+.5
                assert samples[2]['applied_sequence']>samples[1]['applied_sequence']+30
                own=players[report['id']]
                assert sum((a-b)**2 for a,b in zip(own['xz'],access['route'][-1]))**.5<.2, f'Final route goal drift: pose={own["xz"]}, goal={access["route"][-1]}'
                assert report['scenery_camera']['checks']>0
                assert 0<report['scenery_camera']['distance']<12, 'Interior camera must stay below the ceiling'
            if interior and name == 'a':
                evidence=report['interior']
                assert evidence['stage']==7 and evidence['camera_inside']
                entered,exited,wall=evidence['entered'],evidence['exited'],evidence['wall']
                assert entered['xz'][1]<-6.8 and exited['xz'][1]>2.8
                assert entered['y']>exited['y']+2.3
                assert wall['xz'][0]>entered['xz'][0]+2 and abs(wall['xz'][0]+4.9157)<.04
                own=players[report['id']]
                assert abs(own['xz'][0]+8.913)<.15 and abs(own['xz'][1]+7)<.15
                assert own['applied_sequence']>wall['applied_sequence']>entered['applied_sequence']
            if collision or interior:
                assert profile['solids'], 'Expected authored house collision'
                for player in players.values():
                    for solid in profile['solids']:
                        dx=abs(player['xz'][0]-solid['center'][0]); dz=abs(player['xz'][1]-solid['center'][2])
                        assert dx >= solid['size'][0]/2+.35-.001 or dz >= solid['size'][2]/2+.35-.001
                if collision and name == 'a':
                    own=players[report['id']]
                    initial_x=next(p['xz'][0] for p in report['initial']['players'] if p['id']==report['id'])
                    assert abs(own['xz'][0]-initial_x+8) < .8 and abs(own['xz'][1]+3.35) < .001
                    assert report['static_collision'] == {'camera_blocked':True,'camera_restored':True}
            validation_active = 'Insert instance layer "VK_LAYER_KHRONOS_validation"' in log
            if "VK_LAYER_KHRONOS_validation" in os.environ.get("VK_INSTANCE_LAYERS", ""):
                assert validation_active, "Enable VK_LOADER_DEBUG=layer to prove layer activation"
            images.append({"client": name, "blue_pixels": blue, "red_pixels": red,
                           "image_sha256": sha256(png), "gpu": report["gpu"],
                           "validation_active": validation_active})
        if full_map:
            distances = []
            for report in reports:
                initial = next(p for p in report['initial']['players'] if p['id'] == report['id'])
                final = next(p for p in report['final']['players'] if p['id'] == report['id'])
                distances.append(sum((a-b)**2 for a,b in zip(initial['xz'], final['xz']))**.5)
            assert max(distances) > 25, f'Expected travel beyond the old patch: {distances}'
        for player_id in ids:
            poses = [next([*p["xz"],p["y"]] for p in r["final"]["players"] if p["id"] == player_id) for r in reports]
            assert max(abs(a-b) for a, b in zip(*poses)) < .05
        server_log = (output / 'server.log').read_text()
        if chunk_world:
            ready_actors = {int(value) for value in re.findall(r'chunk_ready actor=(\d+)', server_log)}
            assert ready_actors == ids, "Both actual clients must report readiness to their own actor"
        assert 'movement_recovery ' not in server_log, 'Unexpected authoritative movement recovery'
        assert 'panicked at' not in server_log
        result = {"schema": 1, "passed": True, "distinct_ids": sorted(ids),
                  "actual_clients": 2, "images": images, "terrain_clearance": ground_checks,
                  "client_sha256": sha256(client_binary),
                  "server_sha256": sha256(server_binary),
                  "lock_sha256": sha256(Path(__file__).resolve().parents[1] / "Cargo.lock"),
                  "validation_requested": os.environ.get("VK_INSTANCE_LAYERS", ""),
                  "reference_assets": str(references) if references else None,
                  "terrain_world": str(terrain) if terrain else None,
                  "chunk_world": str(chunk_world) if chunk_world else None,
                  "chunk_direction": chunk_direction if chunk_world else None,
                  "chunk_stream": chunk_stream, "chunk_asset_failure":chunk_asset_failure,
                  "staging_root_limit": staging_root_limit,"stream_probe":stream_probe,
                  "chunk_bounds_failure":chunk_bounds_failure,
                  "chunk_image_failure":chunk_image_failure,"chunk_shared_images":chunk_shared_images,
                  "client_chunk_world": str(client_chunk_world) if client_chunk_world else None,
                  "player_assets": str(player_assets) if player_assets else None,
                  "static_collision": collision,
                  "interior": interior,
                  "full_map": full_map,
                  "map_access": map_access,
                  "transport": transport,
                  "render_span":render_span,
                  "jump":jump,
                  "expect_remote_pixels":expect_remote_pixels,
                  "scope": "loopback encrypted transport + two Bevy presentations; no capacity, texture fidelity or latency benchmark"}
        proof.write_text(json.dumps(result, indent=2) + "\n")
        print(json.dumps(result, indent=2))
    finally:
        for process in reversed(processes):
            if process.poll() is None:
                process.terminate()
                try:
                    process.wait(timeout=5)
                except subprocess.TimeoutExpired:
                    process.kill()
                    process.wait()
        for log in logs:
            log.close()
        if tickets:
            tickets.cleanup()


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("output", type=Path)
    parser.add_argument("--binaries", type=Path, default=Path("target/debug"))
    parser.add_argument("--reference-assets", type=Path)
    parser.add_argument("--terrain-world", type=Path)
    parser.add_argument("--chunk-world", type=Path)
    parser.add_argument("--client-chunk-world", type=Path)
    parser.add_argument("--chunk-direction", choices=["x", "z"], default="z")
    parser.add_argument("--offscreen-remote", action="store_true")
    parser.add_argument("--player-assets", type=Path)
    parser.add_argument("--static-collision", action="store_true")
    parser.add_argument("--interior", action="store_true")
    parser.add_argument("--full-map", action="store_true")
    parser.add_argument("--map-access", action="store_true")
    parser.add_argument("--transport", choices=["quinn", "renet"], default="renet")
    parser.add_argument('--world-origin', type=lambda s: [float(v) for v in s.split(',')])
    parser.add_argument('--replication-span', type=float)
    parser.add_argument("--render-span", type=float)
    parser.add_argument("--jump", action="store_true")
    args = parser.parse_args()
    run(args.output.resolve(), args.binaries.resolve(), args.reference_assets.resolve() if args.reference_assets else None, args.terrain_world.resolve() if args.terrain_world else None, args.player_assets.resolve() if args.player_assets else None, args.static_collision, args.interior, args.full_map, args.map_access, args.transport, args.world_origin, args.replication_span, args.render_span, args.jump, args.chunk_world.resolve() if args.chunk_world else None, args.client_chunk_world.resolve() if args.client_chunk_world else None, not args.offscreen_remote, args.chunk_direction)
