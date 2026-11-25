"""GTK sign-in and challenge prompts for the hosted Linux launcher."""
from threading import Event

from gi.repository import GLib, Gtk

from cognito_native import sign_in

BROWSER = 10


class NativeLoginDialog(Gtk.Dialog):
    def __init__(self, launcher):
        super().__init__(title='Sign in to Theia Online', transient_for=launcher.window, modal=True)
        self.launcher = launcher
        self.reply = None
        self.add_button('Cancel', Gtk.ResponseType.CANCEL)
        self.add_button('Sign in with browser', BROWSER)
        self.submit = self.add_button('Sign in', Gtk.ResponseType.OK)
        self.submit.get_style_context().add_class('suggested-action')
        self.set_default_response(Gtk.ResponseType.OK)
        self.set_resizable(False)

        body = self.get_content_area()
        body.set_spacing(8)
        body.set_border_width(18)
        heading = Gtk.Label(label='Sydney development realm', xalign=0)
        heading.get_style_context().add_class('heading')
        body.pack_start(heading, False, False, 0)
        self.username_label = Gtk.Label(label='Username', xalign=0)
        self.username = Gtk.Entry()
        self.username.set_activates_default(True)
        self.secret_label = Gtk.Label(label='Password', xalign=0)
        self.secret = Gtk.Entry()
        self.secret.set_activates_default(True)
        self.secret.set_visibility(False)
        self.secret.set_input_purpose(Gtk.InputPurpose.PASSWORD)
        self.status = Gtk.Label(label='Sign in with your approved tester account.', xalign=0)
        self.status.set_line_wrap(True)
        self.status.set_width_chars(40)
        for item in (self.username_label, self.username, self.secret_label, self.secret, self.status):
            body.pack_start(item, False, False, 0)
        self.connect('response', self.respond)
        self.show_all()
        self.username.grab_focus()

    def dismiss(self):
        self.secret.set_text('')
        self.reply = None
        self.destroy()

    def prompt(self, kind, message, check):
        """Called from the auth worker, with the dialog updated by the GTK thread."""
        ready = Event()
        value = []

        def submitted(answer):
            value.append(answer)
            ready.set()

        def show():
            if self.launcher.login_dialog is self and not self.launcher.cancelled.is_set():
                self.reply = submitted
                self.status.set_text(message)
                self.username_label.hide()
                self.username.hide()
                code = kind.endswith('_code')
                self.secret_label.set_text('Verification code' if code else 'New password')
                self.secret.set_input_purpose(Gtk.InputPurpose.PIN if code else Gtk.InputPurpose.PASSWORD)
                self.submit.set_label('Continue')
                self.set_response_sensitive(Gtk.ResponseType.OK, True)
                self.secret.set_sensitive(True)
                self.secret.grab_focus()
            return GLib.SOURCE_REMOVE

        GLib.idle_add(show)
        while not ready.wait(.1):
            check()
        check()
        return value.pop()

    def busy(self, message):
        self.set_response_sensitive(BROWSER, False)
        self.set_response_sensitive(Gtk.ResponseType.OK, False)
        self.username.set_sensitive(False)
        self.secret.set_sensitive(False)
        self.status.set_text(message)

    def failed(self, message):
        self.reply = None
        self.username_label.show()
        self.username.show()
        self.username.set_sensitive(True)
        self.secret.set_sensitive(True)
        self.secret_label.set_text('Password')
        self.secret.set_input_purpose(Gtk.InputPurpose.PASSWORD)
        self.submit.set_label('Sign in')
        self.set_response_sensitive(Gtk.ResponseType.CANCEL, True)
        self.set_response_sensitive(Gtk.ResponseType.OK, True)
        self.set_response_sensitive(BROWSER, True)
        self.status.set_text(message)
        self.secret.grab_focus()

    def respond(self, _widget, response):
        if response == Gtk.ResponseType.OK:
            if self.reply is not None:
                answer = self.secret.get_text()
                self.secret.set_text('')
                self.busy('Verifying…')
                reply, self.reply = self.reply, None
                reply(answer)
            elif self.launcher.job is None:
                credentials = [self.username.get_text().strip(), self.secret.get_text()]
                self.secret.set_text('')
                if not all(credentials):
                    self.failed('Enter your username and password.')
                    return
                self.busy('Signing in…')
                def authenticate(cancelled):
                    return sign_in(self.launcher.realm.identity, credentials.pop(0),
                                   credentials.pop(), cancelled, self.prompt)
                self.launcher.start_hosted_play(authenticate)
        elif response == BROWSER and self.launcher.job is None:
            self.launcher.dismiss_login()
            self.launcher.start_hosted_play()
        elif response in (Gtk.ResponseType.CANCEL, Gtk.ResponseType.DELETE_EVENT):
            if self.launcher.job is None:
                self.launcher.dismiss_login()
            else:
                self.launcher.cancelled.set()
                self.busy('Cancelling…')
                self.set_response_sensitive(Gtk.ResponseType.CANCEL, False)
