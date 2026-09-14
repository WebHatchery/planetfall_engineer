//! Fluid transport, reactions, phase changes, and terrain products.

use super::*;

impl SimulationWorld {
    pub fn flow_surface(&mut self) {
        let snapshot = self.cells.clone();
        let mut transfers: Vec<(usize, usize, FluidId, u32, i32, u16)> = Vec::new();
        for y in 0..self.height {
            for x in 0..self.width {
                let source_pos = CellPos { x, y };
                let Some(source_index) = self.index(source_pos) else {
                    continue;
                };
                let source = &snapshot[source_index];
                let total = source.surface_volume();
                if total == 0 {
                    continue;
                }
                let limit = source
                    .surface
                    .iter()
                    .map(|m| m.fluid.max_transfer())
                    .min()
                    .unwrap_or(0);
                let options: Vec<_> = self
                    .neighbors(source_pos)
                    .into_iter()
                    .filter_map(|(dest_pos, direction)| {
                        let dest_index = self.index(dest_pos)?;
                        let dest = &snapshot[dest_index];
                        let delta = source.surface_head_hu() - dest.surface_head_hu();
                        let gate_factor = self.devices.surface_flow_factor(source_pos, dest_pos);
                        if delta <= 1
                            || (source.contained != dest.contained
                                && !self.devices.controls_surface_edge(source_pos, dest_pos))
                            || dest.surface_volume() >= CELL_CAPACITY_VU
                            || gate_factor == 0
                        {
                            None
                        } else {
                            Some((
                                dest_pos,
                                direction,
                                (delta.max(0) as u32 / 4).saturating_mul(gate_factor) / 10_000,
                            ))
                        }
                    })
                    .collect();
                let total_weight: u32 = options.iter().map(|o| o.2).sum();
                if total_weight == 0 {
                    continue;
                }
                let transfer_budget = total.min(limit).min(total_weight);
                let mut assigned = 0;
                for &(dest_pos, _, weight) in &options {
                    let Some(destination) = self.index(dest_pos) else {
                        continue;
                    };
                    let amount =
                        (transfer_budget as u64 * weight as u64 / total_weight as u64) as u32;
                    assigned += amount;
                    if amount > 0 {
                        for (fluid, amount, temp, contamination) in mixture_split(source, amount) {
                            transfers.push((
                                source_index,
                                destination,
                                fluid,
                                amount,
                                temp,
                                contamination,
                            ));
                        }
                    }
                }
                if assigned < transfer_budget {
                    if let Some((dest_pos, _, _)) = options.first().copied() {
                        let Some(destination) = self.index(dest_pos) else {
                            continue;
                        };
                        let amount = transfer_budget - assigned;
                        for (fluid, amount, temp, contamination) in mixture_split(source, amount) {
                            transfers.push((
                                source_index,
                                destination,
                                fluid,
                                amount,
                                temp,
                                contamination,
                            ));
                        }
                    }
                }
            }
        }
        self.apply_surface_transfers(transfers);
    }

    fn apply_surface_transfers(&mut self, transfers: Vec<(usize, usize, FluidId, u32, i32, u16)>) {
        for (source, destination, fluid, amount, temp, contamination) in transfers {
            let available = self.cells[source]
                .surface
                .iter()
                .find(|entry| entry.fluid == fluid)
                .map(|entry| entry.volume_vu)
                .unwrap_or(0);
            let moved = amount
                .min(available)
                .min(CELL_CAPACITY_VU.saturating_sub(self.cells[destination].surface_volume()));
            if moved > 0 {
                remove_fluid(&mut self.cells[source].surface, fluid, moved);
                self.cells[destination].add_surface(FluidEntry {
                    fluid,
                    volume_vu: moved,
                    temperature_dk: temp,
                    contamination_bp: contamination,
                });
            }
        }
    }

