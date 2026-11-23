"""Theia-themed native window decoration and small launcher pictograms."""
import cairo
from gi.repository import Gdk, GdkPixbuf, Gtk

from artwork import asset


def style(widget, name):
    widget.get_style_context().add_class(name)
    return widget


def picture(name, width, height):
    pixbuf = GdkPixbuf.Pixbuf.new_from_file_at_scale(str(asset(name)), width, height, True)
    return Gtk.Image.new_from_pixbuf(pixbuf)


def title_art():
    original = GdkPixbuf.Pixbuf.new_from_file(str(asset('title')))
    wordmark = original.new_subpixbuf(61, 0, 1610, 771)
    image = Gtk.Image.new_from_pixbuf(wordmark.scale_simple(219, 105, GdkPixbuf.InterpType.HYPER))
    image.get_accessible().set_name('Theia Online')
    return image


def load_skin(css_file):
    provider = Gtk.CssProvider()
    provider.load_from_path(str(css_file))
    Gtk.StyleContext.add_provider_for_screen(Gdk.Screen.get_default(), provider,
                                              Gtk.STYLE_PROVIDER_PRIORITY_APPLICATION)
    images = {
        'button.action': 'secondary',
        'button.nav-active': 'selected',
        'button.primary': 'primary',
        'button.primary:hover': 'primary-hover',
        '.header-ornament': 'divider',
    }
    rules = [
        f'{selector} {{ background-image: url("{asset(name).as_uri()}"); }}'
        for selector, name in images.items()
    ]
    for selector, name, slice_width in (
        ('.surface', 'panel', 120),
        ('.status-card', 'small-panel', 46),
    ):
        rules.append(f'{selector} {{ border-image: url("{asset(name).as_uri()}") {slice_width} stretch; }}')
    textures = Gtk.CssProvider()
    textures.load_from_data('\n'.join(rules).encode())
    Gtk.StyleContext.add_provider_for_screen(Gdk.Screen.get_default(), textures,
                                              Gtk.STYLE_PROVIDER_PRIORITY_APPLICATION + 1)


class WindowFrame(Gtk.DrawingArea):
    def __init__(self):
        super().__init__()
        self.set_has_window(False)
        original = GdkPixbuf.Pixbuf.new_from_file(str(asset('corner')))
        pixels = original.new_subpixbuf(30, 32, 220, 220)
        self.corner = pixels.scale_simple(39, 39, GdkPixbuf.InterpType.HYPER)
        self.connect('draw', self.paint)

    def paint(self, widget, context):
        width, height = widget.get_allocated_width(), widget.get_allocated_height()
        gold = cairo.LinearGradient(0, 0, width, height * .84)
        for offset, color in (
            (0, (.99, .78, .21)), (.16, (1, .93, .67)), (.38, (.57, .31, .05)),
            (.53, (.96, .74, .24)), (.76, (1, .9, .52)), (1, (.57, .29, .04)),
        ):
            gold.add_color_stop_rgb(offset, *color)
        context.set_line_width(2)
        context.set_source(gold)
        context.rectangle(2, 2, width - 4, height - 4)
        context.stroke()
        context.set_line_width(1)
        context.set_source_rgba(.78, .49, .14, .70)
        context.rectangle(4.5, 4.5, width - 9, height - 9)
        context.stroke()
        for x, y, sx, sy in ((0, 0, 1, 1), (width, 0, -1, 1),
                             (0, height, 1, -1), (width, height, -1, -1)):
            context.save()
            context.translate(x, y)
            context.scale(sx, sy)
            Gdk.cairo_set_source_pixbuf(context, self.corner, 0, 0)
            context.paint()
            context.restore()
        return False


class Glyph(Gtk.DrawingArea):
    def __init__(self, kind, size):
        super().__init__()
        self.kind = kind
        self.set_size_request(size, size)
        self.connect('draw', self.paint)

    def show_kind(self, kind):
        if self.kind != kind:
            self.kind = kind
            self.queue_draw()

    def paint(self, widget, context):
        width, height = widget.get_allocated_width(), widget.get_allocated_height()
        unit = min(width, height)
        context.translate((width - unit)/2, (height - unit)/2)
        context.scale(unit/32, unit/32)
        colors = self.get_style_context().get_color(self.get_state_flags())
        context.set_source_rgba(colors.red, colors.green, colors.blue, colors.alpha)
        if self.kind in ('play', 'install', 'update', 'ready') and not (
            self.get_state_flags() & Gtk.StateFlags.INSENSITIVE
        ):
            gold = cairo.LinearGradient(0, 0, 0, 32)
            gold.add_color_stop_rgb(0, 1, .97, .75)
            gold.add_color_stop_rgb(.50, 1, .78, .29)
            gold.add_color_stop_rgb(1, .69, .40, .10)
            context.set_source(gold)
        context.set_line_width(2)
        context.set_line_cap(cairo.LINE_CAP_ROUND)
        context.set_line_join(cairo.LINE_JOIN_ROUND)
        if self.kind == 'play':
            context.move_to(8, 4)
            context.line_to(26, 16)
            context.line_to(8, 28)
            context.close_path()
            context.fill()
        elif self.kind == 'minimise':
            self.stroke(context, (8, 16), (24, 16))
        elif self.kind == 'maximise':
            context.rectangle(8, 8, 16, 16)
            context.stroke()
        elif self.kind == 'restore':
            self.stroke(context, (12, 7), (25, 7), (25, 20))
            context.rectangle(7, 12, 13, 13)
            context.stroke()
        elif self.kind == 'close':
            self.stroke(context, (9, 9), (23, 23))
            self.stroke(context, (23, 9), (9, 23))
        elif self.kind == 'right':
            context.set_line_width(3)
            self.stroke(context, (12, 7), (21, 16), (12, 25))
        else:
            context.arc(16, 16, 14.1, 0, 6.283185)
            context.stroke()
            context.set_line_width(2.25)
            if self.kind == 'ready':
                self.stroke(context, (9, 16), (14, 21), (23, 11))
            elif self.kind == 'update':
                context.arc(16, 16, 6.3, -.35, 4.83)
                context.stroke()
                self.stroke(context, (19, 9), (22.5, 14), (17.5, 14))
            else:
                self.stroke(context, (16, 8), (16, 20))
                self.stroke(context, (11.5, 16), (16, 20.5), (20.5, 16))
                self.stroke(context, (10, 24), (22, 24))
        return False

    @staticmethod
    def stroke(context, start, *points):
        context.move_to(*start)
        for point in points:
            context.line_to(*point)
        context.stroke()


def resize_edge(x, y, width, height):
    left, right, top, bottom = x < 7, x >= width - 7, y < 7, y >= height - 7
    if top:
        return Gdk.WindowEdge.NORTH_WEST if left else (
            Gdk.WindowEdge.NORTH_EAST if right else Gdk.WindowEdge.NORTH)
    if bottom:
        return Gdk.WindowEdge.SOUTH_WEST if left else (
            Gdk.WindowEdge.SOUTH_EAST if right else Gdk.WindowEdge.SOUTH)
    if left:
        return Gdk.WindowEdge.WEST
    if right:
        return Gdk.WindowEdge.EAST
    return None
