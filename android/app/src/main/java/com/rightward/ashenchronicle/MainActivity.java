package com.rightward.ashenchronicle;

import android.app.AlertDialog;
import android.content.Intent;
import android.net.Uri;
import android.os.Bundle;
import android.os.Environment;
import android.os.Looper;
import android.util.Log;
import android.text.Editable;
import android.text.InputType;
import android.text.TextWatcher;
import android.view.KeyEvent;
import android.view.View;
import android.view.inputmethod.EditorInfo;
import android.view.inputmethod.InputMethodManager;
import android.widget.EditText;
import android.widget.TextView;

import androidx.activity.OnBackPressedCallback;
import androidx.core.view.WindowCompat;
import androidx.core.view.WindowInsetsCompat;
import androidx.core.view.WindowInsetsControllerCompat;

import androidx.documentfile.provider.DocumentFile;

import com.google.androidgamesdk.GameActivity;

import java.io.File;
import java.io.FileOutputStream;
import java.io.IOException;
import java.io.InputStream;
import java.io.InputStreamReader;
import java.io.OutputStream;
import java.io.OutputStreamWriter;
import java.nio.charset.StandardCharsets;
import java.nio.file.AtomicMoveNotSupportedException;
import java.nio.file.Files;
import java.nio.file.StandardCopyOption;

import org.json.JSONException;
import org.json.JSONObject;

public class MainActivity extends GameActivity {
    private static final String TAG = "AshenChronicle";
    private static final String GAME_DIRECTORY_NAME = "The Ashen Chronicle";
    private static final String SHARED_STORAGE_URI_PREFERENCE = "shared_storage_tree_uri";
    private static final String GAME_CONFIG_FILE_NAME = "config.json";
    private static final String GAME_CONFIG_STORAGE_URI_KEY = "shared_storage_tree_uri";
    private static final int GAME_CONFIG_VERSION = 1;
    private static final int REQUEST_CODE_OPEN_TREE = 1001;
    private static final int ANDROID_TEXT_INPUT_TARGET_NONE = -1;

    private AndroidTextInputEditText androidTextInput;
    private int androidTextInputTarget = ANDROID_TEXT_INPUT_TARGET_NONE;
    private boolean suppressAndroidTextInputCallbacks;

