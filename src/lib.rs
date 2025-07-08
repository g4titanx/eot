//! # EOT - EVM Opcode Table
//!
//! A Rust implementation of EVM opcodes for all Ethereum forks,
//! with complete fork inheritance, validation, and metadata.
//!
//! ## Features
//!
//! - **Complete Fork Support**: All Ethereum forks from Frontier to Cancun
//! - **Dynamic Gas Analysis**: Context-aware gas cost calculation with EIP support
//! - **Optimization Analysis**: Automatic detection of gas inefficiencies
//! - **Historical Tracking**: Gas cost evolution across forks
//! - **Comprehensive Validation**: Ensures opcode consistency and accuracy
//!
//! ## Quick Start
//!
//! ```rust
//! use eot::*;
//! use eot::gas::*;
//!
//! // Basic opcode usage
//! let registry = OpcodeRegistry::new();
//! let opcodes = registry.get_opcodes(Fork::London);
//!
//! // Dynamic gas analysis
//! let calculator = DynamicGasCalculator::new(Fork::London);
//! let context = ExecutionContext::new();
//! let gas_cost = calculator.calculate_gas_cost(0x54, &context, &[0x123])?;
//!
//! // Analyze gas usage for a sequence of opcodes
//! let opcodes = vec![0x60, 0x01, 0x60, 0x02, 0x01]; // PUSH1 1, PUSH1 2, ADD
//! let analysis = calculator.analyze_sequence_gas(&opcodes.into_iter().map(|op| (op, vec![])).collect())?;
//! println!("Total gas: {}", analysis.total_gas);
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

#![deny(missing_docs)]
#![warn(clippy::all)]

use std::collections::HashMap;

pub mod forks;
pub use forks::*;

// Core traits and types
pub mod traits;
pub use traits::*;

// Validation and verification
pub mod validation;
pub use validation::*;

// Gas analysis system
pub mod gas;
pub use gas::{
    DynamicGasCalculator, ExecutionContext, GasAnalysis, GasAnalysisResult, GasCostCategory,
};

// Unified opcodes feature for bytecode manipulation tools
#[cfg(feature = "unified-opcodes")]
pub mod unified;
#[cfg(feature = "unified-opcodes")]
pub use unified::UnifiedOpcode;

/// Ethereum hard fork identifiers in chronological order
#[derive(Clone, Copy, Debug, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub enum Fork {
    /// Frontier (July 30, 2015) - Genesis block
    Frontier,
    /// Ice Age (September 7, 2015) - Difficulty bomb introduction
    IceAge,
    /// Homestead (March 14, 2016) - First major upgrade
    Homestead,
    /// DAO Fork (July 20, 2016) - Emergency response to DAO hack
    DaoFork,
    /// Tangerine Whistle (October 18, 2016) - Gas cost adjustments
    TangerineWhistle,
    /// Spurious Dragon (November 22, 2016) - More gas adjustments
    SpuriousDragon,
    /// Byzantium (October 16, 2017) - Metropolis part 1
    Byzantium,
    /// Constantinople (February 28, 2019) - Metropolis part 2
    Constantinople,
    /// Petersburg (February 28, 2019) - Constantinople fix
    Petersburg,
    /// Istanbul (December 8, 2019) - Gas optimizations
    Istanbul,
    /// Muir Glacier (January 2, 2020) - Difficulty bomb delay
    MuirGlacier,
    /// Berlin (April 15, 2021) - Gas cost changes
    Berlin,
    /// London (August 5, 2021) - EIP-1559 fee market
    London,
    /// Altair (October 27, 2021) - Beacon Chain upgrade
    Altair,
    /// Arrow Glacier (December 9, 2021) - Difficulty bomb delay
    ArrowGlacier,
    /// Gray Glacier (June 30, 2022) - Difficulty bomb delay
    GrayGlacier,
    /// Bellatrix (September 6, 2022) - Beacon Chain prep for merge
    Bellatrix,
    /// Paris (September 15, 2022) - The Merge to Proof of Stake
    Paris,
    /// Shanghai (April 12, 2023) - Withdrawals enabled
    Shanghai,
    /// Capella (April 12, 2023) - Beacon Chain withdrawals
    Capella,
    /// Cancun (March 13, 2024) - Proto-danksharding
    Cancun,
    /// Deneb (March 13, 2024) - Beacon Chain blobs
    Deneb,
}

