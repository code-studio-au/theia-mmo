"""Canonical shared PNG variants and bounded dependency verification."""
import re
import struct

from package_world import file_hash

MAX_IMAGES = 512
MAX_IMAGE_BYTES = 16 * 1024 * 1024
MAX_TOTAL_BYTES = 1024 * 1024 * 1024
MAX_RGBA_BYTES = 2 * 1024 * 1024 * 1024
URI = re.compile(r'^\.\./shared/([0-9a-f]{64})-([sl])-(\d+)-(\d+)-(\d+)-(\d+)\.png$')


def valid_sampler(settings):
    return (isinstance(settings, list) and len(settings) == 4
            and all(type(n) is int for n in settings)
            and all(n in (10497, 33071, 33648) for n in settings[:2])
            and settings[2] in (0, 9728, 9729, 9984, 9985, 9986, 9987)
            and settings[3] in (0, 9728, 9729))


def canonical(images):
    if not isinstance(images, list) or len(images) > MAX_IMAGES:
        raise ValueError('Shared image count exceeds budget')
    for image in images:
        if (set(image) != {'sha256', 'bytes', 'color_space', 'sampler'}
                or not isinstance(image['sha256'], list) or len(image['sha256']) != 32
                or any(type(n) is not int or not 0 <= n <= 255 for n in image['sha256'])
                or type(image['bytes']) is not int or not 1 <= image['bytes'] <= MAX_IMAGE_BYTES
                or image['color_space'] not in ('srgb', 'linear') or not valid_sampler(image['sampler'])):
            raise ValueError('Invalid shared image descriptor')
    result = sorted(images, key=lambda i: (i['sha256'], i['color_space'] == 'linear', i['sampler']))
    keys = [filename(i) for i in result]
    if len(set(keys)) != len(keys) or sum(i['bytes'] for i in result) > MAX_TOTAL_BYTES:
        raise ValueError('Duplicate or over-budget shared images')
    return result


def filename(image):
    digest = bytes(image['sha256']).hex()
    color = 's' if image['color_space'] == 'srgb' else 'l'
    return f"{digest}-{color}-{'-'.join(str(n) for n in image['sampler'])}.png"


def sampler(document, texture):
    values = document.get('samplers', [])
    index = texture.get('sampler')
    if index is None:
        settings = {}
    elif type(index) is int and 0 <= index < len(values):
        settings = values[index]
    else:
        raise ValueError('Unknown glTF sampler')
    result = [settings.get('wrapS', 10497), settings.get('wrapT', 10497),
              settings.get('minFilter', 0), settings.get('magFilter', 0)]
    if not valid_sampler(result):
        raise ValueError('Invalid glTF sampler settings')
    return result


def usage_fields(document):
    for material in document.get('materials', []):
        for owner, name, color in [(material.get('pbrMetallicRoughness', {}), 'baseColorTexture', 'srgb'),
                                   (material, 'emissiveTexture', 'srgb'),
                                   (material.get('pbrMetallicRoughness', {}), 'metallicRoughnessTexture', 'linear'),
                                   (material, 'normalTexture', 'linear'),
                                   (material, 'occlusionTexture', 'linear')]:
            if name in owner:
                yield owner[name], color


