"""Blender's baked file-axis/units path for canonical, undeformed static meshes.

Scene placements already include imported model node transforms. Do not apply
those transforms a second time when producing a mesh-local prototype. Nonidentity
FBX geometric transforms need separate qualification and currently reject.
"""
def require_identity_geometry(model,model_id):
    from io_scene_fbx import parse_fbx
    root,_=parse_fbx.parse(str(model))
    objects,=[e for e in root.elems if e.id==b'Objects']
    node,=[e for e in objects.elems if e.id==b'Model' and e.props[0]==model_id]
    for container in node.elems:
        if container.id!=b'Properties70':continue
        for entry in container.elems:
            if entry.id!=b'P':continue
            key=entry.props[0]
            if key in (b'GeometricTranslation',b'GeometricRotation',b'GeometricScaling'):
                expected=(1,1,1) if key==b'GeometricScaling' else (0,0,0)
                if tuple(entry.props[4:])!=expected:raise ValueError('Nonidentity FBX geometric transform requires qualification')
