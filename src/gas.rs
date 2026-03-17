//! Gas cost analysis for EVM opcodes.
//!
//! This module provides:
//! - **Static classification** via [`GasCostCategory`]
//! - **Dynamic calculation** via [`DynamicGasCalculator`] (EIP-2929 warm/cold,
//!   memory expansion, complex call pricing)
//! - **Sequence analysis** via [`GasAnalyzer`]
//! - **Fork comparison** via [`GasComparator`]

pub mod analysis;
pub mod calculator;
pub mod context;

pub use analysis::*;
pub use calculator::*;
pub use context::*;

// ---------------------------------------------------------------------------
// Gas constants
// ---------------------------------------------------------------------------

/// Gas cost: base (Gzero) — STOP, RETURN, REVERT
pub const ZERO: u64 = 0;
/// Gas cost: jumpdest
pub const JUMPDEST: u64 = 1;
/// Gas cost: very low (Gbase) — ADD, SUB, LT, GT, etc.
pub const VERYLOW: u64 = 3;
/// Gas cost: low (Glow) — MUL, DIV, etc.
pub const LOW: u64 = 5;
/// Gas cost: mid (Gmid) — ADDMOD, MULMOD
pub const MID: u64 = 8;
/// Gas cost: high (Ghigh) — JUMP
pub const HIGH: u64 = 8;
/// Gas cost: warm storage read (EIP-2929)
pub const WARM_STORAGE_READ_COST: u64 = 100;
/// Gas cost: cold SLOAD (EIP-2929)
pub const COLD_SLOAD_COST: u64 = 2100;
/// Gas cost: cold account access (EIP-2929)
pub const COLD_ACCOUNT_ACCESS_COST: u64 = 2600;
/// Gas cost: SSTORE set (new value)
pub const SSTORE_SET: u64 = 20000;
/// Gas cost: SSTORE reset (change value)
pub const SSTORE_RESET: u64 = 2900;
/// Gas cost: base transaction
pub const TX_BASE: u64 = 21000;
/// Gas cost: per byte of transaction data (non-zero)
pub const TX_DATA_NON_ZERO: u64 = 16;
/// Gas cost: per byte of transaction data (zero)
pub const TX_DATA_ZERO: u64 = 4;
/// Gas cost: contract creation
pub const CREATE_GAS: u64 = 32000;
/// Gas cost: per word of SHA3 data
pub const KECCAK256_WORD: u64 = 6;
/// Gas cost: per word of copied data
pub const COPY_WORD: u64 = 3;
/// Gas cost: per LOG topic
pub const LOG_TOPIC: u64 = 375;
/// Gas cost: per byte of LOG data
pub const LOG_DATA: u64 = 8;

// ---------------------------------------------------------------------------
// GasCostCategory
// ---------------------------------------------------------------------------

/// Broad classification of opcode gas costs for analysis.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GasCostCategory {
    /// Very cheap operations (1–3 gas).
    VeryLow,
    /// Low cost operations (3–8 gas).
    Low,
    /// Medium cost operations (8–100 gas).
    Medium,
    /// High cost operations (100–2600 gas).
    High,
    /// Very high cost operations (2600+ gas).
    VeryHigh,
    /// Unclassified.
    Unknown,
}

impl GasCostCategory {
    /// Classify an opcode by its base gas cost range.
    pub fn classify(opcode: u8) -> Self {
        match opcode {
            0x01..=0x0b | 0x10..=0x1d | 0x50 | 0x58 | 0x80..=0x9f => Self::VeryLow,
            0x51..=0x53 | 0x56..=0x57 | 0x5a..=0x5b => Self::Low,
            0x20 | 0x30 | 0x32..=0x3a | 0x40..=0x48 => Self::Medium,
            0x54 | 0x31 | 0x3b | 0x3c | 0x3d | 0x3e | 0x3f => Self::High,
            0x55 | 0xf0..=0xff => Self::VeryHigh,
            _ => Self::Unknown,
        }
    }

    /// Returns the typical gas range for this category.
    pub fn gas_range(&self) -> (u64, u64) {
        match self {
            Self::VeryLow => (1, 3),
            Self::Low => (3, 8),
            Self::Medium => (8, 100),
            Self::High => (100, 2600),
            Self::VeryHigh => (2600, u64::MAX),
            Self::Unknown => (0, 0),
        }
    }
}

// ---------------------------------------------------------------------------
// GasAnalysis — lightweight analysis result
// ---------------------------------------------------------------------------

