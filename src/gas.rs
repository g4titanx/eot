//! Dynamic gas cost analysis for EVM opcodes
//! 
//! This module provides context-aware gas cost calculation that accounts for:
//! - EIP-2929 warm/cold storage and account access
//! - Memory expansion costs (quadratic pricing)
//! - Complex call operation pricing
//! - Fork-specific gas cost evolution
//! - Storage state changes (EIP-2200)

use std::collections::HashMap;

pub mod context;
pub mod calculator;
pub mod analysis;

pub use context::*;
pub use calculator::*;
pub use analysis::*;

/// Represents different types of gas costs
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GasCostType {
    /// Fixed cost regardless of context
    Static(u64),
    /// Dynamic cost that depends on execution context
    Dynamic {
        /// Base gas cost before variable factors
        base_cost: u64,
        /// Additional factors that can affect the final cost
        variable_factors: Vec<GasVariableFactor>,
    },
    /// Memory expansion cost
    MemoryExpansion {
        /// Base cost for the operation
        base_cost: u64,
        /// Factor multiplied by memory size
        memory_size_factor: u64,
    },
    /// Complex cost with multiple dependencies
    Complex,
}

/// Factors that can affect variable gas costs
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GasVariableFactor {
    /// Cost depends on whether storage slot was previously accessed
    StorageWarmCold { 
        /// Gas cost for warm (previously accessed) storage
        warm_cost: u64, 
        /// Gas cost for cold (first access) storage
        cold_cost: u64 
    },
    /// Cost depends on whether address was previously accessed
    AddressWarmCold { 
        /// Gas cost for warm (previously accessed) address
        warm_cost: u64, 
        /// Gas cost for cold (first access) address
        cold_cost: u64 
    },
    /// Cost depends on memory expansion
    MemoryExpansion,
    /// Cost depends on value being transferred
    ValueTransfer(u64),
    /// Cost depends on account creation
    AccountCreation(u64),
    /// Cost for copying data
    DataCopy { 
        /// Gas cost per 32-byte word copied
        cost_per_word: u64 
    },
}

/// Gas cost categories for optimization analysis
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GasCostCategory {
    /// Very cheap operations (1-3 gas)
    VeryLow,
    /// Low cost operations (3-8 gas)  
    Low,
    /// Medium cost operations (8-100 gas)
    Medium,
    /// High cost operations (100-2600 gas)
    High,
    /// Very high cost operations (2600+ gas)
    VeryHigh,
    /// Unknown/unclassified operations
    Unknown,
}

/// Result of gas analysis for a sequence of opcodes
#[derive(Debug, Clone)]
pub struct GasAnalysisResult {
    /// Total gas consumed including base transaction cost
    pub total_gas: u64,
    /// Gas breakdown by opcode
    pub breakdown: Vec<(u8, u64)>,
    /// Warnings about expensive operations
    pub warnings: Vec<String>,
    /// Final execution context after simulation
    pub context: ExecutionContext,
    /// Detected optimization opportunities
    pub optimizations: Vec<String>,
}

impl GasAnalysisResult {
    /// Get gas efficiency ratio compared to a baseline
    pub fn efficiency_ratio(&self, baseline_gas: u64) -> f64 {
        self.total_gas as f64 / baseline_gas as f64
    }

    /// Check if gas usage is within acceptable bounds
    pub fn is_within_bounds(&self, max_gas: u64) -> bool {
        self.total_gas <= max_gas
    }

    /// Get the most expensive operations
    pub fn top_expensive_operations(&self, n: usize) -> Vec<(u8, u64)> {
        let mut sorted = self.breakdown.clone();
        sorted.sort_by(|a, b| b.1.cmp(&a.1));
        sorted.into_iter().take(n).collect()
    }

    /// Calculate gas efficiency score (0-100, higher is better)
    pub fn efficiency_score(&self) -> u8 {
        let avg_gas_per_opcode = if !self.breakdown.is_empty() {
            self.total_gas / self.breakdown.len() as u64
        } else {
            0
        };

        // Score based on average gas per opcode (lower is better)
        match avg_gas_per_opcode {
            0..=10 => 100,
            11..=50 => 80,
            51..=200 => 60,
            201..=1000 => 40,
            1001..=5000 => 20,
            _ => 0,
        }
    }