    pub fn flow_steam(&mut self) {
        let snapshot = self.cells.clone();
        let mut moves = Vec::new();
        for y in 0..self.height {
            for x in 0..self.width {
                let pos = CellPos { x, y };
                let Some(source_index) = self.index(pos) else {
                    continue;
                };
                let source = &snapshot[source_index];
                let Some(steam) = source.airborne.iter().find(|m| m.fluid == FluidId::Steam) else {
                    continue;
                };
                let options: Vec<_> = self
                    .neighbors(pos)
                    .into_iter()
                    .filter_map(|(dest, _)| {
                        let di = self.index(dest)?;
                        let target = &snapshot[di];
                        if target.airborne_volume() >= source.airborne_volume()
                            || target.sealed
                            || self.definitions[di].gas_blocked
                        {
                            return None;
                        }
                        let raw = (source.airborne_volume() - target.airborne_volume()) / 5;
                        (raw > 0).then_some((di, raw))
                    })
                    .collect();
                let total_weight: u32 = options.iter().map(|option| option.1).sum();
                let transfer_budget = steam
                    .volume_vu
                    .min(FluidId::Steam.max_transfer())
                    .min(total_weight);
                if transfer_budget == 0 {
                    continue;
                }
                let mut assigned = 0;
                for &(destination, weight) in &options {
                    let amount =
                        (transfer_budget as u64 * weight as u64 / total_weight as u64) as u32;
                    assigned += amount;
                    if amount > 0 {
                        moves.push((source_index, destination, amount));
                    }
                }
                if assigned < transfer_budget {
                    if let Some(&(destination, _)) = options.first() {
                        moves.push((source_index, destination, transfer_budget - assigned));
                    }
                }
            }
        }
        for (source, destination, amount) in moves {
            let available = self.cells[source]
                .airborne
                .iter()
                .find(|entry| entry.fluid == FluidId::Steam)
                .map(|entry| entry.volume_vu)
                .unwrap_or(0);
            let moved = amount
                .min(available)
                .min(CELL_CAPACITY_VU.saturating_sub(self.cells[destination].airborne_volume()));
            if moved > 0 {
                remove_fluid(&mut self.cells[source].airborne, FluidId::Steam, moved);
                self.cells[destination].add_airborne(FluidEntry::new(FluidId::Steam, moved));
            }
        }
        for index in 0..self.cells.len() {
            if self.cells[index].airborne_volume() > 6_000 {
                let pos = CellPos {
                    x: index as u16 % self.width,
                    y: index as u16 / self.width,
                };
                self.events.push(SimEvent::HighPressureSteam { cell: pos });
            }
        }
        for index in 0..self.cells.len() {
            let ambient = self.definitions[index].ambient_temperature_dk;
            if ambient <= WATER_BOIL_DK {
                let amount = self.cells[index]
                    .airborne
                    .iter()
                    .find(|m| m.fluid == FluidId::Steam)
                    .map(|m| m.volume_vu.min(200))
                    .unwrap_or(0)
                    .min(CELL_CAPACITY_VU.saturating_sub(self.cells[index].surface_volume()));
                if amount > 0 && self.cells[index].surface_volume() < CELL_CAPACITY_VU {
                    remove_fluid(&mut self.cells[index].airborne, FluidId::Steam, amount);
                    self.cells[index].add_surface(FluidEntry::new(FluidId::Water, amount));
                }
            }
        }
    }

    pub(super) fn react_materials(&mut self) {
        for index in 0..self.cells.len() {
            let pos = CellPos {
                x: index as u16 % self.width,
                y: index as u16 / self.width,
            };
            let water = volume(&self.cells[index].surface, FluidId::Water);
            let lava = volume(&self.cells[index].surface, FluidId::Lava);
            let reaction = water
                .min(lava)
                .min(250)
                .min(CELL_CAPACITY_VU.saturating_sub(self.cells[index].airborne_volume()));
            if reaction > 0 {
                remove_fluid(&mut self.cells[index].surface, FluidId::Water, reaction);
                remove_fluid(&mut self.cells[index].surface, FluidId::Lava, reaction);
                self.cells[index].add_airborne(FluidEntry {
                    fluid: FluidId::Steam,
                    volume_vu: reaction,
                    temperature_dk: 4_730,
                    contamination_bp: 0,
                });
                self.cells[index].pending_rock_vu += reaction;
                self.ledger.reacted += (reaction * 2) as u64;
                self.ledger.products += reaction as u64;
                self.events.push(SimEvent::MaterialReacted {
                    reaction: "water_lava".into(),
                    cell: pos,
                    volume_vu: reaction,
                });
            }
            let lava = volume(&self.cells[index].surface, FluidId::Lava);
            let slurry = volume(&self.cells[index].surface, FluidId::ToxicSlurry);
            let reaction = lava.min(slurry).min(120);
            if reaction > 0 {
                remove_fluid(&mut self.cells[index].surface, FluidId::Lava, reaction);
                remove_fluid(
                    &mut self.cells[index].surface,
                    FluidId::ToxicSlurry,
                    reaction,
                );
                self.cells[index].pending_vitrified_vu += reaction * 2;
                self.cells[index].ground_contamination_bp = self.cells[index]
                    .ground_contamination_bp
                    .saturating_sub((reaction * 5).min(u16::MAX as u32) as u16);
                self.ledger.reacted += (reaction * 2) as u64;
                self.events.push(SimEvent::MaterialReacted {
                    reaction: "slurry_vitrified".into(),
                    cell: pos,
                    volume_vu: reaction,
                });
            }
            if volume(&self.cells[index].surface, FluidId::ToxicSlurry) > 0
                && !self.cells[index].sealed
            {
                let concentration = self.cells[index]
                    .surface
                    .iter()
                    .find(|m| m.fluid == FluidId::ToxicSlurry)
                    .map(|m| m.contamination_bp)
                    .unwrap_or(0);
                self.cells[index].ground_contamination_bp =
                    self.cells[index].ground_contamination_bp.max(concentration);
            }
        }
    }

