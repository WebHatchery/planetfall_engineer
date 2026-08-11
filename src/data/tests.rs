use super::*;

#[test]
fn embedded_foundation_data_loads() {
    let data = GameData::load().unwrap();
    assert_eq!(data.config.world_width, 32);
    assert_eq!(data.config.world_height, 20);
    assert_eq!(data.content.devices.len(), 10);
}
