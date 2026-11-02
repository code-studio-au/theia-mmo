#!/usr/bin/env python3
"""Bake the explicitly extracted MeshTint wolf sources into a standard Bevy GLB.

Run using build/blender-tools/bin/python. Never executes Unity package code.
"""
import argparse
import hashlib
import json
from pathlib import Path
import shutil

import bpy
import numpy as np

from convert_synty_locomotion import unique

CLIPS = {
    'Idle': 'Polygonal Wolf@Idle.FBX',
    'Walk': 'Polygonal Wolf@Walk Forward WO Root.FBX',
    'Run': 'Polygonal Wolf@Run Forward WO Root.FBX',
    'Attack': 'Polygonal Wolf@Bite Attack.FBX',
    'Hit': 'Polygonal Wolf@Take Damage.FBX',
    'Death': 'Polygonal Wolf@Die.FBX',
}


def convert(source, output):
    source, output = source.resolve(strict=True), output.resolve()
    if output.exists() or output.is_relative_to(source) or source.is_relative_to(output):
        raise FileExistsError('Output must be new and separate from source')
    inputs = json.loads((source / 'extraction.json').read_bytes())['selected']
    for name, binding in inputs.items():
        path = (source / name).resolve(strict=True)
        if not path.is_relative_to(source) or hashlib.sha256(path.read_bytes()).hexdigest() != binding['sha256']:
            raise ValueError('Source payload changed or escaped extraction directory')
    bpy.ops.wm.read_factory_settings(use_empty=True)
    scene = bpy.context.scene
    bpy.ops.import_scene.fbx(filepath=str(unique(source, 'Polygonal Wolf.FBX')), use_anim=False,
                             automatic_bone_orientation=False, use_custom_props=False)
    rigs = [o for o in bpy.data.objects if o.type == 'ARMATURE']
    meshes = [o for o in bpy.data.objects if o.type == 'MESH']
    if len(rigs) != 1 or len(meshes) != 1:
        raise ValueError('Expected one wolf rig and skin')
    rig, mesh = rigs[0], meshes[0]
    rig.name, mesh.name = 'WolfRig', 'WolfMesh'
    bones = sorted(rig.data.bones, key=lambda b: len(b.parent_recursive))
    if len(bones) != 41 or any(sum(g.weight > 1e-8 for g in v.groups) > 4 for v in mesh.data.vertices):
        raise ValueError('Unexpected wolf rig or skin influence count')
    bpy.context.view_layer.objects.active = rig
    bpy.ops.object.mode_set(mode='EDIT')
    for bone in rig.data.edit_bones:
        bone.use_connect = False
    bpy.ops.object.mode_set(mode='OBJECT')
    bones = sorted(rig.data.bones, key=lambda b: len(b.parent_recursive))
    palette = unique(source, 'Polygonal Wolf Brown.png')
    mat_source = unique(source, 'Polygonal Wolf Brown.mat').read_text()
    palette_meta = json.loads((source / 'extraction.json').read_bytes())['selected']
    palette_binding = palette_meta[str(palette.relative_to(source))]['guid']
    if f'_MainTex:\n        m_Texture: {{fileID: 2800000, guid: {palette_binding}, type: 3}}' not in mat_source:
        raise ValueError('Palette does not match the selected Unity material')
    material = bpy.data.materials.new('WolfBrownStandard')
    material.use_nodes = True
    bsdf = next(n for n in material.node_tree.nodes if n.type == 'BSDF_PRINCIPLED')
    bsdf.inputs['Roughness'].default_value = 1.
    bsdf.inputs['Metallic'].default_value = 0.
    image = material.node_tree.nodes.new('ShaderNodeTexImage')
    image.image = bpy.data.images.load(str(palette))
    image.image.colorspace_settings.name = 'sRGB'
    material.node_tree.links.new(image.outputs['Color'], bsdf.inputs['Base Color'])
    mesh.data.materials.clear()
    mesh.data.materials.append(material)
    # No emission approximation: the Unity glow map is deliberately not adopted.
    bind_vertices = [list(mesh.matrix_world @ v.co) for v in mesh.data.vertices]
    actions, references, report = [], {}, {}
    for name, filename in CLIPS.items():
        before = set(bpy.data.objects)
        bpy.ops.import_scene.fbx(filepath=str(unique(source, filename)), use_anim=True,
                                 automatic_bone_orientation=False, use_custom_props=False)
        added = [o for o in bpy.data.objects if o not in before]
        animated = [o for o in added if o.type == 'ARMATURE']
        if len(animated) != 1:
            raise ValueError('Expected one animation rig')
        animated = animated[0]
        hierarchy = lambda r: {b.name: b.parent.name if b.parent else None for b in r.data.bones}
        if hierarchy(animated) != hierarchy(rig):
            raise ValueError('Animation rig mismatch')
        original = animated.animation_data.action
        first, last = map(lambda f: int(round(f)), original.frame_range)
        fps = scene.render.fps / scene.render.fps_base
        if not 1 <= last - first <= 180 or fps != 30:
            raise ValueError('Unexpected clip timing')
        action = bpy.data.actions.new(name)
        action.use_fake_user = True
        rig.animation_data_create()
        rig.animation_data.action = action
        for pose in rig.pose.bones:
            pose.rotation_mode = 'QUATERNION'
        samples, maximum_error = [], 0.
        object_matrix = animated.matrix_world.copy()
        for frame in range(first, last + 1):
            scene.frame_set(frame)
            if max(abs(animated.matrix_world[i][j] - object_matrix[i][j]) for i in range(4) for j in range(4)) > 1e-6:
                raise ValueError('Clip moves its scene object; unsupported root motion')
            conversion = rig.matrix_world.inverted() @ animated.matrix_world
            desired = {b.name: conversion @ animated.pose.bones[b.name].matrix.copy() for b in bones}
            for bone in bones:
                options = {} if bone.parent is None else {'parent_matrix': desired[bone.parent.name],
                                                          'parent_matrix_local': bone.parent.matrix_local}
                pose = rig.pose.bones[bone.name]
                pose.matrix_basis = bone.convert_local_to_pose(desired[bone.name], bone.matrix_local, invert=True, **options)
                for prop in ('location', 'rotation_quaternion', 'scale'):
                    pose.keyframe_insert(prop, frame=frame)
            bpy.context.view_layer.update()
            wanted = {}
            for bone in bones:
                actual = rig.matrix_world @ rig.pose.bones[bone.name].matrix
                expected = animated.matrix_world @ animated.pose.bones[bone.name].matrix
                error = max(abs(actual[i][j] - expected[i][j]) for i in range(4) for j in range(4))
                maximum_error = max(maximum_error, error)
                if error > .0001:
                    raise ValueError(f'Pose mismatch {name}/{frame}/{bone.name}: {error}')
                wanted[bone.name] = [list(row) for row in expected]
            evaluated = mesh.evaluated_get(bpy.context.evaluated_depsgraph_get())
            vertices = [list(evaluated.matrix_world @ v.co) for v in evaluated.data.vertices]
            samples.append({'time': (frame - first) / fps, 'bones_blender_world': wanted, 'vertices_blender_world': vertices})
        if name in ('Idle', 'Walk', 'Run'):
            a = np.array(samples[0]['bones_blender_world']['RigPelvis'])[:3, 3]
            b = np.array(samples[-1]['bones_blender_world']['RigPelvis'])[:3, 3]
            if np.linalg.norm((a - b)[:2]) > .01:
                raise ValueError('Loop carries horizontal displacement')
        report[name] = {'frames': [first, last], 'fps': fps, 'maximum_world_pose_matrix_error': maximum_error}
        references[name] = samples
        actions.append(action)
        rig.animation_data.action = None
        for obj in added:
            bpy.data.objects.remove(obj, do_unlink=True)
    for action in list(bpy.data.actions):
        if action not in actions:
            bpy.data.actions.remove(action)
    for pose in rig.pose.bones:
        pose.matrix_basis.identity()
    scene.frame_set(0)
    world = mesh.matrix_world.copy()
    mesh.parent = None
    mesh.matrix_world = world
    bpy.ops.object.select_all(action='DESELECT')
    mesh.select_set(True)
    bpy.context.view_layer.objects.active = mesh
    bpy.ops.object.transform_apply(location=True, rotation=True, scale=True)
    output.mkdir(parents=True)
    try:
        bpy.ops.export_scene.gltf(filepath=str(output / 'wolf.glb'), export_format='GLB',
            export_animations=True, export_animation_mode='ACTIONS', export_anim_single_armature=True,
            export_rest_position_armature=True, export_frame_range=False, export_force_sampling=True,
            export_anim_slide_to_zero=True, export_vertex_color='NONE', export_all_vertex_colors=False,
            export_skins=True, export_all_influences=False, export_influence_nb=4,
            export_yup=True, export_cameras=False, export_lights=False, export_extras=False)
        (output / 'source-pose-reference.json').write_text(json.dumps({'bind_vertices_blender_world': bind_vertices, 'clips': references}, separators=(',', ':')))
        result = {'blender': bpy.app.version_string, 'bones': 41, 'clips': report, 'inputs': inputs,
                  'glb_sha256': hashlib.sha256((output / 'wolf.glb').read_bytes()).hexdigest(),
                  'material': 'Brown base colour; metallic 0, roughness 1; emission omitted',
                  'scope': 'Offline wolf conversion, not native playback or gameplay adoption'}
        (output / 'conversion.json').write_text(json.dumps(result, indent=2) + '\n')
        return result
    except BaseException:
        shutil.rmtree(output)
        raise


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('source', type=Path)
    parser.add_argument('output', type=Path)
    args = parser.parse_args()
    print(json.dumps(convert(args.source, args.output), indent=2))
