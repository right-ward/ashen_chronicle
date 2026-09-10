use crate::content::EventContent;
use crate::model::GameState;
use flate2::{read::GzDecoder, write::GzEncoder, Compression};
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

const SAVE_FILE_VERSION: u32 = 2;
pub const SAVE_BASE_NAME: &str = "ashen_chronicle_save";
const SAVE_EXTENSION: &str = "json.gz";
const LEGACY_SAVE_FILE_NAME: &str = "ashen_chronicle_save.json";
const GENERATED_EVENT_PREFIXES: &[&str] = &["generated.event.", "generated.evolution.event."];

#[derive(Debug, Serialize)]
struct SaveFile<'a> {
    save_file_version: u32,
    game: &'a GameState,
    #[serde(default)]
    runtime_generated_events: Vec<EventContent>,
}

#[derive(Debug, Deserialize)]
struct LoadedSaveFile {
    save_file_version: u32,
    game: GameState,
    #[serde(default)]
    runtime_generated_events: Vec<EventContent>,
}

const MAX_SAVE_BYTES: u64 = 8 * 1024 * 1024;

pub fn save_game(path: &Path, state: &GameState) -> io::Result<()> {
    let payload = SaveFile {
        save_file_version: SAVE_FILE_VERSION,
        game: state,
        runtime_generated_events: state
            .campaign_content
            .as_ref()
            .map(|content| {
                content
                    .events
                    .iter()
                    .filter(|event| is_runtime_generated_event(&event.id))
                    .cloned()
                    .collect()
            })
            .unwrap_or_default(),
    };
    let json =
        serde_json::to_vec_pretty(&payload).map_err(|err| io::Error::other(err.to_string()))?;

    let file = File::create(path)?;
    let mut encoder = GzEncoder::new(file, Compression::default());
    encoder.write_all(&json)?;
    encoder.finish()?;
    Ok(())
}

pub fn load_game(path: &Path) -> io::Result<GameState> {
    let file = File::open(path)?;
    if file.metadata()?.len() > MAX_SAVE_BYTES {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "save file exceeds the maximum supported size",
        ));
    }
    let mut data = Vec::new();
    file.take(MAX_SAVE_BYTES + 1).read_to_end(&mut data)?;
    if data.len() as u64 > MAX_SAVE_BYTES {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "save file exceeds the maximum supported size",
        ));
    }
    let json = if is_gzip(&data) {
        let decoder = GzDecoder::new(data.as_slice());
        let mut decoded = Vec::new();
        decoder
            .take(MAX_SAVE_BYTES + 1)
            .read_to_end(&mut decoded)
            .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err.to_string()))?;
        if decoded.len() as u64 > MAX_SAVE_BYTES {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "decompressed save exceeds the maximum supported size",
            ));
        }
        decoded
    } else {
        // Backward compatibility for the pre-compression JSON save format.
        data
    };

    let parsed: LoadedSaveFile = serde_json::from_slice(&json)
        .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err.to_string()))?;
    if parsed.save_file_version > SAVE_FILE_VERSION || parsed.save_file_version == 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "Save file version {} is not supported by this build",
                parsed.save_file_version
            ),
        ));
    }
    let mut game = parsed.game;
    if parsed.save_file_version < 2 {
        game.world.time_points = 3;
        game.world.day = 1;
        if game.character.level == 0 {
            game.character.level = 1;
        }
        if game.character.attributes.might == 0
            && game.character.attributes.insight == 0
            && game.character.attributes.endurance == 0
        {
            game.character.attributes.might = 1;
            game.character.attributes.insight = 1;
            game.character.attributes.endurance = 1;
        }
    }
    let mut content = crate::content::load_campaign_content();
    for event in parsed.runtime_generated_events {
        if content
            .events
            .iter()
            .all(|candidate| candidate.id != event.id)
        {
            content.events.push(event);
        }
    }
    game.campaign_content = Some(content);
    Ok(game)
}

