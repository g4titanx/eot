//! Core traits for EVM opcode table system with gas analysis integration

use crate::{Fork, gas::{ExecutionContext, GasCostCategory, DynamicGasCalculator, GasAnalysis}};

/// Extended trait for opcodes with additional utilities including gas analysis
pub trait OpcodeExt: crate::OpCode {
    /// Check if this opcode modifies state
    fn modifies_state(&self) -> bool {
        matches!(
            (*self).into(),
            0x55 | // SSTORE
            0x5d | // TSTORE  
            0xf0 | // CREATE
            0xf1 | // CALL
            0xf2 | // CALLCODE
            0xf4 | // DELEGATECALL
            0xf5 | // CREATE2
            0xff // SELFDESTRUCT
        ) && (*self).into() != 0xfa // STATICCALL is explicitly non-state-modifying
    }

    /// Check if this opcode can cause revert
    fn can_revert(&self) -> bool {
        matches!(
            (*self).into(),
            0xf1 | // CALL
            0xf2 | // CALLCODE
            0xf4 | // DELEGATECALL
            0xf5 | // CREATE2
            0xfa | // STATICCALL
            0xfd // REVERT
        )
    }

    /// Check if this is a push opcode
    fn is_push(&self) -> bool {
        let opcode = (*self).into();
        (0x5f..=0x7f).contains(&opcode) // PUSH0 through PUSH32
    }

    /// Get push data size (0 for PUSH0, 1 for PUSH1, etc.)
    fn push_size(&self) -> Option<u8> {
        let opcode = (*self).into();
        match opcode {
            0x5f => Some(0),                    // PUSH0
            0x60..=0x7f => Some(opcode - 0x5f), // PUSH1-PUSH32
            _ => None,
        }
    }

    /// Check if this is a dup opcode
    fn is_dup(&self) -> bool {
        let opcode = (*self).into();
        (0x80..=0x8f).contains(&opcode)
    }

    /// Check if this is a swap opcode
    fn is_swap(&self) -> bool {
        let opcode = (*self).into();
        (0x90..=0x9f).contains(&opcode)
    }

    /// Get the minimum required stack depth for this opcode
    fn min_stack_depth(&self) -> u8 {
        let metadata = self.metadata();
        if self.is_dup() {
            // DUP1 needs 1 item, DUP2 needs 2 items, etc.
            let opcode = (*self).into();
            opcode - 0x7f // DUP1=0x80, so 0x80-0x7f=1
        } else if self.is_swap() {
            // SWAP1 needs 2 items, SWAP2 needs 3 items, etc.
            let opcode = (*self).into();
            opcode - 0x8e // SWAP1=0x90, so 0x90-0x8e=2
        } else {
            metadata.stack_inputs
        }
    }

    /// Calculate dynamic gas cost for this opcode
    fn calculate_gas_cost(&self, context: &ExecutionContext, operands: &[u64]) -> Result<u64, String> {
        let calculator = DynamicGasCalculator::new(Self::fork());
        calculator.calculate_gas_cost((*self).into(), context, operands)
    }

    /// Get gas cost category for optimization analysis
    fn gas_cost_category(&self) -> GasCostCategory {
        GasCostCategory::classify_opcode((*self).into())
    }

    /// Check if this opcode's gas cost varies with context
    fn has_dynamic_gas_cost(&self) -> bool {
        matches!(
            (*self).into(),
            // Storage operations (warm/cold)
            0x54 | 0x55 |
            // Transient storage
            0x5c | 0x5d |
            // Account access operations
            0x31 | 0x3b | 0x3c | 0x3f |
            // Memory operations (expansion)
            0x51 | 0x52 | 0x53 | 0x5e |
            // Call operations (complex pricing)
            0xf0..=0xff |
            // Copy operations (size dependent)
            0x37 | 0x39 | 0x3e |
            // Hash operations (size dependent)
            0x20 |
            // Log operations (size dependent)
            0xa0..=0xa4
        )
    }

