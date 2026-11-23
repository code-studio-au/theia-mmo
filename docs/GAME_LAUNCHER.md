# Theia Online launcher

The desktop launcher is a separate Linux AMD64 application with its own Debian
package and application menu entry. The package installs the launcher and its
artwork. The launcher then installs the game into a separate directory. The
native game window owns sign-in, registration and character selection.

## Local development use

Build the standalone launcher package with `dpkg-deb` available on the build
machine.

```sh
python3 tools/package_client_launcher.py build/theia-launcher_0.1.3_amd64.deb
```

Install the Debian package through the system's package installer and open
**Theia Online Launcher**. In **Settings**, choose a trusted client release
folder containing `client-release.json`, or a library containing versioned
subfolders. The default game installation is
`$XDG_DATA_HOME/theia-online/game` (normally `~/.local/share/theia-online/game`).
The launcher saves its folder choices in `launcher.json` beside that default
directory.

**Install Game** stages and verifies the selected release, then enables **Play**.
Play opens the game fullscreen by default and minimises the launcher. When the
game closes, the launcher is available again. Forward `--window-size WIDTHxHEIGHT`
after `--` for a windowed launch or use F11 to toggle fullscreen in the client.
When a newer release is found, the main action becomes **Update Game**.
**Updates** also offers **Check for Updates**, **Restore Previous Version** and
**Play Installed Version** while a newer release is offered. Install and update
show verified bytes, percentage and, after enough time to measure the transfer,
a rate and estimated time left. Cancel stops the transfer. The bottom bar
displays the active client version.

The Play screen links to the bundled 0.1.0.1 client preview announcement.

## Launcher updates

Install the 0.1.3 launcher package once to enable checks for future launcher
releases. At startup and on **Updates → Check for Updates**, the launcher looks
in the `launcher/` subfolder of the selected trusted release library for Debian
packages named `theia-launcher_VERSION_amd64.deb`. It checks the package name,
version and architecture, then offers the newest compatible version above the
running version. The Updates tab displays launcher and game client versions
independently, and Play shows a link to any available launcher update.

Select **Update Launcher** to install the offered package with the system APT
installer and desktop administrator approval. **Restart Launcher** becomes
available after the system confirms the offered version is installed. The app
keeps its settings, game installation and launch arguments across the restart.
Publish packages only to a library you trust. This path uses local files.

## Login server status

The launcher probes the development account service's `GET /ready` every five
seconds. Green **Online** requires the account service's readiness response.
Other responses or connection failures show **Unavailable**. The default is
`127.0.0.1:5050`. With a custom numeric loopback bind, set
`THEIA_ACCOUNT_BIND` for the launcher or pass `--login-server 127.0.0.1:PORT`.
Use the matching current account server build.

The endpoint is available on the existing guarded account listener once
startup has completed. The probe sends no credentials, starts no login session
and does not test a complete identity-provider sign-in. Public realm
monitoring remains separate.

## Preparing a client release

Export each validated build to a new version folder in the same library.

```sh
python3 tools/client_install.py export build/launcher-staging build/client-releases/0.1.0.12 --version 0.1.0.12
theia-launcher --release-library build/client-releases -- --window-size 1280x720
```

Stage the compiled `theia-client` and the assets required by that build. In
development, use the normal private account playtest to start and configure
matching local services. Pass its usual client arguments after `--` when
starting the launcher. The same local installer also exposes the `install`,
`status`, `rollback` and `play` subcommands for command-line use.

## Activation and testing

The installer verifies the platform, SHA-256 and size of each file before
switching versions atomically. Installation failures and cancellation preserve
the active version. A desktop update cannot activate an older release after a
concurrent update or silently install a different version from the one offered.
Launch verifies the installed files and starts the game from its active version
directory. A running game can continue using its version files.

Sixteen installer/launcher checks and 25 server library tests pass. The
packaged app was exercised on Linux X11 with two distinct local client builds,
a rejected corrupt update and rollback. Captures cover game not installed,
installing, update required, updating and game ready, with real byte progress.
The 967×600 layout and top-right maximise, restore and minimise controls were
checked. The launcher reported Online against the running private account
service. The installed client then opened native Login at 1280×720 and signed
in to a fresh account roster. Linux Vulkan llvmpipe reported zero validation
diagnostics. See `build/launcher-theme-qualified-65dre1f2/qualification.json`.

The 0.1.2 package opened installed client 0.1.0.4 fullscreen at 1440×900 on
Linux X11 with Vulkan llvmpipe. The launcher minimised on Play and restored
when the client closed. All 19 installer and launcher checks passed.

For 0.1.3, all 22 checks passed with an APT upgrade simulation. On private
Linux X11, the packaged GTK app detected a newer launcher on startup and on
manual check, installed it into an isolated Debian package root and restarted
into the updated app with game 0.1.0.4 still active. The test substituted the
isolated Debian install for the live desktop administrator prompt.

## Distribution remains open

P6-09 began with trusted local full releases. Hosted delivery needs signed
release metadata, HTTPS, verified resumable downloads and a published realm
configuration. Windows still needs its own launcher installer and native
qualification. Measure full package downloads before selecting a delta format.
