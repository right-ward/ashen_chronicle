# Android CI builds

The Android CI workflow produces installable, signed APK artifacts for the supported Android ABIs:

- `android-aarch64` → `arm64-v8a`
- `android-armv7` → `armeabi-v7a`
- `android-x86_64` → `x86_64`

Release tags build all three APKs and upload them to the corresponding GitHub Release. The workflow removes the legacy Android executable tarballs from that release so the Android release artifacts are installable APKs. A `workflow_dispatch` run builds one selected target and exposes the signed APK as a workflow artifact without requiring a release tag.

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

## Local Android project

The Gradle project is under `android/`. It uses Bevy's `GameActivity` integration and expects the Rust `cdylib` to be copied into the ABI-specific `jniLibs` directory before Gradle packages the APK.

The CI workflow performs that native-library build and packaging automatically. Local release signing is optional; when the signing environment variables are absent, the Gradle release build remains unsigned.
