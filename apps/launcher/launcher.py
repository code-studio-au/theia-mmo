"""Theia Online desktop launcher for signed downloads and local releases."""
import argparse
import os
from pathlib import Path
import sys
import threading
import time
import webbrowser

import gi

gi.require_version('Gtk', '3.0')
gi.require_version('Gdk', '3.0')
from gi.repository import Gdk, GdkPixbuf, GLib, Gtk, Pango

from artwork import HERE, REPOSITORY, asset
from chrome import Glyph, WindowFrame, load_skin, picture, resize_edge, style, title_art

if not (HERE / 'assets').is_dir():
    sys.path.insert(0, str(REPOSITORY / 'tools'))

import client_install
from hosted_release import DEFAULT_URL, HostedLibrary, HostedRelease
from launcher_update import (
    SYSTEM_LAUNCHER, VERSION, install_launcher_update, newest_hosted_launcher_release, newest_launcher_release,
)
from state import DEFAULT_LOGIN_BIND, data_directory, game_launch_args, login_address, login_ready, newest_release
from state import read_settings, save_settings
from browser_sign_in import SignInConfig, rehearse_sign_in
from hosted_realm import HostedRealm
from native_login_dialog import NativeLoginDialog
from news_feed import NEWS_URL, fetch_news


def box(vertical=False, spacing=10):
    direction = Gtk.Orientation.VERTICAL if vertical else Gtk.Orientation.HORIZONTAL
    return Gtk.Box(orientation=direction, spacing=spacing)


def label(text='', theme=None):
    widget = Gtk.Label(label=text, xalign=0)
    widget.set_line_wrap(True)
    widget.set_max_width_chars(39)
    if theme:
        style(widget, theme)
    return widget


def button(text, action, theme='action', icon=None, icon_only=False):
    widget = style(Gtk.Button(), theme)
    widget.connect('clicked', lambda _: action())
    if icon is None:
        widget.set_label(text)
    else:
        contents = box(spacing=8)
        contents.set_halign(Gtk.Align.CENTER)
        contents.pack_start(icon, False, False, 0)
        if not icon_only:
            widget.caption = Gtk.Label(label=text)
            contents.pack_start(widget.caption, False, False, 0)
        widget.add(contents)
        widget.get_accessible().set_name(text)
    if icon_only:
        widget.set_tooltip_text(text)
    return widget


def size_text(amount):
    if amount >= 1024**3:
        return f'{amount / 1024**3:.2f} GiB'
    return f'{amount / 1024**2:.1f} MiB'


class Scene(Gtk.DrawingArea):
    def __init__(self):
        super().__init__()
        self.background = GdkPixbuf.Pixbuf.new_from_file(str(asset('landscape')))
        self.hero = GdkPixbuf.Pixbuf.new_from_file(str(asset('hero')))
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
        context.set_source_rgba(.01, .02, .045, .06)
        context.paint()
        scale = height / self.hero.get_height()
        self.image(context, self.hero, -width * .014, 8, scale)
        return False