fn is_runtime_generated_event(event_id: &str) -> bool {
    GENERATED_EVENT_PREFIXES
        .iter()
        .any(|prefix| event_id.starts_with(prefix))
}

pub fn character_save_path(directory: &Path, character_name: &str) -> PathBuf {
    directory.join(format!(
        "{}_{}.{}",
        SAVE_BASE_NAME,
        sanitize_filename_component(character_name),
        SAVE_EXTENSION
    ))
}

pub fn find_save_files(directory: &Path) -> io::Result<Vec<PathBuf>> {
    let mut saves = Vec::new();
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        if file_name.starts_with(&format!("{}_", SAVE_BASE_NAME))
            && file_name.ends_with(&format!(".{}", SAVE_EXTENSION))
        {
            saves.push(path);
        }
    }
    saves.sort();
    Ok(saves)
}

pub fn legacy_save_path(directory: &Path) -> PathBuf {
    directory.join(LEGACY_SAVE_FILE_NAME)
}

pub fn sanitize_filename_component(name: &str) -> String {
    let mut sanitized = String::new();
    for ch in name.trim().chars() {
        if ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | ' ') {
            sanitized.push(ch);
        } else {
            sanitized.push('_');
        }
    }
    let sanitized = sanitized.trim().trim_matches('.').to_string();
    let has_safe_name_char = sanitized.chars().any(|ch| ch.is_ascii_alphanumeric());
    if sanitized.is_empty() || !has_safe_name_char {
        "unnamed".to_string()
    } else {
        sanitized
    }
}

