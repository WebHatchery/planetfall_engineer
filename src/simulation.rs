//! Deterministic terrain, surface-fluid, steam, and reaction simulation.
//!
//! The renderer consumes this state but never participates in its decisions.

use crate::{devices::DeviceSystem, state::CellPos};
use serde::{Deserialize, Serialize};

pub const CELL_CAPACITY_VU: u32 = 8_000;
pub const WATER_BOIL_DK: i32 = 3_730;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum FluidId {
    Lava,
    Steam,
    ToxicSlurry,
    Water,
}

impl FluidId {
    pub const ALL: [Self; 4] = [Self::Lava, Self::Steam, Self::ToxicSlurry, Self::Water];

    pub const fn max_transfer(self) -> u32 {
        match self {
            Self::Water => 400,
            Self::Lava => 120,
            Self::ToxicSlurry => 180,
            Self::Steam => 300,
        }
    }

    pub const fn default_temperature(self) -> i32 {
        match self {
            Self::Water => 2_930,
            Self::Lava => 12_730,
            Self::ToxicSlurry => 3_000,
            Self::Steam => 3_930,
        }
    }

    pub const fn is_airborne(self) -> bool { matches!(self, Self::Steam) }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FluidEntry {
    pub fluid: FluidId,
    pub volume_vu: u32,
    pub temperature_dk: i32,
    pub contamination_bp: u16,
}

impl FluidEntry {
    pub fn new(fluid: FluidId, volume_vu: u32) -> Self {
        Self { fluid, volume_vu, temperature_dk: fluid.default_temperature(), contamination_bp: if fluid == FluidId::ToxicSlurry { 10_000 } else { 0 } }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CellDefinition {
    pub base_height_hu: i16,
    pub minimum_height_hu: i16,
    pub maximum_height_hu: i16,
    pub protected: bool,
    pub gas_blocked: bool,
    pub ambient_temperature_dk: i32,
}

impl Default for CellDefinition {
    fn default() -> Self { Self { base_height_hu: 1_000, minimum_height_hu: 0, maximum_height_hu: 8_000, protected: false, gas_blocked: false, ambient_temperature_dk: 2_930 } }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimCell {
    pub height_hu: i16,
    pub sealed: bool,
    pub ground_contamination_bp: u16,
    pub surface: Vec<FluidEntry>,
    pub airborne: Vec<FluidEntry>,
    pub pending_rock_vu: u32,
    pub pending_vitrified_vu: u32,
}

impl SimCell {
    pub fn empty(height_hu: i16, sealed: bool) -> Self { Self { height_hu, sealed, ground_contamination_bp: 0, surface: Vec::new(), airborne: Vec::new(), pending_rock_vu: 0, pending_vitrified_vu: 0 } }
    pub fn surface_volume(&self) -> u32 { self.surface.iter().map(|m| m.volume_vu).sum() }
    pub fn airborne_volume(&self) -> u32 { self.airborne.iter().map(|m| m.volume_vu).sum() }
    pub fn surface_head_hu(&self) -> i32 { self.height_hu as i32 + self.surface_volume() as i32 }
    pub fn add_surface(&mut self, mut entry: FluidEntry) -> u32 {
        let accepted = entry.volume_vu.min(CELL_CAPACITY_VU.saturating_sub(self.surface_volume()));
        entry.volume_vu = accepted;
        if accepted == 0 { return 0; }
        merge_entry(&mut self.surface, entry);
        accepted
    }
    pub fn add_airborne(&mut self, mut entry: FluidEntry) -> u32 {
        let accepted = entry.volume_vu.min(CELL_CAPACITY_VU.saturating_sub(self.airborne_volume()));
        entry.volume_vu = accepted;
        if accepted == 0 { return 0; }
        merge_entry(&mut self.airborne, entry);
        accepted
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MassLedger {
    pub injected: u64,
    pub drained: u64,
    pub reacted: u64,
    pub products: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SimEvent {
    SourceBackpressure { cell: CellPos, requested_vu: u32, accepted_vu: u32 },
    MaterialReacted { reaction: String, cell: CellPos, volume_vu: u32 },
    HighPressureSteam { cell: CellPos },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationWorld {
    pub width: u16,
    pub height: u16,
    pub definitions: Vec<CellDefinition>,
    pub cells: Vec<SimCell>,
    pub tick: u64,
    pub ledger: MassLedger,
    pub events: Vec<SimEvent>,
    pub devices: DeviceSystem,
}

impl SimulationWorld {
    pub fn new(width: u16, height: u16) -> Self {
        let definitions = vec![CellDefinition::default(); width as usize * height as usize];
        let cells = definitions.iter().map(|d| SimCell::empty(d.base_height_hu, false)).collect();
        Self { width, height, definitions, cells, tick: 0, ledger: MassLedger::default(), events: Vec::new(), devices: DeviceSystem::default() }
    }

    pub fn index(&self, pos: CellPos) -> Option<usize> { (pos.x < self.width && pos.y < self.height).then_some(pos.y as usize * self.width as usize + pos.x as usize) }
    fn neighbors(&self, pos: CellPos) -> Vec<(CellPos, u8)> {
        [(0, -1), (1, 0), (0, 1), (-1, 0)].into_iter().enumerate().filter_map(|(direction, (dx, dy))| {
            let x = pos.x as i16 + dx; let y = pos.y as i16 + dy;
            (x >= 0 && y >= 0 && x < self.width as i16 && y < self.height as i16).then_some((CellPos { x: x as u16, y: y as u16 }, direction as u8))
        }).collect()
    }

    pub fn inject(&mut self, pos: CellPos, fluid: FluidId, volume_vu: u32) {
        let Some(index) = self.index(pos) else { return; };
        let accepted = if fluid.is_airborne() { self.cells[index].add_airborne(FluidEntry::new(fluid, volume_vu)) } else { self.cells[index].add_surface(FluidEntry::new(fluid, volume_vu)) };
        self.ledger.injected += accepted as u64;
        if accepted < volume_vu { self.events.push(SimEvent::SourceBackpressure { cell: pos, requested_vu: volume_vu, accepted_vu: accepted }); }
    }

    pub fn terrain_edit(&mut self, pos: CellPos, action: TerrainAction) -> Result<(), TerrainError> {
        let index = self.index(pos).ok_or(TerrainError::OutOfBounds)?;
        let definition = &self.definitions[index];
        if definition.protected { return Err(TerrainError::Protected); }
        if action == TerrainAction::Raise && self.cells[index].surface_volume() > 0 { return Err(TerrainError::Capacity); }
        let cell = &mut self.cells[index];
        match action {
            TerrainAction::Excavate => cell.height_hu = (cell.height_hu - 250).max(definition.minimum_height_hu),
            TerrainAction::Raise => cell.height_hu = (cell.height_hu + 250).min(definition.maximum_height_hu),
            TerrainAction::Seal => cell.sealed = true,
        }
        Ok(())
    }

    pub fn tick(&mut self) {
        self.events.clear();
        self.tick = self.tick.saturating_add(1);
        self.flow_surface();
        self.flow_steam();
        self.react_materials();
        self.heat_and_phase_change();
        self.apply_terrain_products();
        let mut devices = std::mem::take(&mut self.devices);
        devices.tick(self);
        self.devices = devices;
        self.sort_entries();
    }

    fn flow_surface(&mut self) {
        let snapshot = self.cells.clone();
        let mut transfers: Vec<(usize, usize, FluidId, u32, i32, u16)> = Vec::new();
        for y in 0..self.height { for x in 0..self.width {
            let source_pos = CellPos { x, y }; let source_index = self.index(source_pos).unwrap(); let source = &snapshot[source_index];
            let total = source.surface_volume(); if total == 0 { continue; }
            let limit = source.surface.iter().map(|m| m.fluid.max_transfer()).min().unwrap_or(0);
            let options: Vec<_> = self.neighbors(source_pos).into_iter().filter_map(|(dest_pos, direction)| {
                let dest = &snapshot[self.index(dest_pos).unwrap()]; let delta = source.surface_head_hu() - dest.surface_head_hu();
                let gate_factor = self.devices.surface_flow_factor(source_pos, dest_pos);
                if delta <= 1 || dest.sealed || dest.surface_volume() >= CELL_CAPACITY_VU || gate_factor == 0 { None } else { Some((dest_pos, direction, (delta.max(0) as u32 / 4).saturating_mul(gate_factor) / 10_000) ) }
            }).collect();
            let total_weight: u32 = options.iter().map(|o| o.2).sum(); if total_weight == 0 { continue; }
            let budget = total.min(limit); let mut assigned = 0;
            for (dest_pos, _, weight) in options {
                let amount = (budget as u64 * weight as u64 / total_weight as u64) as u32; assigned += amount;
                if amount > 0 { let (fluid, temp, contamination) = mixture_for(source, amount); transfers.push((source_index, self.index(dest_pos).unwrap(), fluid, amount, temp, contamination)); }
            }
            if assigned < budget { if let Some((dest_pos, _)) = self.neighbors(source_pos).first().copied() { let amount = budget - assigned; let (fluid, temp, contamination) = mixture_for(source, amount); transfers.push((source_index, self.index(dest_pos).unwrap(), fluid, amount, temp, contamination)); } }
        }}
        for (source, destination, fluid, amount, temp, contamination) in transfers { remove_fluid(&mut self.cells[source].surface, fluid, amount); self.cells[destination].add_surface(FluidEntry { fluid, volume_vu: amount, temperature_dk: temp, contamination_bp: contamination }); }
    }

    fn flow_steam(&mut self) {
        let snapshot = self.cells.clone(); let mut moves = Vec::new();
        for y in 0..self.height { for x in 0..self.width { let pos = CellPos { x, y }; let source_index = self.index(pos).unwrap(); let source = &snapshot[source_index]; let Some(steam) = source.airborne.iter().find(|m| m.fluid == FluidId::Steam) else { continue; }; for (dest, _) in self.neighbors(pos) { let di = self.index(dest).unwrap(); let target = &snapshot[di]; if target.airborne_volume() >= source.airborne_volume() || target_airborne_blocked(di) { continue; } let amount = ((source.airborne_volume() - target.airborne_volume()) / 5).min(steam.volume_vu).min(FluidId::Steam.max_transfer()); if amount > 0 { moves.push((source_index, di, amount)); } } }}
        for (source, destination, amount) in moves { remove_fluid(&mut self.cells[source].airborne, FluidId::Steam, amount); self.cells[destination].add_airborne(FluidEntry::new(FluidId::Steam, amount)); }
        for index in 0..self.cells.len() { if self.cells[index].airborne_volume() > 6_000 { let pos = CellPos { x: index as u16 % self.width, y: index as u16 / self.width }; self.events.push(SimEvent::HighPressureSteam { cell: pos }); } }
        for index in 0..self.cells.len() { let ambient = self.definitions[index].ambient_temperature_dk; let cold = ambient <= WATER_BOIL_DK; if cold { let amount = self.cells[index].airborne.iter().find(|m| m.fluid == FluidId::Steam).map(|m| m.volume_vu.min(200)).unwrap_or(0); if amount > 0 && self.cells[index].surface_volume() < CELL_CAPACITY_VU { remove_fluid(&mut self.cells[index].airborne, FluidId::Steam, amount); self.cells[index].add_surface(FluidEntry::new(FluidId::Water, amount)); } } }
    }

    fn react_materials(&mut self) {
        for index in 0..self.cells.len() { let pos = CellPos { x: index as u16 % self.width, y: index as u16 / self.width };
            let water = volume(&self.cells[index].surface, FluidId::Water); let lava = volume(&self.cells[index].surface, FluidId::Lava); let reaction = water.min(lava).min(250);
            if reaction > 0 { remove_fluid(&mut self.cells[index].surface, FluidId::Water, reaction); remove_fluid(&mut self.cells[index].surface, FluidId::Lava, reaction); self.cells[index].add_airborne(FluidEntry { fluid: FluidId::Steam, volume_vu: reaction, temperature_dk: 4_730, contamination_bp: 0 }); self.cells[index].pending_rock_vu += reaction; self.ledger.reacted += (reaction * 2) as u64; self.ledger.products += reaction as u64; self.events.push(SimEvent::MaterialReacted { reaction: "water_lava".into(), cell: pos, volume_vu: reaction }); }
            let lava = volume(&self.cells[index].surface, FluidId::Lava); let slurry = volume(&self.cells[index].surface, FluidId::ToxicSlurry); let reaction = lava.min(slurry).min(120);
            if reaction > 0 { remove_fluid(&mut self.cells[index].surface, FluidId::Lava, reaction); remove_fluid(&mut self.cells[index].surface, FluidId::ToxicSlurry, reaction); self.cells[index].pending_vitrified_vu += reaction * 2; self.cells[index].ground_contamination_bp = self.cells[index].ground_contamination_bp.saturating_sub((reaction * 5).min(u16::MAX as u32) as u16); self.ledger.reacted += (reaction * 2) as u64; self.events.push(SimEvent::MaterialReacted { reaction: "slurry_vitrified".into(), cell: pos, volume_vu: reaction }); }
            if volume(&self.cells[index].surface, FluidId::ToxicSlurry) > 0 && !self.cells[index].sealed { let concentration = self.cells[index].surface.iter().find(|m| m.fluid == FluidId::ToxicSlurry).map(|m| m.contamination_bp).unwrap_or(0); self.cells[index].ground_contamination_bp = self.cells[index].ground_contamination_bp.max(concentration); }
        }
    }

    fn heat_and_phase_change(&mut self) { for index in 0..self.cells.len() { let ambient = self.definitions[index].ambient_temperature_dk; for entry in &mut self.cells[index].surface { let delta = ambient - entry.temperature_dk; if delta != 0 { entry.temperature_dk += delta / 100 + delta.signum(); } } let water = self.cells[index].surface.iter().find(|m| m.fluid == FluidId::Water && m.temperature_dk >= WATER_BOIL_DK).map(|m| m.volume_vu.min(150)).unwrap_or(0); if water > 0 { remove_fluid(&mut self.cells[index].surface, FluidId::Water, water); self.cells[index].add_airborne(FluidEntry::new(FluidId::Steam, water)); } let lava = self.cells[index].surface.iter().find(|m| m.fluid == FluidId::Lava && m.temperature_dk < 9_000).map(|m| m.volume_vu.min(80)).unwrap_or(0); if lava > 0 { remove_fluid(&mut self.cells[index].surface, FluidId::Lava, lava); self.cells[index].pending_rock_vu += lava; } } }
    fn apply_terrain_products(&mut self) {
        for cell in &mut self.cells {
            if cell.pending_rock_vu >= 1_000 { let steps = cell.pending_rock_vu / 1_000; cell.height_hu = cell.height_hu.saturating_add((steps * 1_000).min(i16::MAX as u32) as i16); cell.pending_rock_vu %= 1_000; }
            if cell.pending_vitrified_vu >= 1_000 { let steps = cell.pending_vitrified_vu / 1_000; cell.height_hu = cell.height_hu.saturating_add((steps * 500).min(i16::MAX as u32) as i16); cell.pending_vitrified_vu %= 1_000; }
        }
    }
    fn sort_entries(&mut self) { for cell in &mut self.cells { cell.surface.sort_by_key(|m| m.fluid); cell.airborne.sort_by_key(|m| m.fluid); } }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerrainAction { Excavate, Raise, Seal }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerrainError { OutOfBounds, Protected, Capacity }

fn target_airborne_blocked(_index: usize) -> bool { false }
fn volume(entries: &[FluidEntry], fluid: FluidId) -> u32 { entries.iter().find(|m| m.fluid == fluid).map(|m| m.volume_vu).unwrap_or(0) }
fn remove_fluid(entries: &mut Vec<FluidEntry>, fluid: FluidId, amount: u32) { if let Some(entry) = entries.iter_mut().find(|m| m.fluid == fluid) { entry.volume_vu -= amount.min(entry.volume_vu); } entries.retain(|m| m.volume_vu > 0); }
fn merge_entry(entries: &mut Vec<FluidEntry>, entry: FluidEntry) { if let Some(existing) = entries.iter_mut().find(|m| m.fluid == entry.fluid) { let total = existing.volume_vu + entry.volume_vu; existing.temperature_dk = ((existing.temperature_dk as i64 * existing.volume_vu as i64 + entry.temperature_dk as i64 * entry.volume_vu as i64) / total as i64) as i32; existing.contamination_bp = ((existing.contamination_bp as u64 * existing.volume_vu as u64 + entry.contamination_bp as u64 * entry.volume_vu as u64) / total as u64) as u16; existing.volume_vu = total; } else { entries.push(entry); entries.sort_by_key(|m| m.fluid); } }
fn mixture_for(source: &SimCell, _amount: u32) -> (FluidId, i32, u16) { let entry = source.surface.iter().max_by_key(|m| m.volume_vu).expect("mixture source is nonempty"); (entry.fluid, entry.temperature_dk, entry.contamination_bp) }

#[cfg(test)]
mod tests {
    use super::*;
    fn pos(x: u16, y: u16) -> CellPos { CellPos { x, y } }
    #[test] fn terrain_actions_obey_protection_and_capacity() { let mut world = SimulationWorld::new(3, 1); world.definitions[0].protected = true; assert_eq!(world.terrain_edit(pos(0, 0), TerrainAction::Excavate), Err(TerrainError::Protected)); world.inject(pos(1, 0), FluidId::Water, 100); assert_eq!(world.terrain_edit(pos(1, 0), TerrainAction::Raise), Err(TerrainError::Capacity)); assert!(world.terrain_edit(pos(1, 0), TerrainAction::Excavate).is_ok()); }
    #[test] fn water_flows_downhill_and_respects_limit() { let mut world = SimulationWorld::new(2, 1); world.cells[0].height_hu = 2_000; world.cells[1].height_hu = 0; world.inject(pos(0, 0), FluidId::Water, 1_000); world.tick(); assert_eq!(volume(&world.cells[0].surface, FluidId::Water), 600); assert_eq!(volume(&world.cells[1].surface, FluidId::Water), 400); }
    #[test] fn lava_and_water_make_steam_and_rock() { let mut world = SimulationWorld::new(1, 1); world.inject(pos(0, 0), FluidId::Water, 500); world.inject(pos(0, 0), FluidId::Lava, 500); world.tick(); assert_eq!(volume(&world.cells[0].surface, FluidId::Water), 250); assert_eq!(volume(&world.cells[0].surface, FluidId::Lava), 250); assert_eq!(volume(&world.cells[0].airborne, FluidId::Steam), 250); assert_eq!(world.cells[0].pending_rock_vu, 250); assert!(world.events.iter().any(|event| matches!(event, SimEvent::MaterialReacted { reaction, .. } if reaction == "water_lava"))); }
    #[test] fn slurry_contaminates_unsealed_ground_and_lava_vitrifies() { let mut world = SimulationWorld::new(1, 1); world.inject(pos(0, 0), FluidId::ToxicSlurry, 500); world.inject(pos(0, 0), FluidId::Lava, 500); world.tick(); assert_eq!(world.cells[0].ground_contamination_bp, 10_000); assert_eq!(world.cells[0].pending_vitrified_vu, 240); }
    #[test] fn source_backpressure_is_explicit() { let mut world = SimulationWorld::new(1, 1); world.inject(pos(0, 0), FluidId::Water, 9_000); assert!(world.events.iter().any(|event| matches!(event, SimEvent::SourceBackpressure { accepted_vu: 8_000, .. }))); }
}