    static {
        System.loadLibrary("ashen_chronicle");
    }

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        prepareGameRoot();
        super.onCreate(savedInstanceState);
        initializeAndroidTextInput();
        getOnBackPressedDispatcher().addCallback(this, new OnBackPressedCallback(true) {
            @Override
            public void handleOnBackPressed() {
                if (!dismissAndroidTextInput()) {
                    nativeBackNavigation();
                }
            }
        });
        maybePromptForSharedStorage();
    }

    @Override
    protected void onPause() {
        syncLocalStorageToShared();
        super.onPause();
    }

    @Override
    protected void onStop() {
        // Keep this as a final lifecycle flush even though onPause() already
        // synchronizes, since Android may reach onStop() through a separate
        // lifecycle path.
        syncLocalStorageToShared();
        super.onStop();
    }

    @Override
    protected void onActivityResult(int requestCode, int resultCode, Intent data) {
        super.onActivityResult(requestCode, resultCode, data);

        if (requestCode != REQUEST_CODE_OPEN_TREE || resultCode != RESULT_OK || data == null) {
            return;
        }

        Uri treeUri = data.getData();
        if (treeUri == null) {
            return;
        }

        int takeFlags = data.getFlags()
                & (Intent.FLAG_GRANT_READ_URI_PERMISSION | Intent.FLAG_GRANT_WRITE_URI_PERMISSION);
        if ((takeFlags & Intent.FLAG_GRANT_READ_URI_PERMISSION) == 0
                || (takeFlags & Intent.FLAG_GRANT_WRITE_URI_PERMISSION) == 0) {
            showStorageError("The selected folder did not grant both read and write access.");
            return;
        }

        String previousSelection = getPreferences(MODE_PRIVATE)
                .getString(SHARED_STORAGE_URI_PREFERENCE, null);
        try {
            getContentResolver().takePersistableUriPermission(treeUri, takeFlags);
            if (!hasPersistedTreePermission(treeUri)) {
                throw new SecurityException("The selected folder permission was not persisted");
            }

            DocumentFile selectedTree = DocumentFile.fromTreeUri(this, treeUri);
            if (selectedTree == null || !selectedTree.canRead() || !selectedTree.canWrite()) {
                throw new SecurityException("The selected folder does not support read and write access");
            }
        } catch (SecurityException exception) {
            Log.w(TAG, "Could not persist shared storage permission", exception);
            if (!treeUri.toString().equals(previousSelection)) {
                try {
                    getContentResolver().releasePersistableUriPermission(treeUri, takeFlags);
                } catch (SecurityException releaseException) {
                    Log.w(TAG, "Could not release the rejected folder permission", releaseException);
                }
            }
            showStorageError("The selected folder could not be granted persistent read and write access.");
            return;
        }

        getPreferences(MODE_PRIVATE)
                .edit()
                .putString(SHARED_STORAGE_URI_PREFERENCE, treeUri.toString())
                .apply();

        try {
            if (hasSharedStorageContent(treeUri)) {
                syncSharedStorageToLocal();
            } else {
                syncLocalStorageToShared();
            }
        } catch (IllegalArgumentException | SecurityException exception) {
            Log.w(TAG, "Could not synchronize the selected shared storage folder", exception);
            showStorageError("The folder was selected, but its contents could not be synchronized. The game will retry when storage is available.");
        }
    }

    /**
     * Finish the Android Activity from the native game's Quit action.
     *
     * Bevy's Android integration keeps its activity handle process-global, while
     * android-activity ties that handle to the current Activity instance. A
     * finished Activity must therefore be followed by a fresh process before the
     * launcher creates another game instance.
     */
    private static native void nativeBackNavigation();
    private static native void nativeAndroidTextInputChanged(int target, String text);
    private static native void nativeAndroidTextInputSubmitted(int target);
    private static native void nativeAndroidTextInputDismissed(int target);

    private final class AndroidTextInputEditText extends EditText {
        AndroidTextInputEditText(MainActivity context) {
            super(context);
        }

        @Override
        public boolean onKeyPreIme(int keyCode, KeyEvent event) {
            if (keyCode == KeyEvent.KEYCODE_BACK
                    && event.getAction() == KeyEvent.ACTION_UP
                    && androidTextInputTarget != ANDROID_TEXT_INPUT_TARGET_NONE) {
                dismissAndroidTextInput();
                return true;
            }
            return super.onKeyPreIme(keyCode, event);
        }
    }

    private void initializeAndroidTextInput() {
        View contentView = findViewById(android.R.id.content);
        if (!(contentView instanceof android.view.ViewGroup)) {
            Log.w(TAG, "Could not initialize Android text input: content view is not a ViewGroup");
            return;
        }

        AndroidTextInputEditText input = new AndroidTextInputEditText(this);
        input.setSingleLine(true);
        input.setInputType(InputType.TYPE_CLASS_TEXT | InputType.TYPE_TEXT_FLAG_NO_SUGGESTIONS);
        input.setImeOptions(
                EditorInfo.IME_ACTION_DONE
                        | EditorInfo.IME_FLAG_NO_EXTRACT_UI
                        | EditorInfo.IME_FLAG_NO_FULLSCREEN);
        input.setTextColor(0x00000000);
        input.setBackgroundColor(0x00000000);
        input.setCursorVisible(false);
        input.setAlpha(0.0f);
        input.setFocusable(true);
        input.setFocusableInTouchMode(true);
        input.setImportantForAutofill(View.IMPORTANT_FOR_AUTOFILL_NO);
        input.addTextChangedListener(new TextWatcher() {
            @Override
            public void beforeTextChanged(CharSequence source, int start, int count, int after) {
            }

            @Override
            public void onTextChanged(CharSequence source, int start, int before, int count) {
            }

            @Override
            public void afterTextChanged(Editable editable) {
                if (!suppressAndroidTextInputCallbacks
                        && androidTextInputTarget != ANDROID_TEXT_INPUT_TARGET_NONE) {
                    nativeAndroidTextInputChanged(androidTextInputTarget, editable.toString());
                }
            }
        });
        input.setOnEditorActionListener(new TextView.OnEditorActionListener() {
            @Override
            public boolean onEditorAction(
                    TextView view,
                    int actionId,
                    KeyEvent event) {
                if (androidTextInputTarget == ANDROID_TEXT_INPUT_TARGET_NONE) {
                    return false;
                }

                boolean enterKey =
                        event != null
                                && event.getKeyCode() == KeyEvent.KEYCODE_ENTER
                                && event.getAction() == KeyEvent.ACTION_UP;
                if (actionId == EditorInfo.IME_ACTION_DONE || enterKey) {
                    nativeAndroidTextInputSubmitted(androidTextInputTarget);
                    return true;
                }
                return false;
            }
        });

        android.view.ViewGroup.LayoutParams params =
                new android.view.ViewGroup.LayoutParams(1, 1);
        ((android.view.ViewGroup) contentView).addView(input, params);
        androidTextInput = input;
    }

    public void setAndroidTextInput(int target, String text) {
        Runnable update = () -> setAndroidTextInputOnUiThread(target, text);
        if (Looper.myLooper() == Looper.getMainLooper()) {
            update.run();
        } else {
            runOnUiThread(update);
        }
    }

    private void setAndroidTextInputOnUiThread(int target, String text) {
        if (androidTextInput == null) {
            return;
        }

        androidTextInputTarget = target;
        suppressAndroidTextInputCallbacks = true;
        androidTextInput.setText(text == null ? "" : text);
        androidTextInput.setSelection(androidTextInput.length());
        suppressAndroidTextInputCallbacks = false;

        if (target == ANDROID_TEXT_INPUT_TARGET_NONE) {
            InputMethodManager manager =
                    (InputMethodManager) getSystemService(INPUT_METHOD_SERVICE);
            if (manager != null) {
                manager.hideSoftInputFromWindow(androidTextInput.getWindowToken(), 0);
            }
            androidTextInput.clearFocus();
            return;
        }

        androidTextInput.requestFocus();
        showAndroidTextInputKeyboard();
    }

    private void showAndroidTextInputKeyboard() {
        if (androidTextInput == null
                || androidTextInputTarget == ANDROID_TEXT_INPUT_TARGET_NONE) {
            return;
        }

        androidTextInput.post(() -> {
            if (androidTextInput == null
                    || androidTextInputTarget == ANDROID_TEXT_INPUT_TARGET_NONE) {
                return;
            }

            androidTextInput.requestFocus();

            WindowInsetsControllerCompat insetsController =
                    WindowCompat.getInsetsController(getWindow(), androidTextInput);
            if (insetsController != null) {
                insetsController.show(WindowInsetsCompat.Type.ime());
            }

            InputMethodManager manager =
                    (InputMethodManager) getSystemService(INPUT_METHOD_SERVICE);
            if (manager != null) {
                manager.restartInput(androidTextInput);
                manager.showSoftInput(
                        androidTextInput,
                        InputMethodManager.SHOW_IMPLICIT);
            }
        });
    }

    private boolean dismissAndroidTextInput() {
        if (androidTextInput == null
                || androidTextInputTarget == ANDROID_TEXT_INPUT_TARGET_NONE) {
            return false;
        }

        int target = androidTextInputTarget;
        androidTextInputTarget = ANDROID_TEXT_INPUT_TARGET_NONE;
        suppressAndroidTextInputCallbacks = true;
        androidTextInput.setText("");
        suppressAndroidTextInputCallbacks = false;

        InputMethodManager manager =
                (InputMethodManager) getSystemService(INPUT_METHOD_SERVICE);
        if (manager != null) {
            manager.hideSoftInputFromWindow(androidTextInput.getWindowToken(), 0);
        }
        androidTextInput.clearFocus();
        nativeAndroidTextInputDismissed(target);
        return true;
    }

    public void requestGameExit() {
        if (!isFinishing()) {
            finishAndRemoveTask();
        }
    }

    @Override
    protected void onDestroy() {
        boolean finishing = isFinishing();
        super.onDestroy();

        if (finishing) {
            // Defer process termination until the GameActivity destroy callback
            // and the current Android lifecycle callback have fully unwound.
            new android.os.Handler(android.os.Looper.getMainLooper())
                    .post(() -> android.os.Process.killProcess(android.os.Process.myPid()));
        }
    }

    @Override
    public void onWindowFocusChanged(boolean hasFocus) {
        super.onWindowFocusChanged(hasFocus);

        if (hasFocus) {
            hideSystemUi();
            showAndroidTextInputKeyboard();
        }
    }

    private File resolveGameRoot() {
        File externalFiles = getExternalFilesDir(Environment.DIRECTORY_DOCUMENTS);
        if (externalFiles != null) {
            return new File(externalFiles, GAME_DIRECTORY_NAME);
        }

        return new File(getFilesDir(), GAME_DIRECTORY_NAME);
    }

    public String getNativeGameRootPath() {
        return resolveGameRoot().getAbsolutePath();
    }

    private void prepareGameRoot() {
        restorePrivateConfig();
        File root = resolveGameRoot();
        File data = new File(root, "data");
        File mods = new File(data, "mods");
        File assets = new File(data, "assets");
        File saves = new File(root, "saves");

        if (!data.mkdirs() && !data.isDirectory()) {
            return;
        }
        if (!mods.mkdirs() && !mods.isDirectory()) {
            return;
        }
        if (!saves.mkdirs() && !saves.isDirectory()) {
            return;
        }

        try {
            copyAssetDirectory("mods", mods);
            copyAssetDirectory("assets", assets);
            copyAssetFile("base_content.json", new File(data, "base_content.json"));
            File baseContent = new File(data, "base_content.json");
            if (baseContent.isFile()) {
                baseContent.setWritable(false, false);
                baseContent.setReadOnly();
            }
            syncSharedStorageToLocal();
        } catch (IOException exception) {
            Log.w(TAG, "Could not prepare the Android game root", exception);
        }
    }

    private void maybePromptForSharedStorage() {
        if (getSharedStorageTree() != null) {
            return;
        }

        new AlertDialog.Builder(this)
                .setTitle("Choose game folder")
                .setMessage("Choose or create a folder for The Ashen Chronicle. The game will use it for saves, mods, and editable assets so they remain accessible outside the app. You can continue with private app storage for now.")
                .setPositiveButton("Choose folder", (dialog, which) -> requestStorageTree())
                .setNegativeButton("Continue", null)
                .show();
    }

    private void requestStorageTree() {
        Intent intent = new Intent(Intent.ACTION_OPEN_DOCUMENT_TREE);
        intent.addFlags(Intent.FLAG_GRANT_READ_URI_PERMISSION
                | Intent.FLAG_GRANT_WRITE_URI_PERMISSION
                | Intent.FLAG_GRANT_PERSISTABLE_URI_PERMISSION
                | Intent.FLAG_GRANT_PREFIX_URI_PERMISSION);
        startActivityForResult(intent, REQUEST_CODE_OPEN_TREE);
    }

    public void requestStorageTreeFromOptions() {
        requestStorageTree();
    }

    public String getSharedStorageDisplayName() {
        DocumentFile tree = getSharedStorageTree();
        if (tree == null) {
            return null;
        }
        String name = tree.getName();
        return name == null ? null : name;
    }

    private DocumentFile getSharedStorageTree() {
        String uriString = getPreferences(MODE_PRIVATE)
                .getString(SHARED_STORAGE_URI_PREFERENCE, null);
        if (uriString == null) {
            return null;
        }

        Uri treeUri = Uri.parse(uriString);
        if (!hasPersistedTreePermission(treeUri)) {
            Log.w(TAG, "Stored shared storage URI no longer has a persisted read/write grant");
            clearSharedStorageSelection();
            return null;
        }

        try {
            DocumentFile tree = DocumentFile.fromTreeUri(this, treeUri);
            if (tree == null) {
                clearSharedStorageSelection();
                return null;
            }
            if (tree.canRead()) {
                return tree;
            }

            // Keep a valid grant when a provider has a transient read failure.
            Log.w(TAG, "Stored shared storage folder is currently unreadable");
            return null;
        } catch (IllegalArgumentException exception) {
            Log.w(TAG, "Stored shared storage URI is invalid", exception);
            clearSharedStorageSelection();
            return null;
        } catch (SecurityException exception) {
            Log.w(TAG, "Could not access the stored shared storage folder", exception);
            return null;
        }
    }

    private boolean hasPersistedTreePermission(Uri treeUri) {
        for (android.content.UriPermission permission : getContentResolver().getPersistedUriPermissions()) {
            if (treeUri.equals(permission.getUri())
                    && permission.isReadPermission()
                    && permission.isWritePermission()) {
                return true;
            }
        }
        return false;
    }

    private void clearSharedStorageSelection() {
        getPreferences(MODE_PRIVATE)
                .edit()
                .remove(SHARED_STORAGE_URI_PREFERENCE)
                .apply();

        File privateConfig = new File(getFilesDir(), GAME_CONFIG_FILE_NAME);
        if (privateConfig.isFile() && !privateConfig.delete()) {
            Log.w(TAG, "Could not remove stale private game configuration");
        }
    }

    private boolean hasSharedStorageContent(Uri treeUri) {
        try {
            DocumentFile tree = DocumentFile.fromTreeUri(this, treeUri);
            if (tree == null || !tree.canRead()) {
                return false;
            }

            DocumentFile data = findDirectory(tree, "data");
            DocumentFile saves = findDirectory(tree, "saves");
            return hasFiles(findDirectory(data, "mods"))
                    || hasFiles(findDirectory(data, "assets"))
                    || hasFiles(saves);
        } catch (IllegalArgumentException | SecurityException exception) {
            Log.w(TAG, "Could not inspect selected shared storage contents", exception);
            return false;
        }
    }

    private boolean hasFiles(DocumentFile directory) {
        if (directory == null || !directory.isDirectory()) {
            return false;
        }

        for (DocumentFile child : directory.listFiles()) {
            if (child.isFile()) {
                return true;
            }
            if (child.isDirectory() && hasFiles(child)) {
                return true;
            }
        }
        return false;
    }

    private void syncSharedStorageToLocal() {
        DocumentFile tree = getSharedStorageTree();
        if (tree == null) {
            return;
        }

        File root = resolveGameRoot();
        File data = new File(root, "data");
        File mods = new File(data, "mods");
        File saves = new File(root, "saves");

        try {
            if (!tree.canRead()) {
                Log.w(TAG, "Selected shared storage folder is not readable");
                return;
            }

            if (!importSharedConfig(tree)) {
                writePrivateConfig(tree.getUri());
                copyPrivateConfigToShared(tree);
            }

            DocumentFile sharedData = findDirectory(tree, "data");
            if (sharedData != null) {
                DocumentFile sharedMods = findDirectory(sharedData, "mods");
                if (sharedMods != null) {
                    copyDocumentDirectoryToLocal(sharedMods, mods);
                }
                DocumentFile sharedAssets = findDirectory(sharedData, "assets");
                if (sharedAssets != null) {
                    copyDocumentDirectoryToLocal(
                            sharedAssets,
                            new File(data, "assets"));
                }
            }

            DocumentFile sharedSaves = findDirectory(tree, "saves");
            if (sharedSaves != null) {
                copyDocumentDirectoryToLocal(sharedSaves, saves);
            }
        } catch (IOException | SecurityException exception) {
            Log.w(TAG, "Could not import shared game storage", exception);
        }
    }

    private void syncLocalStorageToShared() {
        DocumentFile tree = getSharedStorageTree();
        if (tree == null) {
            return;
        }

        File root = resolveGameRoot();
        File data = new File(root, "data");
        File mods = new File(data, "mods");
        File assets = new File(data, "assets");
        File saves = new File(root, "saves");

        try {
            if (!tree.canWrite()) {
                Log.w(TAG, "Selected shared storage folder is not writable");
                return;
            }

            writePrivateConfig(tree.getUri());
            copyPrivateConfigToShared(tree);

            DocumentFile sharedData = findOrCreateDirectory(tree, "data");
            DocumentFile sharedMods = findOrCreateDirectory(sharedData, "mods");
            DocumentFile sharedAssets = findOrCreateDirectory(sharedData, "assets");
            DocumentFile sharedSaves = findOrCreateDirectory(tree, "saves");
            copyLocalDirectoryToDocument(mods, sharedMods);
            copyLocalDirectoryToDocument(assets, sharedAssets);
            copyLocalDirectoryToDocument(saves, sharedSaves);
        } catch (IOException | SecurityException exception) {
            Log.w(TAG, "Could not export shared game storage", exception);
        }
    }

    private void restorePrivateConfig() {
        String storedUri = getPreferences(MODE_PRIVATE)
                .getString(SHARED_STORAGE_URI_PREFERENCE, null);
        if (storedUri != null && hasPersistedTreePermission(Uri.parse(storedUri))) {
            return;
        }

        // Preferences are authoritative when valid. The config mirror is only
        // a recovery path when its URI still has a persisted Android grant.
        Uri configuredUri = readPrivateConfigUri();
        if (configuredUri != null
                && hasPersistedTreePermission(configuredUri)
                && !configuredUri.toString().equals(storedUri)) {
            getPreferences(MODE_PRIVATE)
                    .edit()
                    .putString(SHARED_STORAGE_URI_PREFERENCE, configuredUri.toString())
                    .apply();
            return;
        }

        if (storedUri != null) {
            clearSharedStorageSelection();
        }
    }

    private boolean importSharedConfig(DocumentFile tree) throws IOException {
        DocumentFile config = findDocumentFile(tree, GAME_CONFIG_FILE_NAME);
        if (config == null || !config.isFile()) {
            return false;
        }

        Uri configuredUri;
        try (InputStream input = getContentResolver().openInputStream(config.getUri())) {
            if (input == null) {
                throw new IOException("Could not open shared game configuration");
            }
            configuredUri = readConfigUri(input);
        }

        if (configuredUri == null
                || !configuredUri.toString().equals(tree.getUri().toString())) {
            Log.w(TAG, "Ignoring shared game configuration for a different or invalid storage tree");
            return false;
        }

        getPreferences(MODE_PRIVATE)
                .edit()
                .putString(SHARED_STORAGE_URI_PREFERENCE, configuredUri.toString())
                .apply();
        writePrivateConfig(configuredUri);
        return true;
    }

    private void writePrivateConfig(Uri treeUri) throws IOException {
        File config = new File(getFilesDir(), GAME_CONFIG_FILE_NAME);
        try (OutputStream output = new java.io.FileOutputStream(config);
             OutputStreamWriter writer = new OutputStreamWriter(output, StandardCharsets.UTF_8)) {
            writeConfig(writer, treeUri);
        }
    }

    private void copyPrivateConfigToShared(DocumentFile tree) throws IOException {
        File config = new File(getFilesDir(), GAME_CONFIG_FILE_NAME);
        if (!config.isFile()) {
            return;
        }
        copyLocalFileToDocument(config, tree);
    }

    private Uri readPrivateConfigUri() {
        File config = new File(getFilesDir(), GAME_CONFIG_FILE_NAME);
        if (!config.isFile()) {
            return null;
        }

        try (InputStream input = new java.io.FileInputStream(config)) {
            return readConfigUri(input);
        } catch (IOException exception) {
            Log.w(TAG, "Could not read private game configuration", exception);
            return null;
        }
    }

    private Uri readConfigUri(InputStream input) throws IOException {
        StringBuilder content = new StringBuilder();
        try (InputStreamReader reader = new InputStreamReader(input, StandardCharsets.UTF_8)) {
            char[] buffer = new char[4096];
            int length;
            while ((length = reader.read(buffer)) != -1) {
                content.append(buffer, 0, length);
            }
        }

        try {
            JSONObject config = new JSONObject(content.toString());
            if (config.optInt("version", -1) != GAME_CONFIG_VERSION) {
                return null;
            }

            String uriString = config.optString(GAME_CONFIG_STORAGE_URI_KEY, null);
            if (uriString == null || uriString.isEmpty()) {
                return null;
            }
            return Uri.parse(uriString);
        } catch (JSONException | IllegalArgumentException exception) {
            Log.w(TAG, "Ignoring malformed game configuration", exception);
            return null;
        }
    }

    private void writeConfig(OutputStreamWriter writer, Uri treeUri) throws IOException {
        JSONObject config = new JSONObject();
        try {
            config.put("version", GAME_CONFIG_VERSION);
            config.put(GAME_CONFIG_STORAGE_URI_KEY, treeUri.toString());
            writer.write(config.toString(2));
            writer.write('\n');
        } catch (JSONException exception) {
            throw new IOException("Could not build game configuration", exception);
        }
    }

    private DocumentFile findDirectory(DocumentFile parent, String name) {
        if (parent == null || !parent.isDirectory()) {
            return null;
        }

        for (DocumentFile child : parent.listFiles()) {
            if (child.isDirectory() && name.equals(child.getName())) {
                return child;
            }
        }
        return null;
    }

    private DocumentFile findOrCreateDirectory(DocumentFile parent, String name) throws IOException {
        DocumentFile existing = findDirectory(parent, name);
        if (existing != null) {
            return existing;
        }

        DocumentFile conflictingFile = findDocumentFile(parent, name);
        if (conflictingFile != null && !conflictingFile.isDirectory() && !conflictingFile.delete()) {
            throw new IOException("Could not replace shared file " + name);
        }

        DocumentFile created = parent.createDirectory(name);
        if (created == null) {
            throw new IOException("Could not create shared directory " + name);
        }
        return created;
    }

    private DocumentFile findDocumentFile(DocumentFile parent, String name) {
        if (parent == null || !parent.isDirectory()) {
            return null;
        }

        for (DocumentFile child : parent.listFiles()) {
            if (name.equals(child.getName())) {
                return child;
            }
        }
        return null;
    }

    private void copyDocumentDirectoryToLocal(DocumentFile source, File destination) throws IOException {
        if (destination.exists() && !destination.isDirectory()) {
            throw new IOException("Shared directory conflicts with local file " + destination);
        }
        if (!destination.isDirectory() && !destination.mkdirs()) {
            throw new IOException("Could not create " + destination);
        }

        for (DocumentFile child : source.listFiles()) {
            String name = child.getName();
            if (!isSafeDocumentName(name)) {
                Log.w(TAG, "Skipping unsafe shared-storage name: " + name);
                continue;
            }

            File target = new File(destination, name);
            try {
                if (child.isDirectory()) {
                    copyDocumentDirectoryToLocal(child, target);
                } else if (child.isFile()) {
                    copyDocumentFileToLocal(child, target);
                }
            } catch (IOException | SecurityException exception) {
                Log.w(TAG, "Could not import shared entry " + child.getName(), exception);
            }
        }
    }

    private void copyDocumentFileToLocal(DocumentFile source, File destination) throws IOException {
        if (destination.isDirectory()) {
            throw new IOException("Shared file conflicts with local directory " + destination);
        }

        File parent = destination.getParentFile();
        if (parent == null) {
            throw new IOException("Local destination has no parent: " + destination);
        }
        if (!parent.isDirectory() && !parent.mkdirs()) {
            throw new IOException("Could not create " + parent);
        }

        File temporary = File.createTempFile("ac_", ".tmp", parent);
        try {
            try (InputStream sourceStream = getContentResolver().openInputStream(source.getUri())) {
                if (sourceStream == null) {
                    throw new IOException("Could not open shared file " + source.getName());
                }

                try (FileOutputStream output = new FileOutputStream(temporary)) {
                    byte[] buffer = new byte[8192];
                    int length;
                    while ((length = sourceStream.read(buffer)) != -1) {
                        output.write(buffer, 0, length);
                    }
                    output.getFD().sync();
                }
            }

            try {
                Files.move(
                        temporary.toPath(),
                        destination.toPath(),
                        StandardCopyOption.ATOMIC_MOVE,
                        StandardCopyOption.REPLACE_EXISTING);
            } catch (AtomicMoveNotSupportedException exception) {
                Files.move(
                        temporary.toPath(),
                        destination.toPath(),
                        StandardCopyOption.REPLACE_EXISTING);
            }
        } finally {
            if (temporary.exists() && !temporary.delete()) {
                Log.w(TAG, "Could not remove temporary local-storage file " + temporary);
            }
        }
    }

    private void copyLocalDirectoryToDocument(File source, DocumentFile destination) throws IOException {
        if (!source.isDirectory()) {
            return;
        }

        File[] children = source.listFiles();
        if (children == null) {
            throw new IOException("Could not list local directory " + source);
        }

        for (File child : children) {
            if (!isSafeDocumentName(child.getName())) {
                Log.w(TAG, "Skipping unsafe local-storage name: " + child.getName());
                continue;
            }

            try {
                if (child.isDirectory()) {
                    DocumentFile target = findOrCreateDirectory(destination, child.getName());
                    copyLocalDirectoryToDocument(child, target);
                } else if (child.isFile()) {
                    copyLocalFileToDocument(child, destination);
                }
            } catch (IOException | SecurityException exception) {
                Log.w(TAG, "Could not sync local file " + child, exception);
            }
        }
    }

    private void copyLocalFileToDocument(File source, DocumentFile destinationDirectory) throws IOException {
        DocumentFile target = findDocumentFile(destinationDirectory, source.getName());
        if (target != null && target.isDirectory()) {
            if (!target.delete()) {
                throw new IOException("Could not replace shared directory " + target.getName());
            }
            target = null;
        }

        if (target == null) {
            target = destinationDirectory.createFile("application/octet-stream", source.getName());
            if (target == null) {
                throw new IOException("Could not create shared file " + source.getName());
            }
        }

        try (InputStream input = new java.io.FileInputStream(source)) {
            OutputStream output = getContentResolver().openOutputStream(target.getUri(), "wt");
            if (output == null) {
                throw new IOException("Could not open shared file " + source.getName() + " for writing");
            }

            try (OutputStream destination = output) {
                byte[] buffer = new byte[8192];
                int length;
                while ((length = input.read(buffer)) != -1) {
                    destination.write(buffer, 0, length);
                }
            }
        }
    }

    private boolean isSafeDocumentName(String name) {
        return name != null
                && !name.isEmpty()
                && !name.equals(".")
                && !name.equals("..")
                && name.indexOf('/') < 0
                && name.indexOf('\\') < 0;
    }

    private void showStorageError(String message) {
        if (!isFinishing()) {
            new AlertDialog.Builder(this)
                    .setTitle("Storage unavailable")
                    .setMessage(message)
                    .setPositiveButton("Continue", null)
                    .show();
        }
    }

    private void copyAssetDirectory(String assetPath, File destination) throws IOException {
        String[] children = getAssets().list(assetPath);
        if (children == null) {
            return;
        }

        if (children.length == 0) {
            copyAssetFile(assetPath, destination);
            return;
        }

        if (!destination.exists() && !destination.mkdirs()) {
            throw new IOException("Could not create " + destination);
        }

        for (String child : children) {
            File target = new File(destination, child);
            String childAssetPath = assetPath + "/" + child;
            String[] nested = getAssets().list(childAssetPath);
            if (nested != null && nested.length > 0) {
                copyAssetDirectory(childAssetPath, target);
            } else if (!target.exists()) {
                copyAssetFile(childAssetPath, target);
            }
        }
    }

    private void copyAssetFile(String assetPath, File destination) throws IOException {
        File parent = destination.getParentFile();
        if (parent != null && !parent.isDirectory() && !parent.mkdirs()) {
            throw new IOException("Could not create " + parent);
        }

        try (InputStream input = getAssets().open(assetPath);
             FileOutputStream output = new FileOutputStream(destination)) {
            byte[] buffer = new byte[8192];
            int length;
            while ((length = input.read(buffer)) != -1) {
                output.write(buffer, 0, length);
            }
        }
    }

    private void hideSystemUi() {
        View decorView = getWindow().getDecorView();
        decorView.setSystemUiVisibility(
                View.SYSTEM_UI_FLAG_IMMERSIVE_STICKY
                        | View.SYSTEM_UI_FLAG_LAYOUT_STABLE
                        | View.SYSTEM_UI_FLAG_LAYOUT_HIDE_NAVIGATION
                        | View.SYSTEM_UI_FLAG_LAYOUT_FULLSCREEN
                        | View.SYSTEM_UI_FLAG_HIDE_NAVIGATION
                        | View.SYSTEM_UI_FLAG_FULLSCREEN
        );
    }
}