class Launcher:
    PAGES = ('Play', 'News', 'Settings')

    def __init__(self, args, game_args):
        self.prefs = args.settings_file.expanduser().resolve()
        self.available = None
        self.launcher_update = None
        self.launcher_installed = None
        self.launcher_feedback = None
        self.job = None
        self.game = None
        self.closing = False
        self.feedback = None
        self.probe_pending = False
        self.login_server = args.login_server or login_address(DEFAULT_LOGIN_BIND)
        self.realm = args.realm
        self.realm_feedback = None
        self.login_dialog = None
        self.cancelled = threading.Event()
        self.game_args = game_launch_args(game_args)
        self.test_login = args.test_login
        self.developer_tools = args.developer_tools
        self.test_login_status = 'Use the browser to sign in to the test identity provider.'
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
        self.library = args.release_library.expanduser().resolve() if args.release_library else None
        self.local_play = args.local_sandbox
        keys = sorted((HERE / 'release-keys').glob('*.der'))
        self.hosted = HostedLibrary(DEFAULT_URL, keys) if keys else None
        self.public_realm = bool(self.hosted and not (self.realm or self.local_play or self.test_login))
        self.show_login_status = bool(self.realm or self.public_realm or self.library
                                      or self.developer_tools or args.login_server)

        load_skin(HERE / 'launcher.css')
        self.window = Gtk.Window(title='Theia Online Launcher')
        self.window.set_icon_from_file(str(asset('icon')))
        self.window.set_decorated(False)
        self.window.set_default_size(1270, 720)
        self.window.set_size_request(950, 590)
        self.window.add_events(Gdk.EventMask.BUTTON_PRESS_MASK)
        self.window.connect('button-press-event', self.window_drag)
        self.window.connect('delete-event', self.close)
        self.window.connect('destroy', Gtk.main_quit)
        self.window.connect('window-state-event', self.window_state)

        chrome = Gtk.Overlay()
        self.window.add(chrome)
        root = box(vertical=True, spacing=0)
        chrome.add(root)
        frame = WindowFrame()
        chrome.add_overlay(frame)
        chrome.set_overlay_pass_through(frame, True)
        self.header = self.build_header()
        root.pack_start(self.header, False, False, 0)

        overlay = Gtk.Overlay()
        overlay.add(Scene())
        root.pack_start(overlay, True, True, 0)
        surface = style(box(vertical=True), 'surface')
        surface.set_size_request(406, -1)
        surface.set_halign(Gtk.Align.END)
        surface.set_valign(Gtk.Align.START)
        surface.set_margin_top(15)
        surface.set_margin_end(17)
        overlay.add_overlay(surface)
        self.stack = Gtk.Stack()
        self.stack.set_hhomogeneous(True)
        self.stack.set_vhomogeneous(False)
        self.stack.set_transition_type(Gtk.StackTransitionType.CROSSFADE)
        scroll = Gtk.ScrolledWindow()
        scroll.set_policy(Gtk.PolicyType.NEVER, Gtk.PolicyType.AUTOMATIC)
        scroll.set_propagate_natural_height(True)
        scroll.set_propagate_natural_width(True)
        scroll.add(self.stack)
        surface.pack_start(scroll, True, True, 0)

        def fit_panel(_, allocation):
            height = max(100, allocation.height - 63)
            if scroll.get_max_content_height() != height:
                GLib.idle_add(scroll.set_max_content_height, height)

        overlay.connect('size-allocate', fit_panel)
        self.build_pages()
        root.pack_start(self.build_footer(), False, False, 0)
        self.select_page('Play')
        self.window.show_all()
        self.feedback = settings_error
        self.refresh_view()
        self.check(settings_error)
        threading.Thread(target=self.load_news, daemon=True).start()
        GLib.timeout_add(400, self.poll_game)
        if self.show_login_status:
            GLib.timeout_add_seconds(5, self.probe_login)
            if not self.public_realm:
                self.probe_login()

    def build_header(self):
        header = style(Gtk.Overlay(), 'app-header')
        row = box(spacing=36)
        header.add(row)
        logo = title_art()
        logo.set_margin_start(40)
        logo.set_margin_top(12)
        logo.set_margin_bottom(5)
        row.pack_start(logo, False, False, 0)
        navigation = style(box(spacing=5), 'navigation')
        navigation.set_valign(Gtk.Align.END)
        navigation.set_margin_bottom(5)
        navigation.set_margin_end(70)
        row.pack_start(navigation, True, True, 0)
        icons = (Glyph('play', 25), picture('news-icon', 25, 25), picture('settings-icon', 25, 25))
        self.page_buttons = {}
        for page, icon in zip(self.PAGES, icons):
            control = button(page, lambda page=page: self.select_page(page), 'nav', icon)
            navigation.pack_start(control, True, True, 0)
            self.page_buttons[page] = control

        controls = box(spacing=2)
        controls.set_halign(Gtk.Align.END)
        controls.set_valign(Gtk.Align.START)
        controls.set_margin_top(6)
        controls.set_margin_end(12)
        self.maximise_glyph = Glyph('maximise', 23)
        self.maximise_button = button('Maximise', self.toggle_maximise, 'window-control',
                                      self.maximise_glyph, icon_only=True)
        for control in (
            button('Minimise', self.window.iconify, 'window-control', Glyph('minimise', 23), icon_only=True),
            self.maximise_button,
            button('Close', self.window.close, 'window-control', Glyph('close', 23), icon_only=True),
        ):
            controls.pack_start(control, False, False, 0)
        header.add_overlay(controls)

        ornament = style(Gtk.Box(), 'header-ornament')
        ornament.set_halign(Gtk.Align.CENTER)
        ornament.set_valign(Gtk.Align.START)
        ornament.set_size_request(160, 12)
        header.add_overlay(ornament)
        header.set_overlay_pass_through(ornament, True)
        return header

    def build_footer(self):
        footer = style(box(vertical=True, spacing=10), 'footer')
        row = box(spacing=16)
        footer.pack_start(row, False, False, 0)
        status = style(box(spacing=15), 'status-card')
        row.pack_start(status, True, True, 0)
        self.install_glyph = style(Glyph('install', 52), 'status-icon')
        self.install_glyph.set_valign(Gtk.Align.CENTER)
        status.pack_start(self.install_glyph, False, False, 0)
        text = box(vertical=True, spacing=4)
        text.set_valign(Gtk.Align.CENTER)
        status.pack_start(text, True, True, 0)
        self.status = label('Checking local installation…', 'section')
        text.pack_start(self.status, False, False, 0)
        self.progress_row = box(spacing=9)
        self.progress = Gtk.ProgressBar()
        self.progress.set_valign(Gtk.Align.CENTER)
        self.progress_row.pack_start(self.progress, True, True, 0)
        self.percent = label('0%', 'progress-value')
        self.progress_row.pack_start(self.percent, False, False, 0)
        text.pack_start(self.progress_row, False, False, 0)
        self.detail = label('', 'detail')
        self.detail.set_max_width_chars(84)
        text.pack_start(self.detail, False, False, 0)
        actions = box(spacing=6)
        actions.set_valign(Gtk.Align.CENTER)
        status.pack_end(actions, False, False, 0)
        self.cancel_button = button('Cancel', self.cancel, 'action', Glyph('close', 20), icon_only=True)
        actions.pack_start(self.cancel_button, False, False, 0)
        actions.pack_start(button('Launcher Settings', lambda: self.select_page('Settings'), 'action',
                                  picture('settings-icon', 22, 22), icon_only=True), False, False, 0)

        self.primary_glyph = Glyph('install', 29)
        self.primary_button = button('Install Game', self.primary, 'primary', self.primary_glyph)
        row.pack_end(self.primary_button, False, False, 0)

        bottom = style(box(spacing=8), 'footer-line')
        footer.pack_start(bottom, False, False, 0)
        if self.show_login_status:
            self.login_dot = style(Gtk.Box(), 'login-dot')
            self.login_dot.set_valign(Gtk.Align.CENTER)
            bottom.pack_start(self.login_dot, False, False, 0)
            status_label = label('Realm Status' if self.realm or self.public_realm else 'Login Server Status',
                                 'server-label')
            bottom.pack_start(status_label, False, False, 0)
            self.login_label = label('Checking…', 'detail')
            bottom.pack_start(self.login_label, False, False, 0)
        self.version_label = label('', 'version')
        bottom.pack_end(self.version_label, False, False, 0)
        return footer

    def toggle_maximise(self):
        if self.window.is_maximized():
            self.window.unmaximize()
        else:
            self.window.maximize()

    def window_state(self, _, event):
        maximised = bool(event.new_window_state & Gdk.WindowState.MAXIMIZED)
        self.maximise_glyph.show_kind('restore' if maximised else 'maximise')
        title = 'Restore' if maximised else 'Maximise'
        self.maximise_button.get_accessible().set_name(title)
        self.maximise_button.set_tooltip_text(title)
        return False

    def window_drag(self, widget, event):
        if event.button != 1:
            return False
        target = Gtk.get_event_widget(event)
        while target is not None and target is not widget:
            if isinstance(target, Gtk.Button):
                return False
            target = target.get_parent()
        origin_x, origin_y = widget.get_window().get_origin()[-2:]
        x, y = event.x_root - origin_x, event.y_root - origin_y
        edge = resize_edge(x, y, widget.get_allocated_width(), widget.get_allocated_height())
        pointer = (event.button, int(event.x_root), int(event.y_root), event.time)
        if edge is not None and not widget.is_maximized() and event.type == Gdk.EventType.BUTTON_PRESS:
            widget.begin_resize_drag(edge, *pointer)
            return True
        if y < self.header.get_allocated_height():
            if event.type == Gdk.EventType.DOUBLE_BUTTON_PRESS:
                self.toggle_maximise()
            elif event.type == Gdk.EventType.BUTTON_PRESS:
                widget.begin_move_drag(*pointer)
            return True
        return False

    def probe_login(self):
        realm = self.realm
        if self.public_realm and realm is None:
            return GLib.SOURCE_CONTINUE
        if not self.probe_pending:
            self.probe_pending = True

            def probe():
                online = realm.ready() if realm else login_ready(self.login_server)
                GLib.idle_add(self.login_state, online)

            threading.Thread(target=probe, daemon=True).start()
        return GLib.SOURCE_CONTINUE

    def login_state(self, online):
        self.probe_pending = False
        inactive = 'Sign in to start' if self.realm and self.realm.gateway else 'Unavailable'
        self.login_label.set_text('Online' if online else inactive)
        for widget in (self.login_dot, self.login_label):
            color = widget.get_style_context()
            color.remove_class('online' if not online else 'unavailable')
            color.add_class('online' if online else 'unavailable')
        return GLib.SOURCE_REMOVE

    def page(self, name, title):
        page = box(vertical=True, spacing=12)
        heading = label(title, 'heading')
        if name == 'Play':
            lead = box(spacing=7)
            lead.pack_start(label('✦', 'heading-star'), False, False, 0)
            lead.pack_start(heading, False, False, 0)
            page.pack_start(lead, False, False, 0)
        else:
            page.pack_start(heading, False, False, 0)
        page.pack_start(Gtk.Separator(), False, False, 1)
        self.stack.add_named(page, name)
        return page

    def build_pages(self):
        play = self.page('Play', 'Latest News')
        self.latest_news = box(vertical=True)
        play.pack_start(self.latest_news, False, False, 0)
        play.pack_start(button('All News', lambda: self.select_page('News')), False, False, 0)

        news = self.page('News', 'News')
        self.news_articles = box(vertical=True)
        news.pack_start(self.news_articles, False, False, 0)
        news.pack_start(button('News on the Website', lambda: webbrowser.open(NEWS_URL)), False, False, 0)
        for panel in (self.latest_news, self.news_articles):
            panel.add(label('Loading news…', 'detail'))

        settings = self.page('Settings', 'Settings')
        settings.pack_start(label(f'Launcher {VERSION}', 'detail'), False, False, 0)
        self.check_button = button('Check for Updates', self.check)
        settings.pack_start(self.check_button, False, False, 0)
        self.launcher_release_label = label('', 'detail')
        settings.pack_start(self.launcher_release_label, False, False, 0)
        settings.pack_start(label('Game installation', 'section'), False, False, 0)
        self.root_label = self.path_label(settings)
        self.root_button = button('Choose Game Folder…', self.choose_install_root)
        settings.pack_start(self.root_button, False, False, 0)
        if self.developer_tools:
            self.rollback_button = button('Restore Previous Version', self.rollback)
            settings.pack_start(self.rollback_button, False, False, 0)
            self.play_installed = button('Play Installed Version', self.play)
            settings.pack_start(self.play_installed, False, False, 0)
        if self.test_login:
            self.test_login_label = label(self.test_login_status, 'detail')
            settings.pack_start(self.test_login_label, False, False, 0)
            self.test_login_button = button('Test Sign-in', self.start_test_login)
            settings.pack_start(self.test_login_button, False, False, 0)

    def load_news(self):
        try:
            articles = fetch_news()
        except (OSError, ValueError):
            articles = []
        GLib.idle_add(self.show_news, articles)

    def show_news(self, articles):
        if self.closing:
            return GLib.SOURCE_REMOVE
        for panel, entries in ((self.latest_news, articles[:1]), (self.news_articles, articles)):
            for child in panel.get_children():
                panel.remove(child)
            if not entries:
                panel.add(label('News is unavailable right now.', 'detail'))
            for article in entries:
                card = style(Gtk.Button(), 'news-story')
                card.connect('clicked', lambda _, url=article.url: webbrowser.open(url))
                card.get_accessible().set_name(article.title)
                copy = box(vertical=True, spacing=5)
                copy.pack_start(label(article.title, 'news-title'), False, False, 0)
                if article.summary:
                    copy.pack_start(label(article.summary, 'detail'), False, False, 0)
                card.add(copy)
                panel.pack_start(card, False, False, 0)
            panel.show_all()
        return GLib.SOURCE_REMOVE

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

    @property
    def launcher_action(self):
        if self.launcher_installed:
            return 'Restart Launcher'
        if self.launcher_update:
            return 'Update Launcher'
        return None

    def refresh_view(self):
        current, previous = self.record['current'], self.record['previous']
        busy = self.job is not None
        running = self.game is not None
        ready = not (busy or running)
        transferring = self.job == 'install'
        update = bool(current and self.available)
        game_action = 'Play' if transferring or (current and not update) else (
            'Update Game' if update else 'Install Game'
        )
        launcher_action = self.launcher_action
        play_ready = (current == self.realm.client_version if self.realm else self.local_play)
        action_name = launcher_action or game_action
        self.primary_button.caption.set_text(action_name)
        self.primary_button.get_accessible().set_name(action_name)
        self.primary_glyph.show_kind('update' if launcher_action else {
            'Play': 'play', 'Update Game': 'update', 'Install Game': 'install',
        }[game_action])
        self.install_glyph.show_kind('update' if launcher_action or update else 'ready' if current else 'install')
        can_act = play_ready if game_action == 'Play' else bool(self.library or self.hosted)
        self.primary_button.set_sensitive(ready and bool(launcher_action or can_act))
        if self.test_login:
            signing_in = self.job == 'test-login'
            self.test_login_label.set_text(self.test_login_status)
            self.test_login_button.set_label('Cancel Sign-in' if signing_in else 'Test Sign-in')
            self.test_login_button.set_sensitive(ready or (signing_in and not self.cancelled.is_set()))
        self.check_button.set_sensitive(ready)
        if self.developer_tools:
            game_ready = ready and not launcher_action
            self.rollback_button.set_sensitive(game_ready and bool(previous))
            self.play_installed.set_visible(update)
            self.play_installed.set_sensitive(game_ready and play_ready)
        self.root_button.set_sensitive(ready)
        self.cancel_button.set_visible(transferring or bool(self.realm and self.job == 'play'))
        self.cancel_button.set_sensitive(not self.cancelled.is_set())
        self.progress_row.set_visible(transferring)
        if self.job == 'launcher-update':
            launcher_status = 'Verifying the package and opening the system installer…'
        elif self.launcher_feedback:
            launcher_status = self.launcher_feedback
        elif self.launcher_installed:
            launcher_status = f'Version {self.launcher_installed} installed. Restart to finish.'
        elif self.launcher_update:
            launcher_status = f'Version {self.launcher_update[1]} is available.'
        else:
            launcher_status = ''
        self.launcher_release_label.set_text(launcher_status)
        self.launcher_release_label.set_visible(bool(launcher_status))

        self.root_label.set_text(str(self.install_root))
        self.root_label.set_tooltip_text(str(self.install_root))
        self.version_label.set_text(f'Client v{current}' if current else 'Client not installed')
        if self.available:
            package, release, total = self.available
            size = f'{size_text(total)} download' if isinstance(package, HostedRelease) else size_text(total)
            offer = f'Version {release} available  ·  {size}'
        elif self.library:
            offer = 'No newer release in the selected folder.' if current else 'No compatible release found.'
        elif self.hosted:
            offer = 'No newer client release.' if current else 'Check for a client release.'
        else:
            offer = 'Use --release-library to select a local development export.'
        if self.job is None:
            if running:
                title = 'Game is running'
            elif self.launcher_installed:
                title = 'Restart required'
            elif launcher_action:
                title = 'Launcher update required'
            elif update:
                title = 'Update required'
            elif current:
                title = 'Game ready'
            else:
                title = 'Game not installed'
            self.status.set_text(title)
        if not transferring:
            play_error = ''
            if current and not play_ready and not self.available:
                play_error = (self.realm_feedback or
                              (f'Install client {self.realm.client_version} to play.' if self.realm
                               else 'Theia realm information is unavailable. Check for updates to retry.'))
            self.detail.set_text(launcher_status if launcher_action else self.feedback or play_error or offer)


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

    def choose_install_root(self):
        chosen = self.pick_folder('Choose the game installation folder', self.install_root)
        if chosen is None:
            return
        try:
            save_settings(self.prefs, chosen)
        except OSError as error:
            self.feedback = str(error)
            self.refresh_view()
            return
        self.install_root = chosen.resolve()
        self.record = {'current': None, 'previous': None}
        self.available = None
        self.check()

    def start(self, job, message, operation, done):
        if self.job is not None or self.game is not None:
            return
        self.job = job
        self.feedback = None
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
        job = self.job
        self.job = None
        if self.closing:
            self.window.destroy()
            return GLib.SOURCE_REMOVE
        if error:
            if job == 'play' and self.login_dialog is not None:
                if self.cancelled.is_set():
                    self.dismiss_login()
                else:
                    self.login_dialog.failed(error)
            elif job == 'launcher-update':
                self.launcher_feedback = error
            elif job == 'test-login':
                self.test_login_status = error
            else:
                self.feedback = error
        else:
            done(result)
        self.refresh_view()
        return GLib.SOURCE_REMOVE

    def check(self, message=None):
        def discover():
            launcher = None
            launcher_error = None
            try:
                if self.hosted and not self.library:
                    launcher = newest_hosted_launcher_release(self.hosted, VERSION)
                else:
                    launcher = newest_launcher_release(self.library, VERSION)
            except (OSError, client_install.InstallError) as error:
                launcher_error = str(error)
            previous_offer = self.launcher_update
            if self.hosted and not self.library and previous_offer and (
                    launcher is None or client_install.version_key(launcher[1]) < client_install.version_key(previous_offer[1])):
                launcher = previous_offer
            try:
                record = client_install.active(self.install_root)
            except (OSError, client_install.InstallError) as error:
                return self.record, None, str(error), launcher, launcher_error, self.realm, None
            if launcher:
                return record, None, None, launcher, launcher_error, self.realm, None
            realm, realm_error = self.realm, None
            if self.public_realm:
                try:
                    realm = HostedRealm.from_public(self.hosted.realm_metadata())
                except (OSError, client_install.InstallError):
                    realm, realm_error = None, 'Theia realm information is unavailable. Check for updates to retry.'
            try:
                if not self.library and self.hosted:
                    available = self.hosted.newest(record['current'], VERSION, install_root=self.install_root)
                else:
                    available = newest_release(self.library, record['current'])
            except (OSError, client_install.InstallError) as error:
                available, client_error = None, str(error)
            else:
                client_error = None
            return record, available, client_error, None, launcher_error, realm, realm_error

        def checked(result):
            (self.record, self.available, error, self.launcher_update, self.launcher_feedback,
             self.realm, self.realm_feedback) = result
            self.feedback = error or message
            if self.public_realm:
                if self.realm:
                    self.probe_login()
                elif self.realm_feedback:
                    self.login_state(False)

        self.start('check', 'Checking for releases…', discover, checked)

    def update_launcher(self):
        if self.launcher_installed:
            try:
                os.execv(SYSTEM_LAUNCHER, [str(SYSTEM_LAUNCHER), *sys.argv[1:]])
            except OSError as error:
                self.launcher_feedback = str(error)
                self.refresh_view()
            return
        offer = self.launcher_update
        if offer is None:
            return
        self.launcher_feedback = None

        def complete(release):
            self.launcher_installed = release
            self.launcher_update = None

        self.start('launcher-update', 'Updating launcher…', lambda: install_launcher_update(offer), complete)

    def primary(self):
        if self.launcher_action:
            self.update_launcher()
        elif self.available:
            self.install()
        elif self.record['current']:
            self.play()
        else:
            self.check()

    def start_test_login(self):
        if self.job == 'test-login':
            self.cancelled.set()
            self.test_login_status = 'Cancelling test sign-in…'
            self.refresh_view()
            return
        if not self.test_login or self.job or self.game:
            return
        self.test_login_status = 'Finish signing in in your browser.'

        def complete(_subject):
            self.test_login_status = 'Test sign-in succeeded.'

        self.start('test-login', 'Signing in…',
                   lambda: rehearse_sign_in(self.test_login, self.cancelled.is_set), complete)

    def install(self):
        if self.launcher_action or not self.available:
            return
        package, offered_version, _ = self.available
        self.progress.set_fraction(0)
        self.percent.set_text('0%')
        self.detail.set_text(f'Preparing v{offered_version}…')
        started = time.monotonic()
        last_draw = 0.

        def report(done, total):
            nonlocal last_draw
            now = time.monotonic()
            if done == total or now - last_draw >= .09:
                last_draw = now
                GLib.idle_add(self.draw_progress, done, total, now - started)

        def complete(release):
            self.check(f'Client {release} installed')

        def apply_release():
            if isinstance(package, HostedRelease):
                return package.install(self.install_root, report, self.cancelled.is_set,
                                       offered_version=offered_version)
            return client_install.install(package, self.install_root, report, self.cancelled.is_set,
                                          offered_version=offered_version)

        self.start('install', 'Installing game…' if not self.record['current'] else 'Updating game…',
                   apply_release, complete)

    def draw_progress(self, done, total, elapsed):
        fraction = done / total
        self.progress.set_fraction(fraction)
        self.percent.set_text(f'{fraction:.0%}')
        text = f'{size_text(done)} / {size_text(total)}'
        if done and elapsed >= 1:
            rate = done / elapsed
            seconds = int((total - done) / rate)
            text += f'  ·  {size_text(rate)}/s'
            if seconds >= 60:
                minutes, seconds = divmod(seconds, 60)
                text += f'  ·  About {minutes}m {seconds}s left'
            elif seconds:
                text += f'  ·  About {seconds}s left'
        self.detail.set_text(text)
        return GLib.SOURCE_REMOVE

    def cancel(self):
        self.cancelled.set()
        self.cancel_button.set_sensitive(False)
        self.status.set_text('Cancelling…')

    def rollback(self):
        if not self.developer_tools or self.launcher_action:
            return
        self.start('rollback', 'Verifying the previous version…',
                   lambda: client_install.rollback(self.install_root),
                   lambda release: self.check(f'Client {release} restored'))

    def play(self):
        if self.launcher_action or not self.record['current'] or self.job is not None or self.game is not None:
            return
        if self.realm:
            if self.record['current'] != self.realm.client_version:
                return
            if self.realm.native_sign_in:
                if self.login_dialog is None:
                    self.login_dialog = NativeLoginDialog(self)
                else:
                    self.login_dialog.present()
            else:
                self.start_hosted_play()
            return
        if self.local_play:
            self.start('play', 'Verifying installed client…',
                       lambda: client_install.launch(self.install_root, self.game_args), self.playing)

    def start_hosted_play(self, authenticate=None):
        if self.launcher_action or not self.realm:
            return
        self.start('play', 'Signing in…' if authenticate else 'Sign in with your browser…',
                   lambda: self.realm.launch(self.install_root, self.game_args, self.cancelled.is_set,
                                             lambda message: GLib.idle_add(self.realm_progress, message),
                                             authenticate=authenticate),
                   self.playing)

    def dismiss_login(self):
        if self.login_dialog is not None:
            dialog, self.login_dialog = self.login_dialog, None
            dialog.dismiss()

    def realm_progress(self, message):
        if self.job == 'play' and not self.cancelled.is_set():
            self.status.set_text(message)
            if self.login_dialog is not None:
                self.login_dialog.status.set_text(message)
        return GLib.SOURCE_REMOVE

    def playing(self, process):
        self.dismiss_login()
        self.game = process
        self.window.iconify()

    def poll_game(self):
        if self.game is not None:
            code = self.game.poll()
            if code is not None:
                error = getattr(self.game, 'theia_hosted_error', None)
                self.game = None
                self.feedback = error or (None if code == 0 else f'Game exited with code {code}')
                self.refresh_view()
                if self.window.get_window().get_state() & Gdk.WindowState.ICONIFIED:
                    self.window.present()
        return GLib.SOURCE_CONTINUE

    def close(self, *_):
        if self.job == 'test-login' or (self.realm and self.job == 'play'):
            self.closing = True
            self.cancelled.set()
            return True
        if self.job == 'install':
            self.closing = True
            self.cancel()
            return True
        return self.job is not None


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--install-root', type=Path)
    parser.add_argument('--release-library', type=Path, help='Use a local development release folder')
    parser.add_argument('--developer-tools', action='store_true', help='Show previous-client controls in Settings')
    parser.add_argument('--local-sandbox', action='store_true', help='Play without a realm in local development')
    parser.add_argument('--login-server', type=login_address, default=os.environ.get('THEIA_ACCOUNT_BIND'),
                        help='Show readiness for a local game login service at this loopback address')
    parser.add_argument('--settings-file', type=Path, default=data_directory() / 'launcher.json')
    parser.add_argument('--test-login-config', type=Path,
                        help='Opt in to an isolated browser sign-in rehearsal')
    parser.add_argument('--realm-config', type=Path, help='Connect Play to the configured hosted realm')
    args = sys.argv[1:]
    split = args.index('--') if '--' in args else len(args)
    options = parser.parse_args(args[:split])
    try:
        options.realm = HostedRealm(options.realm_config.resolve(strict=True)) if options.realm_config else None
        if sum(map(bool, (options.realm, options.test_login_config, options.local_sandbox))) > 1:
            parser.error('Choose a hosted realm, local sandbox, or test sign-in rehearsal')
        options.test_login = SignInConfig.load(options.test_login_config) if options.test_login_config else None
    except (OSError, client_install.InstallError) as error:
        parser.error(f'Could not load the launcher sign-in configuration: {error}')
    if not Gtk.init_check()[0]:
        parser.exit(1, 'Theia Launcher needs a desktop display.\n')
    Launcher(options, args[split + 1:])
    Gtk.main()


if __name__ == '__main__':
    main()
