"""Embedded GTK sign-in and challenge prompts for the hosted Linux launcher."""
from threading import Event
import webbrowser

from gi.repository import GLib, Gtk

from client_install import InstallError
from cognito_native import sign_in

ACCOUNT_URL = 'https://theiaonline.com/account/'


class NativeLoginPanel(Gtk.Box):
    def __init__(self, launcher, button, label):
        super().__init__(orientation=Gtk.Orientation.VERTICAL, spacing=1)
        self.launcher = launcher
        self.reply = None
        self.get_style_context().add_class('sign-in-panel')
        heading = label('Sign in to Theia', 'heading')
        heading.set_halign(Gtk.Align.START)
        self.pack_start(heading, False, False, 0)
        self.pack_start(Gtk.Separator(), False, False, 0)
        body = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=1)
        body.get_style_context().add_class('panel-body')
        self.pack_start(body, False, False, 0)
        self.username_label = label('_Username', 'sign-in-label')
        self.username = Gtk.Entry()
        self.secret_label = label('_Password', 'sign-in-label')
        self.secret = Gtk.Entry()
        self.secret.set_visibility(False)
        self.secret.set_input_purpose(Gtk.InputPurpose.PASSWORD)
        self.field_labels = Gtk.SizeGroup(mode=Gtk.SizeGroupMode.HORIZONTAL)
        fields = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=6)
        for field, entry in ((self.username_label, self.username), (self.secret_label, self.secret)):
            field.set_use_underline(True)
            field.set_mnemonic_widget(entry)
            self.field_labels.add_widget(field)
            entry.set_width_chars(12)
            entry.connect('activate', lambda _: self.respond())
            group = Gtk.Box(spacing=12)
            group.pack_start(field, False, False, 0)
            group.pack_start(entry, True, True, 0)
            fields.pack_start(group, False, False, 0)
        body.pack_start(fields, False, False, 0)
        self.status = label('', 'sign-in-status')
        self.status.set_no_show_all(True)
        body.pack_start(self.status, False, False, 0)
        self.submit = button('Sign in', self.respond, 'sign-in-submit')
        self.help_button = button('Account help', self.open_account, 'sign-in-secondary')
        self.help_button.set_tooltip_text('Set up your account or reset your password on the Theia website')
        self.actions = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=6)
        self.actions.pack_start(self.submit, False, False, 0)
        self.actions.pack_start(self.help_button, False, False, 0)
        body.pack_start(self.actions, False, False, 0)
        self.show_status('')

    def show_status(self, message):
        self.status.set_text(message)
        self.status.set_visible(bool(message))
        self.actions.set_margin_top(0 if message else 5)

    def dismiss(self):
        self.secret.set_text('')
        self.reply = None

    def prompt(self, kind, message, check):
        """Called from the auth worker, with the panel updated by the GTK thread."""
        if kind == 'new_password' and self.launcher.public_realm:
            raise InstallError('Finish setting up this account using Account help.')
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
        self.help_button.set_sensitive(False)
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
        self.submit.set_sensitive(True)
        self.help_button.set_sensitive(True)
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

    def open_account(self):
        self.secret.set_text('')
        try:
            if webbrowser.open(ACCOUNT_URL):
                return
        except (OSError, webbrowser.Error):
            pass
        self.show_status(f'Open {ACCOUNT_URL} in your browser for account help.')

    def cancel(self):
        self.dismiss()
        if self.launcher.job is None:
            self.launcher.dismiss_login()
        else:
            self.launcher.cancelled.set()
            self.busy('Cancelling…')
