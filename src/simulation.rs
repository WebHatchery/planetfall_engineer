//! Deterministic terrain, surface-fluid, steam, reaction, and ledger state.
//!
//! The renderer consumes this state but never participates in its decisions.

use crate::{
    devices::DeviceSystem,
    economy::{FabricationError, FabricationState, PowerLedger, PowerSource, ResourceDeposit},
    state::CellPos,
};
use serde::{Deserialize, Serialize};

mod process;
pub use process::volume;

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

    pub const fn is_airborne(self) -> bool {
        matches!(self, Self::Steam)
    }
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
        Self {
            fluid,
            volume_vu,
            temperature_dk: fluid.default_temperature(),
            contamination_bp: if fluid == FluidId::ToxicSlurry {
                10_000
            } else {
                0
            },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CellDefinition {
    pub base_height_hu: i16,
    pub minimum_height_hu: i16,
    pub maximum_height_hu: i16,
    pub protected: bool,
    pub gas_blocked: bool,
    #[serde(default)]
    pub surface_drain_rate_vu: u32,
    pub ambient_temperature_dk: i32,
}

impl Default for CellDefinition {
    fn default() -> Self {
        Self {
            base_height_hu: 1_000,
            minimum_height_hu: 0,
            maximum_height_hu: 8_000,
            protected: false,
            gas_blocked: false,
            surface_drain_rate_vu: 0,
            ambient_temperature_dk: 2_930,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimCell {
    pub height_hu: i16,
    pub sealed: bool,
    #[serde(default)]
    pub contained: bool,
    pub ground_contamination_bp: u16,
    pub surface: Vec<FluidEntry>,
    pub airborne: Vec<FluidEntry>,
    pub pending_rock_vu: u32,
    pub pending_vitrified_vu: u32,
    #[serde(default)]
    pub formed_rock_vu: u32,
    #[serde(default)]
    pub formed_vitrified_vu: u32,
}

impl SimCell {
    pub fn empty(height_hu: i16, sealed: bool) -> Self {
        Self {
            height_hu,
            sealed,
            contained: false,
            ground_contamination_bp: 0,
            surface: Vec::new(),
            airborne: Vec::new(),
            pending_rock_vu: 0,
            pending_vitrified_vu: 0,
            formed_rock_vu: 0,
            formed_vitrified_vu: 0,
        }
    }

    pub fn surface_volume(&self) -> u32 {
        self.surface.iter().map(|material| material.volume_vu).sum()
    }

    pub fn airborne_volume(&self) -> u32 {
        self.airborne
            .iter()
            .map(|material| material.volume_vu)
            .sum()
    }

    pub fn surface_head_hu(&self) -> i32 {
        self.height_hu as i32 + self.surface_volume() as i32
    }

    pub fn add_surface(&mut self, mut entry: FluidEntry) -> u32 {
        let accepted = entry
            .volume_vu
            .min(CELL_CAPACITY_VU.saturating_sub(self.surface_volume()));
        entry.volume_vu = accepted;
        if accepted > 0 {
            merge_entry(&mut self.surface, entry);
        }
        accepted
    }

    pub fn add_airborne(&mut self, mut entry: FluidEntry) -> u32 {
        let accepted = entry
            .volume_vu
            .min(CELL_CAPACITY_VU.saturating_sub(self.airborne_volume()));
        entry.volume_vu = accepted;
        if accepted > 0 {
            merge_entry(&mut self.airborne, entry);
        }
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceState {
    pub position: CellPos,
    pub fluid: FluidId,
    pub rate_vu: u32,
    pub enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SimEvent {
    SourceBackpressure {
        cell: CellPos,
        requested_vu: u32,
        accepted_vu: u32,
    },
    MaterialReacted {
        reaction: String,
        cell: CellPos,
        volume_vu: u32,
    },
    HighPressureSteam {
        cell: CellPos,
    },
    DepositRecovered {
        id: String,
        yield_fu: u32,
    },
    PowerBrownout {
        entity_id: u32,
        demand_eu: u32,
    },
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
    pub sources: Vec<SourceState>,
    pub deposits: Vec<ResourceDeposit>,
    pub power_sources: Vec<PowerSource>,
    pub fabrication: FabricationState,
    pub power: PowerLedger,
}

impl SimulationWorld {
    pub fn new(width: u16, height: u16) -> Self {
        let definitions = vec![CellDefinition::default(); width as usize * height as usize];
        let cells = definitions
            .iter()
            .map(|definition| SimCell::empty(definition.base_height_hu, false))
            .collect();
        Self {
            width,
            height,
            definitions,
            cells,
            tick: 0,
            ledger: MassLedger::default(),
            events: Vec::new(),
            devices: DeviceSystem::default(),
            sources: Vec::new(),
            deposits: Vec::new(),
            power_sources: Vec::new(),
            fabrication: FabricationState::default(),
            power: PowerLedger::default(),
        }
    }

    pub fn index(&self, pos: CellPos) -> Option<usize> {
        (pos.x < self.width && pos.y < self.height)
            .then_some(pos.y as usize * self.width as usize + pos.x as usize)
    }

    fn neighbors(&self, pos: CellPos) -> Vec<(CellPos, u8)> {
        [(0, -1), (1, 0), (0, 1), (-1, 0)]
            .into_iter()
            .enumerate()
            .filter_map(|(direction, (dx, dy))| {
                let x = pos.x as i16 + dx;
                let y = pos.y as i16 + dy;
                (x >= 0 && y >= 0 && x < self.width as i16 && y < self.height as i16).then_some((
                    CellPos {
                        x: x as u16,
                        y: y as u16,
                    },
                    direction as u8,
                ))
            })
            .collect()
    }

    pub fn inject(&mut self, pos: CellPos, fluid: FluidId, volume_vu: u32) {
        let Some(index) = self.index(pos) else { return };
        let accepted = if fluid.is_airborne() {
            self.cells[index].add_airborne(FluidEntry::new(fluid, volume_vu))
        } else {
            self.cells[index].add_surface(FluidEntry::new(fluid, volume_vu))
        };
        self.ledger.injected += accepted as u64;
        if accepted < volume_vu {
            self.events.push(SimEvent::SourceBackpressure {
                cell: pos,
                requested_vu: volume_vu,
                accepted_vu: accepted,
            });
        }
    }

    pub fn add_source(&mut self, position: CellPos, fluid: FluidId, rate_vu: u32) {
        self.sources.push(SourceState {
            position,
            fluid,
            rate_vu,
            enabled: false,
        });
    }

    pub fn set_sources_enabled(&mut self, enabled: bool) {
        for source in &mut self.sources {
            source.enabled = enabled;
        }
    }

    pub fn set_power_source_enabled(&mut self, id: &str, enabled: bool) -> bool {
        if let Some(source) = self.power_sources.iter_mut().find(|source| source.id == id) {
            source.enabled = enabled;
            true
        } else {
            false
        }
    }

    pub fn add_deposit(
        &mut self,
        id: impl Into<String>,
        position: CellPos,
        yield_fu: u32,
        asset_id: impl Into<String>,
    ) {
        self.deposits.push(ResourceDeposit {
            id: id.into(),
            position,
            yield_fu,
            asset_id: asset_id.into(),
            depleted: false,
        });
    }

    pub fn add_power_source(
        &mut self,
        id: impl Into<String>,
        position: CellPos,
        output_eu_per_tick: u32,
        asset_id: impl Into<String>,
    ) {
        self.power_sources.push(PowerSource {
            id: id.into(),
            position,
            output_eu_per_tick,
            enabled: true,
            asset_id: asset_id.into(),
        });
    }

    pub fn recover_deposit(&mut self, id: &str) -> Result<u32, FabricationError> {
        let deposit = self
            .deposits
            .iter_mut()
            .find(|deposit| deposit.id == id)
            .ok_or_else(|| FabricationError::DepositNotFound(id.into()))?;
        if deposit.depleted {
            return Err(FabricationError::DepositDepleted(id.into()));
        }
        deposit.depleted = true;
        self.fabrication.recovered_fu += deposit.yield_fu;
        self.fabrication.available_fu += deposit.yield_fu;
        self.events.push(SimEvent::DepositRecovered {
            id: deposit.id.clone(),
            yield_fu: deposit.yield_fu,
        });
        Ok(deposit.yield_fu)
    }

    pub fn recover_deposit_at(&mut self, position: CellPos) -> Result<u32, FabricationError> {
        let id = self
            .deposits
            .iter()
            .find(|deposit| deposit.position == position)
            .map(|deposit| deposit.id.clone())
            .ok_or_else(|| {
                FabricationError::DepositNotFound(format!("{},{}", position.x, position.y))
            })?;
        self.recover_deposit(&id)
    }

    pub fn total_material_volume(&self) -> u64 {
        self.cells
            .iter()
            .map(|cell| {
                cell.surface
                    .iter()
                    .map(|entry| entry.volume_vu as u64)
                    .sum::<u64>()
                    + cell
                        .airborne
                        .iter()
                        .map(|entry| entry.volume_vu as u64)
                        .sum::<u64>()
                    + u64::from(cell.pending_rock_vu)
                    + u64::from(cell.pending_vitrified_vu)
                    + u64::from(cell.formed_rock_vu)
                    + u64::from(cell.formed_vitrified_vu)
            })
            .sum::<u64>()
            + self
                .devices
                .devices
                .iter()
                .map(|device| device.stored_vu as u64)
                .sum::<u64>()
    }

    pub fn mass_balance_error(&self) -> i64 {
        self.ledger.injected as i64
            - self.ledger.drained as i64
            - self.total_material_volume() as i64
    }

    pub fn terrain_edit(
        &mut self,
        pos: CellPos,
        action: TerrainAction,
    ) -> Result<(), TerrainError> {
        let index = self.index(pos).ok_or(TerrainError::OutOfBounds)?;
        if self.definitions[index].protected {
            return Err(TerrainError::Protected);
        }
        if action == TerrainAction::Raise && self.cells[index].surface_volume() > 0 {
            return Err(TerrainError::Capacity);
        }
        let definition = &self.definitions[index];
        let cell = &mut self.cells[index];
        match action {
            TerrainAction::Excavate => {
                cell.height_hu = (cell.height_hu - 250).max(definition.minimum_height_hu)
            }
            TerrainAction::Raise => {
                cell.height_hu = (cell.height_hu + 250).min(definition.maximum_height_hu)
            }
            TerrainAction::Seal => cell.sealed = true,
        }
        Ok(())
    }

    pub fn tick(&mut self) {
        self.events.clear();
        self.tick = self.tick.saturating_add(1);
        for source in self
            .sources
            .clone()
            .into_iter()
            .filter(|source| source.enabled)
        {
            self.inject(source.position, source.fluid, source.rate_vu);
        }
        self.flow_surface();
        self.drain_surface_outlets();
        self.flow_steam();
        self.react_materials();
        self.heat_and_phase_change();
        self.apply_terrain_products();
        let authored_supply: u32 = self
            .power_sources
            .iter()
            .filter(|source| source.enabled)
            .map(|source| source.output_eu_per_tick)
            .sum();
        let mut devices = std::mem::take(&mut self.devices);
        for (entity_id, demand_eu) in devices.allocate_power(&mut self.power, authored_supply) {
            self.events.push(SimEvent::PowerBrownout {
                entity_id,
                demand_eu,
            });
        }
        devices.tick(self);
        self.devices = devices;
        self.sort_entries();
    }

    fn drain_surface_outlets(&mut self) {
        for index in 0..self.cells.len() {
            let mut remaining = self.definitions[index].surface_drain_rate_vu;
            if remaining == 0 {
                continue;
            }
            let fluids: Vec<_> = self.cells[index]
                .surface
                .iter()
                .map(|entry| (entry.fluid, entry.volume_vu))
                .collect();
            for (fluid, available) in fluids {
                let amount = available.min(remaining);
                remove_fluid(&mut self.cells[index].surface, fluid, amount);
                self.ledger.drained += u64::from(amount);
                remaining -= amount;
                if remaining == 0 {
                    break;
                }
            }
        }
    }

    fn sort_entries(&mut self) {
        for cell in &mut self.cells {
            cell.surface.sort_by_key(|material| material.fluid);
            cell.airborne.sort_by_key(|material| material.fluid);
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerrainAction {
    Excavate,
    Raise,
    Seal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerrainError {
    OutOfBounds,
    Protected,
    Capacity,
}

fn remove_fluid(entries: &mut Vec<FluidEntry>, fluid: FluidId, amount: u32) {
    if let Some(entry) = entries.iter_mut().find(|entry| entry.fluid == fluid) {
        entry.volume_vu -= amount.min(entry.volume_vu);
    }
    entries.retain(|entry| entry.volume_vu > 0);
}

fn merge_entry(entries: &mut Vec<FluidEntry>, entry: FluidEntry) {
    if let Some(existing) = entries
        .iter_mut()
        .find(|material| material.fluid == entry.fluid)
    {
        let total = existing.volume_vu + entry.volume_vu;
        existing.temperature_dk = ((existing.temperature_dk as i64 * existing.volume_vu as i64
            + entry.temperature_dk as i64 * entry.volume_vu as i64)
            / total as i64) as i32;
        existing.contamination_bp = ((existing.contamination_bp as u64 * existing.volume_vu as u64
            + entry.contamination_bp as u64 * entry.volume_vu as u64)
            / total as u64) as u16;
        existing.volume_vu = total;
    } else {
        entries.push(entry);
        entries.sort_by_key(|material| material.fluid);
    }
}
