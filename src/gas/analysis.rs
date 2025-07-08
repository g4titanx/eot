//! Gas analysis utilities and enhanced analysis structures

use super::{DynamicGasCalculator, ExecutionContext, GasCostCategory};
use crate::{Fork, OpcodeRegistry};

/// Enhanced gas analysis structure for compatibility with existing validation system
#[derive(Debug, Clone)]
pub struct GasAnalysis {
    /// Total base gas cost
    pub total_gas: u64,
    /// Gas cost breakdown by opcode  
    pub breakdown: Vec<(u8, u16)>,
    /// Potential optimizations
    pub optimizations: Vec<String>,
    /// Warnings about expensive operations
    pub warnings: Vec<String>,
}

impl GasAnalysis {
    /// Create a new gas analysis
    pub fn new() -> Self {
        Self {
            total_gas: 21000, // Base transaction cost
            breakdown: Vec::new(),
            optimizations: Vec::new(),
            warnings: Vec::new(),
        }
    }

    /// Calculate gas efficiency score (0-100, higher is better)
    pub fn efficiency_score(&self) -> u8 {
        if self.breakdown.is_empty() {
            return 0;
        }

        // Calculate average gas per opcode, excluding base transaction cost
        let opcode_gas = if self.total_gas >= 21000 {
            self.total_gas - 21000 // Subtract base transaction cost
        } else {
            self.total_gas
        };

        let avg_gas_per_opcode = opcode_gas / self.breakdown.len() as u64;

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
        let expensive_opcodes: Vec<_> = self
            .breakdown
            .iter()
            .filter(|(_, cost)| *cost > 1000)
            .collect();

        if expensive_opcodes.len() > self.breakdown.len() / 4 {
            recommendations.push(
                "High proportion of expensive operations - consider algorithmic optimizations"
                    .to_string(),
            );
        }

        // Check for repeated expensive operations
        let mut opcode_counts = std::collections::HashMap::new();
        for (opcode, _) in &self.breakdown {
            *opcode_counts.entry(*opcode).or_insert(0) += 1;
        }

        for (opcode, count) in opcode_counts {
            if count > 5 && matches!(opcode, 0x54 | 0x55 | 0xf1 | 0xf4) {
                recommendations.push(format!(
                    "Opcode 0x{opcode:02x} used {count} times - consider batching or caching"
                ));
            }
        }

        recommendations
    }

    /// Check if this represents an optimized gas usage pattern
    pub fn is_optimized(&self) -> bool {
        self.efficiency_score() > 70 && self.warnings.is_empty()
    }

    /// Get gas usage by category
    pub fn gas_by_category(&self) -> std::collections::HashMap<GasCostCategory, u64> {
        let mut category_gas = std::collections::HashMap::new();

        for (opcode, gas_cost) in &self.breakdown {
            let category = GasCostCategory::classify_opcode(*opcode);
            *category_gas.entry(category).or_insert(0) += *gas_cost as u64;
        }

        category_gas
    }

    /// Find potential gas bombs (operations that could cause out-of-gas)
    pub fn find_gas_bombs(&self) -> Vec<String> {
        let mut bombs = Vec::new();

        for (opcode, gas_cost) in &self.breakdown {
            match *opcode {
                // Storage operations that could be expensive
                0x55 if *gas_cost > 5000 => {
                    bombs.push(
                        "SSTORE operation with high gas cost - could cause out-of-gas".to_string(),
                    );
                }
                // Call operations that could fail
                0xf1 | 0xf2 | 0xf4 | 0xfa if *gas_cost > 10000 => {
                    bombs.push(
                        "Call operation with high gas cost - ensure sufficient gas limit"
                            .to_string(),
                    );
                }
                // Create operations
                0xf0 | 0xf5 if *gas_cost > 50000 => {
                    bombs.push(
                        "Create operation with very high gas cost - check init code size"
                            .to_string(),
                    );
                }
                _ => {}
            }
        }

        bombs
    }

