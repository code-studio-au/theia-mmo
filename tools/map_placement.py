"""wow.export ADT/WMO placement transforms, expressed in the OBJ's Y-up frame.

Derived from the upstream Blender importer; see docs/MAP_OBJECTS.md for provenance.
No Blender or scipy dependency: these are ordinary homogeneous transforms.
"""
import math

import numpy as np
from trimesh.transformations import euler_matrix, quaternion_matrix

MAX_SIZE = 51200 / 3
TO_Y_UP = euler_matrix(-math.pi / 2, 0, 0)


def scale(row):
    value = float(row.get('ScaleFactor') or 1)
    if not math.isfinite(value) or value <= 0:
        raise ValueError('Placement scale must be finite and positive')
    return np.diag([value, value, value, 1.])


def translation(values):
    if not all(math.isfinite(v) for v in values):
        raise ValueError('Nonfinite placement coordinates')
    matrix = np.eye(4)
    matrix[:3, 3] = values
    return matrix


def adt_transform(row):
    if row['Type'] not in ('m2', 'wmo'):
        raise ValueError(f"Unsupported placement type: {row['Type']}")
    x, y, z = [float(row['Position' + axis]) for axis in 'XYZ']
    rx, ry, rz = [float(row['Rotation' + axis]) for axis in 'XYZ']
    if not all(math.isfinite(v) for v in (rx, ry, rz)):
        raise ValueError('Nonfinite placement rotation')
    # Upstream Blender parent: (MAX-X, -(MAX-Z), Y), Euler XYZ
    # (RotationZ, RotationX, 90+RotationY), followed by OBJ's +90 X.
    parent = translation([MAX_SIZE - x, -(MAX_SIZE - z), y])
    rotation = euler_matrix(*map(math.radians, (rz, rx, 90 + ry)))
    return TO_Y_UP @ parent @ rotation @ TO_Y_UP.T @ scale(row)


def doodad_transform(row):
    position = [float(row['Position' + axis]) for axis in 'XYZ']
    quaternion = [float(row['Rotation' + axis]) for axis in 'WXYZ']
    if not all(math.isfinite(v) for v in quaternion) or sum(v * v for v in quaternion) < 1e-12:
        raise ValueError('Invalid WMO doodad quaternion')
    # Relative to the WMO's raw OBJ root; WMO CSV quaternions are WXYZ.
    return TO_Y_UP @ translation(position) @ quaternion_matrix(quaternion) @ TO_Y_UP.T @ scale(row)
