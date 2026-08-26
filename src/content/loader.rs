use super::definitions::*;
use std::collections::HashSet;
use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

const CONTENT_FILE_NAME: &str = "base_content.json";
const MODS_DIR_NAME: &str = "mods";

#[derive(Debug, Clone)]
pub struct DataRootCandidate {
    pub root: PathBuf,
    pub has_base_content: bool,
    pub has_mods_directory: bool,
}

#[derive(Debug, Clone)]
struct DiscoveredMod {
    manifest: ModManifest,
    manifest_path: PathBuf,
}

impl ContentLoadReport {
    fn with_content(content: CampaignContent) -> Self {
        Self {
            content,
            loaded_mods: Vec::new(),
            warnings: Vec::new(),
        }
    }
}

pub fn load_campaign_content() -> CampaignContent {
    load_campaign_content_report().content
}

pub fn load_campaign_content_report() -> ContentLoadReport {
    let mut report = ContentLoadReport::with_content(default_campaign_content());
    let candidates = data_root_candidates();
    let Some(root) = select_data_root(&candidates) else {
        report.warnings.push(
            "No external data root with base_content.json was found; using embedded base content."
                .into(),
        );
        return report;
    };

    let base_path = root.join(CONTENT_FILE_NAME);
    match load_content_file(&base_path) {
        Ok(content) => report.content = content,
        Err(err) => {
            report.warnings.push(format!(
                "Could not load base content from {}: {err}; using embedded base content.",
                base_path.display()
            ));
            return report;
        }
    }

    let base_location_names: HashSet<&str> = report
        .content
        .world
        .locations
        .iter()
        .map(|entry| entry.name.as_str())
        .collect();
    let base_faction_names: HashSet<&str> = report
        .content
        .factions
        .iter()
        .map(|entry| entry.name.as_str())
        .collect();
    let base_events = std::mem::take(&mut report.content.events);
    let base_event_ids: HashSet<String> =
        base_events.iter().map(|event| event.id.clone()).collect();
    report.content.events = filter_valid_events(
        base_events,
        &base_location_names,
        &base_faction_names,
        &base_event_ids,
        &HashSet::new(),
        "base content",
        &mut report.warnings,
    );

    let mods_root = root.join(MODS_DIR_NAME);
    if !mods_root.is_dir() {
        report
            .warnings
            .push(format!("Mods directory not found: {}", mods_root.display()));
        report.warnings.extend(report.content.validate());
        return report;
    }

    let mut discovered_mods = discover_mods(&mods_root, &mut report.warnings);
    discovered_mods.sort_by(|left, right| {
        left.manifest
            .priority
            .cmp(&right.manifest.priority)
            .then_with(|| left.manifest.id.cmp(&right.manifest.id))
    });

    let mut seen_mod_ids = HashSet::new();
    for discovered in discovered_mods {
        if !discovered.manifest.enabled || !seen_mod_ids.insert(discovered.manifest.id.clone()) {
            if discovered.manifest.enabled {
                report.warnings.push(format!(
                    "skipping duplicate mod id {}",
                    discovered.manifest.id
                ));
            }
            continue;
        }
        let manifest = discovered.manifest.clone();
        match load_mod_content(&discovered.manifest_path, &manifest) {
            Ok(mod_content) => {
                merge_campaign_content(&mut report.content, mod_content, &mut report.warnings);
                report.loaded_mods.push(manifest);
            }
            Err(err) => report.warnings.push(format!(
                "could not load mod {} ({}) from {}: {}",
                manifest.id,
                manifest.name,
                discovered.manifest_path.display(),
                err
            )),
        }
    }

    report.warnings.extend(report.content.validate());
    report
}

pub fn data_root_candidates() -> Vec<DataRootCandidate> {
    let mut roots = Vec::new();
    let mut push_root = |root: PathBuf| {
        let normalized = fs::canonicalize(&root).unwrap_or(root);
        if roots
            .iter()
            .all(|candidate: &DataRootCandidate| candidate.root != normalized)
        {
            roots.push(DataRootCandidate {
                has_base_content: normalized.join(CONTENT_FILE_NAME).is_file(),
                has_mods_directory: normalized.join(MODS_DIR_NAME).is_dir(),
                root: normalized,
            });
        }
    };

    if let Ok(current_dir) = env::current_dir() {
        push_root(current_dir.join("data"));
    }
    if let Ok(exe) = env::current_exe() {
        if let Some(dir) = exe.parent() {
            push_root(dir.join("data"));
            if let Some(parent) = dir.parent() {
                push_root(parent.join("data"));
            }
        }
    }
    roots
}

fn select_data_root(candidates: &[DataRootCandidate]) -> Option<PathBuf> {
    candidates
        .iter()
        .find(|candidate| candidate.has_base_content && candidate.has_mods_directory)
        .or_else(|| {
            candidates
                .iter()
                .find(|candidate| candidate.has_base_content)
        })
        .map(|candidate| candidate.root.clone())
}