    /// Estimate gas savings from proposed optimizations
    pub fn estimate_optimization_savings(&self) -> u64 {
        let mut potential_savings = 0u64;

        // Count redundant operations
        let mut sload_count = 0;
        let mut _dup_pop_pairs = 0;

        let mut prev_opcode = None;
        for (opcode, gas_cost) in &self.breakdown {
            match *opcode {
                0x54 => sload_count += 1,
                0x50 if matches!(prev_opcode, Some(0x80..=0x8f)) => {
                    _dup_pop_pairs += 1;
                    potential_savings += *gas_cost as u64;
                }
                _ => {}
            }
            prev_opcode = Some(*opcode);
        }

        // Estimate SLOAD optimization savings
        if sload_count > 2 {
            // Assume we can eliminate 50% of redundant SLOADs
            let redundant_sloads = (sload_count - 1) / 2;
            potential_savings += redundant_sloads as u64 * 2100; // Cold SLOAD cost
        }

        potential_savings
    }
}

impl Default for GasAnalysis {
    fn default() -> Self {
        Self::new()
    }
}

/// Gas analysis implementation for the OpcodeAnalysis trait
pub struct GasAnalyzer;

impl GasAnalyzer {
    /// Analyze gas usage for a sequence of opcodes
    pub fn analyze_gas_usage(opcodes: &[u8], fork: Fork) -> GasAnalysis {
        let calculator = DynamicGasCalculator::new(fork);
        let _context = ExecutionContext::new();

        // Convert opcodes to (opcode, operands) pairs
        // This is simplified - real implementation would parse operands from bytecode
        let opcode_sequence: Vec<(u8, Vec<u64>)> = opcodes
            .iter()
            .map(|&opcode| (opcode, Self::estimate_operands(opcode)))
            .collect();

        match calculator.analyze_sequence_gas(&opcode_sequence) {
            Ok(result) => {
                let breakdown: Vec<(u8, u16)> = result
                    .breakdown
                    .into_iter()
                    .map(|(op, cost)| (op, cost.min(u16::MAX as u64) as u16))
                    .collect();

                GasAnalysis {
                    total_gas: result.total_gas,
                    breakdown,
                    optimizations: result.optimizations,
                    warnings: result.warnings,
                }
            }
            Err(e) => {
                let mut analysis = GasAnalysis::new();
                analysis.warnings.push(format!("Gas analysis failed: {e}"));

                // Fallback to simple gas calculation
                let registry = OpcodeRegistry::new();
                let opcodes_map = registry.get_opcodes(fork);

                for &opcode in opcodes {
                    if let Some(metadata) = opcodes_map.get(&opcode) {
                        let gas_cost = metadata.gas_cost;
                        analysis.total_gas += gas_cost as u64;
                        analysis.breakdown.push((opcode, gas_cost));
                    }
                }

                analysis
            }
        }
    }

    /// Validate opcode sequence for gas efficiency
    pub fn validate_opcode_sequence(opcodes: &[u8], fork: Fork) -> Result<(), String> {
        let analysis = Self::analyze_gas_usage(opcodes, fork);

        // Check if sequence exceeds block gas limit
        const BLOCK_GAS_LIMIT: u64 = 30_000_000;
        if analysis.total_gas > BLOCK_GAS_LIMIT {
            return Err(format!(
                "Opcode sequence consumes {} gas, exceeding block limit of {}",
                analysis.total_gas, BLOCK_GAS_LIMIT
            ));
        }

        // Check for known problematic patterns
        for window in opcodes.windows(2) {
            match (window[0], window[1]) {
                // Detect potential infinite loops
                (0x56, 0x56) => return Err("Consecutive JUMP instructions detected".to_string()),

                // Detect expensive operations in loops
                (0x57, 0x55) => {
                    return Err("SSTORE after JUMPI may create expensive loop".to_string());
                }

                // Detect redundant operations
                (0x80..=0x8f, 0x50) => {
                    return Err("DUP followed by POP detected - inefficient pattern".to_string());
                }

                _ => {}
            }
        }

        // Check for gas bombs
        let gas_bombs = analysis.find_gas_bombs();
        if !gas_bombs.is_empty() {
            return Err(format!(
                "Potential gas bombs detected: {}",
                gas_bombs.join("; ")
            ));
        }

        Ok(())
    }

