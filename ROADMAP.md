# The Ashen Chronicle

## Roadmap

The roadmap tracks current and upcoming development. Detailed completed milestone history is kept in [`docs/roadmap-history.md`](docs/roadmap-history.md).

## Current state
### v0.50.14: Android storage and touch input fixes
- Corrected Android bundled-asset paths so the packaged `data/` content is copied into the app-local game root before native gameplay starts.
- Added Android-side diagnostics for IME events and the `BrowserBack` key event so device input-delivery behavior can be verified without changing the existing text or navigation consumers.
- Added tap-versus-drag gating for graphical choice buttons so touching and dragging a scrollable panel no longer immediately confirms a choice.
- Updated the Android Gradle fallback version metadata to match the project version.
- Fixed the touch-choice interaction query so touch-button state can be cleared without a Rust mutability error under the current stable toolchain.
- Android device verification of storage synchronization, IME delivery, system Back delivery, and touch gesture behavior remains outstanding.

### v0.50.12: Android GameActivity entry-point export fix
- Replaced Bevy's generated private Android entry-point wrapper with a public `android_main` symbol in the library target that produces the packaged native `.so`.
- Kept artifact-level CI validation of the exported `android_main` symbol using the Android NDK's `llvm-nm`.
- Kept a non-empty native-library output check before APK packaging.

### v0.50.11: Android GameActivity entry-point fix
- Moved Bevy's Android entry-point function into the library target that produces the packaged native `.so`.
- Restored artifact-level CI validation of the exported `android_main` symbol using the Android NDK's `llvm-nm`.
- Kept a non-empty native-library output check before APK packaging.

### v0.50.10: Android entry-point CI guard fix
- Replaced the stripped-library `android_main` symbol check with source/configuration validation that is compatible with the release profile.
- Kept the native Android build and added a non-empty native-library output check before APK packaging.

### v0.50.9: Android armv7 CI symbol-check fix
- Corrected Android CI native-library validation to inspect defined `android_main` symbols instead of requiring dynamic export visibility, allowing the armv7 build to proceed to APK packaging.

### v0.50.8: Android SAF storage access
- Replaced Android All Files Access with user-selected Storage Access Framework directory access.
- Kept Rust filesystem I/O on the app-scoped Android game root while mirroring editable mods and saves to the selected shared folder.
- Persisted the selected folder permission across launches and synchronized shared data into the local game root on startup.
- Migrated existing local mods/saves into an empty newly selected shared folder without overwriting an existing shared game dataset.
- Kept the bundled protected base content out of the user-editable shared storage mirror.

### v0.50.7: Android launch crash fix
- Added Bevy's Android entry-point macro so GameActivity can invoke the native application entry point.
- Added an Android CI guard that fails the build when `android_main` is not exported by the native library.

### v0.50.6: Android APK CI packaging and signing
- Added CI packaging for installable signed Android APKs using the existing Gradle/GameActivity project.
- Preserved the existing Android x86_64, AArch64, and ARMv7 targets with per-ABI native library packaging.
- Added workflow-dispatch support for building a selected Android target without a release tag.
- Added secure release signing through GitHub Actions secrets, with short-lived CI-only signing for manual test builds when persistent secrets are unavailable.
- Added APK signature verification and Android build-log artifacts.
- Release-tag Android assets are now APKs; the legacy standalone Android executable tarballs are removed from the published release while desktop release artifacts remain unchanged.

### v0.50.5: Cross-platform game-root selection and Android screen audit
- Added native desktop game-root use/recreate/alternate-folder selection.
- Hardened protected base-content replacement and corrected lifecycle Load Back navigation.
- Corrected Combat semantic Cancel for Android system Back and completed the static audit of all existing Bevy screens against the shared responsive/touch architecture; final manual Android verification remains outstanding.

### v0.50.4: Android touch, soft-keyboard, and system Back input
- Added reusable contextual touch behavior to the shared Bevy presentation layer while preserving the frontend-neutral semantic input queues and desktop keyboard mappings.
- Ordinary Bevy choice buttons remain direct-touch actions, while character-creation fields and the developer-console input gain contextual touch focus controls.
- Added a compact contextual Tab button beside the developer-console input without reserving a permanent control row.
- Added touch-drag scrolling for shared scrollable panels so long gameplay, records, dialogue, console, and lifecycle content remains usable on touch-only devices.
- Added shared Android soft-keyboard IME integration; committed IME text is routed through the existing keyboard text-input path, and text focus controls keyboard visibility without Android-specific gameplay logic.
- Added Android Back key handling through the shared semantic Cancel input, including suppression of the navigation action when the soft keyboard is being dismissed.
- Completed semantic delete handling in the developer console while retaining existing physical keyboard behavior.

## Next

Run the Android CI workflow for v0.50.14, install the resulting APK on a physical Android device, verify that packaged assets populate the app-local game root and that selected shared storage synchronizes correctly, then inspect the Android input diagnostics while testing text entry and system Back before making any further input-layer changes.

## Longer-term direction

Continue modularizing the codebase by responsibility rather than by file size alone. Keep runtime state, content loading, gameplay actions, presentation, persistence, and event processing independently understandable and testable.

Major gameplay work should remain data-driven where practical, preserve backward compatibility, and include focused tests for behavior affected by the change.
