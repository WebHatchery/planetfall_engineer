//! Authoritative integer world state, fixed-tick shell, and versioned save data.

use crate::data::GameConfig;
use crate::mission::{CampaignProgress, MissionId, MissionState};
use crate::simulation::SimulationWorld;
use macroquad_toolkit::persistence::{load_from_slot_with_migration, save_to_slot_with_version};
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const SIM_TICKS_PER_SECOND: u64 = 10;
pub const MAX_TICKS_PER_FRAME: u32 = 8;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CellPos { pub x: u16, pub y: u16 }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CellState { pub height_hu: i16, pub sealed: bool }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldState {
    pub width: u16,
    pub height: u16,
    pub cells: Vec<CellState>,
}

impl WorldState {
    pub fn new(width: usize, height: usize) -> Self {
        let cells = (0..height).flat_map(|y| (0..width).map(move |x| {
            let ridge = ((x as i16 - width as i16 / 2).abs() + (y as i16 - height as i16 / 2).abs()) * 125;
            CellState { height_hu: 1000 + ridge, sealed: x == 0 || y == 0 || x + 1 == width || y + 1 == height }
        })).collect();
        Self { width: width as u16, height: height as u16, cells }
    }

    pub fn index(&self, pos: CellPos) -> Option<usize> {
        (pos.x < self.width && pos.y < self.height)
            .then_some(pos.y as usize * self.width as usize + pos.x as usize)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TimeControl { Paused, OneX, TwoX, FourX }

impl TimeControl { pub fn multiplier(self) -> u64 { match self { Self::Paused => 0, Self::OneX => 1, Self::TwoX => 2, Self::FourX => 4 } } }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveData { pub schema_version: u32, pub version: String, pub tick: u64, pub world: WorldState, pub simulation: SimulationWorld, pub selected: CellPos, pub mission: MissionState, pub campaign: CampaignProgress }

#[derive(Debug, Clone)]
pub struct GameSession {
    pub world: WorldState,
    pub simulation: SimulationWorld,
    pub mission: MissionState,
    pub campaign: CampaignProgress,
    pub tick: u64,
    pub selected: CellPos,
    pub time_control: TimeControl,
    accumulator: f64,
}

impl GameSession {
    pub fn new(config: &GameConfig) -> Self {
        let mut mission = MissionState::new(MissionId::L01FirstFlow);
        mission.start();
        Self { world: WorldState::new(config.world_width, config.world_height), simulation: SimulationWorld::new(config.world_width as u16, config.world_height as u16), mission, campaign: CampaignProgress::default(), tick: 0,
            selected: CellPos { x: (config.world_width / 2) as u16, y: (config.world_height / 2) as u16 },
            time_control: TimeControl::Paused, accumulator: 0.0 }
    }

    pub fn update(&mut self, frame_seconds: f32) -> u32 {
        let scaled = frame_seconds.max(0.0) as f64 * self.time_control.multiplier() as f64;
        self.accumulator += scaled;
        let mut ticks = 0;
        while self.accumulator >= 1.0 / SIM_TICKS_PER_SECOND as f64 && ticks < MAX_TICKS_PER_FRAME {
            self.accumulator -= 1.0 / SIM_TICKS_PER_SECOND as f64;
            self.tick(); ticks += 1;
        }
        ticks
    }

    pub fn tick(&mut self) { self.tick = self.tick.saturating_add(1); self.simulation.tick(); for (cell, sim_cell) in self.world.cells.iter_mut().zip(&self.simulation.cells) { cell.height_hu = sim_cell.height_hu; cell.sealed = sim_cell.sealed; } }

    pub fn move_selected(&mut self, dx: i16, dy: i16) {
        let x = (self.selected.x as i16 + dx).clamp(0, self.world.width as i16 - 1) as u16;
        let y = (self.selected.y as i16 + dy).clamp(0, self.world.height as i16 - 1) as u16;
        self.selected = CellPos { x, y };
    }

    pub fn state_hash(&self) -> u64 {
        let mut hash = 1469598103934665603u64;
        for byte in serde_json::to_vec(&(self.tick, &self.world, &self.simulation, &self.mission, &self.campaign, self.selected)).unwrap() { hash ^= byte as u64; hash = hash.wrapping_mul(1099511628211); }
        hash
    }

    pub fn to_save(&self, version: &str) -> SaveData { SaveData { schema_version: 3, version: version.into(), tick: self.tick, world: self.world.clone(), simulation: self.simulation.clone(), selected: self.selected, mission: self.mission.clone(), campaign: self.campaign.clone() } }
    pub fn from_save(save: SaveData) -> Self { Self { world: save.world, simulation: save.simulation, mission: save.mission, campaign: save.campaign, tick: save.tick, selected: save.selected, time_control: TimeControl::Paused, accumulator: 0.0 } }
}

pub fn save_session(session: &GameSession, config: &GameConfig) -> Result<(), String> { save_to_slot_with_version(&config.game_name, &config.save_slot, &session.to_save(&config.version), &config.version) }

pub fn load_session(config: &GameConfig) -> Result<GameSession, String> {
    let save: SaveData = load_from_slot_with_migration(&config.game_name, &config.save_slot, &config.version, |_, value| migrate_save_value(value, config))?;
    Ok(GameSession::from_save(save))
}

fn migrate_save_value(value: Value, config: &GameConfig) -> Result<SaveData, String> {
    serde_json::from_value(value.get("data").cloned().unwrap_or(value)).map_err(|e| format!("Save is not a Phase 1 foundation save: {e}")).and_then(|save: SaveData| {
        if save.world.width as usize != config.world_width || save.world.height as usize != config.world_height { Err("Save world dimensions do not match content".into()) } else { Ok(save) }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    fn config() -> GameConfig { GameConfig { game_name: "test".into(), display_name: "Test".into(), save_slot: "test".into(), version: "1".into(), world_width: 8, world_height: 8 } }
    #[test] fn row_major_coordinates_are_stable() { let w = WorldState::new(8, 8); assert_eq!(w.index(CellPos { x: 2, y: 3 }), Some(26)); assert!(w.index(CellPos { x: 8, y: 0 }).is_none()); }
    #[test] fn partitioning_does_not_change_tick_hash() { let c = config(); let mut a = GameSession::new(&c); let mut b = GameSession::new(&c); a.time_control = TimeControl::OneX; b.time_control = TimeControl::OneX; for _ in 0..10 { a.update(0.1); } b.update(0.25); b.update(0.25); b.update(0.5); assert_eq!(a.state_hash(), b.state_hash()); }
    #[test] fn paused_session_does_not_tick() { let c = config(); let mut s = GameSession::new(&c); s.update(10.0); assert_eq!(s.tick, 0); }
    #[test] fn save_round_trip_is_exact() { let c = config(); let mut s = GameSession::new(&c); s.tick(); s.mission.on_tick(&s.simulation); s.mission.checkpoint(); s.campaign.record_success(MissionId::L01FirstFlow, 700); let save = s.to_save("1"); let restored = GameSession::from_save(save); assert_eq!(restored.state_hash(), s.state_hash()); assert_eq!(restored.mission.checkpoint_tick, 1); assert_eq!(restored.campaign.best_ticks[0], Some(700)); }
}