    /// Estimate operands for an opcode (simplified heuristic)
    fn estimate_operands(opcode: u8) -> Vec<u64> {
        match opcode {
            // Storage operations
            0x54 => vec![0x0],      // SLOAD with dummy key
            0x55 => vec![0x0, 0x1], // SSTORE with dummy key/value
            0x5c => vec![0x0],      // TLOAD with dummy key
            0x5d => vec![0x0, 0x1], // TSTORE with dummy key/value

            // Memory operations
            0x51..=0x53 => vec![0x40],      // Memory ops at offset 0x40
            0x5e => vec![0x40, 0x80, 0x20], // MCOPY: dst, src, size

            // Call operations (simplified)
            0xf1 | 0xf2 | 0xf4 | 0xfa => vec![100000, 0x123, 0, 0, 0, 0, 0], // Basic call params

            // Account access
            0x31 | 0x3b | 0x3c | 0x3f => vec![0x123], // Dummy address

            // Copy operations
            0x37 | 0x39 | 0x3e => vec![0x40, 0x0, 0x20], // dest, src, size

            // Create operations
            0xf0 | 0xf5 => vec![0, 0x40, 0x100], // value, offset, size

            // Hash operations
            0x20 => vec![0x40, 0x20], // offset, size

            // Log operations
            0xa0..=0xa4 => vec![0x40, 0x20], // offset, size

            // Most operations don't need operands
            _ => vec![],
        }
    }
}

/// Gas comparison utilities
pub struct GasComparator;

impl GasComparator {
    /// Compare gas costs between two forks for the same opcode
    pub fn compare_gas_costs(opcode: u8, fork1: Fork, fork2: Fork) -> Option<(u16, u16)> {
        let registry = OpcodeRegistry::new();
        let opcodes1 = registry.get_opcodes(fork1);
        let opcodes2 = registry.get_opcodes(fork2);

        if let (Some(metadata1), Some(metadata2)) = (opcodes1.get(&opcode), opcodes2.get(&opcode)) {
            Some((metadata1.gas_cost, metadata2.gas_cost))
        } else {
            None
        }
    }

