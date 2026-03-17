//! Gas analysis utilities: static analysis, fork comparison, and optimization advice.

use super::{DynamicGasCalculator, GasAnalysis};
use crate::{Fork, OpCode};

// ---------------------------------------------------------------------------
// GasAnalyzer — static sequence analysis
// ---------------------------------------------------------------------------

/// Analyses gas usage for opcode sequences without full execution context.
pub struct GasAnalyzer;

impl GasAnalyzer {
    /// Analyse gas usage for a raw opcode byte sequence.
    ///
    /// Operands are estimated heuristically; for precise analysis, use
    /// [`DynamicGasCalculator::analyze_sequence_gas`] with explicit operands.
    pub fn analyze_gas_usage(opcodes: &[u8], fork: Fork) -> GasAnalysis {
        let calc = DynamicGasCalculator::new(fork);
        let pairs: Vec<(u8, Vec<u64>)> = opcodes
            .iter()
            .map(|&op| (op, Self::estimate_operands(op)))
            .collect();

        match calc.analyze_sequence_gas(&pairs) {
            Ok(a) => a,
            Err(e) => {
                let mut analysis = GasAnalysis::new();
                analysis.warnings.push(format!("Gas analysis failed: {e}"));
                for &op in opcodes {
                    let cost = OpCode::from_byte(op).gas_cost(fork);
                    analysis.total_gas += cost as u64;
                    analysis.breakdown.push((op, cost));
                }
                analysis
            }
        }
    }

    /// Validate that an opcode sequence doesn't have obvious gas issues.
    pub fn validate_opcode_sequence(opcodes: &[u8], fork: Fork) -> Result<(), String> {
        let analysis = Self::analyze_gas_usage(opcodes, fork);
        const BLOCK_LIMIT: u64 = 30_000_000;
        if analysis.total_gas > BLOCK_LIMIT {
            return Err(format!(
                "Sequence uses {} gas, exceeding block limit {BLOCK_LIMIT}",
                analysis.total_gas
            ));
        }
        for window in opcodes.windows(2) {
            match (window[0], window[1]) {
                (0x56, 0x56) => return Err("Consecutive JUMP instructions".into()),
                (0x57, 0x55) => return Err("SSTORE after JUMPI — potential expensive loop".into()),
                (0x80..=0x8f, 0x50) => return Err("DUP followed by POP — redundant".into()),
                _ => {}
            }
        }
        let bombs = analysis.find_gas_bombs();
        if !bombs.is_empty() {
            return Err(format!("Gas bombs: {}", bombs.join("; ")));
        }
        Ok(())
    }

    /// Heuristic operand estimation for static analysis.
    fn estimate_operands(opcode: u8) -> Vec<u64> {
        match opcode {
            0x54 => vec![0x0],
            0x55 => vec![0x0, 0x1],
            0x5c => vec![0x0],
            0x5d => vec![0x0, 0x1],
            0x51..=0x53 => vec![0x40],
            0x5e => vec![0x40, 0x80, 0x20],
            0xf1 | 0xf2 | 0xf4 | 0xfa => vec![100_000, 0x123, 0, 0, 0, 0, 0],
            0x31 | 0x3b | 0x3c | 0x3f => vec![0x123],
            0x37 | 0x39 | 0x3e => vec![0x40, 0x0, 0x20],
            0xf0 | 0xf5 => vec![0, 0x40, 0x100],
            0x20 => vec![0x40, 0x20],
            0xa0..=0xa4 => vec![0x40, 0x20],
            _ => vec![],
        }
    }
}

// ---------------------------------------------------------------------------
// GasComparator — fork comparison
// ---------------------------------------------------------------------------

/// Compare gas costs between forks.
pub struct GasComparator;

/// A change in an opcode between two forks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpcodeChange {
    /// The opcode byte.
    pub opcode: u8,
    /// Kind of change.
    pub change_type: ChangeType,
    /// Previous value (gas cost), if any.
    pub old_value: Option<u16>,
    /// New value (gas cost), if any.
    pub new_value: Option<u16>,
}

/// The kind of change between forks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChangeType {
    /// Opcode was added.
    Added,
    /// Gas cost changed.
    GasCostChanged,
}

