"""Contracts for regenerated alpha and composed state families."""
import unittest
from PIL import Image, ImageDraw
from regenerate_export import finish_master, safe_border, derive, slice_insets, tint, PAD
from validate import nine_slice

class RegenerationTests(unittest.TestCase):
    def test_near_opaque_interior_and_faint_alpha_debris(self):
        im=Image.new('RGBA',(40,40),(200,10,80,1))
        ImageDraw.Draw(im).rectangle((8,8,31,31),fill=(10,30,80,253))
        out=finish_master(im)
        self.assertEqual(out.size,(24,24))
        self.assertEqual(out.getchannel('A').getextrema(),(255,255))

    def test_hollow_frame_remains_hollow(self):
        im=Image.new('RGBA',(60,60))
        ImageDraw.Draw(im).ellipse((5,5,54,54),outline=(240,160,30,253),width=6)
        out=finish_master(im)
        self.assertEqual(out.getpixel((out.width//2,out.height//2))[3],0)
        self.assertEqual(max(out.getchannel('A').getdata()),255)

    def test_padding_has_zero_alpha_and_bled_rgb(self):
        out=safe_border(Image.new('RGBA',(12,12),(240,170,20,255)))
        self.assertEqual(out.getpixel((7,12)),(240,170,20,0))
        self.assertEqual(out.getpixel((8,12)),(240,170,20,255))

    def test_coloured_symbol_survives_selection(self):
        masters={'base':Image.new('RGBA',(40,40),(5,20,60,255)), 'glyph':Image.new('RGBA',(10,10),(30,80,255,255))}
        rule={'master':'base','operation':'button_glyph','glyph':'glyph'}
        default=derive('test',rule,masters,(40,40),False)
        selected=derive('test',rule,masters,(40,40),False,'selected')
        self.assertEqual(default.getpixel((20,20)),selected.getpixel((20,20)))
        self.assertNotEqual(default.getpixel((2,2)),selected.getpixel((2,2)))
        self.assertEqual(default.getchannel('A').tobytes(),selected.getchannel('A').tobytes())


    def test_notice_badge_is_outside_horizontal_stretch_region(self):
        size=(240,64)
        im=Image.new('RGBA',size,(10,30,60,255))
        ImageDraw.Draw(im).ellipse((0,0,63,63),fill=(240,180,30,255))
        im=safe_border(im)
        insets=slice_insets('notice.warning',{'operation':'notice'},size)
        out=nine_slice(im,insets,(800,im.height))
        self.assertGreaterEqual(insets[0],size[1]+PAD)
        self.assertEqual(im.crop((0,0,72,im.height)).tobytes(),out.crop((0,0,72,out.height)).tobytes())

    def test_rarity_recolours_gold_rim_without_tinting_inset_or_item(self):
        im=Image.new('RGBA',(100,100),(5,20,60,255))
        draw=ImageDraw.Draw(im)
        draw.rectangle((0,0,99,99),outline=(240,160,20,255),width=3)
        draw.rectangle((40,40,60,60),fill=(240,160,20,255))
        out=tint(im,'purple',border_only=True)
        self.assertNotEqual(out.getpixel((1,50)),im.getpixel((1,50)))
        self.assertEqual(out.getpixel((7,50)),im.getpixel((7,50)))
        self.assertEqual(out.getpixel((50,50)),im.getpixel((50,50)))
        self.assertEqual(out.getchannel('A').tobytes(),im.getchannel('A').tobytes())

if __name__=='__main__':unittest.main()