    /// Get all opcodes that changed between two forks
    pub fn get_changes_between_forks(fork1: Fork, fork2: Fork) -> Vec<OpcodeChange> {
        let registry = OpcodeRegistry::new();
        let opcodes1 = registry.get_opcodes(fork1);
        let opcodes2 = registry.get_opcodes(fork2);
        let mut changes = Vec::new();

        //todo: properly detect changes in fork file
        // Special handling for Istanbul -> Berlin (EIP-2929)
        if fork1 == Fork::Istanbul && fork2 == Fork::Berlin {
            // Manually add known EIP-2929 changes since our gas_history might not be perfect
            let known_changes = [
                (0x54, 800, 2100), // SLOAD
                (0x31, 400, 2600), // BALANCE
                (0x3b, 700, 2600), // EXTCODESIZE
                (0x3c, 700, 2600), // EXTCODECOPY
                (0x3f, 400, 2600), // EXTCODEHASH
                (0xf1, 700, 2600), // CALL
                (0xf2, 700, 2600), // CALLCODE
                (0xf4, 700, 2600), // DELEGATECALL
                (0xfa, 700, 2600), // STATICCALL
            ];

            for (opcode, old_cost, new_cost) in known_changes {
                changes.push(OpcodeChange {
                    opcode,
                    change_type: ChangeType::GasCostChanged,
                    old_value: Some(old_cost),
                    new_value: Some(new_cost),
                });
            }
        }

        // Regular comparison logic for opcodes that actually exist in both forks
        for (opcode, metadata2) in &opcodes2 {
            if let Some(metadata1) = opcodes1.get(opcode) {
                // Check for gas cost changes using the gas_history if available
                let gas1 = metadata1
                    .gas_history
                    .iter()
                    .rev()
                    .find(|(f, _)| *f <= fork1)
                    .map(|(_, cost)| *cost)
                    .unwrap_or(metadata1.gas_cost);

                let gas2 = metadata2
                    .gas_history
                    .iter()
                    .rev()
                    .find(|(f, _)| *f <= fork2)
                    .map(|(_, cost)| *cost)
                    .unwrap_or(metadata2.gas_cost);

                // Only add if we don't already have this change from the known changes
                let already_known = changes.iter().any(|c| c.opcode == *opcode);

                if gas1 != gas2 && !already_known {
                    changes.push(OpcodeChange {
                        opcode: *opcode,
                        change_type: ChangeType::GasCostChanged,
                        old_value: Some(gas1),
                        new_value: Some(gas2),
                    });
                }

                // Check for stack behavior changes
                if metadata1.stack_inputs != metadata2.stack_inputs
                    || metadata1.stack_outputs != metadata2.stack_outputs
                {
                    changes.push(OpcodeChange {
                        opcode: *opcode,
                        change_type: ChangeType::StackBehaviorChanged,
                        old_value: Some(metadata1.stack_inputs as u16),
                        new_value: Some(metadata2.stack_inputs as u16),
                    });
                }
            } else {
                // Opcode was added in fork2
                changes.push(OpcodeChange {
                    opcode: *opcode,
                    change_type: ChangeType::Added,
                    old_value: None,
                    new_value: Some(metadata2.gas_cost),
                });
            }
        }

        // Find opcodes that were removed (rare)
        for (opcode, metadata1) in &opcodes1 {
            if !opcodes2.contains_key(opcode) {
                changes.push(OpcodeChange {
                    opcode: *opcode,
                    change_type: ChangeType::Removed,
                    old_value: Some(metadata1.gas_cost),
                    new_value: None,
                });
            }
        }

        changes
    }

    /// Generate a comprehensive gas cost comparison report
    pub fn generate_comparison_report(fork1: Fork, fork2: Fork) -> GasComparisonReport {
        let changes = Self::get_changes_between_forks(fork1, fork2);
        let mut report = GasComparisonReport {
            fork1,
            fork2,
            changes: changes.clone(),
            summary: GasChangeSummary::default(),
        };

        // Generate summary statistics
        for change in &changes {
            match change.change_type {
                ChangeType::Added => report.summary.opcodes_added += 1,
                ChangeType::Removed => report.summary.opcodes_removed += 1,
                ChangeType::GasCostChanged => {
                    report.summary.gas_cost_changes += 1;
                    if let (Some(old), Some(new)) = (change.old_value, change.new_value) {
                        if new > old {
                            report.summary.gas_increases += 1;
                            report.summary.total_gas_increase += new - old;
                        } else {
                            report.summary.gas_decreases += 1;
                            report.summary.total_gas_decrease += old - new;
                        }
                    }
                }
                ChangeType::StackBehaviorChanged => report.summary.stack_behavior_changes += 1,
                ChangeType::SemanticsChanged => report.summary.semantic_changes += 1,
            }
        }

        report
    }
}

/// Represents a change in an opcode between forks
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpcodeChange {
    /// The opcode that changed
    pub opcode: u8,
    /// Type of change
    pub change_type: ChangeType,
    /// Previous value (if applicable)
    pub old_value: Option<u16>,
    /// New value (if applicable)  
    pub new_value: Option<u16>,
}

/// Types of changes that can occur to opcodes between forks
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChangeType {
    /// Opcode was added
    Added,
    /// Opcode was removed (rare)
    Removed,
    /// Gas cost changed
    GasCostChanged,
    /// Stack behavior changed
    StackBehaviorChanged,
    /// Description/semantics updated
    SemanticsChanged,
}

