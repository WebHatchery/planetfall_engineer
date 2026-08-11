use super::*;

#[test]
fn embedded_registry_validates_all_slice_references() {
    let registry = ContentRegistry::load().unwrap();
    registry.validate().unwrap();
    assert_eq!(registry.maps.len(), 14);
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
