#!/usr/bin/env python3
"""Convert owned Synty modular FBX data and qualified animation GLB to Bevy assets.

Run with tools/synty-conversion-requirements.txt. Unity scripts are never executed.
Outputs are private build artifacts; source meshes/textures must not enter Git.
"""
import argparse
import hashlib
import json
from pathlib import Path
import tarfile

BODIES = ('Male', 'Female')
PARTS = ('Torso', 'Hips', 'ArmUpperLeft', 'ArmUpperRight', 'ArmLowerLeft',
         'ArmLowerRight', 'HandLeft', 'HandRight', 'LegLeft', 'LegRight')
OUTFITS = (1, 3)


def selected_names(body=0, face=0, hair=1, beard=0, outfit=0):
    gender = BODIES[body]
    result = [f'Chr_{part}_{gender}_{OUTFITS[outfit]:02}' for part in PARTS]
    result += [f'Chr_Head_{gender}_{face:02}', f'Chr_Eyebrow_{gender}_01']
    if hair:
        result.append(f'Chr_Hair_{hair:02}')
    if beard and body == 0:
        result.append(f'Chr_FacialHair_Male_{beard:02}')
    return result


def extract(package, directory):
    wanted = {'Models/FixedScale/ModularCharacters.fbx': 'modular.fbx',
              'Textures/Textures/PolygonFantasyHero_Texture_01_A.png': 'palette.png',
              **{f'Textures/Masks/PolygonFantasyHero_Texture_Mask_{n:02}.png': f'mask-{n}.png'
                 for n in (3, 4)}}
    found = {}
    with tarfile.open(package, 'r:gz') as archive:
        members = archive.getmembers()
        for member in members:
            if not member.isfile() or not member.name.endswith('/pathname'):
                continue
            raw = archive.extractfile(member).read(4097)
            if len(raw) > 4096:
                raise ValueError('Oversize Unity pathname')
            path = raw.decode('utf-8').splitlines()[0]
            for suffix, output in wanted.items():
                if path == 'Assets/PolygonFantasyHeroCharacters/' + suffix:
                    if output in found:
                        raise ValueError('Ambiguous modular source')
                    found[output] = (path, member.name.removesuffix('/pathname') + '/asset')
        if set(found) != set(wanted.values()):
            raise ValueError('Required modular sources are missing')
        provenance = {}
        for output, (path, member) in found.items():
            data = archive.extractfile(member).read(32 * 1024 * 1024 + 1)
            if len(data) > 32 * 1024 * 1024:
                raise ValueError('Oversize modular source')
            (directory / output).write_bytes(data)
            provenance[output] = dict(path=path, sha256=hashlib.sha256(data).hexdigest())
    return provenance


def palettes(source, output):
    import numpy as np
    from PIL import Image
    base = np.array(Image.open(source/'palette.png').convert('RGB'))
    skin_mask = np.array(Image.open(source/'mask-3.png').convert('RGB'))
    hair_mask = np.array(Image.open(source/'mask-4.png').convert('RGB'))
    skin = [(255, 204, 174), (209, 162, 117), (144, 104, 80), (113, 76, 57)]
    hair = [(67, 54, 34), (40, 38, 36), (218, 179, 107), (165, 80, 42)]
    for s, colour in enumerate(skin):
        for h, hair_colour in enumerate(hair):
            pixels = base.copy()
            pixels[hair_mask[:, :, 2] <= 127] = hair_colour
            pixels[skin_mask[:, :, 0] <= 127] = colour
            pixels[skin_mask[:, :, 2] <= 127] = [round(c * .8) for c in colour]
            pixels[skin_mask[:, :, 1] <= 127] = [round(c * .9) for c in colour]
            Image.fromarray(pixels).save(output/f'palette-{s}-{h}.png')