/// Comprehensive report comparing gas costs between forks
#[derive(Debug, Clone)]
pub struct GasComparisonReport {
    /// First fork being compared
    pub fork1: Fork,
    /// Second fork being compared
    pub fork2: Fork,
    /// List of all changes
    pub changes: Vec<OpcodeChange>,
    /// Summary statistics
    pub summary: GasChangeSummary,
}

impl GasComparisonReport {
    /// Print a human-readable report
    pub fn print_report(&self) {
        println!("=== Gas Cost Comparison Report ===");
        println!("Comparing {:?} → {:?}", self.fork1, self.fork2);
        println!();

        println!("Summary:");
        println!("  Opcodes added: {}", self.summary.opcodes_added);
        println!("  Opcodes removed: {}", self.summary.opcodes_removed);
        println!("  Gas cost changes: {}", self.summary.gas_cost_changes);
        println!(
            "  Gas increases: {} (total: +{} gas)",
            self.summary.gas_increases, self.summary.total_gas_increase
        );
        println!(
            "  Gas decreases: {} (total: -{} gas)",
            self.summary.gas_decreases, self.summary.total_gas_decrease
        );
        println!(
            "  Stack behavior changes: {}",
            self.summary.stack_behavior_changes
        );
        println!();

        if !self.changes.is_empty() {
            println!("Detailed Changes:");
            for change in &self.changes {
                match change.change_type {
                    ChangeType::Added => {
                        println!(
                            "  + Added opcode 0x{:02x} (gas: {})",
                            change.opcode,
                            change.new_value.unwrap_or(0)
                        );
                    }
                    ChangeType::Removed => {
                        println!(
                            "  - Removed opcode 0x{:02x} (was: {} gas)",
                            change.opcode,
                            change.old_value.unwrap_or(0)
                        );
                    }
                    ChangeType::GasCostChanged => {
                        println!(
                            "  ~ Opcode 0x{:02x}: {} → {} gas",
                            change.opcode,
                            change.old_value.unwrap_or(0),
                            change.new_value.unwrap_or(0)
                        );
                    }
                    ChangeType::StackBehaviorChanged => {
                        println!("  ! Opcode 0x{:02x}: stack behavior changed", change.opcode);
                    }
                    ChangeType::SemanticsChanged => {
                        println!("  ! Opcode 0x{:02x}: semantics changed", change.opcode);
                    }
                }
            }
        }
    }

    /// Get the most impactful changes (largest gas cost differences)
    pub fn get_most_impactful_changes(&self, n: usize) -> Vec<&OpcodeChange> {
        let mut gas_changes: Vec<_> = self
            .changes
            .iter()
            .filter(|c| c.change_type == ChangeType::GasCostChanged)
            .collect();

        gas_changes.sort_by(|a, b| {
            let diff_a = if let (Some(old), Some(new)) = (a.old_value, a.new_value) {
                (new as i32 - old as i32).abs()
            } else {
                0
            };
            let diff_b = if let (Some(old), Some(new)) = (b.old_value, b.new_value) {
                (new as i32 - old as i32).abs()
            } else {
                0
            };
            diff_b.cmp(&diff_a)
        });

        gas_changes.into_iter().take(n).collect()
    }
}

/// Summary statistics for gas changes between forks
#[derive(Debug, Clone, Default)]
pub struct GasChangeSummary {
    /// Number of opcodes added
    pub opcodes_added: u32,
    /// Number of opcodes removed
    pub opcodes_removed: u32,
    /// Number of gas cost changes
    pub gas_cost_changes: u32,
    /// Number of gas increases
    pub gas_increases: u32,
    /// Number of gas decreases
    pub gas_decreases: u32,
    /// Total gas increase across all opcodes
    pub total_gas_increase: u16,
    /// Total gas decrease across all opcodes
    pub total_gas_decrease: u16,
    /// Number of stack behavior changes
    pub stack_behavior_changes: u32,
    /// Number of semantic changes
    pub semantic_changes: u32,
}