impl GasComparator {
    /// Compare the static gas cost of a single opcode between two forks.
    pub fn compare_gas_costs(opcode: u8, fork1: Fork, fork2: Fork) -> Option<(u16, u16)> {
        let op = OpCode::new(opcode)?;
        Some((op.gas_cost(fork1), op.gas_cost(fork2)))
    }

    /// List all opcode changes between two forks.
    pub fn get_changes_between_forks(fork1: Fork, fork2: Fork) -> Vec<OpcodeChange> {
        let mut changes = Vec::new();
        for op in OpCode::iter_all() {
            let in1 = op.is_valid_in(fork1);
            let in2 = op.is_valid_in(fork2);
            if !in1 && in2 {
                changes.push(OpcodeChange {
                    opcode: op.byte(),
                    change_type: ChangeType::Added,
                    old_value: None,
                    new_value: Some(op.gas_cost(fork2)),
                });
            } else if in1 && in2 {
                let g1 = op.gas_cost(fork1);
                let g2 = op.gas_cost(fork2);
                if g1 != g2 {
                    changes.push(OpcodeChange {
                        opcode: op.byte(),
                        change_type: ChangeType::GasCostChanged,
                        old_value: Some(g1),
                        new_value: Some(g2),
                    });
                }
            }
        }
        changes
    }

    /// Generate a printable comparison report.
    pub fn generate_comparison_report(fork1: Fork, fork2: Fork) -> GasComparisonReport {
        let changes = Self::get_changes_between_forks(fork1, fork2);
        let mut summary = GasChangeSummary::default();
        for c in &changes {
            match c.change_type {
                ChangeType::Added => summary.opcodes_added += 1,
                ChangeType::GasCostChanged => {
                    summary.gas_cost_changes += 1;
                    if let (Some(old), Some(new)) = (c.old_value, c.new_value) {
                        if new > old {
                            summary.gas_increases += 1;
                            summary.total_gas_increase += (new - old) as u32;
                        } else {
                            summary.gas_decreases += 1;
                            summary.total_gas_decrease += (old - new) as u32;
                        }
                    }
                }
            }
        }
        GasComparisonReport {
            fork1,
            fork2,
            changes,
            summary,
        }
    }
}

/// Full comparison report between two forks.
#[derive(Debug, Clone)]
pub struct GasComparisonReport {
    /// First fork.
    pub fork1: Fork,
    /// Second fork.
    pub fork2: Fork,
    /// Individual changes.
    pub changes: Vec<OpcodeChange>,
    /// Summary statistics.
    pub summary: GasChangeSummary,
}

impl GasComparisonReport {
    /// Print the report to stdout.
    pub fn print_report(&self) {
        println!(
            "=== Gas Comparison: {:?} -> {:?} ===",
            self.fork1, self.fork2
        );
        println!("  Added: {}", self.summary.opcodes_added);
        println!("  Gas changes: {}", self.summary.gas_cost_changes);
        println!(
            "  Increases: {} (+{} total)",
            self.summary.gas_increases, self.summary.total_gas_increase
        );
        println!(
            "  Decreases: {} (-{} total)",
            self.summary.gas_decreases, self.summary.total_gas_decrease
        );
        for c in &self.changes {
            match c.change_type {
                ChangeType::Added => {
                    let name = OpCode::from_byte(c.opcode).name();
                    println!(
                        "  + {name} (0x{:02x}): {} gas",
                        c.opcode,
                        c.new_value.unwrap_or(0)
                    );
                }
                ChangeType::GasCostChanged => {
                    let name = OpCode::from_byte(c.opcode).name();
                    println!(
                        "  ~ {name} (0x{:02x}): {} -> {}",
                        c.opcode,
                        c.old_value.unwrap_or(0),
                        c.new_value.unwrap_or(0)
                    );
                }
            }
        }
    }

    /// Top N most impactful gas changes.
    pub fn get_most_impactful_changes(&self, n: usize) -> Vec<&OpcodeChange> {
        let mut gas_changes: Vec<_> = self
            .changes
            .iter()
            .filter(|c| c.change_type == ChangeType::GasCostChanged)
            .collect();
        gas_changes.sort_by(|a, b| {
            let da = a
                .old_value
                .zip(a.new_value)
                .map(|(o, n)| (n as i32 - o as i32).unsigned_abs())
                .unwrap_or(0);
            let db = b
                .old_value
                .zip(b.new_value)
                .map(|(o, n)| (n as i32 - o as i32).unsigned_abs())
                .unwrap_or(0);
            db.cmp(&da)
        });
        gas_changes.into_iter().take(n).collect()
    }
}