def convert(package, animation, output):
    import bpy
    from mathutils import Matrix
    package, animation = package.resolve(strict=True), animation.resolve(strict=True)
    output = output.resolve()
    output.mkdir(parents=True, exist_ok=False)
    source = output/'source'; source.mkdir()
    provenance = extract(package, source)
    palettes(source, output)
    bpy.ops.wm.read_factory_settings(use_empty=True)
    scene = bpy.context.scene
    scene.render.fps = 30
    bpy.ops.import_scene.fbx(filepath=str(source/'modular.fbx'), use_anim=False,
                             automatic_bone_orientation=False, use_custom_props=False)
    rig = next(o for o in bpy.data.objects if o.type == 'ARMATURE')
    rig.name = 'ModularRig'
    for bone in rig.data.bones:
        if bone.name.endswith('.001'):
            bone.name = bone.name[:-4] + '_1'
    wanted = set()
    for body in range(2):
        for face in range(4):
            for outfit in range(2):
                wanted.update(selected_names(body, face, 0, 0, outfit))
    wanted.update(f'Chr_Hair_{i:02}' for i in range(1, 5))
    wanted.update(f'Chr_FacialHair_Male_{i:02}' for i in range(1, 4))
    meshes = [o for o in bpy.data.objects if o.type == 'MESH' and o.name in wanted]
    if {m.name for m in meshes} != wanted:
        raise ValueError('Curated character parts are missing')
    for obj in list(bpy.data.objects):
        if obj.type == 'MESH' and obj not in meshes:
            bpy.data.objects.remove(obj, do_unlink=True)
    original_objects = set(bpy.data.objects)
    bpy.ops.import_scene.gltf(filepath=str(animation))
    added = set(bpy.data.objects) - original_objects
    animated = next(o for o in added if o.type == 'ARMATURE')
    animation_sources = list(bpy.data.actions)
    for track in list(animated.animation_data.nla_tracks):
        animated.animation_data.nla_tracks.remove(track)
    # Keep the qualified weapon sockets and their canonical names for grip masks.
    bpy.context.view_layer.objects.active = rig
    bpy.ops.object.mode_set(mode='EDIT')
    for bone in rig.data.edit_bones:
        bone.use_connect = False
    conversion = rig.matrix_world.inverted() @ animated.matrix_world
    for name in ('Prop_L', 'Prop_R'):
        original = animated.data.bones[name]
        bone = rig.data.edit_bones.new(name)
        bone.parent = rig.data.edit_bones[original.parent.name]
        bone.matrix = conversion @ original.matrix_local
        bone.length = original.length
        bone.use_deform = False
    bpy.ops.object.mode_set(mode='OBJECT')
    bones = list(rig.data.bones)
    material = bpy.data.materials.new('HeroPalette')
    material.use_nodes = True
    principled = next(n for n in material.node_tree.nodes if n.type == 'BSDF_PRINCIPLED')
    principled.inputs['Roughness'].default_value = .8
    texture = material.node_tree.nodes.new('ShaderNodeTexImage')
    texture.image = bpy.data.images.load(str(output/'palette-0-0.png'))
    texture.image.colorspace_settings.name = 'sRGB'
    material.node_tree.links.new(texture.outputs['Color'], principled.inputs['Base Color'])
    max_influences = 0
    for mesh in meshes:
        mesh.data.materials.clear(); mesh.data.materials.append(material)
        for vertex in mesh.data.vertices:
            influences = [g for g in vertex.groups if g.weight > 1e-8]
            if not influences or any(mesh.vertex_groups[g.group].name not in rig.data.bones for g in influences):
                raise ValueError('Missing/unknown skin binding')
            max_influences = max(max_influences, len(influences))
    if max_influences > 4:
        raise ValueError('Curated modular parts need an explicit skin-reduction qualification')
    actions, reports = [], {}
    rig.animation_data_create()
    for pose in rig.pose.bones:
        pose.rotation_mode = 'QUATERNION'
    for original in animation_sources:
        name = original.name
        original.name = 'Source_' + name
        animated.animation_data.action = original
        animated.animation_data.action_slot = original.slots[0]
        first, last = map(round, original.frame_range)
        action = bpy.data.actions.new(name); action.use_fake_user = True
        rig.animation_data.action = action
        error = 0.
        for frame in range(first, last + 1):
            scene.frame_set(frame)
            desired = {}
            for bone in bones:
                if bone.name in animated.pose.bones:
                    # Transfer the world-space deformation, preserving the target bind axes.
                    source_bone = animated.data.bones[bone.name]
                    desired[bone.name] = (conversion @ animated.pose.bones[bone.name].matrix
                        @ source_bone.matrix_local.inverted() @ conversion.inverted() @ bone.matrix_local)
                else:
                    desired[bone.name] = desired[bone.parent.name] @ bone.parent.matrix_local.inverted() @ bone.matrix_local
                kwargs = {} if bone.parent is None else dict(parent_matrix=desired[bone.parent.name], parent_matrix_local=bone.parent.matrix_local)
                pose = rig.pose.bones[bone.name]
                pose.matrix_basis = bone.convert_local_to_pose(desired[bone.name], bone.matrix_local, invert=True, **kwargs)
                for channel in ('location', 'rotation_quaternion', 'scale'):
                    pose.keyframe_insert(channel, frame=frame)
            bpy.context.view_layer.update()
            error = max(error, max(abs((rig.matrix_world @ rig.pose.bones[b.name].matrix)[i][j] -
                    (rig.matrix_world @ desired[b.name])[i][j]) for b in bones for i in range(4) for j in range(4)))
        if error > .0001:
            raise ValueError(f'Retarget pose mismatch in {name}: {error}')
        reports[name] = dict(frames=last-first+1, world_pose_error=error)
        actions.append(action)
    rig.animation_data.action = None
    for obj in added:
        bpy.data.objects.remove(obj, do_unlink=True)
    for action in list(bpy.data.actions):
        if action not in actions:
            bpy.data.actions.remove(action)
    for pose in rig.pose.bones:
        pose.matrix_basis = Matrix.Identity(4)
    scene.frame_set(0)
    for mesh in meshes:
        world = mesh.matrix_world.copy(); mesh.parent = None; mesh.matrix_world = world
        bpy.ops.object.select_all(action='DESELECT'); mesh.select_set(True)
        bpy.context.view_layer.objects.active = mesh
        bpy.ops.object.transform_apply(location=True, rotation=True, scale=True)
    settings = dict(export_format='GLB', export_animations=True, export_animation_mode='ACTIONS',
        export_anim_single_armature=True, export_rest_position_armature=True, export_frame_range=False,
        export_force_sampling=True, export_anim_slide_to_zero=True, export_vertex_color='NONE',
        export_all_vertex_colors=False, export_skins=True, export_yup=True, export_cameras=False,
        export_lights=False, export_extras=False, use_selection=True, export_influence_nb=4)
    bpy.ops.object.select_all(action='DESELECT'); rig.select_set(True)
    for mesh in meshes: mesh.select_set(True)
    bpy.ops.export_scene.gltf(filepath=str(output/'character-variants.glb'), **settings)
    default_names = set(selected_names())
    for mesh in meshes: mesh.select_set(mesh.name in default_names)
    bpy.ops.export_scene.gltf(filepath=str(output/'character.glb'), **settings)
    report = dict(schema=1, blender=bpy.app.version_string, package_sha256=hashlib.sha256(package.read_bytes()).hexdigest(),
        animation_sha256=hashlib.sha256(animation.read_bytes()).hexdigest(), sources=provenance,
        bones=len(bones), max_influences=max_influences, parts=sorted(wanted), default_parts=sorted(default_names), clips=reports)
    (output/'modular-hero.json').write_text(json.dumps(report, indent=2)+'\n')
    return report


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--package', type=Path, required=True)
    parser.add_argument('--animation', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    result = convert(args.package, args.animation, args.output)
    print(json.dumps({k:result[k] for k in ('bones', 'max_influences', 'clips')}))