    /// Check if this is a control flow opcode
    fn is_control_flow(&self) -> bool {
        matches!(
            (*self).into(),
            0x00 | // STOP
            0x56 | // JUMP
            0x57 | // JUMPI
            0x5b | // JUMPDEST
            0xf3 | // RETURN
            0xfd | // REVERT
            0xfe | // INVALID
            0xff   // SELFDESTRUCT
        )
    }

    /// Check if this opcode affects memory
    fn affects_memory(&self) -> bool {
        matches!(
            (*self).into(),
            0x51..=0x53 | // MLOAD, MSTORE, MSTORE8
            0x5e |        // MCOPY
            0x37 | 0x39 | 0x3e | // Copy operations
            0x20 |        // KECCAK256
            0xa0..=0xa4 | // LOG operations
            0xf0..=0xff   // Call/Create operations
        )
    }

    /// Check if this opcode affects storage
    fn affects_storage(&self) -> bool {
        matches!(
            (*self).into(),
            0x54 | 0x55 | // SLOAD, SSTORE
            0x5c | 0x5d   // TLOAD, TSTORE
        )
    }

    /// Get estimated gas cost for this opcode (simplified, without context)
    fn estimated_gas_cost(&self) -> u16 {
        self.gas_cost()
    }

    /// Check if this opcode is deprecated or discouraged
    fn is_deprecated(&self) -> bool {
        matches!(
            (*self).into(),
            0xf2 | // CALLCODE (use DELEGATECALL instead)
            0xff   // SELFDESTRUCT (being phased out)
        )
    }

    /// Get optimization recommendations for this opcode
    fn optimization_recommendations(&self) -> Vec<String> {
        let mut recommendations = Vec::new();
        let opcode = (*self).into();

        match opcode {
            0x60 if Self::fork() >= Fork::Shanghai => {
                // PUSH1 0x00 can be replaced with PUSH0
                recommendations.push("Consider using PUSH0 instead of PUSH1 0x00 to save gas".to_string());
            }
            0x54 => {
                recommendations.push("Consider caching SLOAD results in memory if used multiple times".to_string());
                if Self::fork() >= Fork::Berlin {
                    recommendations.push("Pre-warm storage slots when possible to benefit from lower gas costs".to_string());
                }
            }
            0x55 => {
                recommendations.push("Consider using TSTORE for temporary values that don't need persistence".to_string());
                recommendations.push("Pack storage variables to minimize SSTORE operations".to_string());
            }
            0xf1 | 0xf2 | 0xf4 | 0xfa => {
                recommendations.push("Minimize external calls as they are expensive and can fail".to_string());
                if Self::fork() >= Fork::Berlin {
                    recommendations.push("Pre-warm target addresses when possible".to_string());
                }
            }
            0xf0 | 0xf5 => {
                recommendations.push("Consider using CREATE2 for deterministic addresses".to_string());
                if Self::fork() >= Fork::Shanghai {
                    recommendations.push("Be aware of initcode size limits (EIP-3860)".to_string());
                }
            }
            _ => {}
        }

        recommendations
    }
}

/// Automatic implementation for all OpCode types
impl<T: crate::OpCode> OpcodeExt for T {}

/// Trait for comparing opcodes between forks
pub trait OpcodeComparison {
    /// Compare gas costs between two forks for the same opcode
    fn compare_gas_costs(opcode: u8, fork1: Fork, fork2: Fork) -> Option<(u16, u16)>;

    /// Get all opcodes that changed between two forks
    fn get_changes_between_forks(fork1: Fork, fork2: Fork) -> Vec<OpcodeChange>;
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

/// Trait for fork-specific validation rules
pub trait ForkValidation {
    /// Validate that all opcodes in this fork are consistent
    fn validate_fork_consistency(fork: Fork) -> Result<(), Vec<String>>;