    /// Get recommendations for gas optimization
    pub fn get_optimization_recommendations(&self) -> Vec<String> {
        let mut recommendations = self.optimizations.clone();

        // Analyze patterns in the breakdown
        let expensive_opcodes: Vec<_> = self.breakdown
            .iter()
            .filter(|(_, cost)| *cost > 1000)
            .collect();

        if expensive_opcodes.len() > self.breakdown.len() / 4 {
            recommendations.push(
                "High proportion of expensive operations - consider algorithmic optimizations".to_string()
            );
        }

        // Check for repeated expensive operations
        let mut opcode_counts = HashMap::new();
        for (opcode, _) in &self.breakdown {
            *opcode_counts.entry(*opcode).or_insert(0) += 1;
        }

        for (opcode, count) in opcode_counts {
            if count > 5 && matches!(opcode, 0x54 | 0x55 | 0xf1 | 0xf4) {
                recommendations.push(format!(
                    "Opcode 0x{:02x} used {} times - consider batching or caching",
                    opcode, count
                ));
            }
        }

        recommendations
    }

    /// Check if this represents an optimized gas usage pattern
    pub fn is_optimized(&self) -> bool {
        self.efficiency_score() > 70 && self.warnings.is_empty()
    }
}

/// Utility functions for gas cost classification
impl GasCostCategory {
    /// Classify an opcode by its gas cost category
    pub fn classify_opcode(opcode: u8) -> Self {
        match opcode {
            // Very cheap operations (1-3 gas)
            0x01..=0x0b | 0x10..=0x1d | 0x50 | 0x58 | 0x80..=0x9f => Self::VeryLow,
            
            // Low cost operations (3-8 gas)
            0x51..=0x53 | 0x56..=0x57 | 0x5a..=0x5b => Self::Low,
            
            // Medium cost operations (8-100 gas)
            0x20 | 0x30 | 0x32..=0x3a | 0x40..=0x48 => Self::Medium,
            
            // High cost operations (100-2600 gas) - specific opcodes
            0x54 | 0x31 | 0x3b | 0x3c | 0x3d | 0x3e | 0x3f => Self::High,
            
            // Very high cost operations (2600+ gas)
            0x55 | 0xf0..=0xff => Self::VeryHigh,
            
            _ => Self::Unknown,
        }
    }

    /// Get the typical gas range for this category
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gas_cost_category_classification() {
        assert_eq!(GasCostCategory::classify_opcode(0x01), GasCostCategory::VeryLow); // ADD
        assert_eq!(GasCostCategory::classify_opcode(0x54), GasCostCategory::High);    // SLOAD
        assert_eq!(GasCostCategory::classify_opcode(0x55), GasCostCategory::VeryHigh); // SSTORE
    }

    #[test]
    fn test_gas_analysis_result_efficiency_score() {
        let result = GasAnalysisResult {
            total_gas: 21030, // Base + 30 gas
            breakdown: vec![(0x01, 3), (0x02, 3), (0x03, 3)], // Simple operations
            warnings: vec![],
            context: ExecutionContext::default(),
            optimizations: vec![],
        };

        assert!(result.efficiency_score() > 80);
    }

    #[test]
    fn test_top_expensive_operations() {
        let result = GasAnalysisResult {
            total_gas: 50000,
            breakdown: vec![
                (0x54, 2100), // SLOAD
                (0x01, 3),    // ADD
                (0x55, 5000), // SSTORE
                (0x02, 3),    // MUL
            ],
            warnings: vec![],
            context: ExecutionContext::default(),
            optimizations: vec![],
        };

        let top_ops = result.top_expensive_operations(2);
        assert_eq!(top_ops.len(), 2);
        assert_eq!(top_ops[0], (0x55, 5000)); // SSTORE should be most expensive
        assert_eq!(top_ops[1], (0x54, 2100)); // SLOAD should be second
    }
}