/// Summary statistics for a fork comparison.
#[derive(Debug, Clone, Default)]
pub struct GasChangeSummary {
    /// Opcodes added.
    pub opcodes_added: u32,
    /// Gas cost changes.
    pub gas_cost_changes: u32,
    /// Number of increases.
    pub gas_increases: u32,
    /// Number of decreases.
    pub gas_decreases: u32,
    /// Total gas increase across all opcodes.
    pub total_gas_increase: u32,
    /// Total gas decrease across all opcodes.
    pub total_gas_decrease: u32,
}

// ---------------------------------------------------------------------------
// GasOptimizationAdvisor
// ---------------------------------------------------------------------------

/// Provides fork-specific optimization recommendations.
pub struct GasOptimizationAdvisor;

impl GasOptimizationAdvisor {
    /// Fork-specific optimization tips.
    pub fn get_fork_optimizations(fork: Fork) -> Vec<String> {
        let mut recs = Vec::new();
        if fork >= Fork::Shanghai {
            recs.push("Use PUSH0 instead of PUSH1 0x00 (saves 1 gas per use)".into());
        }
        if fork >= Fork::Cancun {
            recs.push(
                "Use TSTORE/TLOAD for temporary storage (100 gas vs 2100+ for SSTORE/SLOAD)".into(),
            );
            recs.push("Use MCOPY for memory copying (more efficient than manual loops)".into());
        }
        if fork >= Fork::Berlin {
            recs.push("Pre-warm storage slots and addresses to reduce cold access costs".into());
        }
        recs.push("Pack storage variables to minimise SSTORE operations".into());
        recs.push("Minimise external calls — they are expensive and can fail".into());
        recs
    }

    /// Analyse a raw opcode pattern and suggest improvements.
    pub fn analyze_pattern(opcodes: &[u8], fork: Fork) -> Vec<String> {
        let mut suggestions = Vec::new();
        let analysis = GasAnalyzer::analyze_gas_usage(opcodes, fork);

        let mut consecutive_sloads = 0u32;
        let mut push_zeros = 0u32;
        for window in opcodes.windows(2) {
            if window == [0x54, 0x54] {
                consecutive_sloads += 1;
            }
            if window == [0x60, 0x00] && fork >= Fork::Shanghai {
                push_zeros += 1;
            }
        }
        if consecutive_sloads > 0 {
            suggestions.push(format!(
                "{consecutive_sloads} consecutive SLOADs — consider caching in memory"
            ));
        }
        if push_zeros > 0 {
            suggestions.push(format!(
                "{push_zeros} PUSH1 0x00 sequences — replace with PUSH0 (saves {push_zeros} gas)"
            ));
        }
        if analysis.efficiency_score() < 50 {
            suggestions.push("Low gas efficiency — consider algorithmic improvements".into());
        }
        suggestions
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn analyze_simple() {
        let a = GasAnalyzer::analyze_gas_usage(&[0x01, 0x02, 0x03], Fork::London);
        assert!(a.total_gas >= crate::gas::TX_BASE);
        assert_eq!(a.breakdown.len(), 3);
    }

    #[test]
    fn fork_comparison() {
        let changes = GasComparator::get_changes_between_forks(Fork::Istanbul, Fork::Berlin);
        assert!(!changes.is_empty());
        assert!(changes.iter().any(|c| c.opcode == 0x54)); // SLOAD
        assert!(changes.iter().any(|c| c.opcode == 0x31)); // BALANCE
    }

    #[test]
    fn optimization_advisor() {
        let recs = GasOptimizationAdvisor::get_fork_optimizations(Fork::Shanghai);
        assert!(recs.iter().any(|r| r.contains("PUSH0")));
    }

    #[test]
    fn pattern_analysis() {
        let ops = vec![0x60, 0x00, 0x54, 0x54, 0x55];
        let suggestions = GasOptimizationAdvisor::analyze_pattern(&ops, Fork::Shanghai);
        assert!(!suggestions.is_empty());
    }
}
