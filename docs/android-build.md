# Android CI builds

The Android CI workflow produces installable, signed APK artifacts for the supported Android ABIs:

- `android-aarch64` → `arm64-v8a`
- `android-armv7` → `armeabi-v7a`
- `android-x86_64` → `x86_64`

A `workflow_dispatch` run supports two operations. `build` builds the selected ABI and exposes the signed APK as a workflow artifact without touching a GitHub Release. `release` rebuilds all three APKs from an existing `release_tag` and publishes them to that release. The `release` operation requires the persistent Android signing secrets. It is intended for release recovery or republishing a tag without deleting or moving the tag.

For manual runs, the workflow can generate a short-lived CI-only signing key when the Android signing secrets are absent. This is intended for personal/device testing. Release-tag builds require the persistent repository signing secrets below.

## Required repository secrets

Release builds expect these GitHub Actions repository secrets:

- `ANDROID_KEYSTORE_BASE64`: the base64-encoded Java keystore containing the release signing key
- `ANDROID_KEYSTORE_PASSWORD`: keystore password
- `ANDROID_KEY_ALIAS`: alias of the signing key
- `ANDROID_KEY_PASSWORD`: signing key password

The private keystore is decoded only into the runner's temporary directory and is never committed to the repository.

A new local keystore can be created with the JDK's `keytool`, for example:

```text
keytool -genkeypair -v \
  -keystore ashen-chronicle-release.keystore \
  -alias ashen-chronicle \
  -keyalg RSA \
  -keysize 2048 \
  -validity 10000
```

Encode that keystore for the `ANDROID_KEYSTORE_BASE64` secret without adding the file to the repository. The remaining secret values must match the keystore created above.

## Android storage

Android uses the Storage Access Framework rather than `MANAGE_EXTERNAL_STORAGE`. On first launch, the game asks the player to choose or create a folder for `The Ashen Chronicle` using the system folder picker.

The selected folder permission is persisted by Android. Rust continues to use an app-scoped filesystem directory for normal `std::fs` access, while the Android activity mirrors the editable `data/mods` and `saves` directories between that local root and the selected shared folder. The bundled `data/base_content.json` remains protected and is not synchronized into user-editable shared storage.

When the selected folder is empty, the existing local mods and saves are copied into it. When it already contains game data, that shared data is imported into the local game root instead. Subsequent saves/mod changes are synchronized back when the activity stops.

The selected directory should be the game folder itself (for example, `Documents/The Ashen Chronicle`), not its parent `Documents` directory.

## Local Android project

The Gradle project is under `android/`. It uses Bevy's `GameActivity` integration and expects the Rust `cdylib` to be copied into the ABI-specific `jniLibs` directory before Gradle packages the APK.

The CI workflow performs that native-library build and packaging automatically. APK packaging deliberately compresses the native `.so` library so the directly distributed APK is substantially smaller; this uses Gradle's `packagingOptions.jniLibs.useLegacyPackaging` setting. Local release signing is optional; when the signing environment variables are absent, the Gradle release build remains unsigned.