    /// Check for any known issues or edge cases in this fork
    fn check_known_issues(fork: Fork) -> Vec<String>;
}

/// Enhanced trait for opcode analysis with gas considerations
pub trait OpcodeAnalysis {
    /// Analyze gas usage patterns for a sequence of opcodes
    fn analyze_gas_usage(opcodes: &[u8], fork: Fork) -> GasAnalysis;

    /// Check if a sequence of opcodes is valid for a given fork
    fn validate_opcode_sequence(opcodes: &[u8], fork: Fork) -> Result<(), String>;

    /// Get optimization suggestions for a sequence of opcodes
    fn get_optimization_suggestions(opcodes: &[u8], fork: Fork) -> Vec<String> {
        let analysis = Self::analyze_gas_usage(opcodes, fork);
        analysis.get_optimization_recommendations()
    }

    /// Estimate gas savings from proposed optimizations
    fn estimate_gas_savings(opcodes: &[u8], fork: Fork) -> u64 {
        let analysis = Self::analyze_gas_usage(opcodes, fork);
        analysis.estimate_optimization_savings()
    }
}

/// Implementation of the OpcodeComparison trait using the gas analysis system
impl OpcodeComparison for crate::OpcodeRegistry {
    fn compare_gas_costs(opcode: u8, fork1: Fork, fork2: Fork) -> Option<(u16, u16)> {
        use crate::gas::GasComparator;
        GasComparator::compare_gas_costs(opcode, fork1, fork2)
    }

    fn get_changes_between_forks(fork1: Fork, fork2: Fork) -> Vec<OpcodeChange> {
        use crate::gas::{GasComparator, analysis::{ChangeType as GasChangeType}};
        
        let gas_changes = GasComparator::get_changes_between_forks(fork1, fork2);
        gas_changes.into_iter().map(|gc| OpcodeChange {
            opcode: gc.opcode,
            change_type: match gc.change_type {
                GasChangeType::Added => ChangeType::Added,
                GasChangeType::Removed => ChangeType::Removed,
                GasChangeType::GasCostChanged => ChangeType::GasCostChanged,
                GasChangeType::StackBehaviorChanged => ChangeType::StackBehaviorChanged,
                GasChangeType::SemanticsChanged => ChangeType::SemanticsChanged,
            },
            old_value: gc.old_value,
            new_value: gc.new_value,
        }).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::forks::*;

    #[test]
    fn test_gas_cost_category_classification() {
        let add_opcode = Frontier::ADD;
        assert_eq!(add_opcode.gas_cost_category(), GasCostCategory::VeryLow);

        let sload_opcode = Berlin::SLOAD;
        assert_eq!(sload_opcode.gas_cost_category(), GasCostCategory::High);
    }

    #[test]
    fn test_dynamic_gas_detection() {
        let add_opcode = Frontier::ADD;
        assert!(!add_opcode.has_dynamic_gas_cost());

        let sload_opcode = Berlin::SLOAD;
        assert!(sload_opcode.has_dynamic_gas_cost());
    }

    #[test]
    fn test_control_flow_detection() {
        let add_opcode = Frontier::ADD;
        assert!(!add_opcode.is_control_flow());

        let jump_opcode = Frontier::JUMP;
        assert!(jump_opcode.is_control_flow());
    }

    #[test]
    fn test_memory_affects_detection() {
        let add_opcode = Frontier::ADD;
        assert!(!add_opcode.affects_memory());

        let mload_opcode = Frontier::MLOAD;
        assert!(mload_opcode.affects_memory());
    }

    #[test]
    fn test_storage_affects_detection() {
        let add_opcode = Frontier::ADD;
        assert!(!add_opcode.affects_storage());

        let sload_opcode = Frontier::SLOAD;
        assert!(sload_opcode.affects_storage());
    }

    #[test]
    fn test_optimization_recommendations() {
        let sload_opcode = Berlin::SLOAD;
        let recommendations = sload_opcode.optimization_recommendations();
        
        assert!(!recommendations.is_empty());
        assert!(recommendations.iter().any(|r| r.contains("caching")));
    }
}
