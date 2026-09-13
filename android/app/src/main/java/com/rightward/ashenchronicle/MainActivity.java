package com.rightward.ashenchronicle;

import android.app.AlertDialog;
import android.content.Intent;
import android.net.Uri;
import android.os.Bundle;
import android.os.Environment;
import android.provider.Settings;
import android.view.View;

import com.google.androidgamesdk.GameActivity;

import java.io.File;
import java.io.FileOutputStream;
import java.io.IOException;

public class MainActivity extends GameActivity {
    private static final String GAME_DIRECTORY_NAME = "The Ashen Chronicle";
    private static final String PROMPT_PREFERENCE = "storage_prompt_shown";

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
    public void onWindowFocusChanged(boolean hasFocus) {
        super.onWindowFocusChanged(hasFocus);

        if (hasFocus) {
            hideSystemUi();
        }
    }

    private File resolveGameRoot() {
        if (Environment.isExternalStorageManager()) {
            File documents = Environment.getExternalStoragePublicDirectory(Environment.DIRECTORY_DOCUMENTS);
            return new File(documents, GAME_DIRECTORY_NAME);
        }

        File fallback = getExternalFilesDir(Environment.DIRECTORY_DOCUMENTS);
        if (fallback != null) {
            return new File(fallback, GAME_DIRECTORY_NAME);
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
            copyAssetDirectory("data/mods", mods);
            copyAssetFile("data/base_content.json", new File(data, "base_content.json"));
            File baseContent = new File(data, "base_content.json");
            if (baseContent.isFile()) {
                baseContent.setWritable(false, false);
                baseContent.setReadOnly();
            }
        } catch (IOException ignored) {
            // The Rust startup path will report a failure if the root cannot be initialized.
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

        try (java.io.InputStream input = getAssets().open(assetPath);
             FileOutputStream output = new FileOutputStream(destination)) {
            byte[] buffer = new byte[8192];
            int length;
            while ((length = input.read(buffer)) != -1) {
                output.write(buffer, 0, length);
            }
        }
    }

    private void maybePromptForSharedStorage() {
        if (Environment.isExternalStorageManager()) {
            return;
        }
        if (getPreferences(MODE_PRIVATE).getBoolean(PROMPT_PREFERENCE, false)) {
            return;
        }
        getPreferences(MODE_PRIVATE).edit().putBoolean(PROMPT_PREFERENCE, true).apply();

        new AlertDialog.Builder(this)
                .setTitle("Shared storage access")
                .setMessage("The game can use Documents/The Ashen Chronicle for saves and editable mods. Without access, the game will continue using an app-owned fallback location; installing, changing, or removing mods there will not be available.")
                .setPositiveButton("Open Settings", (dialog, which) -> openStorageSettings())
                .setNegativeButton("Continue", null)
                .show();
    }

    private void openStorageSettings() {
        try {
            Intent intent = new Intent(
                    Settings.ACTION_MANAGE_APP_ALL_FILES_ACCESS_PERMISSION,
                    Uri.parse("package:" + getPackageName())
            );
            startActivity(intent);
        } catch (Exception ignored) {
            Intent intent = new Intent(Settings.ACTION_MANAGE_ALL_FILES_ACCESS_PERMISSION);
            startActivity(intent);
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
