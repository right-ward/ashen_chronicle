use ashen_chronicle::content::CampaignContent;

#[test]
fn bundled_expansion_content_deserializes() {
    let expansion = serde_json::from_str::<CampaignContent>(include_str!("../data/mods/ashen_expansion/content.json"))
        .expect("ashen expansion content must deserialize");
    assert_eq!(expansion.version, 1);
    assert!(expansion
        .world
        .locations
        .iter()
        .any(|location| location.id == "location.resonant_forge"));
}

#[test]
fn bundled_depth_expansion_content_deserializes() {
    let expansion = serde_json::from_str::<CampaignContent>(include_str!("../data/mods/echoes_depth/content.json"))
        .expect("echoes depth content must deserialize");
    assert_eq!(expansion.version, 1);
    assert!(expansion
        .world
        .locations
        .iter()
        .any(|location| location.id == "location.broken_stage"));
}
