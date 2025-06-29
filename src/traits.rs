//! Core traits for EVM opcode table system

use crate::Fork;

/// Extended trait for opcodes with additional utilities
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

/// Utility trait for opcode analysis
pub trait OpcodeAnalysis {
    /// Analyze gas usage patterns for a sequence of opcodes
    fn analyze_gas_usage(opcodes: &[u8], fork: Fork) -> GasAnalysis;

    /// Check if a sequence of opcodes is valid for a given fork
    fn validate_opcode_sequence(opcodes: &[u8], fork: Fork) -> Result<(), String>;
}

/// Gas analysis result
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
