"""The independent Theia Online desktop launcher for trusted local releases."""
import argparse
from pathlib import Path
import sys
import threading
import time

import gi

gi.require_version('Gtk', '3.0')
gi.require_version('Gdk', '3.0')
from gi.repository import Gdk, GdkPixbuf, GLib, Gtk, Pango

HERE = Path(__file__).resolve().parent
REPOSITORY = HERE.parents[1]
ASSETS = HERE / 'assets'
SPRITES = ASSETS / 'sprites' if ASSETS.is_dir() else REPOSITORY / 'content/ui/sprites-v3/sprites'
if ASSETS.is_dir():
    assets = {name: ASSETS / (name + '.png') for name in ('title', 'courtyard', 'hero', 'icon')}
else:
    sys.path.insert(0, str(REPOSITORY / 'tools'))
    frontend = REPOSITORY / 'content/ui/frontend'
    assets = dict(title=frontend / 'title.png', courtyard=frontend / 'character-courtyard-v1.png',
                  hero=frontend / 'account-adventurer-v1.png',
                  icon=REPOSITORY / 'content/ui/sprites-v3/sprites/crest/tree.png')

import client_install
from state import data_directory, newest_release, read_settings, save_settings


def box(vertical=False, spacing=10):
    direction = Gtk.Orientation.VERTICAL if vertical else Gtk.Orientation.HORIZONTAL
    return Gtk.Box(orientation=direction, spacing=spacing)


def label(text='', style=None):
    widget = Gtk.Label(label=text, xalign=0)
    widget.set_line_wrap(True)
    if style:
        widget.get_style_context().add_class(style)
    return widget


def button(text, action, style=None):
    widget = Gtk.Button.new_with_label(text)
    widget.connect('clicked', lambda _: action())
    if style:
        widget.get_style_context().add_class(style)
    return widget


def size_text(amount):
    if amount >= 1024**3:
        return f'{amount / 1024**3:.2f} GiB'
    return f'{amount / 1024**2:.1f} MiB'


class Scene(Gtk.DrawingArea):
    def __init__(self):
        super().__init__()
        self.background = GdkPixbuf.Pixbuf.new_from_file(str(assets['courtyard']))
        self.hero = GdkPixbuf.Pixbuf.new_from_file(str(assets['hero']))
        self.connect('draw', self.paint)

    @staticmethod
    def image(context, pixbuf, x, y, scale):
        context.save()
        context.translate(x, y)
        context.scale(scale, scale)
        Gdk.cairo_set_source_pixbuf(context, pixbuf, 0, 0)
        context.paint()
        context.restore()

    def paint(self, widget, context):
        width, height = widget.get_allocated_width(), widget.get_allocated_height()
        image = self.background
        scale = max(width / image.get_width(), height / image.get_height())
        self.image(context, image, (width - image.get_width() * scale) / 2,
                   (height - image.get_height() * scale) / 2, scale)
        context.set_source_rgba(.01, .035, .065, .19)
        context.paint()
        scale = height * .99 / self.hero.get_height()
        self.image(context, self.hero, -width * .025, height - self.hero.get_height() * scale, scale)
        return False


