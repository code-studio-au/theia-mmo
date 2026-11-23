# Theia Online launcher

The desktop launcher is a separate Linux AMD64 application with its own Debian
package and application menu entry. The package installs the launcher and its
artwork. The launcher then installs the game into a separate directory. The
native game window owns sign-in, registration and character selection.

## Local development use

Build the standalone launcher package with `dpkg-deb` available on the build
machine.

```sh
python3 tools/package_client_launcher.py build/theia-launcher_0.1.0_amd64.deb
```

Install the Debian package through the system's package installer and open
**Theia Online Launcher**. In **Settings**, choose a trusted client release
folder containing `client-release.json`, or a library containing versioned
subfolders. The default game installation is
`$XDG_DATA_HOME/theia-online/game` (normally `~/.local/share/theia-online/game`).
The launcher saves its folder choices in `launcher.json` beside that default
directory.

**Install Game** stages and verifies the selected release, then enables **Play**.
**Updates** shows the latest compatible version in the chosen library above the
installed version. **Check for Updates** rescans the library, **Install Update**
applies the new release, and **Restore Previous Version** rolls back the last
switch. The main panel shows the available release, and the status bar displays
the active game version. Progress shows bytes verified, with cancellation during
install or update. News and server status are waiting for actual feeds.

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

Fourteen automated checks cover release selection, settings, concurrent and
stale offers, cancellation, rollback, corruption and executable handoff. The
packaged app was exercised on Linux X11 with two distinct local client builds,
a rejected corrupt update and rollback. The current installed client opened its
native Login at 1280×720 and signed in to a fresh account roster using private
playtest services. Linux Vulkan llvmpipe reported zero validation diagnostics.
See `build/launcher-desktop-qualified-z5dig57a/qualification.json`.

## Distribution remains open

P6-09 began with trusted local full releases. Hosted delivery needs signed
release metadata, HTTPS, verified resumable downloads and a published realm
configuration. Windows still needs its own launcher installer and native
qualification. Measure full package downloads before selecting a delta format.