fn load_content_file(path: &Path) -> io::Result<CampaignContent> {
    let data = fs::read_to_string(path)?;
    serde_json::from_str(&data)
        .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err.to_string()))
}

fn load_mod_content(manifest_path: &Path, manifest: &ModManifest) -> io::Result<CampaignContent> {
    let content_path = manifest_path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(&manifest.content_file);
    load_content_file(&content_path)
}

fn discover_mods(mods_root: &Path, warnings: &mut Vec<String>) -> Vec<DiscoveredMod> {
    let Ok(entries) = fs::read_dir(mods_root) else {
        warnings.push(format!(
            "could not read mods directory {}",
            mods_root.display()
        ));
        return Vec::new();
    };
    let mut found = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let manifest_path = path.join("manifest.json");
        if !manifest_path.is_file() {
            warnings.push(format!(
                "skipping mod directory without manifest: {}",
                path.display()
            ));
            continue;
        }
        match fs::read_to_string(&manifest_path)
            .ok()
            .and_then(|data| serde_json::from_str::<ModManifest>(&data).ok())
        {
            Some(manifest) => found.push(DiscoveredMod {
                manifest,
                manifest_path,
            }),
            None => warnings.push(format!(
                "could not parse mod manifest {}",
                manifest_path.display()
            )),
        }
    }
    found
}

fn merge_campaign_content(
    base: &mut CampaignContent,
    incoming: CampaignContent,
    warnings: &mut Vec<String>,
) {
    base.world.region = incoming.world.region;
    merge_vec_by_key(
        &mut base.world.locations,
        incoming.world.locations,
        |entry| entry.id.clone(),
    );
    merge_vec_by_key(&mut base.factions, incoming.factions, |entry| {
        entry.id.clone()
    });
    merge_vec_by_key(&mut base.npcs, incoming.npcs, |entry| entry.id.clone());
    merge_vec_by_key(&mut base.quests, incoming.quests, |entry| entry.id.clone());
    merge_vec_by_key(&mut base.encounters, incoming.encounters, |entry| {
        entry.location_name.clone()
    });
    merge_vec_by_key(&mut base.atmospheres, incoming.atmospheres, |entry| {
        entry.location_name.clone()
    });
    merge_vec_by_key(&mut base.item_visuals, incoming.item_visuals, |entry| {
        entry.item_name.clone()
    });

    let locations: HashSet<&str> = base
        .world
        .locations
        .iter()
        .map(|entry| entry.name.as_str())
        .collect();
    let factions: HashSet<&str> = base
        .factions
        .iter()
        .map(|entry| entry.name.as_str())
        .collect();
    let existing: HashSet<String> = base.events.iter().map(|event| event.id.clone()).collect();
    let known: HashSet<String> = existing
        .iter()
        .cloned()
        .chain(incoming.events.iter().map(|event| event.id.clone()))
        .collect();
    let accepted = filter_valid_events(
        incoming.events,
        &locations,
        &factions,
        &known,
        &existing,
        "mod content",
        warnings,
    );
    base.events.extend(accepted);
}

fn filter_valid_events(
    events: Vec<EventContent>,
    locations: &HashSet<&str>,
    factions: &HashSet<&str>,
    known_ids: &HashSet<String>,
    existing_ids: &HashSet<String>,
    source: &str,
    warnings: &mut Vec<String>,
) -> Vec<EventContent> {
    let mut accepted = Vec::new();
    let mut seen_ids = existing_ids.clone();
    let mut pending = Vec::new();
    for event in events {
        let mut issues = validate_event_content(&event, locations, factions, known_ids);
        if !seen_ids.insert(event.id.clone()) {
            issues.push(format!("duplicate event id {}", event.id));
        }
        if issues.is_empty() {
            pending.push(event);
        } else {
            warnings.push(format!(
                "rejecting event '{}' from {}: {}",
                event.id,
                source,
                issues.join("; ")
            ));
        }
    }
    loop {
        let available: HashSet<String> = existing_ids
            .iter()
            .cloned()
            .chain(accepted.iter().map(|event: &EventContent| event.id.clone()))
            .chain(pending.iter().map(|event| event.id.clone()))
            .collect();
        let mut removed = false;
        let mut next = Vec::new();
        for event in pending {
            let prior = event
                .conditions
                .as_ref()
                .and_then(|conditions| conditions.prior_event_id.as_deref());
            if let Some(prior_id) = prior.filter(|id| !available.contains(*id)) {
                warnings.push(format!(
                    "rejecting event '{}' from {}: prior event '{}' was rejected or unavailable",
                    event.id, source, prior_id
                ));
                removed = true;
            } else {
                next.push(event);
            }
        }
        pending = next;
        if !removed {
            break;
        }
    }
    accepted.extend(pending);
    accepted
}

