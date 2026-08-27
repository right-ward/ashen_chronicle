use super::loader::{data_root_candidates, load_campaign_content_report};
use std::fmt::Write;

pub fn campaign_content_load_diagnostics() -> String {
    let candidates = data_root_candidates();
    let selected_root = candidates
        .iter()
        .find(|candidate| candidate.has_base_content && candidate.has_mods_directory)
        .or_else(|| {
            candidates
                .iter()
                .find(|candidate| candidate.has_base_content)
        })
        .map(|candidate| candidate.root.clone());
    let report = load_campaign_content_report();
    let base_source = if report
        .warnings
        .iter()
        .any(|warning| warning.contains("using embedded base content"))
        || selected_root.is_none()
    {
        "embedded"
    } else {
        "external"
    };

    let mut output = String::new();
    let _ = writeln!(
        output,
        "selected data root: {}",
        selected_root
            .as_ref()
            .map(|root| root.display().to_string())
            .unwrap_or_else(|| "<none>".into())
    );
    let _ = writeln!(output, "base source: {base_source}");

    writeln!(output, "data root candidates:").unwrap();
    if candidates.is_empty() {
        output.push_str("  <none>\n");
    } else {
        for candidate in &candidates {
            let marker = selected_root
                .as_ref()
                .map(|root| *root == candidate.root)
                .unwrap_or(false);
            let _ = writeln!(
                output,
                "  {}{} base={} mods={}",
                if marker { "* " } else { "  " },
                candidate.root.display(),
                candidate.has_base_content,
                candidate.has_mods_directory
            );
        }
    }

    let _ = writeln!(output, "loaded mods: {}", report.loaded_mods.len());
    if report.loaded_mods.is_empty() {
        output.push_str("  <none>\n");
    } else {
        for manifest in &report.loaded_mods {
            let _ = writeln!(
                output,
                "  {} | {} | version={} priority={} enabled={}",
                manifest.id, manifest.name, manifest.version, manifest.priority, manifest.enabled
            );
        }
    }

    let _ = writeln!(output, "warnings: {}", report.warnings.len());
    if report.warnings.is_empty() {
        output.push_str("  <none>\n");
    } else {
        for warning in &report.warnings {
            let _ = writeln!(output, "  {warning}");
        }
    }

    let _ = writeln!(
        output,
        "loader content: regions=1 locations={} encounters={} events={} quests={} npcs={} factions={} items={}",
        report.content.world.locations.len(),
        report.content.encounters.len(),
        report.content.events.len(),
        report.content.quests.len(),
        report.content.npcs.len(),
        report.content.factions.len(),
        report.content.item_visuals.len(),
    );

    output.trim_end().to_string()
}
