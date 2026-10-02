- Kept the explicit Android task/process termination required to avoid reusing Bevy's process-global Android activity handle after Quit.
- Deferred the process kill until after the Java Activity's destroy callback has fully unwound, avoiding an immediate process death inside `GameActivity.onDestroy()`.
- Hardened the Android Quit path to target the post-Quit relaunch splash regression by allowing Activity teardown to unwind before process termination.

### v0.51.15: Atmospheric quit-confirmation variants
- Tightened the semantic relationship between each quit question, its leave/stay responses, and its ASCII art motif.
- Preserved whole-bundle random selection so each atmospheric presentation remains internally matched.
- Expanded the quit-confirmation set from 9 to 16 variants, including substantially darker themes of fading names, unseen voices, doubled heartbeats, mirrors that refuse to reflect departures, and an unseen presence.
- Bumped the project and Android fallback version metadata to 0.51.15.