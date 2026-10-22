#!/usr/bin/env python3
"""Bake selected Synty FBX poses onto the sample skin and export standard GLB.

Run with the isolated Blender Python environment. Inputs are explicit extracted
source files; Unity scripts/controllers are never executed. No game defaults change.
"""
import argparse
import hashlib
import json
import math
from pathlib import Path
import shutil

import bpy
import numpy as np

CLIPS = {'Idle': 'A_Idle_Standing_Masc.fbx', 'Walk': 'A_Walk_F_Masc.fbx',
         'Run': 'A_Run_F_Masc.fbx', 'Jump': 'A_Jump_Idle_Masc.fbx'}


def unique(root, name):
    paths = list(root.rglob(name))
    if len(paths) != 1:
        raise ValueError(f'Expected one explicit source {name}')
    return paths[0]


def convert(source, output, four_weight_preview=False):
    source, output = source.resolve(strict=True), output.resolve()
    if output.exists() or output.is_relative_to(source):
        raise FileExistsError('Conversion output must be new and outside source')
    inputs = json.loads((source / 'extraction.json').read_bytes())['selected']
    for name, binding in inputs.items():
        if hashlib.sha256((source / name).read_bytes()).hexdigest() != binding['sha256']:
            raise ValueError('Extracted source changed')
    bpy.ops.wm.read_factory_settings(use_empty=True)
    scene = bpy.context.scene
    scene.render.fps, scene.render.fps_base = 30, 1.
    bpy.ops.import_scene.fbx(filepath=str(unique(source, 'PolygonSyntyCharacter.fbx')), use_anim=False,
                             automatic_bone_orientation=False, use_custom_props=False)
    rigs = [o for o in bpy.data.objects if o.type == 'ARMATURE']
    meshes = [o for o in bpy.data.objects if o.type == 'MESH']
    if len(rigs) != 1 or len(meshes) != 1:
        raise ValueError('Expected the sample character skin and one armature')
    rig, mesh = rigs[0], meshes[0]
    rig.name, mesh.name = 'SyntyRig', 'CharacterMesh'
    # The mesh FBX connects Hips to Root's tail, while animation sources translate
    # Hips independently. Connectivity is a Blender editing constraint, absent
    # from glTF; clear it without changing the bind matrices or parentage.
    bpy.context.view_layer.objects.active = rig
    bpy.ops.object.mode_set(mode='EDIT')
    for bone in rig.data.edit_bones:
        bone.use_connect = False
    bpy.ops.object.mode_set(mode='OBJECT')
    bones = sorted(rig.data.bones, key=lambda bone: len(bone.parent_recursive))
    if len(bones) != 50:
        raise ValueError('Unexpected sample skeleton')
    bone_indices = {bone.name: n for n, bone in enumerate(bones)}
    source_weights = []
    for vertex in mesh.data.vertices:
        weights = sorted(((bone_indices[mesh.vertex_groups[g.group].name], g.weight)
                          for g in vertex.groups if g.weight > 1e-8), key=lambda item: -item[1])
        if not weights or len(weights) > 8:
            raise ValueError('Unsupported source skin influences')
        source_weights.append(weights)
    joint_indices = np.zeros((len(source_weights), 8), dtype=int)
    weights = np.zeros((len(source_weights), 8))
    for n, values in enumerate(source_weights):
        for k, (joint, weight) in enumerate(values):
            joint_indices[n, k], weights[n, k] = joint, weight
    weights /= weights.sum(axis=1)[:, None]
    four_weights = weights.copy();four_weights[:, 4:] = 0
    four_weights /= four_weights.sum(axis=1)[:, None]
    vertices = np.array([[*vertex.co, 1.] for vertex in mesh.data.vertices])
    material = bpy.data.materials.new('SyntyDummyStandard')
    material.use_nodes = True
    principled = next(n for n in material.node_tree.nodes if n.type == 'BSDF_PRINCIPLED')
    principled.inputs['Roughness'].default_value = .5
    principled.inputs['Metallic'].default_value = 0.
    texture = material.node_tree.nodes.new('ShaderNodeTexImage')
    texture.image = bpy.data.images.load(str(unique(source, 'T_Polygon_Dummy_01.png')), check_existing=True)
    texture.image.colorspace_settings.name = 'sRGB'
    material.node_tree.links.new(texture.outputs['Color'], principled.inputs['Base Color'])
    mesh.data.materials.clear();mesh.data.materials.append(material)
    actions, report, references = [], {}, {}
    for name, filename in CLIPS.items():
        before = set(bpy.data.objects)
        bpy.ops.import_scene.fbx(filepath=str(unique(source, filename)), use_anim=True,
                                 automatic_bone_orientation=False, use_custom_props=False)
        added = [o for o in bpy.data.objects if o not in before]
        source_rigs = [o for o in added if o.type == 'ARMATURE']
        if len(source_rigs) != 1:
            raise ValueError('Animation source must contain exactly one rig')
        animated = source_rigs[0]
        if set(animated.data.bones.keys()) != set(rig.data.bones.keys()):
            raise ValueError('Animation skeleton names differ')
        if any((bone.parent.name if bone.parent else None) !=
               (rig.data.bones[bone.name].parent.name if rig.data.bones[bone.name].parent else None)
               for bone in animated.data.bones):
            raise ValueError('Animation skeleton parentage differs')
        original = animated.animation_data.action
        first, last = (int(round(v)) for v in original.frame_range)
        if not 1 <= last - first <= 180 or scene.render.fps != 30 or scene.render.fps_base != 1.:
            raise ValueError('Unsupported source animation timing')
        rest_error = max(abs(bone.matrix_local[i][j] - animated.data.bones[bone.name].matrix_local[i][j])
                         for bone in bones for i in range(4) for j in range(4))
        action = bpy.data.actions.new(name);action.use_fake_user = True
        rig.animation_data_create();rig.animation_data.action = action
        for pose in rig.pose.bones:
            pose.rotation_mode = 'QUATERNION'
        samples, roots, maximum_error, skin_error = [], [], 0., 0.
        for frame in range(first, last + 1):
            scene.frame_set(frame)
            conversion = rig.matrix_world.inverted() @ animated.matrix_world
            desired = {bone.name: conversion @ animated.pose.bones[bone.name].matrix.copy() for bone in bones}
            # Converting pose matrices with explicit desired parent matrices
            # avoids assuming identical bind poses or copying incompatible curves.
            for bone in bones:
                options = {} if bone.parent is None else {'parent_matrix': desired[bone.parent.name],
                                                          'parent_matrix_local': bone.parent.matrix_local}
                pose = rig.pose.bones[bone.name]
                pose.matrix_basis = bone.convert_local_to_pose(desired[bone.name], bone.matrix_local, invert=True, **options)
                pose.keyframe_insert('location', frame=frame)
                pose.keyframe_insert('rotation_quaternion', frame=frame)
                pose.keyframe_insert('scale', frame=frame)
            bpy.context.view_layer.update()
            transforms = np.array([rig.matrix_world @ rig.pose.bones[bone.name].matrix @ bone.matrix_local.inverted()
                                   @ rig.matrix_world.inverted() @ mesh.matrix_world for bone in bones])
            deformed = np.einsum('nkij,nj->nki', transforms[joint_indices], vertices)
            full_skin = np.einsum('nk,nki->ni', weights, deformed)
            four_skin = np.einsum('nk,nki->ni', four_weights, deformed)
            skin_error = max(skin_error, float(np.linalg.norm(full_skin[:, :3] - four_skin[:, :3], axis=1).max()))
            wanted = {}
            for bone in bones:
                actual = rig.matrix_world @ rig.pose.bones[bone.name].matrix
                expected = animated.matrix_world @ animated.pose.bones[bone.name].matrix
                error = max(abs(actual[i][j] - expected[i][j]) for i in range(4) for j in range(4))
                maximum_error = max(maximum_error, error)
                if error > .0001:
                    raise ValueError(f'Pose bake mismatch {name}/{frame}/{bone.name}: {error}')
                wanted[bone.name] = [list(row) for row in expected]
            samples.append({'time': (frame - first) / 30., 'bones_blender_world': wanted})
            roots.append(list((animated.matrix_world @ animated.pose.bones['Root'].matrix).translation))
        root_xz_span = max(max(p[axis] for p in roots) - min(p[axis] for p in roots) for axis in [0, 1])
        if root_xz_span > .005:
            raise ValueError('Source root has horizontal locomotion; authority must not inherit it')
        if skin_error > .005 and not four_weight_preview:
            raise ValueError(f'Standard four-influence skin error exceeds 5mm in {name}: {skin_error}')
        report[name] = {'frames': [first, last], 'fps': 30, 'source_bind_matrix_difference': rest_error,
                        'maximum_world_pose_matrix_error': maximum_error, 'root_horizontal_span_metres': root_xz_span,
                        'maximum_four_influence_skin_error_metres': skin_error}
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
    # glTF skinning ignores the mesh node's transform. Make the mesh an identity
    # scene root so ordinary bounds and skin evaluation describe the same metres.
    mesh_world = mesh.matrix_world.copy()
    mesh.parent = None
    mesh.matrix_world = mesh_world
    bpy.ops.object.select_all(action='DESELECT')
    mesh.select_set(True)
    bpy.context.view_layer.objects.active = mesh
    bpy.ops.object.transform_apply(location=True, rotation=True, scale=True)
    output.mkdir(parents=True)
    try:
        export = dict(export_format='GLB',
            export_animations=True, export_animation_mode='ACTIONS', export_anim_single_armature=True,
            export_rest_position_armature=True, export_frame_range=False, export_force_sampling=True,
            export_anim_slide_to_zero=True, export_vertex_color='NONE', export_all_vertex_colors=False,
            export_skins=True,
            export_yup=True, export_cameras=False, export_lights=False, export_extras=False)
        bpy.ops.export_scene.gltf(filepath=str(output / 'character-source-skin.glb'), export_all_influences=True, **export)
        bpy.ops.export_scene.gltf(filepath=str(output / 'character.glb'), export_all_influences=False, export_influence_nb=4, **export)
        result = {'blender': bpy.app.version_string, 'bones': len(bones),
                  'source_bind_poses_are_identical': all(r['source_bind_matrix_difference'] < 1e-5 for r in report.values()),
                  'skin_vertices_reduced_to_four_influences': int(np.count_nonzero(weights[:, 4:].sum(axis=1))),
                  'maximum_discarded_weight': float(weights[:, 4:].sum(axis=1).max()),
                  'four_weight_preview_opt_in': four_weight_preview,
                  'four_weight_skin_within_5mm_gate': all(r['maximum_four_influence_skin_error_metres'] <= .005 for r in report.values()),
                  'clips': report, 'material': 'Explicit standard base-colour palette, metallic 0, roughness 0.5',
                  'inputs': inputs, 'scope': 'isolated sample character conversion; not game character adoption'}
        (output / 'conversion.json').write_text(json.dumps(result, indent=2) + '\n')
        (output / 'source-pose-reference.json').write_text(json.dumps(references, separators=(',', ':')) + '\n')
        return result
    except BaseException:
        shutil.rmtree(output)
        raise


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('source', type=Path)
    parser.add_argument('output', type=Path)
    parser.add_argument('--four-weight-preview', action='store_true', help='Allow a measured preview above the 5mm gate; preserve the full source skin GLB separately')
    args = parser.parse_args()
    print(json.dumps(convert(args.source, args.output, args.four_weight_preview), indent=2))
