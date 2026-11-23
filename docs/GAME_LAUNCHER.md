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