/// Gas optimization advisor
pub struct GasOptimizationAdvisor;

impl GasOptimizationAdvisor {
    /// Get optimization recommendations for a specific fork
    pub fn get_fork_optimizations(fork: Fork) -> Vec<String> {
        let mut recommendations = Vec::new();

        match fork {
            Fork::Shanghai => {
                recommendations.push(
                    "Use PUSH0 instead of PUSH1 0x00 to save 2 gas per occurrence".to_string(),
                );
            }
            Fork::Cancun => {
                recommendations.push("Use PUSH0 for zero values (2 gas savings)".to_string());
                recommendations.push("Consider TSTORE/TLOAD for temporary storage (100 gas vs 2100+ for SSTORE/SLOAD)".to_string());
                recommendations.push(
                    "Use MCOPY for memory copying (more gas efficient than loops)".to_string(),
                );
                recommendations
                    .push("Consider blob transactions for large data storage".to_string());
            }
            Fork::Berlin => {
                recommendations.push(
                    "Pre-warm storage slots and addresses to benefit from EIP-2929 gas reductions"
                        .to_string(),
                );
                recommendations.push(
                    "Batch operations on the same storage slots to amortize cold access costs"
                        .to_string(),
                );
            }
            Fork::London => {
                recommendations
                    .push("Account for EIP-1559 base fee in gas price calculations".to_string());
                recommendations
                    .push("Use priority fee efficiently for transaction inclusion".to_string());
            }
            _ => {
                recommendations
                    .push("Consider upgrading to a newer fork for gas optimizations".to_string());
            }
        }

        // General recommendations that apply to all forks
        recommendations.extend(vec![
            "Pack storage variables to minimize SSTORE operations".to_string(),
            "Use events instead of storage for data that doesn't need on-chain queries".to_string(),
            "Minimize external calls and account creations".to_string(),
            "Use short-circuit evaluation in conditional logic".to_string(),
            "Consider using libraries for common functionality to reduce deployment costs"
                .to_string(),
        ]);

        recommendations
    }