def validate_references(document, images, *, allow_material_extensions=True):
    expected = {'../shared/' + filename(i): i for i in canonical(images)}
    if allow_material_extensions:
        from static_material_extensions import validate
        validate(document)
    elif document.get('extensionsUsed') or document.get('extensionsRequired'):
        raise ValueError('Shared-image GLBs must be unextended')
    if any('uri' in b for b in document.get('buffers', [])):
        raise ValueError('External geometry buffers are unbound')
    referenced = set()
    from material_profiles import image_uses, SAMPLER
    for uri,color in image_uses(document):
        if uri not in expected or expected[uri]['color_space']!=color or expected[uri]['sampler']!=SAMPLER:
            raise ValueError('Unbound projected material image/role/sampler')
        referenced.add(uri)
    for image in document.get('images', []):
        if 'uri' in image:
            uri = image['uri']
            if uri not in expected or 'bufferView' in image or image.get('mimeType', 'image/png') != 'image/png':
                raise ValueError('Unbound or noncanonical shared image URI')
            referenced.add(uri)
    if referenced != set(expected):
        raise ValueError('Unused shared image declaration')
    textures = document.get('textures', [])
    uses = {}
    for field, color in usage_fields(document):
        index = field['index']
        if type(index) is not int or not 0 <= index < len(textures):
            raise ValueError('Unknown material texture')
        uses.setdefault(index, set()).add(color)
    if any(len(value) > 1 for value in uses.values()):
        raise ValueError('Shared texture must separate linear and sRGB uses')
    for n, texture in enumerate(textures):
        source = texture['source']
        if type(source) is not int or not 0 <= source < len(document.get('images', [])):
            raise ValueError('Unknown texture image')
        image = document['images'][source]
        if 'uri' in image:
            entry = expected[image['uri']]
            color = 'linear' if 'linear' in uses.get(n, set()) else 'srgb'
            if entry['color_space'] != color or entry['sampler'] != sampler(document, texture):
                raise ValueError('Shared image variant disagrees with glTF loader settings')


def png_pixels(path):
    with path.open('rb') as file:
        header = file.read(33)
    if len(header) != 33:
        raise ValueError('Missing shared PNG header')
    width, height = struct.unpack('>II', header[16:24])
    pixels = width * height
    if (header[:8] != b'\x89PNG\r\n\x1a\n' or header[12:16] != b'IHDR'
            or struct.unpack('>I', header[8:12])[0] != 13 or header[24] != 8
            or header[25] not in (0, 2, 3, 4, 6) or header[26] != 0 or header[27] != 0 or header[28] > 1
            or not 1 <= width <= 8192 or not 1 <= height <= 8192 or pixels > 16 * 1024 * 1024):
        raise ValueError('Unsupported or over-budget shared PNG header')
    return pixels


def collect(document, root, *, allow_material_extensions=True):
    root = root.resolve(strict=True)
    entries = {}
    rgba_bytes = 0
    from material_profiles import image_uses
    projected={uri for uri,_ in image_uses(document)}
    dependencies=list(document.get('images',[]))+[{'uri':uri} for uri in sorted(projected)]
    for image in dependencies:
        if 'uri' not in image or image['uri'] in entries:
            continue
        match = URI.fullmatch(image['uri'])
        if not match:
            raise ValueError('Noncanonical shared image URI')
        digest, color, *settings = match.groups()
        entry = {'sha256': list(bytes.fromhex(digest)), 'bytes': 1,
                 'color_space': 'srgb' if color == 's' else 'linear', 'sampler': list(map(int, settings))}
        path = root / 'shared' / filename(entry)
        if path.resolve(strict=True) != path:
            raise ValueError('Shared image path alias is not allowed')
        size, actual = file_hash(path, MAX_IMAGE_BYTES)
        entry['bytes'] = size
        if actual.hex() != digest:
            raise ValueError('Shared image hash disagrees with URI')
        rgba_bytes += png_pixels(path) * 4
        if image['uri'] in projected:
            with path.open('rb') as file:header=file.read(29)
            width,height=struct.unpack('>II',header[16:24])
            if max(width,height)>4096 or header[25] not in (2,6) or header[28]!=0:
                raise ValueError('Unsupported bounded projected-material PNG')
        if rgba_bytes > MAX_RGBA_BYTES:
            raise ValueError('Shared PNG base-level budget exceeded')
        entries[image['uri']] = entry
    result = canonical(list(entries.values()))
    validate_references(document, result, allow_material_extensions=allow_material_extensions)
    return result


def verify(document, root, declared, *, allow_material_extensions=True):
    if collect(document, root, allow_material_extensions=allow_material_extensions) != canonical(declared):
        raise ValueError('Shared image declarations disagree with actual dependencies')
