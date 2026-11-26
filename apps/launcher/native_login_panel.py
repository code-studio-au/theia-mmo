"""Embedded GTK sign-in and challenge prompts for the hosted Linux launcher."""
from threading import Event

from gi.repository import GLib, Gtk

from cognito_native import sign_in


class NativeLoginPanel(Gtk.Box):
    def __init__(self, launcher, button, label):
        super().__init__(orientation=Gtk.Orientation.VERTICAL, spacing=6)
        self.launcher = launcher
        self.reply = None
        self.set_valign(Gtk.Align.CENTER)
        self.get_style_context().add_class('sign-in-panel')
        heading = label('Sign in to Theia', 'sign-in-heading')
        heading.set_halign(Gtk.Align.START)
        self.field_width = Gtk.SizeGroup(mode=Gtk.SizeGroupMode.HORIZONTAL)
        self.field_width.add_widget(heading)
        self.pack_start(heading, False, False, 0)
        self.pack_start(Gtk.Separator(), False, False, 0)
        self.username_label = label('_Username', 'sign-in-label')
        self.username = Gtk.Entry()
        self.secret_label = label('_Password', 'sign-in-label')
        self.secret = Gtk.Entry()
        self.secret.set_visibility(False)
        self.secret.set_input_purpose(Gtk.InputPurpose.PASSWORD)
        for field, entry in ((self.username_label, self.username), (self.secret_label, self.secret)):
            field.set_use_underline(True)
            field.set_mnemonic_widget(entry)
            entry.set_width_chars(12)
            entry.set_max_width_chars(12)
            entry.set_halign(Gtk.Align.START)
            self.field_width.add_widget(entry)
            entry.connect('activate', lambda _: self.respond())
            group = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=3)
            group.pack_start(field, False, False, 0)
            group.pack_start(entry, False, False, 0)
            self.pack_start(group, False, False, 0)
        self.status = label('', 'sign-in-status')
        self.status.set_no_show_all(True)
        self.pack_start(self.status, False, False, 0)
        actions = Gtk.Box(spacing=6)
        self.back = button('Cancel', self.cancel, 'sign-in-secondary')
        self.submit = button('Sign in', self.respond, 'sign-in-submit')
        self.browser = button('Use browser', self.use_browser, 'sign-in-secondary')
        self.browser.set_tooltip_text('Sign in with browser')
        actions.pack_start(self.back, False, False, 0)
        actions.pack_start(self.browser, False, False, 0)
        actions.pack_end(self.submit, True, True, 0)
        self.pack_start(actions, False, False, 0)

    def show_status(self, message):
        self.status.set_text(message)
        self.status.set_visible(bool(message))

    def dismiss(self):
        self.secret.set_text('')
        self.reply = None

    def prompt(self, kind, message, check):
        """Called from the auth worker, with the panel updated by the GTK thread."""
        ready = Event()
        value = []

        def submitted(answer):
            value.append(answer)
            ready.set()

        def show():
            if self.launcher.login_panel is self and not self.launcher.cancelled.is_set():
                self.reply = submitted
                self.show_status(message)
                self.username.get_parent().hide()
                code = kind.endswith('_code')
                self.secret_label.set_text_with_mnemonic('Verification _code' if code else 'New _password')
                self.secret.set_input_purpose(Gtk.InputPurpose.PIN if code else Gtk.InputPurpose.PASSWORD)
                self.submit.set_label('Continue')
                self.submit.set_sensitive(True)
                self.secret.set_sensitive(True)
                self.secret.grab_focus()
            return GLib.SOURCE_REMOVE

        GLib.idle_add(show)
        while not ready.wait(.1):
            check()
        check()
        return value.pop()

    def busy(self, message):
        self.browser.set_sensitive(False)
        self.submit.set_sensitive(False)
        self.username.set_sensitive(False)
        self.secret.set_sensitive(False)
        self.show_status(message)

    def failed(self, message):
        self.reply = None
        self.username.get_parent().show()
        self.username.set_sensitive(True)
        self.secret.set_sensitive(True)
        self.secret_label.set_text_with_mnemonic('_Password')
        self.secret.set_input_purpose(Gtk.InputPurpose.PASSWORD)
        self.submit.set_label('Sign in')
        self.back.set_sensitive(True)
        self.submit.set_sensitive(True)
        self.browser.set_sensitive(True)
        self.show_status(message)
        self.secret.grab_focus()

    def respond(self):
        if not self.submit.get_sensitive():
            return
        if self.reply is not None:
            answer = self.secret.get_text()
            if not answer:
                self.show_status('Enter a response to continue.')
                return
            self.secret.set_text('')
            self.busy('Verifying…')
            reply, self.reply = self.reply, None
            reply(answer)
        elif self.launcher.job is None:
            credentials = [self.username.get_text().strip(), self.secret.get_text()]
            self.secret.set_text('')
            if not all(credentials):
                self.failed('Enter your username and password.')
                if not credentials[0]:
                    self.username.grab_focus()
                return
            self.busy('Signing in…')

            def authenticate(cancelled):
                return sign_in(self.launcher.realm.identity, credentials.pop(0),
                               credentials.pop(), cancelled, self.prompt)

            self.launcher.start_hosted_play(authenticate)

    def use_browser(self):
        if self.launcher.job is None:
            self.launcher.dismiss_login()
            self.launcher.start_hosted_play()

    def cancel(self):
        self.dismiss()
        if self.launcher.job is None:
            self.launcher.dismiss_login()
        else:
            self.launcher.cancelled.set()
            self.busy('Cancelling…')
            self.back.set_sensitive(False)
