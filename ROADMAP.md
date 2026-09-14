# The Ashen Chronicle

## Roadmap

The roadmap tracks current and upcoming development. Detailed completed milestone history is kept in [`docs/roadmap-history.md`](docs/roadmap-history.md).

## Current state
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

Run the Android CI workflow, install the resulting APK on a physical Android device, verify the supported touch/storage flows, then continue with #225.

## Longer-term direction

Continue modularizing the codebase by responsibility rather than by file size alone. Keep runtime state, content loading, gameplay actions, presentation, persistence, and event processing independently understandable and testable.

Major gameplay work should remain data-driven where practical, preserve backward compatibility, and include focused tests for behavior affected by the change.