fn validate_event_content(
    event: &EventContent,
    locations: &HashSet<&str>,
    factions: &HashSet<&str>,
    known_ids: &HashSet<String>,
) -> Vec<String> {
    let mut issues = Vec::new();
    if event.id.trim().is_empty() {
        issues.push("empty id".into());
    }
    if event.trigger.trim().is_empty() {
        issues.push("empty trigger".into());
    }
    if event.weight == 0 {
        issues.push("zero weight".into());
    }
    if let Some(chance) = event.chance_percent {
        if chance > 100 {
            issues.push(format!("invalid chance {}", chance));
        }
    }
    if event.effects.is_empty() {
        issues.push("no effects".into());
    }
    if let Some(conditions) = &event.conditions {
        for location in &conditions.locations {
            if !locations.contains(location.as_str()) {
                issues.push(format!("unknown location {}", location));
            }
        }
        if let (Some(min_day), Some(max_day)) = (conditions.min_day, conditions.max_day) {
            if min_day > max_day {
                issues.push("min_day greater than max_day".into());
            }
        }
        if let Some(prior) = conditions.prior_event_id.as_deref() {
            if !known_ids.contains(prior) {
                issues.push(format!("unknown prior event id {}", prior));
            }
        }
        if let Some(faction) = conditions.faction_name.as_deref() {
            if !factions.contains(faction) {
                issues.push(format!("unknown faction {}", faction));
            }
        }
        if (conditions.min_reputation.is_some() || conditions.max_reputation.is_some())
            && conditions.faction_name.is_none()
        {
            issues.push("reputation condition requires faction_name".into());
        }
        if let (Some(min), Some(max)) = (conditions.min_reputation, conditions.max_reputation) {
            if min > max {
                issues.push("min_reputation greater than max_reputation".into());
            }
        }
        if conditions
            .required_item_name
            .as_deref()
            .map(|name| name.trim().is_empty())
            .unwrap_or(false)
        {
            issues.push("required_item_name cannot be empty".into());
        }
        if conditions
            .required_condition_name
            .as_deref()
            .map(|name| name.trim().is_empty())
            .unwrap_or(false)
        {
            issues.push("required_condition_name cannot be empty".into());
        }
    }
    issues
}

fn merge_vec_by_key<T, F>(base: &mut Vec<T>, incoming: Vec<T>, key_fn: F)
where
    T: Clone,
    F: Fn(&T) -> String,
{
    for item in incoming {
        let key = key_fn(&item);
        if let Some(position) = base.iter().position(|existing| key_fn(existing) == key) {
            base[position] = item;
        } else {
            base.push(item);
        }
    }
}

fn default_campaign_content() -> CampaignContent {
    serde_json::from_str(include_str!("../../data/base_content.json"))
        .expect("embedded base content JSON must remain valid")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_event(id: &str) -> EventContent {
        EventContent {
            id: id.into(),
            trigger: "travel_arrival".into(),
            weight: 1,
            chance_percent: Some(100),
            cooldown_turns: None,
            conditions: None,
            effects: vec![EventEffectContent::Pause],
        }
    }

    #[test]
    fn data_root_candidates_do_not_duplicate_canonical_paths() {
        let candidates = data_root_candidates();
        let mut roots = HashSet::new();
        for candidate in candidates {
            assert!(roots.insert(candidate.root));
        }
    }

    #[test]
    fn invalid_events_are_rejected_and_reported() {
        let mut warnings = Vec::new();
        let locations = HashSet::from(["Ashen Gate"]);
        let factions = HashSet::from(["Cinder Wardens"]);
        let known = HashSet::from(["good.event".into(), "bad.event".into()]);
        let mut invalid = valid_event("bad.event");
        invalid.weight = 0;
        invalid.conditions = Some(EventConditionContent {
            locations: vec!["Unknown Place".into()],
            ..Default::default()
        });
        let accepted = filter_valid_events(
            vec![valid_event("good.event"), invalid],
            &locations,
            &factions,
            &known,
            &HashSet::new(),
            "test content",
            &mut warnings,
        );
        assert_eq!(accepted.len(), 1);
        assert_eq!(accepted[0].id, "good.event");
        assert!(warnings[0].contains("zero weight"));
        assert!(warnings[0].contains("unknown location"));
    }

    #[test]
    fn rejected_prior_event_rejects_dependents() {
        let mut warnings = Vec::new();
        let locations = HashSet::from(["Ashen Gate"]);
        let factions = HashSet::from(["Cinder Wardens"]);
        let known = HashSet::from(["bad.event".into(), "followup.event".into()]);
        let mut bad = valid_event("bad.event");
        bad.weight = 0;
        let mut followup = valid_event("followup.event");
        followup.conditions = Some(EventConditionContent {
            prior_event_id: Some("bad.event".into()),
            ..Default::default()
        });
        let accepted = filter_valid_events(
            vec![bad, followup],
            &locations,
            &factions,
            &known,
            &HashSet::new(),
            "test content",
            &mut warnings,
        );
        assert!(accepted.is_empty());
        assert!(warnings
            .iter()
            .any(|warning| warning.contains("followup.event") && warning.contains("unavailable")));
    }
}