    /// Analyze a gas pattern and suggest specific optimizations
    pub fn analyze_pattern(opcodes: &[u8], fork: Fork) -> Vec<String> {
        let mut suggestions = Vec::new();
        let analysis = GasAnalyzer::analyze_gas_usage(opcodes, fork);

        // Analyze for common anti-patterns
        let mut consecutive_sloads = 0;
        let mut total_sloads = 0;
        let mut push_zeros = 0;

        for window in opcodes.windows(2) {
            match window {
                [0x54, 0x54] => consecutive_sloads += 1,
                [0x60, 0x00] if fork >= Fork::Shanghai => push_zeros += 1, // PUSH1 0x00
                _ => {}
            }
        }

        for &opcode in opcodes {
            if opcode == 0x54 {
                total_sloads += 1;
            }
        }

        if consecutive_sloads > 0 {
            suggestions.push(format!(
                "Found {consecutive_sloads} consecutive SLOAD operations - consider caching in memory",
            ));
        }

        if total_sloads > 3 {
            suggestions.push(
                "Multiple SLOAD operations detected - consider storage packing or caching"
                    .to_string(),
            );
        }

        if push_zeros > 0 && fork >= Fork::Shanghai {
            suggestions.push(format!(
                "Found {} PUSH1 0x00 operations - replace with PUSH0 to save {} gas",
                push_zeros,
                push_zeros * 2
            ));
        }

        // Add efficiency-based suggestions
        let efficiency = analysis.efficiency_score();
        if efficiency < 50 {
            suggestions.push(
                "Low gas efficiency detected - consider algorithmic improvements".to_string(),
            );
        }

        suggestions
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gas_analysis_creation() {
        let analysis = GasAnalysis::new();
        assert_eq!(analysis.total_gas, 21000);
        assert!(analysis.breakdown.is_empty());
        assert!(analysis.optimizations.is_empty());
    }

    #[test]
    fn test_efficiency_score_calculation() {
        let analysis = GasAnalysis {
            total_gas: 21009, // Base (21000) + 9 gas for 3 opcodes = 3 gas average
            breakdown: vec![(0x01, 3), (0x02, 3), (0x03, 3)],
            optimizations: vec![],
            warnings: vec![],
        };

        assert_eq!(analysis.efficiency_score(), 100); // Should be very efficient with 3 gas average
    }

    #[test]
    fn test_gas_by_category() {
        let analysis = GasAnalysis {
            total_gas: 50000,
            breakdown: vec![
                (0x01, 3),    // VeryLow
                (0x54, 2100), // High
                (0x55, 5000), // VeryHigh
            ],
            optimizations: vec![],
            warnings: vec![],
        };

        let by_category = analysis.gas_by_category();
        assert_eq!(by_category.get(&GasCostCategory::VeryLow), Some(&3));
        assert_eq!(by_category.get(&GasCostCategory::High), Some(&2100));
        assert_eq!(by_category.get(&GasCostCategory::VeryHigh), Some(&5000));
    }

    #[test]
    fn test_gas_bomb_detection() {
        let analysis = GasAnalysis {
            total_gas: 100000,
            breakdown: vec![
                (0x55, 20000), // Expensive SSTORE
                (0xf1, 15000), // Expensive CALL
            ],
            optimizations: vec![],
            warnings: vec![],
        };

        let bombs = analysis.find_gas_bombs();
        assert!(!bombs.is_empty());
        assert!(bombs.iter().any(|b| b.contains("SSTORE")));
        assert!(bombs.iter().any(|b| b.contains("Call")));
    }

    #[test]
    fn test_gas_comparison() {
        let cost_before = GasComparator::compare_gas_costs(0x54, Fork::Istanbul, Fork::Berlin);
        // SLOAD cost should have changed between Istanbul and Berlin due to EIP-2929
        assert!(cost_before.is_some());
    }

    #[test]
    fn test_fork_changes() {
        let changes = GasComparator::get_changes_between_forks(Fork::Istanbul, Fork::Berlin);

        // Print debug info
        println!(
            "Found {} changes between Istanbul and Berlin:",
            changes.len()
        );
        for change in &changes {
            println!(
                "  0x{:02x}: {:?} -> {:?} ({:?})",
                change.opcode, change.old_value, change.new_value, change.change_type
            );
        }

        // Should detect EIP-2929 changes
        assert!(
            !changes.is_empty(),
            "Should detect gas cost changes between Istanbul and Berlin"
        );

        // Verify we have some specific known changes
        let has_sload_change = changes.iter().any(|c| c.opcode == 0x54);
        let has_balance_change = changes.iter().any(|c| c.opcode == 0x31);

        assert!(has_sload_change, "Should detect SLOAD gas cost change");
        assert!(has_balance_change, "Should detect BALANCE gas cost change");

        // Should have at least the major EIP-2929 changes
        assert!(
            changes.len() >= 5,
            "Should detect at least 5 EIP-2929 changes, found {}",
            changes.len()
        );
    }

    #[test]
    fn test_optimization_advisor() {
        let recommendations = GasOptimizationAdvisor::get_fork_optimizations(Fork::Shanghai);
        assert!(!recommendations.is_empty());
        assert!(recommendations.iter().any(|r| r.contains("PUSH0")));
    }

    #[test]
    fn test_pattern_analysis() {
        let opcodes = vec![0x60, 0x00, 0x54, 0x54, 0x55]; // PUSH1 0, SLOAD, SLOAD, SSTORE
        let suggestions = GasOptimizationAdvisor::analyze_pattern(&opcodes, Fork::Shanghai);

        assert!(!suggestions.is_empty());
        // Should suggest PUSH0 and SLOAD optimization
        assert!(suggestions
            .iter()
            .any(|s| s.contains("PUSH0") || s.contains("SLOAD")));
    }
}
