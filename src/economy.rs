//! Mission-local fabrication stock, deposits, and planetary field power.

use crate::state::CellPos;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceDeposit {
    pub id: String,
    pub position: CellPos,
    pub yield_fu: u32,
    pub asset_id: String,
    #[serde(default)]
    pub depleted: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FabricationState {
    pub authored_start_fu: u32,
    pub recovered_fu: u32,
    pub available_fu: u32,
    pub reserved_fu: u32,
    pub spent_fu: u32,
    pub refunded_fu: u32,
}

impl FabricationState {
    pub fn new(authored_start_fu: u32) -> Self {
        Self {
            authored_start_fu,
            available_fu: authored_start_fu,
            ..Self::default()
        }
    }

    pub fn can_reserve(&self, amount: u32) -> bool {
        amount <= self.available_fu
    }

    pub fn reserve(&mut self, amount: u32) -> Result<(), FabricationError> {
        if !self.can_reserve(amount) {
            return Err(FabricationError::InsufficientStock {
                requested_fu: amount,
                available_fu: self.available_fu,
            });
        }
        self.available_fu -= amount;
        self.reserved_fu += amount;
        Ok(())
    }

    pub fn release_reservation(&mut self, amount: u32) {
        let released = amount.min(self.reserved_fu);
        self.reserved_fu -= released;
        self.available_fu += released;
    }

    pub fn spend_reservation(&mut self, amount: u32) -> Result<(), FabricationError> {
        if amount > self.reserved_fu {
            return Err(FabricationError::ReservationMismatch {
                requested_fu: amount,
                reserved_fu: self.reserved_fu,
            });
        }
        self.reserved_fu -= amount;
        self.spent_fu += amount;
        Ok(())
    }

    pub fn spend_immediately(&mut self, amount: u32) -> Result<(), FabricationError> {
        if !self.can_reserve(amount) {
            return Err(FabricationError::InsufficientStock {
                requested_fu: amount,
                available_fu: self.available_fu,
            });
        }
        self.available_fu -= amount;
        self.spent_fu += amount;
        Ok(())
    }

    pub fn refund(&mut self, amount: u32) {
        self.available_fu += amount;
        self.refunded_fu += amount;
    }

    pub fn balance_error(&self) -> i64 {
        i64::from(self.available_fu) + i64::from(self.reserved_fu) + i64::from(self.spent_fu)
            - i64::from(self.authored_start_fu)
            - i64::from(self.recovered_fu)
            - i64::from(self.refunded_fu)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PowerSource {
    pub id: String,
    pub position: CellPos,
    pub output_eu_per_tick: u32,
    pub enabled: bool,
    pub asset_id: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PowerLedger {
    pub authored_generation_eu: u32,
    pub turbine_generation_eu: u32,
    pub allocated_demand_eu: u32,
    pub curtailed_supply_eu: u32,
    pub deficit_eu: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum PowerClass {
    Safety,
    Transport,
    Process,
    Interface,
}

impl PowerLedger {
    pub fn supply_eu(&self) -> u32 {
        self.authored_generation_eu + self.turbine_generation_eu
    }

    pub fn reset(&mut self, authored_generation_eu: u32, turbine_generation_eu: u32) {
        *self = Self {
            authored_generation_eu,
            turbine_generation_eu,
            ..Self::default()
        };
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FabricationError {
    DepositNotFound(String),
    DepositDepleted(String),
    InsufficientStock {
        requested_fu: u32,
        available_fu: u32,
    },
    ReservationMismatch {
        requested_fu: u32,
        reserved_fu: u32,
    },
}

impl fmt::Display for FabricationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DepositNotFound(id) => write!(formatter, "deposit {id} is not on this map"),
            Self::DepositDepleted(id) => write!(formatter, "deposit {id} is already depleted"),
            Self::InsufficientStock {
                requested_fu,
                available_fu,
            } => write!(
                formatter,
                "need {requested_fu} fabU but only {available_fu} fabU is available"
            ),
            Self::ReservationMismatch {
                requested_fu,
                reserved_fu,
            } => write!(
                formatter,
                "cannot spend {requested_fu} fabU from {reserved_fu} fabU reserved"
            ),
        }
    }
}
