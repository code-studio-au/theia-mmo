"""Regression checks for extraction, alpha, geometry and atlas failure modes."""
import json
from pathlib import Path
import tempfile
import unittest
import numpy as np
from PIL import Image, ImageDraw
from build import build, finish, pack, state_image, STATES
from validate import nine_slice, validate

ROOT = Path(__file__).resolve().parents[2]


class SpritePipelineTests(unittest.TestCase):
    def test_white_matte_removed_but_enclosed_highlight_survives(self):
        im = Image.new('RGBA', (30, 30))
        d = ImageDraw.Draw(im)
        d.rectangle((4, 4, 25, 25), fill='white')
        d.rectangle((5, 5, 24, 24), fill=(12, 20, 30, 255))
        d.rectangle((13, 13, 16, 16), fill='white')
        out, bounds, metrics = finish(im)
        self.assertEqual(bounds, (5, 5, 25, 25))
        self.assertEqual(out.getpixel((12, 12)), (255, 255, 255, 255))
        self.assertEqual(metrics['exterior_matte_pixels_removed'], 84)

    def test_keyed_highlight_repaired_without_filling_intentional_void(self):
        im = Image.new('RGBA', (20, 20), (150, 90, 10, 255))
        im.putpixel((7, 7), (255, 255, 255, 0))
        im.putpixel((12, 12), (0, 0, 0, 0))
        out, _, _ = finish(im, solid=True)
        self.assertEqual(out.getpixel((11, 11))[3], 255)
        self.assertEqual(out.getpixel((16, 16))[3], 0)

    def test_states_keep_partial_alpha_and_silhouette(self):
        im = Image.new('RGBA', (8, 8), (30, 60, 120, 128))
        im.putpixel((0, 0), (0, 0, 0, 0))
        for state in STATES:
            self.assertEqual(state_image(im, state).getchannel('A').tobytes(), im.getchannel('A').tobytes())

    def test_nine_slice_preserves_corner_pixels(self):
        im = Image.fromarray(np.arange(10*12*4, dtype=np.uint8).reshape((10, 12, 4)))
        out = nine_slice(im, [3, 2, 4, 3], (101, 73))
        self.assertEqual(out.crop((0, 0, 3, 2)).tobytes(), im.crop((0, 0, 3, 2)).tobytes())
        self.assertEqual(out.crop((97, 70, 101, 73)).tobytes(), im.crop((8, 7, 12, 10)).tobytes())
        with self.assertRaises(AssertionError):
            nine_slice(im, [3, 2, 4, 3], (7, 5))

    def test_atlas_round_trip_including_transparent_rgb(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            inputs = {'a': Image.new('RGBA', (39, 20), (120, 30, 50, 0)), 'b': Image.new('RGBA', (33, 47), (5, 20, 200, 170))}
            records = {k: {} for k in inputs}
            sheets = pack(inputs, root, records)
            for name, im in inputs.items():
                record = records[name]
                atlas = Image.open(root/sheets[record['atlas']]['file'])
                self.assertEqual(atlas.crop(record['rect']).tobytes(), im.tobytes())

    def test_wrong_archive_rejected_before_output_changes(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            source = root/'wrong.zip'; source.write_bytes(b'not the approved source')
            output = root/'existing'; output.mkdir(); (output/'keep.txt').write_text('keep')
            recipes = json.loads((ROOT/'tools/ui_sprites/recipes.json').read_text())
            with self.assertRaisesRegex(ValueError, 'approved'):
                build(source, output, recipes)
            self.assertEqual((output/'keep.txt').read_text(), 'keep')

    def test_final_export_contract(self):
        _, _, result = validate(ROOT/'content/ui/sprites-v1')
        self.assertEqual(result['sprites'], 416)
        self.assertEqual(result['families'], 42)


if __name__ == '__main__':
    unittest.main()
