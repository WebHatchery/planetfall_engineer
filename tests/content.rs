//! Embedded content registry validation coverage.

use planetfall_engineer::content::ContentRegistry;

#[test]
fn embedded_registry_validates_all_slice_references() {
    let registry = ContentRegistry::load().unwrap();
    registry.validate().unwrap();
    assert_eq!(registry.maps.len(), 15);
}

#[test]
fn validator_reports_missing_showcase_and_duplicate_ids() {
    let mut registry = ContentRegistry::load().unwrap();
    registry.devices[0].showcase_map_id = "missing_map".into();
    registry.devices.push(registry.devices[0].clone());
    let errors = registry.validate().unwrap_err().join(" | ");
    assert!(errors.contains("missing showcase map"));
    assert!(errors.contains("duplicate id"));
}

#[test]
fn campaign_records_use_campaign_maps_and_in_bounds_authored_points() {
    let registry = ContentRegistry::load().unwrap();
    for mission in &registry.missions {
        let map = registry.map(&mission.map_id).unwrap();
        assert_eq!(map.kind, "campaign");
        assert!(mission
            .deposits
            .iter()
            .all(|deposit| deposit.position[0] < map.width && deposit.position[1] < map.height));
        assert!(mission
            .power_sources
            .iter()
            .all(|source| source.position[0] < map.width && source.position[1] < map.height));
    }
}

#[test]
fn every_tutorial_step_has_player_facing_prompt_data() {
    let registry = ContentRegistry::load().unwrap();
    for tutorial in &registry.tutorials {
        assert_eq!(tutorial.steps.len(), tutorial.prompts.len());
        for step in &tutorial.steps {
            let prompt = registry.tutorial_prompt(step).unwrap();
            assert!(!prompt.title.is_empty());
            assert!(!prompt.instruction.is_empty());
            assert!(!prompt.action.is_empty());
        }
    }
}

#[test]
fn validator_rejects_out_of_bounds_points_and_bad_schedule_balance() {
    let mut registry = ContentRegistry::load().unwrap();
    registry.missions[0].deposits[0].position = [32, 20];
    registry.missions[1].surge_end_tick = Some(800);
    let errors = registry.validate().unwrap_err().join(" | ");
    assert!(errors.contains("outside"));
    assert!(errors.contains("surge schedule"));
}