fn is_gzip(data: &[u8]) -> bool {
    data.len() >= 2 && data[0] == 0x1f && data[1] == 0x8b
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content::{EventConditionContent, EventEffectContent};
    use crate::model::{create_new_state, EventCooldown, WorldMode};
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_dir() -> PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should be valid")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("ashen_chronicle_save_tests_{}", stamp));
        fs::create_dir_all(&path).expect("temporary directory should be created");
        path
    }

    #[test]
    fn compressed_save_round_trips() {
        let dir = temp_dir();
        let path = character_save_path(&dir, "Ash Walker");
        let mut state = create_new_state(
            "Test World",
            WorldMode::New,
            "Tester".to_string(),
            "Ash Walker".to_string(),
        );
        state.character.turn = 7;
        state.world.event_cooldowns.push(EventCooldown {
            event_id: "travel.ruined-road".to_string(),
            ready_at_turn: 11,
        });

        let generation = state.world.generation;
        let original_description = state.world.locations[0].description.clone();
        let original_exits = state.world.locations[0].exits.clone();
        state.world.locations[0].description = "A scar left by the first warden.".to_string();
        state.world.locations[0].dangerous = !state.world.locations[0].dangerous;
        state
            .world
            .record_history(7, "The world has changed.".to_string());
        let mutated_description = state.world.locations[0].description.clone();
        let history_len = state.world.history.len();

        save_game(&path, &state).expect("save should succeed");
        let bytes = fs::read(&path).expect("save should exist");
        assert!(is_gzip(&bytes));

        let loaded = load_game(&path).expect("load should succeed");
        assert_eq!(loaded.character.turn, 7);
        assert_eq!(loaded.character.name, "Tester");
        assert_eq!(loaded.world.event_cooldowns, state.world.event_cooldowns);
        assert_eq!(loaded.rng_state, state.rng_state);
        assert_eq!(loaded.world.generation, generation);
        assert_eq!(loaded.world.locations[0].description, mutated_description);
        assert_ne!(loaded.world.locations[0].description, original_description);
        assert_eq!(loaded.world.locations[0].exits, original_exits);
        assert_eq!(loaded.world.history.len(), history_len);
        fs::remove_dir_all(&dir).expect("temporary directory should be removed");
    }

    #[test]
    fn runtime_generated_events_round_trip_separately_from_authored_content() {
        let dir = temp_dir();
        let path = character_save_path(&dir, "Generated Events");
        let mut state = create_new_state(
            "Test World",
            WorldMode::New,
            "Tester".to_string(),
            "Ash Walker".to_string(),
        );
        let generated = EventContent {
            id: "generated.evolution.event.test".to_string(),
            trigger: "travel_arrival".to_string(),
            weight: 1,
            chance_percent: Some(100),
            cooldown_turns: Some(12),
            conditions: Some(EventConditionContent {
                dangerous: Some(true),
                ..Default::default()
            }),
            effects: vec![EventEffectContent::History {
                text: "A generated sign remains after the save.".to_string(),
            }],
        };
        let authored_id = state
            .campaign_content
            .as_ref()
            .unwrap()
            .events
            .first()
            .map(|event| event.id.clone())
            .expect("authored event should exist");
        state
            .campaign_content
            .as_mut()
            .unwrap()
            .events
            .push(generated.clone());

        save_game(&path, &state).expect("save should succeed");
        let loaded = load_game(&path).expect("load should succeed");
        let events = &loaded.campaign_content.as_ref().unwrap().events;
        assert!(events.iter().any(|event| event.id == authored_id));
        assert!(events.iter().any(|event| event.id == generated.id));
        assert_eq!(
            events
                .iter()
                .filter(|event| event.id == generated.id)
                .count(),
            1
        );
        fs::remove_dir_all(&dir).expect("temporary directory should be removed");
    }

    #[test]
    fn legacy_json_save_still_loads() {
        let dir = temp_dir();
        let path = legacy_save_path(&dir);
        let state = create_new_state(
            "Test World",
            WorldMode::New,
            "Tester".to_string(),
            "Ash Walker".to_string(),
        );
        let payload = SaveFile {
            save_file_version: SAVE_FILE_VERSION,
            game: &state,
            runtime_generated_events: Vec::new(),
        };
        fs::write(
            &path,
            serde_json::to_vec_pretty(&payload).expect("json should serialize"),
        )
        .expect("legacy save should be written");

        let loaded = load_game(&path).expect("legacy save should load");
        assert_eq!(loaded.character.name, state.character.name);
        fs::remove_dir_all(&dir).expect("temporary directory should be removed");
    }

    #[test]
    fn sanitize_filename_component_is_filesystem_safe() {
        assert_eq!(
            sanitize_filename_component("Ash/Walker:Night"),
            "Ash_Walker_Night"
        );
        assert_eq!(sanitize_filename_component("..."), "unnamed");
        assert_eq!(sanitize_filename_component("  Geralt  "), "Geralt");
    }

    #[test]
    fn character_save_path_appends_character_name() {
        let path = character_save_path(Path::new("."), "Ash Walker");
        assert_eq!(
            path,
            PathBuf::from("./ashen_chronicle_save_Ash Walker.json.gz")
        );
    }

    #[test]
    fn oversized_gzip_is_rejected_after_decompression() {
        let dir = temp_dir();
        let path = dir.join("oversized.json.gz");
        let file = File::create(&path).expect("save should be created");
        let mut encoder = GzEncoder::new(file, Compression::default());
        let chunk = vec![b'x'; 1024 * 1024];
        for _ in 0..9 {
            encoder
                .write_all(&chunk)
                .expect("test payload should write");
        }
        encoder.finish().expect("gzip should finish");

        let err = load_game(&path).expect_err("oversized gzip should fail");
        assert_eq!(err.kind(), io::ErrorKind::InvalidData);
        assert!(err.to_string().contains("decompressed"));
        fs::remove_dir_all(&dir).expect("temporary directory should be removed");
    }

    #[test]
    fn invalid_gzip_is_reported_as_invalid_data() {
        let dir = temp_dir();
        let path = dir.join("broken.json.gz");
        fs::write(&path, [0x1f, 0x8b, 0x00, 0x01]).expect("broken gzip should be written");

        let err = load_game(&path).expect_err("invalid gzip should fail");
        assert_eq!(err.kind(), io::ErrorKind::InvalidData);
        fs::remove_dir_all(&dir).expect("temporary directory should be removed");
    }
}