/// Gas analysis result for a sequence of opcodes.
#[derive(Debug, Clone)]
pub struct GasAnalysis {
    /// Total gas (including base transaction cost).
    pub total_gas: u64,
    /// Per-opcode gas breakdown: `(opcode_byte, gas_cost)`.
    pub breakdown: Vec<(u8, u16)>,
    /// Optimization suggestions.
    pub optimizations: Vec<String>,
    /// Warnings about expensive operations.
    pub warnings: Vec<String>,
}

impl GasAnalysis {
    /// Creates a new analysis with base transaction cost.
    pub fn new() -> Self {
        Self {
            total_gas: TX_BASE,
            breakdown: Vec::new(),
            optimizations: Vec::new(),
            warnings: Vec::new(),
        }
    }

    /// Gas efficiency score (0–100, higher is better).
    pub fn efficiency_score(&self) -> u8 {
        if self.breakdown.is_empty() {
            return 0;
        }
        let opcode_gas = self.total_gas.saturating_sub(TX_BASE);
        let avg = opcode_gas / self.breakdown.len() as u64;
        match avg {
            0..=10 => 100,
            11..=50 => 80,
            51..=200 => 60,
            201..=1000 => 40,
            1001..=5000 => 20,
            _ => 0,
        }
    }

    /// Returns optimization recommendations based on the breakdown.
    pub fn get_optimization_recommendations(&self) -> Vec<String> {
        let mut recs = self.optimizations.clone();
        let mut counts = std::collections::HashMap::new();
        for &(op, _) in &self.breakdown {
            *counts.entry(op).or_insert(0u32) += 1;
        }
        for (op, count) in counts {
            if count > 5 && matches!(op, 0x54 | 0x55 | 0xf1 | 0xf4) {
                recs.push(format!(
                    "Opcode 0x{op:02x} used {count} times — consider batching or caching"
                ));
            }
        }
        recs
    }

    /// Returns `true` if the analysis looks well-optimised.
    pub fn is_optimized(&self) -> bool {
        self.efficiency_score() > 70 && self.warnings.is_empty()
    }

    /// Gas usage grouped by [`GasCostCategory`].
    pub fn gas_by_category(&self) -> std::collections::HashMap<GasCostCategory, u64> {
        let mut map = std::collections::HashMap::new();
        for &(op, cost) in &self.breakdown {
            *map.entry(GasCostCategory::classify(op)).or_insert(0) += cost as u64;
        }
        map
    }

    /// Detects operations that could cause out-of-gas.
    pub fn find_gas_bombs(&self) -> Vec<String> {
        let mut bombs = Vec::new();
        for &(op, cost) in &self.breakdown {
            match op {
                0x55 if cost > 5000 => {
                    bombs.push("SSTORE with high gas cost — could cause out-of-gas".into());
                }
                0xf1 | 0xf2 | 0xf4 | 0xfa if cost > 10000 => {
                    bombs.push("Call operation with high gas cost — ensure sufficient gas".into());
                }
                0xf0 | 0xf5 if cost > 50000 => {
                    bombs.push("Create operation with very high gas cost".into());
                }
                _ => {}
            }
        }
        bombs
    }

    /// Estimates potential gas savings from optimizations.
    pub fn estimate_optimization_savings(&self) -> u64 {
        let mut savings = 0u64;
        let mut sload_count = 0u32;
        let mut prev = None;
        for &(op, cost) in &self.breakdown {
            if op == 0x54 {
                sload_count += 1;
            }
            if matches!(prev, Some(0x80..=0x8f)) && op == 0x50 {
                savings += cost as u64;
            }
            prev = Some(op);
        }
        if sload_count > 2 {
            savings += ((sload_count - 1) / 2) as u64 * COLD_SLOAD_COST;
        }
        savings
    }
}

impl Default for GasAnalysis {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn category_classification() {
        assert_eq!(GasCostCategory::classify(0x01), GasCostCategory::VeryLow);
        assert_eq!(GasCostCategory::classify(0x54), GasCostCategory::High);
        assert_eq!(GasCostCategory::classify(0x55), GasCostCategory::VeryHigh);
    }

    #[test]
    fn efficiency_score() {
        let a = GasAnalysis {
            total_gas: TX_BASE + 9,
            breakdown: vec![(0x01, 3), (0x02, 3), (0x03, 3)],
            warnings: vec![],
            optimizations: vec![],
        };
        assert!(a.efficiency_score() >= 80);
    }
}