class Launcher:
    PAGES = {
        'Play': 'icon/pointer',
        'News': 'icon/book',
        'Updates': 'icon/loading',
        'Account': 'icon/character',
        'Settings': 'control/gear/default',
    }

    def __init__(self, args, game_args):
        self.prefs = args.settings_file.expanduser().resolve()
        self.available = None
        self.job = None
        self.game = None
        self.closing = False
        self.cancelled = threading.Event()
        self.game_args = game_args
        self.record = {'current': None, 'previous': None}
        config = {}
        settings_error = None
        try:
            config = read_settings(self.prefs)
        except (OSError, client_install.InstallError) as error:
            settings_error = str(error)
        self.install_root = (args.install_root or Path(
            config.get('install_root', data_directory() / 'game')
        )).expanduser().resolve()
        library = args.release_library or config.get('release_library')
        self.library = Path(library).expanduser().resolve() if library else None

        css = Gtk.CssProvider()
        css.load_from_path(str(HERE / 'launcher.css'))
        Gtk.StyleContext.add_provider_for_screen(Gdk.Screen.get_default(), css,
                                                  Gtk.STYLE_PROVIDER_PRIORITY_APPLICATION)
        artwork = Gtk.CssProvider()
        art_rules = []
        for selector, image in (('', 'default'), (':hover', 'hover'), (':disabled', 'disabled')):
            path = (SPRITES / 'button/primary' / f'{image}.png').as_uri()
            art_rules.append(f'button.primary{selector} {{ background-image: url("{path}"); }}')
        artwork.load_from_data('\n'.join(art_rules).encode())
        Gtk.StyleContext.add_provider_for_screen(Gdk.Screen.get_default(), artwork,
                                                  Gtk.STYLE_PROVIDER_PRIORITY_APPLICATION + 1)
        self.window = Gtk.Window(title='Theia Online Launcher')
        self.window.set_icon_from_file(str(assets['icon']))
        self.window.set_default_size(1230, 720)
        self.window.set_size_request(880, 560)
        self.window.connect('delete-event', self.close)
        self.window.connect('destroy', Gtk.main_quit)
        root = box(vertical=True, spacing=0)
        self.window.add(root)

        header = Gtk.HeaderBar(show_close_button=True)
        logo = GdkPixbuf.Pixbuf.new_from_file_at_scale(str(assets['title']), 159, 68, True)
        header.pack_start(Gtk.Image.new_from_pixbuf(logo))
        nav = box(spacing=5)
        self.page_buttons = {}
        for page, icon in self.PAGES.items():
            control = button(page, lambda page=page: self.select_page(page))
            pixbuf = GdkPixbuf.Pixbuf.new_from_file_at_scale(str(SPRITES / (icon + '.png')),
                                                             20, 20, True)
            control.set_image(Gtk.Image.new_from_pixbuf(pixbuf))
            control.set_always_show_image(True)
            nav.pack_start(control, False, False, 0)
            self.page_buttons[page] = control
        header.set_custom_title(nav)
        self.window.set_titlebar(header)

        overlay = Gtk.Overlay()
        overlay.add(Scene())
        root.pack_start(overlay, True, True, 0)
        surface = box(vertical=True, spacing=13)
        surface.set_size_request(360, -1)
        surface.set_halign(Gtk.Align.END)
        surface.set_valign(Gtk.Align.START)
        surface.set_margin_top(28)
        surface.set_margin_end(22)
        surface.get_style_context().add_class('surface')
        overlay.add_overlay(surface)
        self.stack = Gtk.Stack()
        self.stack.set_hhomogeneous(True)
        self.stack.set_vhomogeneous(False)
        self.stack.set_transition_type(Gtk.StackTransitionType.CROSSFADE)
        surface.pack_start(self.stack, True, True, 0)
        self.build_pages()

        footer = box(vertical=True, spacing=9)
        footer.get_style_context().add_class('footer')
        root.pack_start(footer, False, False, 0)
        row = box(spacing=16)
        footer.pack_start(row, False, False, 0)
        status = box(vertical=True, spacing=8)
        status.get_style_context().add_class('status-card')
        row.pack_start(status, True, True, 0)
        status_top = box()
        status.pack_start(status_top, False, False, 0)
        self.status = label('Checking local installation…', 'section')
        status_top.pack_start(self.status, True, True, 0)
        self.cancel_button = button('Cancel', self.cancel)
        status_top.pack_end(self.cancel_button, False, False, 0)
        self.progress = Gtk.ProgressBar()
        self.progress.set_show_text(True)
        status.pack_start(self.progress, False, False, 0)
        self.primary_button = button('Install Game', self.primary, 'primary')
        row.pack_end(self.primary_button, False, False, 0)

        bottom = box()
        footer.pack_start(bottom, False, False, 0)
        bottom.pack_start(label('Server status  ·  Unknown', 'muted'), False, False, 0)
        self.version_label = label('', 'muted')
        bottom.pack_end(self.version_label, False, False, 0)
        self.select_page('Play')
        self.window.show_all()
        self.refresh_view()
        if settings_error:
            self.status.set_text(settings_error)
        else:
            self.check()
        GLib.timeout_add(400, self.poll_game)

    def page(self, name, title):
        page = box(vertical=True, spacing=12)
        page.pack_start(label(title, 'heading'), False, False, 0)
        page.pack_start(Gtk.Separator(), False, False, 1)
        self.stack.add_named(page, name)
        return page

    def build_pages(self):
        play = self.page('Play', 'Latest News')
        self.summary = label('', 'detail')
        play.pack_start(self.summary, False, False, 0)
        play.pack_start(button('Release details', lambda: self.select_page('Updates')), False, False, 0)

        news = self.page('News', 'News')
        news.pack_start(label('No news feed is configured.', 'detail'), False, False, 0)

        updates = self.page('Updates', 'Updates')
        self.release_label = label('', 'detail')
        updates.pack_start(self.release_label, False, False, 0)
        self.check_button = button('Check for Updates', self.check)
        updates.pack_start(self.check_button, False, False, 0)
        self.update_button = button('Install Update', self.install)
        updates.pack_start(self.update_button, False, False, 0)
        self.rollback_button = button('Restore Previous Version', self.rollback)
        updates.pack_start(self.rollback_button, False, False, 0)

        account = self.page('Account', 'Account')
        copy = label('Sign in and manage your characters in the game client.', 'detail')
        copy.set_max_width_chars(38)
        account.pack_start(copy, False, False, 0)
        self.account_button = button('Open Game', self.play)
        account.pack_start(self.account_button, False, False, 0)

        settings = self.page('Settings', 'Settings')
        settings.pack_start(label('Release library', 'section'), False, False, 0)
        self.source_label = self.path_label(settings)
        self.source_button = button('Choose Release Folder…', self.choose_source)
        settings.pack_start(self.source_button, False, False, 0)
        settings.pack_start(label('Game installation', 'section'), False, False, 0)
        self.root_label = self.path_label(settings)
        self.root_button = button('Choose Game Folder…', self.choose_install_root)
        settings.pack_start(self.root_button, False, False, 0)

    @staticmethod
    def path_label(parent):
        widget = label('', 'detail')
        widget.set_ellipsize(Pango.EllipsizeMode.END)
        widget.set_max_width_chars(38)
        parent.pack_start(widget, False, False, 0)
        return widget

    def select_page(self, page):
        self.stack.set_visible_child_name(page)
        for name, control in self.page_buttons.items():
            style = control.get_style_context()
            if name == page:
                style.add_class('nav-active')
            else:
                style.remove_class('nav-active')

    def refresh_view(self):
        current, previous = self.record['current'], self.record['previous']
        busy = self.job is not None
        running = self.game is not None
        ready = not (busy or running)
        self.primary_button.set_label('Play' if current else 'Install Game')
        self.primary_button.set_sensitive(ready)
        self.account_button.set_sensitive(ready and bool(current))
        self.check_button.set_sensitive(ready)
        self.update_button.set_sensitive(ready and self.available is not None)
        self.update_button.set_label('Install Update' if current else 'Install Game')
        self.rollback_button.set_sensitive(ready and bool(previous))
        self.source_button.set_sensitive(ready)
        self.root_button.set_sensitive(ready)
        self.cancel_button.set_visible(self.job == 'install')
        self.cancel_button.set_sensitive(not self.cancelled.is_set())
        self.progress.set_visible(self.job == 'install')
        self.source_label.set_text(str(self.library) if self.library else 'No folder selected')
        self.source_label.set_tooltip_text(str(self.library) if self.library else None)
        self.root_label.set_text(str(self.install_root))
        self.root_label.set_tooltip_text(str(self.install_root))
        self.version_label.set_text(f'Client {current}' if current else 'Client not installed')
        if self.available:
            _, release, total = self.available
            offer = f'Version {release} available  ·  {size_text(total)}'
        elif self.library:
            offer = 'No newer release in the selected folder.' if current else 'No compatible release found.'
        else:
            offer = 'Choose a release folder in Settings.'
        self.release_label.set_text(offer + (f'\nPrevious version {previous}' if previous else ''))
        self.summary.set_text(offer)

    def pick_folder(self, title, folder):
        chooser = Gtk.FileChooserNative.new(title, self.window, Gtk.FileChooserAction.SELECT_FOLDER,
                                             'Choose', 'Cancel')
        chooser.set_modal(True)
        chooser.set_local_only(True)
        if folder and folder.is_dir():
            chooser.set_current_folder(str(folder))
        try:
            return Path(chooser.get_filename()) if chooser.run() == Gtk.ResponseType.ACCEPT else None
        finally:
            chooser.destroy()

    def set_paths(self, root, library):
        try:
            save_settings(self.prefs, root, library)
        except OSError as error:
            self.status.set_text(str(error))
            return
        self.install_root, self.library = root, library
        self.record = {'current': None, 'previous': None}
        self.available = None
        self.check()

    def choose_source(self):
        chosen = self.pick_folder('Choose a local client package or release library', self.library)
        if chosen is not None:
            self.set_paths(self.install_root, chosen)

    def choose_install_root(self):
        chosen = self.pick_folder('Choose the game installation folder', self.install_root)
        if chosen is not None:
            self.set_paths(chosen, self.library)

    def start(self, job, message, operation, done):
        if self.job is not None or self.game is not None:
            return
        self.job = job
        self.cancelled.clear()
        self.status.set_text(message)
        self.refresh_view()

        def work():
            try:
                result = operation()
            except (OSError, client_install.InstallError) as error:
                GLib.idle_add(self.finished, done, None, str(error))
            else:
                GLib.idle_add(self.finished, done, result, None)

        threading.Thread(target=work).start()

    def finished(self, done, result, error):
        self.job = None
        if self.closing:
            self.window.destroy()
            return GLib.SOURCE_REMOVE
        if error:
            self.status.set_text(error)
        else:
            done(result)
        self.refresh_view()
        return GLib.SOURCE_REMOVE

    def check(self, message=None):
        def discover():
            record = client_install.active(self.install_root)
            try:
                available = newest_release(self.library, record['current'])
            except (OSError, client_install.InstallError) as error:
                return record, None, str(error)
            return record, available, None

        def checked(result):
            self.record, self.available, error = result
            fallback = 'Ready to play' if self.record['current'] else (
                'Ready to install' if self.available else 'Choose a client release'
            )
            self.status.set_text(error or message or fallback)

        self.start('check', 'Checking for releases…', discover, checked)

    def primary(self):
        if self.record['current']:
            self.play()
        elif self.available:
            self.install()
        else:
            self.choose_source()

    def install(self):
        if not self.available:
            return
        package, offered_version, _ = self.available
        self.progress.set_fraction(0)
        self.progress.set_text('0%')
        last_draw = 0.

        def report(done, total):
            nonlocal last_draw
            now = time.monotonic()
            if done == total or now - last_draw >= .09:
                last_draw = now
                GLib.idle_add(self.draw_progress, done, total)

        def complete(release):
            self.check(f'Client {release} installed')

        def apply_release():
            return client_install.install(package, self.install_root, report, self.cancelled.is_set,
                                          offered_version=offered_version)

        self.start('install', 'Installing game…' if not self.record['current'] else 'Updating game…',
                   apply_release, complete)

    def draw_progress(self, done, total):
        fraction = done / total
        self.progress.set_fraction(fraction)
        self.progress.set_text(f'{fraction:.0%}   ·   {size_text(done)} / {size_text(total)}')
        return GLib.SOURCE_REMOVE

    def cancel(self):
        self.cancelled.set()
        self.cancel_button.set_sensitive(False)
        self.status.set_text('Cancelling installation…')

    def rollback(self):
        self.start('rollback', 'Verifying the previous version…',
                   lambda: client_install.rollback(self.install_root),
                   lambda release: self.check(f'Client {release} restored'))

    def play(self):
        if not self.record['current']:
            return
        self.start('play', 'Verifying installed client…',
                   lambda: client_install.launch(self.install_root, self.game_args), self.playing)

    def playing(self, process):
        self.game = process
        self.status.set_text('Game is running')

    def poll_game(self):
        if self.game is not None:
            code = self.game.poll()
            if code is not None:
                self.game = None
                self.status.set_text('Ready to play' if code == 0 else f'Game exited with code {code}')
                self.refresh_view()
        return GLib.SOURCE_CONTINUE

    def close(self, *_):
        if self.job == 'install':
            self.closing = True
            self.cancel()
            return True
        return self.job is not None


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--install-root', type=Path)
    parser.add_argument('--release-library', type=Path)
    parser.add_argument('--settings-file', type=Path, default=data_directory() / 'launcher.json')
    args = sys.argv[1:]
    split = args.index('--') if '--' in args else len(args)
    options = parser.parse_args(args[:split])
    if not Gtk.init_check()[0]:
        parser.exit(1, 'Theia Launcher needs a desktop display.\n')
    Launcher(options, args[split + 1:])
    Gtk.main()


if __name__ == '__main__':
    main()
