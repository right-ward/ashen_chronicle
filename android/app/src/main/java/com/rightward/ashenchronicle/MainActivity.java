package com.rightward.ashenchronicle;

import android.app.AlertDialog;
import android.content.Intent;
import android.net.Uri;
import android.os.Bundle;
import android.os.Environment;
import android.util.Log;
import android.view.View;

import androidx.documentfile.provider.DocumentFile;

import com.google.androidgamesdk.GameActivity;

import java.io.File;
import java.io.FileOutputStream;
import java.io.IOException;
import java.io.InputStream;
import java.io.OutputStream;

public class MainActivity extends GameActivity {
    private static final String TAG = "AshenChronicle";
    private static final String GAME_DIRECTORY_NAME = "The Ashen Chronicle";
    private static final String SHARED_STORAGE_URI_PREFERENCE = "shared_storage_tree_uri";
    private static final int REQUEST_CODE_OPEN_TREE = 1001;

    static {
        System.loadLibrary("ashen_chronicle");
    }

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        prepareGameRoot();
        super.onCreate(savedInstanceState);
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

    @Override
    public void onWindowFocusChanged(boolean hasFocus) {
        super.onWindowFocusChanged(hasFocus);

        if (hasFocus) {
            hideSystemUi();
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

        DocumentFile data = findDirectory(tree, "data");
        DocumentFile saves = findDirectory(tree, "saves");
        return hasChildren(data) || hasChildren(saves);
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
            DocumentFile sharedData = findOrCreateDirectory(tree, "data");
            DocumentFile sharedSaves = findOrCreateDirectory(tree, "saves");
            copyLocalDirectoryToDocument(data, sharedData);
            copyLocalDirectoryToDocument(saves, sharedSaves);
        } catch (IOException exception) {
            Log.w(TAG, "Could not export shared game storage", exception);
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
