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
import android.view.Gravity;
import android.view.KeyEvent;
import android.view.View;
import android.view.inputmethod.EditorInfo;
import android.view.inputmethod.InputMethodManager;
import android.widget.EditText;
import android.widget.TextView;

import androidx.activity.OnBackPressedCallback;

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
        try {
            getContentResolver().takePersistableUriPermission(treeUri, takeFlags);
            getPreferences(MODE_PRIVATE)
                    .edit()
                    .putString(SHARED_STORAGE_URI_PREFERENCE, treeUri.toString())
                    .apply();

            if (hasSharedStorageContent(treeUri)) {
                syncSharedStorageToLocal();
            } else {
                syncLocalStorageToShared();
            }
        } catch (SecurityException exception) {
            Log.w(TAG, "Could not persist shared storage permission", exception);
            getPreferences(MODE_PRIVATE).edit().remove(SHARED_STORAGE_URI_PREFERENCE).apply();
            showStorageError("The selected folder could not be granted persistent access.");
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
        input.setImeOptions(EditorInfo.IME_ACTION_DONE);
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

    private void prepareGameRoot() {
        restorePrivateConfig();
        File root = resolveGameRoot();
        File data = new File(root, "data");
        File mods = new File(data, "mods");
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
                .setMessage("Choose or create a folder for The Ashen Chronicle. The game will use it for saves and editable mods so they remain accessible outside the app. You can continue with private app storage for now.")
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

        try {
            DocumentFile tree = DocumentFile.fromTreeUri(this, Uri.parse(uriString));
            if (tree != null && tree.canRead() && tree.canWrite()) {
                return tree;
            }
        } catch (IllegalArgumentException exception) {
            Log.w(TAG, "Stored shared storage URI is invalid", exception);
        }

        getPreferences(MODE_PRIVATE).edit().remove(SHARED_STORAGE_URI_PREFERENCE).apply();
        return null;
    }

    private boolean hasSharedStorageContent(Uri treeUri) {
        DocumentFile tree;
        try {
            tree = DocumentFile.fromTreeUri(this, treeUri);
        } catch (IllegalArgumentException exception) {
            return false;
        }
        if (tree == null || !tree.canRead()) {
            return false;
        }

        DocumentFile config = findDocumentFile(tree, GAME_CONFIG_FILE_NAME);
        DocumentFile data = findDirectory(tree, "data");
        DocumentFile saves = findDirectory(tree, "saves");
        return (config != null && config.isFile()) || hasChildren(data) || hasChildren(saves);
    }

    private void syncSharedStorageToLocal() {
        DocumentFile tree = getSharedStorageTree();
        if (tree == null || !tree.canRead()) {
            return;
        }

        File root = resolveGameRoot();
        File data = new File(root, "data");
        File mods = new File(data, "mods");
        File saves = new File(root, "saves");

        try {
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
            }

            DocumentFile sharedSaves = findDirectory(tree, "saves");
            if (sharedSaves != null) {
                copyDocumentDirectoryToLocal(sharedSaves, saves);
            }
        } catch (IOException exception) {
            Log.w(TAG, "Could not import shared game storage", exception);
        }
    }

    private void syncLocalStorageToShared() {
        DocumentFile tree = getSharedStorageTree();
        if (tree == null || !tree.canWrite()) {
            return;
        }

        File root = resolveGameRoot();
        File data = new File(root, "data");
        File saves = new File(root, "saves");

        try {
            writePrivateConfig(tree.getUri());
            copyPrivateConfigToShared(tree);

            DocumentFile sharedData = findOrCreateDirectory(tree, "data");
            DocumentFile sharedSaves = findOrCreateDirectory(tree, "saves");
            copyLocalDirectoryToDocument(data, sharedData);
            copyLocalDirectoryToDocument(saves, sharedSaves);
        } catch (IOException exception) {
            Log.w(TAG, "Could not export shared game storage", exception);
        }
    }

    private void restorePrivateConfig() {
        Uri configuredUri = readPrivateConfigUri();
        if (configuredUri == null) {
            return;
        }

        getPreferences(MODE_PRIVATE)
                .edit()
                .putString(SHARED_STORAGE_URI_PREFERENCE, configuredUri.toString())
                .apply();
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

    private boolean hasChildren(DocumentFile directory) {
        return directory != null && directory.isDirectory() && directory.listFiles().length > 0;
    }

    private void copyDocumentDirectoryToLocal(DocumentFile source, File destination) throws IOException {
        if (!destination.exists() && !destination.mkdirs()) {
            throw new IOException("Could not create " + destination);
        }

        for (DocumentFile child : source.listFiles()) {
            String name = child.getName();
            if (!isSafeDocumentName(name)) {
                Log.w(TAG, "Skipping unsafe shared-storage name: " + name);
                continue;
            }

            File target = new File(destination, name);
            if (child.isDirectory()) {
                copyDocumentDirectoryToLocal(child, target);
            } else if (child.isFile()) {
                copyDocumentFileToLocal(child, target);
            }
        }
    }

    private void copyDocumentFileToLocal(DocumentFile source, File destination) throws IOException {
        File parent = destination.getParentFile();
        if (parent != null && !parent.isDirectory() && !parent.mkdirs()) {
            throw new IOException("Could not create " + parent);
        }

        try (InputStream input = getContentResolver().openInputStream(source.getUri());
             FileOutputStream output = new FileOutputStream(destination)) {
            if (input == null) {
                throw new IOException("Could not open shared file " + source.getName());
            }

            byte[] buffer = new byte[8192];
            int length;
            while ((length = input.read(buffer)) != -1) {
                output.write(buffer, 0, length);
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
            } catch (IOException exception) {
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

        try (InputStream input = new java.io.FileInputStream(source);
             OutputStream output = getContentResolver().openOutputStream(target.getUri(), "wt")) {
            if (output == null) {
                throw new IOException("Could not open shared file " + source.getName() + " for writing");
            }

            byte[] buffer = new byte[8192];
            int length;
            while ((length = input.read(buffer)) != -1) {
                output.write(buffer, 0, length);
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
