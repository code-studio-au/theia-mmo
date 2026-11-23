# Theia Online launcher

The launcher installs and starts the game. The native game window owns sign-in,
registration and character selection. The launcher does not need account
credentials or access to character saves.

Use the supplied Theia Online launcher concept as the visual reference. Reuse
the game's navy panels, gold trim, illustrated title and sapphire Play button.
Play, News, Updates, Account and Settings follow the reference layout. Account
hands off to the game for sign-in. News waits for a real release feed. Server
status displays Unknown until it has a verified source. An update shows its
actual progress, and Play reports the installed version it will open.

The first P2-14/P3-01 launcher slice uses an exported local build. Verify the
target platform and package contents, stage each version separately, and switch
the active version after verification completes. Keep the preceding version and
store preferences outside the install directories. A failed or cancelled update
leaves the current build playable. Launch an installed game through its normal
native entry point.

For developer testing, export each validated build as another version. The
launcher adopts the new version on the next launch. Installed clients need the
matching server protocol and gameplay profiles. Build exports and activation
remain local until release distribution is authorized.

A later distribution slice adds signed release metadata, HTTPS delivery and
verified resumable downloads. Start with complete versioned packages. Measure
package size and update time before adding delta patching. Keep the installer
independent of the game process so a running game can finish before activation.

Acceptance starts with two local builds and a failed update. Install the first
version, launch to native Login, stage the second and show real progress.
Activate it, launch to native Login again, and verify preferences survive.
Reject wrong platform, invalid paths, corrupted files and refused activation.
Confirm that a failed update leaves the first verified build available.

## Local installer implemented

`tools/client_install.py` exports a prepared client tree, stages an update
with SHA-256 and size verification, switches the current version atomically,
and retains the previous version for rollback. Progress reports actual bytes
verified. The version directories are separate from user preferences. The
installer operates on trusted local exports.

```sh
python3 tools/client_install.py export build/launcher-staging build/client-release --version 0.1.0.12
python3 tools/client_install.py install build/client-release build/local-client-install
python3 tools/client_install.py status build/local-client-install
python3 tools/client_install.py play build/local-client-install -- --window-size 1280x720
```

Use a staging directory containing the built `theia-client` (or the Windows
`theia-client.exe`) and its release assets. Pass the same game arguments the
installed build normally receives. The desktop launcher is the next slice.

Nine installer tests cover update, rollback, cancellation, corrupt files,
invalid paths, concurrent update refusal and the actual executable handoff.
Two distinct Linux release binaries, 283,873,240 and 283,846,888 bytes after
stripping debug information, installed and rendered from their respective
version directories, with zero Vulkan diagnostics. A corrupted third release
failed before activation. Rollback restored the earlier active version, and
preferences outside the release directory retained their bytes. This was an
offscreen Linux Vulkan llvmpipe check. See
`build/launcher-local-update-v2/qualification.json`.