    pub(super) fn heat_and_phase_change(&mut self) {
        for index in 0..self.cells.len() {
            let ambient = self.definitions[index].ambient_temperature_dk;
            for entry in &mut self.cells[index].surface {
                let delta = ambient - entry.temperature_dk;
                if delta != 0 {
                    entry.temperature_dk += delta / 100 + delta.signum();
                }
            }
            let water = self.cells[index]
                .surface
                .iter()
                .find(|m| m.fluid == FluidId::Water && m.temperature_dk >= WATER_BOIL_DK)
                .map(|m| m.volume_vu.min(150))
                .unwrap_or(0)
                .min(CELL_CAPACITY_VU.saturating_sub(self.cells[index].airborne_volume()));
            if water > 0 {
                remove_fluid(&mut self.cells[index].surface, FluidId::Water, water);
                self.cells[index].add_airborne(FluidEntry::new(FluidId::Steam, water));
            }
            let lava = self.cells[index]
                .surface
                .iter()
                .find(|m| m.fluid == FluidId::Lava && m.temperature_dk < 9_000)
                .map(|m| m.volume_vu.min(80))
                .unwrap_or(0);
            if lava > 0 {
                remove_fluid(&mut self.cells[index].surface, FluidId::Lava, lava);
                self.cells[index].pending_rock_vu += lava;
            }
        }
    }

    pub(super) fn apply_terrain_products(&mut self) {
        for cell in &mut self.cells {
            if cell.pending_rock_vu >= 1_000 {
                let steps = cell.pending_rock_vu / 1_000;
                cell.height_hu = cell
                    .height_hu
                    .saturating_add((steps * 1_000).min(i16::MAX as u32) as i16);
                cell.formed_rock_vu += steps * 1_000;
                cell.pending_rock_vu %= 1_000;
            }
            if cell.pending_vitrified_vu >= 1_000 {
                let steps = cell.pending_vitrified_vu / 1_000;
                cell.height_hu = cell
                    .height_hu
                    .saturating_add((steps * 500).min(i16::MAX as u32) as i16);
                cell.formed_vitrified_vu += steps * 1_000;
                cell.pending_vitrified_vu %= 1_000;
            }
        }
    }
}

pub fn volume(entries: &[FluidEntry], fluid: FluidId) -> u32 {
    entries
        .iter()
        .find(|m| m.fluid == fluid)
        .map(|m| m.volume_vu)
        .unwrap_or(0)
}

fn mixture_split(source: &SimCell, amount: u32) -> Vec<(FluidId, u32, i32, u16)> {
    let total = source.surface_volume();
    let mut remaining = amount.min(total);
    let mut split = Vec::new();
    for (index, entry) in source.surface.iter().enumerate() {
        let part = if index + 1 == source.surface.len() {
            remaining
        } else {
            (amount as u64 * entry.volume_vu as u64 / total as u64) as u32
        }
        .min(entry.volume_vu)
        .min(remaining);
        if part > 0 {
            split.push((
                entry.fluid,
                part,
                entry.temperature_dk,
                entry.contamination_bp,
            ));
            remaining -= part;
        }
    }
    split
}