/// EVM opcode groups for better organization
#[derive(Clone, Copy, Debug, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub enum Group {
    /// Stop and Arithmetic Operations (0x00-0x0f)
    StopArithmetic,
    /// Comparison & Bitwise Logic Operations (0x10-0x1f)
    ComparisonBitwiseLogic,
    /// SHA3 (0x20)
    Sha3,
    /// Environmental Information (0x30-0x3f)
    EnvironmentalInformation,
    /// Block Information (0x40-0x4f)
    BlockInformation,
    /// Stack, Memory, Storage and Flow Operations (0x50-0x5f)
    StackMemoryStorageFlow,
    /// Push Operations (0x60-0x7f)
    Push,
    /// Duplication Operations (0x80-0x8f)
    Duplication,
    /// Exchange Operations (0x90-0x9f)
    Exchange,
    /// Logging Operations (0xa0-0xa4)
    Logging,
    /// System operations (0xf0-0xff)
    System,
}

/// Opcode metadata with complete information
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OpcodeMetadata {
    /// The opcode byte value
    pub opcode: u8,
    /// Opcode name (e.g., "ADD", "PUSH1")
    pub name: &'static str,
    /// Base gas cost
    pub gas_cost: u16,
    /// Number of items popped from stack
    pub stack_inputs: u8,
    /// Number of items pushed to stack
    pub stack_outputs: u8,
    /// Human-readable description
    pub description: &'static str,
    /// Fork where this opcode was introduced
    pub introduced_in: Fork,
    /// Opcode group/category
    pub group: Group,
    /// EIP number that introduced this opcode (if applicable)
    pub eip: Option<u16>,
    /// Gas cost changes across forks
    pub gas_history: &'static [(Fork, u16)],
}

/// Core trait that all opcode enums must implement
pub trait OpCode: From<u8> + Into<u8> + Clone + Copy + std::fmt::Debug {
    /// Get complete metadata for this opcode
    fn metadata(&self) -> OpcodeMetadata;

    /// Get the fork this opcode enum represents
    fn fork() -> Fork;

    /// Get all opcodes available in this fork
    fn all_opcodes() -> Vec<Self>;

    /// Check if an opcode exists in this fork
    fn has_opcode(opcode: u8) -> bool {
        Self::all_opcodes().iter().any(|op| (*op).into() == opcode)
    }

    /// Get gas cost for this opcode in this fork
    fn gas_cost(&self) -> u16 {
        let metadata = self.metadata();

        // Find the most recent gas cost for this fork
        let fork = Self::fork();
        metadata
            .gas_history
            .iter()
            .rev()
            .find(|(f, _)| *f <= fork)
            .map(|(_, cost)| *cost)
            .unwrap_or(metadata.gas_cost)
    }

    /// Get stack inputs for this opcode
    fn stack_inputs(&self) -> u8 {
        self.metadata().stack_inputs
    }
    /// Get stack outputs for this opcode
    fn stack_outputs(&self) -> u8 {
        self.metadata().stack_outputs
    }
    /// Get opcode group
    fn group(&self) -> Group {
        self.metadata().group
    }
    /// Get opcode description
    fn description(&self) -> &'static str {
        self.metadata().description
    }
    /// Get fork where this opcode was introduced
    fn introduced_in(&self) -> Fork {
        self.metadata().introduced_in
    }
    /// Get EIP number for this opcode
    fn eip(&self) -> Option<u16> {
        self.metadata().eip
    }
}

/// Fork inheritance utility to get all opcodes available in a specific fork
pub trait ForkOpcodes {
    /// Get all opcodes available in this fork (including inherited ones)
    fn get_opcodes_for_fork(fork: Fork) -> HashMap<u8, OpcodeMetadata>;

    /// Check if a specific opcode is available in a fork
    fn is_opcode_available(fork: Fork, opcode: u8) -> bool {
        Self::get_opcodes_for_fork(fork).contains_key(&opcode)
    }

    /// Get the fork where an opcode was introduced
    fn opcode_introduced_in(opcode: u8) -> Option<Fork>;
}

/// Comprehensive opcode registry that manages all forks
pub struct OpcodeRegistry {
    opcodes: HashMap<Fork, HashMap<u8, OpcodeMetadata>>,
}

