#!/usr/bin/env python3
"""Render private cosmetic thumbnails from the same curated GLB used by the game.

Run with build/blender-tools/bin/python. Output stays beside the private player
assets, never in Git. No inferred/generated appearances or source package scripts.
"""
import argparse
import hashlib
import json
from pathlib import Path

from convert_modular_hero import selected_names


def render(directory):
    import bpy
    from mathutils import Vector
    from PIL import Image
    directory = directory.resolve(strict=True)
    source = directory / 'character-variants.glb'
    bpy.ops.wm.read_factory_settings(use_empty=True)
    bpy.ops.import_scene.gltf(filepath=str(source))
    scene = bpy.context.scene
    scene.render.engine = 'CYCLES'
    scene.cycles.device = 'CPU'
    scene.cycles.samples = 16
    scene.cycles.use_denoising = True
    scene.render.resolution_x = scene.render.resolution_y = 256
    scene.render.resolution_percentage = 100
    scene.render.film_transparent = True
    scene.render.image_settings.file_format = 'PNG'
    scene.render.image_settings.color_mode = 'RGBA'
    scene.view_settings.view_transform = 'AgX'
    scene.render.threads_mode = 'FIXED'
    scene.render.threads = 4
    rig = next(o for o in bpy.data.objects if o.type == 'ARMATURE')
    for track in list(rig.animation_data.nla_tracks):
        rig.animation_data.nla_tracks.remove(track)
    rig.animation_data.action = bpy.data.actions['Idle']
    rig.animation_data.action_slot = bpy.data.actions['Idle'].slots[0]
    scene.frame_set(1)
    meshes = [o for o in bpy.data.objects if o.type == 'MESH']
    # Retain source UV detail (including the face variants), desaturating only
    # the presentation so the tiles do not imply a different selected palette.
    for material in bpy.data.materials:
        if not material.use_nodes:
            continue
        surface = material.node_tree.nodes.get('Principled BSDF')
        if surface and surface.inputs['Base Color'].is_linked:
            link = surface.inputs['Base Color'].links[0]
            source_socket = link.from_socket
            material.node_tree.links.remove(link)
            gray = material.node_tree.nodes.new('ShaderNodeRGBToBW')
            material.node_tree.links.new(source_socket, gray.inputs['Color'])
            material.node_tree.links.new(gray.outputs['Val'], surface.inputs['Base Color'])
    camera_data = bpy.data.cameras.new('CatalogueCamera')
    camera = bpy.data.objects.new('CatalogueCamera', camera_data)
    scene.collection.objects.link(camera)
    scene.camera = camera
    camera_data.type = 'ORTHO'
    for name, position, energy, size in [('Key',(-3,-4,5),700,4),('Fill',(3,-2,2),300,3),('Rim',(1,3,4),850,3)]:
        light = bpy.data.lights.new(name, 'AREA')
        light.energy, light.shape, light.size = energy, 'DISK', size
        obj = bpy.data.objects.new(name, light)
        scene.collection.objects.link(obj)
        obj.location = position
        obj.rotation_euler = (Vector((0,0,1.2))-obj.location).to_track_quat('-Z','Y').to_euler()
    output = directory / 'thumbnails'
    output.mkdir(exist_ok=True)
    atlas = Image.new('RGBA',(2048,1024))
    records=[]
    for body in range(2):
        choices = [(0,body)] + [(f,v) for f,n in [(1,4),(2,5),(3,4),(6,2)] for v in range(n)]
        for index,(field,value) in enumerate(choices):
            values = [body,0,1,0,0,0,0]
            values[field] = value
            if body == 1: values[3]=0
            names = set(selected_names(body=body,face=values[1],hair=values[2],beard=values[3],outfit=values[6]))
            for mesh in meshes: mesh.hide_render = mesh.name not in names
            full = field in (0,6)
            target = Vector((0,0, .87 if full else 1.55))
            camera.location = target + Vector((.35,-3,.06))
            camera.rotation_euler = (target-camera.location).to_track_quat('-Z','Y').to_euler()
            camera_data.ortho_scale = 2.1 if full else .75
            path=output / f'{body}-{field}-{value}.png'
            scene.render.filepath=str(path)
            bpy.ops.render.render(write_still=True)
            tile=body*16+index
            with Image.open(path) as picture:
                assert picture.getbbox(), 'Empty model thumbnail'
                atlas.paste(picture,(tile%8*256,tile//8*256))
            records.append(dict(body=body,field=field,value=value,tile=tile))
    path=directory/'appearance-thumbnails.png'
    atlas.save(path)
    report=dict(schema=1,source_sha256=hashlib.sha256(source.read_bytes()).hexdigest(),
                atlas_sha256=hashlib.sha256(path.read_bytes()).hexdigest(),blender=bpy.app.version_string,
                columns=8,tile_size=256,records=records)
    (directory/'appearance-thumbnails.json').write_text(json.dumps(report,indent=2)+'\n')


if __name__ == '__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--players',type=Path,required=True)
    render(parser.parse_args().players)