impl OpcodeRegistry {
    /// Create a new opcode registry with all known opcodes
    pub fn new() -> Self {
        let mut registry = Self {
            opcodes: HashMap::new(),
        };

        // Register all forks
        registry.register_fork::<forks::Frontier>();
        registry.register_fork::<forks::Homestead>();
        registry.register_fork::<forks::Byzantium>();
        registry.register_fork::<forks::Constantinople>();
        registry.register_fork::<forks::Istanbul>();
        registry.register_fork::<forks::Berlin>();
        registry.register_fork::<forks::London>();
        registry.register_fork::<forks::Shanghai>();
        registry.register_fork::<forks::Cancun>();

        registry
    }

    fn register_fork<T: OpCode>(&mut self) {
        let fork = T::fork();
        let mut opcodes = HashMap::new();

        for opcode_enum in T::all_opcodes() {
            let byte_val: u8 = opcode_enum.into();
            let metadata = opcode_enum.metadata();
            opcodes.insert(byte_val, metadata);
        }

        self.opcodes.insert(fork, opcodes);
    }

    /// Get all opcodes available in a specific fork
    pub fn get_opcodes(&self, fork: Fork) -> HashMap<u8, OpcodeMetadata> {
        let mut result = HashMap::new();

        // Collect opcodes from all previous forks (inheritance)
        for f in self.opcodes.keys() {
            if *f <= fork {
                if let Some(fork_opcodes) = self.opcodes.get(f) {
                    result.extend(fork_opcodes.clone());
                }
            }
        }

        result
    }

    /// Check if a specific opcode is available in a fork
    pub fn is_opcode_available(&self, fork: Fork, opcode: u8) -> bool {
        self.get_opcodes(fork).contains_key(&opcode)
    }

    /// Validate opcode consistency across forks
    pub fn validate(&self) -> Result<(), Vec<String>> {
        validation::validate_registry(self)
    }
}

impl Default for OpcodeRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Macro to generate opcode enums with metadata
#[macro_export]
macro_rules! opcodes {
    (
        $(#[$meta:meta])*
        $enum_name:ident => $fork:ident {
            $(
                $opcode:literal => $name:ident {
                    gas: $gas:literal,
                    inputs: $inputs:literal,
                    outputs: $outputs:literal,
                    description: $description:literal,
                    introduced_in: $introduced:ident,
                    group: $group:ident,
                    eip: $eip:expr,
                    gas_history: [$($gas_fork:ident => $gas_cost:literal),*],
                }
            ),* $(,)?
        }
    ) => {
        $(#[$meta])*
        #[derive(Clone, Copy, Debug, PartialOrd, Ord, PartialEq, Eq, Hash)]
        pub enum $enum_name {
            $(
                #[doc = $description]
                $name,
            )*
        }

        impl From<u8> for $enum_name {
            fn from(value: u8) -> Self {
                match value {
                    $(
                        $opcode => Self::$name,
                    )*
                    _ => panic!("Invalid opcode 0x{:02x} for fork {}", value, stringify!($fork)),
                }
            }
        }

        impl From<$enum_name> for u8 {
            fn from(opcode: $enum_name) -> Self {
                match opcode {
                    $(
                        $enum_name::$name => $opcode,
                    )*
                }
            }
        }

        impl $crate::OpCode for $enum_name {
            fn metadata(&self) -> $crate::OpcodeMetadata {
                match self {
                    $(
                        Self::$name => $crate::OpcodeMetadata {
                            opcode: $opcode,
                            name: stringify!($name),
                            gas_cost: $gas,
                            stack_inputs: $inputs,
                            stack_outputs: $outputs,
                            description: $description,
                            introduced_in: $crate::Fork::$introduced,
                            group: $crate::Group::$group,
                            eip: $eip,
                            gas_history: &[
                                $(
                                    ($crate::Fork::$gas_fork, $gas_cost),
                                )*
                            ],
                        },
                    )*
                }
            }

            fn fork() -> $crate::Fork {
                $crate::Fork::$fork
            }

            fn all_opcodes() -> Vec<Self> {
                vec![
                    $(
                        Self::$name,
                    )*
                ]
            }
        }

        impl std::fmt::Display for $enum_name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.metadata().name)
            }
        }
    };
}
