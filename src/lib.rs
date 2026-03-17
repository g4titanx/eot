//! # EOT - EVM Opcode Table
//!
//! A comprehensive Rust library for EVM opcodes with fork-aware metadata,
//! gas cost tracking, and bytecode analysis utilities.
//!
//! ## Architecture
//!
//! The core type is [`OpCode`], a transparent newtype over `u8` backed by a
//! static `[Option<OpCodeInfo>; 256]` lookup table. This gives O(1) opcode
//! lookups with zero heap allocation.
//!
//! Fork-specific gas costs are handled by [`OpCode::gas_cost`], which applies
//! known EIP gas changes on top of the base cost stored in [`OpCodeInfo`].
//!
//! ## Quick Start
//!
//! ```
//! use eot::{OpCode, Fork};
//!
//! let add = OpCode::ADD;
//! assert_eq!(add.gas_cost(Fork::Frontier), 3);
//! assert!(add.is_valid_in(Fork::Frontier));
//!
//! // Parse from byte
//! let op = OpCode::new(0x60).unwrap();
//! assert_eq!(op, OpCode::PUSH1);
//! ```

#![deny(missing_docs)]
#![warn(clippy::all)]

use std::fmt;
use std::str::FromStr;

pub mod gas;
pub mod validation;

#[cfg(feature = "unified-opcodes")]
pub mod unified;
#[cfg(feature = "unified-opcodes")]
pub use unified::UnifiedOpcode;

// Re-export key gas types
pub use gas::{DynamicGasCalculator, ExecutionContext, GasAnalysis, GasCostCategory};

// ---------------------------------------------------------------------------
// Fork
// ---------------------------------------------------------------------------

/// Ethereum hard forks that affect EVM opcode behavior.
///
/// Only execution-layer forks that introduce new opcodes or change gas costs
/// are included. Consensus-only upgrades (Altair, Bellatrix, Capella, Deneb,
/// etc.) are omitted because they don't alter the opcode set.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Fork {
    /// Frontier (July 30, 2015) — genesis block.
    Frontier,
    /// Homestead (March 14, 2016) — added DELEGATECALL.
    Homestead,
    /// Tangerine Whistle (October 18, 2016) — EIP-150 gas repricing.
    TangerineWhistle,
    /// Spurious Dragon (November 22, 2016) — EIP-161, EIP-170.
    SpuriousDragon,
    /// Byzantium (October 16, 2017) — added REVERT, RETURNDATASIZE,
    /// RETURNDATACOPY, STATICCALL.
    Byzantium,
    /// Constantinople (February 28, 2019) — added SHL, SHR, SAR, CREATE2,
    /// EXTCODEHASH.
    Constantinople,
    /// Petersburg (February 28, 2019) — reverted EIP-1283 SSTORE changes.
    Petersburg,
    /// Istanbul (December 8, 2019) — EIP-1884 gas repricing, added CHAINID
    /// and SELFBALANCE.
    Istanbul,
    /// Berlin (April 15, 2021) — EIP-2929 state access gas changes.
    Berlin,
    /// London (August 5, 2021) — EIP-1559, added BASEFEE.
    London,
    /// Paris / The Merge (September 15, 2022) — DIFFICULTY becomes PREVRANDAO.
    Paris,
    /// Shanghai (April 12, 2023) — added PUSH0 (EIP-3855).
    Shanghai,
    /// Cancun (March 13, 2024) — added TLOAD, TSTORE, MCOPY, BLOBHASH,
    /// BLOBBASEFEE.
    Cancun,
    /// Prague (May 7, 2025).
    Prague,
    /// Fusaka / Fulu-Osaka (December 3, 2025) — EOF (EIP-7692), CLZ, PeerDAS.
    Fusaka,
}

impl Fork {
    /// All EVM-relevant forks in chronological order.
    pub const fn ordered() -> &'static [Self] {
        &[
            Self::Frontier,
            Self::Homestead,
            Self::TangerineWhistle,
            Self::SpuriousDragon,
            Self::Byzantium,
            Self::Constantinople,
            Self::Petersburg,
            Self::Istanbul,
            Self::Berlin,
            Self::London,
            Self::Paris,
            Self::Shanghai,
            Self::Cancun,
            Self::Prague,
            Self::Fusaka,
        ]
    }

    /// The latest supported fork.
    pub const fn latest() -> Self {
        Self::Fusaka
    }
}

impl fmt::Display for Fork {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

// ---------------------------------------------------------------------------
// Group
// ---------------------------------------------------------------------------

/// Opcode categories following the Yellow Paper grouping.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Group {
    /// 0x00–0x0b: Stop and arithmetic.
    StopArithmetic,
    /// 0x10–0x1e: Comparison and bitwise logic.
    ComparisonBitwiseLogic,
    /// 0x20: Keccak-256.
    Sha3,
    /// 0x30–0x3f: Environmental information.
    EnvironmentalInformation,
    /// 0x40–0x4a: Block information.
    BlockInformation,
    /// 0x50–0x5f: Stack, memory, storage and flow.
    StackMemoryStorageFlow,
    /// 0x60–0x7f: Push operations.
    Push,
    /// 0x80–0x8f: Duplication operations.
    Duplication,
    /// 0x90–0x9f: Exchange operations.
    Exchange,
    /// 0xa0–0xa4: Logging.
    Logging,
    /// 0xf0–0xff: System operations.
    System,
    /// 0xd0–0xd3, 0xe0–0xee: EOF (EVM Object Format) operations.
    Eof,
}

// ---------------------------------------------------------------------------
// OpCodeInfo
// ---------------------------------------------------------------------------

/// Static metadata for a single EVM opcode.
///
/// Stored in the global `OPCODE_TABLE` and returned by [`OpCode::info`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OpCodeInfo {
    /// Mnemonic name (e.g. `"ADD"`, `"PUSH1"`).
    pub name: &'static str,
    /// Number of stack items consumed.
    pub inputs: u8,
    /// Number of stack items produced.
    pub outputs: u8,
    /// Base gas cost at the fork where the opcode was introduced.
    ///
    /// Use [`OpCode::gas_cost`] for fork-aware costs.
    pub base_gas: u16,
    /// Opcode category.
    pub group: Group,
    /// Fork where this opcode was first available.
    pub introduced_in: Fork,
    /// EIP that introduced this opcode, if any.
    pub eip: Option<u16>,
    /// Bytes of immediate data following the opcode (e.g. 1 for PUSH1).
    pub immediate_size: u8,
    /// Whether this opcode halts execution (STOP, RETURN, REVERT, etc.).
    pub terminates: bool,
}

impl OpCodeInfo {
    /// Create a new `OpCodeInfo` with default optional fields.
    const fn new(
        name: &'static str,
        inputs: u8,
        outputs: u8,
        gas: u16,
        group: Group,
        fork: Fork,
    ) -> Self {
        Self {
            name,
            inputs,
            outputs,
            base_gas: gas,
            group,
            introduced_in: fork,
            eip: None,
            immediate_size: 0,
            terminates: false,
        }
    }

    /// Set the EIP number.
    const fn eip(mut self, eip: u16) -> Self {
        self.eip = Some(eip);
        self
    }

    /// Set the immediate data size in bytes.
    const fn imm(mut self, size: u8) -> Self {
        self.immediate_size = size;
        self
    }

    /// Mark this opcode as terminating execution.
    const fn term(mut self) -> Self {
        self.terminates = true;
        self
    }

    /// Net stack effect (outputs − inputs).
    pub const fn stack_diff(&self) -> i16 {
        self.outputs as i16 - self.inputs as i16
    }
}

// ---------------------------------------------------------------------------
// OpCode — core type
// ---------------------------------------------------------------------------

/// A single EVM opcode, stored as a transparent `u8` wrapper.
///
/// Known opcodes have entries in the global `OPCODE_TABLE` and can be
/// looked up in O(1) via [`OpCode::info`].
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct OpCode(u8);

// ---------------------------------------------------------------------------
// opcodes! macro — generates constants + static lookup table
// ---------------------------------------------------------------------------

macro_rules! opcodes {
    ($(
        $byte:literal => $name:ident
            ($in:expr, $out:expr, $gas:expr, $group:ident, $fork:ident)
            $([ $($chain:tt)* ])?
    ;)*) => {
        impl OpCode {
            $(
                #[doc = concat!("`", stringify!($name), "` (`0x", stringify!($byte), "`)")]
                pub const $name: Self = Self($byte);
            )*
        }

        static OPCODE_TABLE: [Option<OpCodeInfo>; 256] = {
            const NONE: Option<OpCodeInfo> = None;
            let mut t = [NONE; 256];
            $(
                t[$byte as usize] = Some(
                    OpCodeInfo::new(
                        stringify!($name),
                        $in, $out, $gas,
                        Group::$group,
                        Fork::$fork,
                    )
                    $( .$($chain)* )?
                );
            )*
            t
        };
    };
}

// ---------------------------------------------------------------------------
// All 149 EVM opcodes
// ---------------------------------------------------------------------------

opcodes! {
    // -- 0x00–0x0b: Stop and Arithmetic ------------------------------------
    0x00 => STOP       (0, 0,  0, StopArithmetic, Frontier) [term()];
    0x01 => ADD        (2, 1,  3, StopArithmetic, Frontier);
    0x02 => MUL        (2, 1,  5, StopArithmetic, Frontier);
    0x03 => SUB        (2, 1,  3, StopArithmetic, Frontier);
    0x04 => DIV        (2, 1,  5, StopArithmetic, Frontier);
    0x05 => SDIV       (2, 1,  5, StopArithmetic, Frontier);
    0x06 => MOD        (2, 1,  5, StopArithmetic, Frontier);
    0x07 => SMOD       (2, 1,  5, StopArithmetic, Frontier);
    0x08 => ADDMOD     (3, 1,  8, StopArithmetic, Frontier);
    0x09 => MULMOD     (3, 1,  8, StopArithmetic, Frontier);
    0x0a => EXP        (2, 1, 10, StopArithmetic, Frontier);
    0x0b => SIGNEXTEND (2, 1,  5, StopArithmetic, Frontier);

    // -- 0x10–0x1d: Comparison & Bitwise Logic -----------------------------
    0x10 => LT     (2, 1, 3, ComparisonBitwiseLogic, Frontier);
    0x11 => GT     (2, 1, 3, ComparisonBitwiseLogic, Frontier);
    0x12 => SLT    (2, 1, 3, ComparisonBitwiseLogic, Frontier);
    0x13 => SGT    (2, 1, 3, ComparisonBitwiseLogic, Frontier);
    0x14 => EQ     (2, 1, 3, ComparisonBitwiseLogic, Frontier);
    0x15 => ISZERO (1, 1, 3, ComparisonBitwiseLogic, Frontier);
    0x16 => AND    (2, 1, 3, ComparisonBitwiseLogic, Frontier);
    0x17 => OR     (2, 1, 3, ComparisonBitwiseLogic, Frontier);
    0x18 => XOR    (2, 1, 3, ComparisonBitwiseLogic, Frontier);
    0x19 => NOT    (1, 1, 3, ComparisonBitwiseLogic, Frontier);
    0x1a => BYTE   (2, 1, 3, ComparisonBitwiseLogic, Frontier);
    0x1b => SHL    (2, 1, 3, ComparisonBitwiseLogic, Constantinople) [eip(145)];
    0x1c => SHR    (2, 1, 3, ComparisonBitwiseLogic, Constantinople) [eip(145)];
    0x1d => SAR    (2, 1, 3, ComparisonBitwiseLogic, Constantinople) [eip(145)];
    0x1e => CLZ    (1, 1, 3, ComparisonBitwiseLogic, Fusaka)        [eip(7939)];

    // -- 0x20: Keccak-256 --------------------------------------------------
    0x20 => KECCAK256 (2, 1, 30, Sha3, Frontier);

    // -- 0x30–0x3f: Environmental Information ------------------------------
    0x30 => ADDRESS        (0, 1,  2, EnvironmentalInformation, Frontier);
    0x31 => BALANCE        (1, 1, 20, EnvironmentalInformation, Frontier);
    0x32 => ORIGIN         (0, 1,  2, EnvironmentalInformation, Frontier);
    0x33 => CALLER         (0, 1,  2, EnvironmentalInformation, Frontier);
    0x34 => CALLVALUE      (0, 1,  2, EnvironmentalInformation, Frontier);
    0x35 => CALLDATALOAD   (1, 1,  3, EnvironmentalInformation, Frontier);
    0x36 => CALLDATASIZE   (0, 1,  2, EnvironmentalInformation, Frontier);
    0x37 => CALLDATACOPY   (3, 0,  3, EnvironmentalInformation, Frontier);
    0x38 => CODESIZE       (0, 1,  2, EnvironmentalInformation, Frontier);
    0x39 => CODECOPY       (3, 0,  3, EnvironmentalInformation, Frontier);
    0x3a => GASPRICE       (0, 1,  2, EnvironmentalInformation, Frontier);
    0x3b => EXTCODESIZE    (1, 1, 20, EnvironmentalInformation, Frontier);
    0x3c => EXTCODECOPY    (4, 0, 20, EnvironmentalInformation, Frontier);
    0x3d => RETURNDATASIZE (0, 1,  2, EnvironmentalInformation, Byzantium) [eip(211)];
    0x3e => RETURNDATACOPY (3, 0,  3, EnvironmentalInformation, Byzantium) [eip(211)];
    0x3f => EXTCODEHASH    (1, 1, 400, EnvironmentalInformation, Constantinople) [eip(1052)];

    // -- 0x40–0x4a: Block Information --------------------------------------
    0x40 => BLOCKHASH   (1, 1, 20, BlockInformation, Frontier);
    0x41 => COINBASE    (0, 1,  2, BlockInformation, Frontier);
    0x42 => TIMESTAMP   (0, 1,  2, BlockInformation, Frontier);
    0x43 => NUMBER      (0, 1,  2, BlockInformation, Frontier);
    0x44 => DIFFICULTY  (0, 1,  2, BlockInformation, Frontier);
    0x45 => GASLIMIT    (0, 1,  2, BlockInformation, Frontier);
    0x46 => CHAINID     (0, 1,  2, BlockInformation, Istanbul)  [eip(1344)];
    0x47 => SELFBALANCE (0, 1,  5, BlockInformation, Istanbul)  [eip(1884)];
    0x48 => BASEFEE     (0, 1,  2, BlockInformation, London)    [eip(3198)];
    0x49 => BLOBHASH    (1, 1,  3, BlockInformation, Cancun)    [eip(4844)];
    0x4a => BLOBBASEFEE (0, 1,  2, BlockInformation, Cancun)    [eip(7516)];

    // -- 0x50–0x5f: Stack, Memory, Storage and Flow ------------------------
    0x50 => POP      (1, 0,    2, StackMemoryStorageFlow, Frontier);
    0x51 => MLOAD    (1, 1,    3, StackMemoryStorageFlow, Frontier);
    0x52 => MSTORE   (2, 0,    3, StackMemoryStorageFlow, Frontier);
    0x53 => MSTORE8  (2, 0,    3, StackMemoryStorageFlow, Frontier);
    0x54 => SLOAD    (1, 1,   50, StackMemoryStorageFlow, Frontier);
    0x55 => SSTORE   (2, 0, 5000, StackMemoryStorageFlow, Frontier);
    0x56 => JUMP     (1, 0,    8, StackMemoryStorageFlow, Frontier);
    0x57 => JUMPI    (2, 0,   10, StackMemoryStorageFlow, Frontier);
    0x58 => PC       (0, 1,    2, StackMemoryStorageFlow, Frontier);
    0x59 => MSIZE    (0, 1,    2, StackMemoryStorageFlow, Frontier);
    0x5a => GAS      (0, 1,    2, StackMemoryStorageFlow, Frontier);
    0x5b => JUMPDEST (0, 0,    1, StackMemoryStorageFlow, Frontier);
    0x5c => TLOAD    (1, 1,  100, StackMemoryStorageFlow, Cancun)   [eip(1153)];
    0x5d => TSTORE   (2, 0,  100, StackMemoryStorageFlow, Cancun)   [eip(1153)];
    0x5e => MCOPY    (3, 0,    3, StackMemoryStorageFlow, Cancun)   [eip(5656)];
    0x5f => PUSH0    (0, 1,    2, Push, Shanghai) [eip(3855)];

    // -- 0x60–0x7f: Push Operations ----------------------------------------
    0x60 => PUSH1  (0, 1, 3, Push, Frontier) [imm(1)];
    0x61 => PUSH2  (0, 1, 3, Push, Frontier) [imm(2)];
    0x62 => PUSH3  (0, 1, 3, Push, Frontier) [imm(3)];
    0x63 => PUSH4  (0, 1, 3, Push, Frontier) [imm(4)];
    0x64 => PUSH5  (0, 1, 3, Push, Frontier) [imm(5)];
    0x65 => PUSH6  (0, 1, 3, Push, Frontier) [imm(6)];
    0x66 => PUSH7  (0, 1, 3, Push, Frontier) [imm(7)];
    0x67 => PUSH8  (0, 1, 3, Push, Frontier) [imm(8)];
    0x68 => PUSH9  (0, 1, 3, Push, Frontier) [imm(9)];
    0x69 => PUSH10 (0, 1, 3, Push, Frontier) [imm(10)];
    0x6a => PUSH11 (0, 1, 3, Push, Frontier) [imm(11)];
    0x6b => PUSH12 (0, 1, 3, Push, Frontier) [imm(12)];
    0x6c => PUSH13 (0, 1, 3, Push, Frontier) [imm(13)];
    0x6d => PUSH14 (0, 1, 3, Push, Frontier) [imm(14)];
    0x6e => PUSH15 (0, 1, 3, Push, Frontier) [imm(15)];
    0x6f => PUSH16 (0, 1, 3, Push, Frontier) [imm(16)];
    0x70 => PUSH17 (0, 1, 3, Push, Frontier) [imm(17)];
    0x71 => PUSH18 (0, 1, 3, Push, Frontier) [imm(18)];
    0x72 => PUSH19 (0, 1, 3, Push, Frontier) [imm(19)];
    0x73 => PUSH20 (0, 1, 3, Push, Frontier) [imm(20)];
    0x74 => PUSH21 (0, 1, 3, Push, Frontier) [imm(21)];
    0x75 => PUSH22 (0, 1, 3, Push, Frontier) [imm(22)];
    0x76 => PUSH23 (0, 1, 3, Push, Frontier) [imm(23)];
    0x77 => PUSH24 (0, 1, 3, Push, Frontier) [imm(24)];
    0x78 => PUSH25 (0, 1, 3, Push, Frontier) [imm(25)];
    0x79 => PUSH26 (0, 1, 3, Push, Frontier) [imm(26)];
    0x7a => PUSH27 (0, 1, 3, Push, Frontier) [imm(27)];
    0x7b => PUSH28 (0, 1, 3, Push, Frontier) [imm(28)];
    0x7c => PUSH29 (0, 1, 3, Push, Frontier) [imm(29)];
    0x7d => PUSH30 (0, 1, 3, Push, Frontier) [imm(30)];
    0x7e => PUSH31 (0, 1, 3, Push, Frontier) [imm(31)];
    0x7f => PUSH32 (0, 1, 3, Push, Frontier) [imm(32)];

    // -- 0x80–0x8f: Duplication Operations ---------------------------------
    0x80 => DUP1  ( 1,  2, 3, Duplication, Frontier);
    0x81 => DUP2  ( 2,  3, 3, Duplication, Frontier);
    0x82 => DUP3  ( 3,  4, 3, Duplication, Frontier);
    0x83 => DUP4  ( 4,  5, 3, Duplication, Frontier);
    0x84 => DUP5  ( 5,  6, 3, Duplication, Frontier);
    0x85 => DUP6  ( 6,  7, 3, Duplication, Frontier);
    0x86 => DUP7  ( 7,  8, 3, Duplication, Frontier);
    0x87 => DUP8  ( 8,  9, 3, Duplication, Frontier);
    0x88 => DUP9  ( 9, 10, 3, Duplication, Frontier);
    0x89 => DUP10 (10, 11, 3, Duplication, Frontier);
    0x8a => DUP11 (11, 12, 3, Duplication, Frontier);
    0x8b => DUP12 (12, 13, 3, Duplication, Frontier);
    0x8c => DUP13 (13, 14, 3, Duplication, Frontier);
    0x8d => DUP14 (14, 15, 3, Duplication, Frontier);
    0x8e => DUP15 (15, 16, 3, Duplication, Frontier);
    0x8f => DUP16 (16, 17, 3, Duplication, Frontier);

    // -- 0x90–0x9f: Exchange Operations ------------------------------------
    0x90 => SWAP1  ( 2,  2, 3, Exchange, Frontier);
    0x91 => SWAP2  ( 3,  3, 3, Exchange, Frontier);
    0x92 => SWAP3  ( 4,  4, 3, Exchange, Frontier);
    0x93 => SWAP4  ( 5,  5, 3, Exchange, Frontier);
    0x94 => SWAP5  ( 6,  6, 3, Exchange, Frontier);
    0x95 => SWAP6  ( 7,  7, 3, Exchange, Frontier);
    0x96 => SWAP7  ( 8,  8, 3, Exchange, Frontier);
    0x97 => SWAP8  ( 9,  9, 3, Exchange, Frontier);
    0x98 => SWAP9  (10, 10, 3, Exchange, Frontier);
    0x99 => SWAP10 (11, 11, 3, Exchange, Frontier);
    0x9a => SWAP11 (12, 12, 3, Exchange, Frontier);
    0x9b => SWAP12 (13, 13, 3, Exchange, Frontier);
    0x9c => SWAP13 (14, 14, 3, Exchange, Frontier);
    0x9d => SWAP14 (15, 15, 3, Exchange, Frontier);
    0x9e => SWAP15 (16, 16, 3, Exchange, Frontier);
    0x9f => SWAP16 (17, 17, 3, Exchange, Frontier);

    // -- 0xa0–0xa4: Logging Operations -------------------------------------
    0xa0 => LOG0 (2, 0,  375, Logging, Frontier);
    0xa1 => LOG1 (3, 0,  750, Logging, Frontier);
    0xa2 => LOG2 (4, 0, 1125, Logging, Frontier);
    0xa3 => LOG3 (5, 0, 1500, Logging, Frontier);
    0xa4 => LOG4 (6, 0, 1875, Logging, Frontier);

    // -- 0xf0–0xff: System Operations --------------------------------------
    0xf0 => CREATE       (3, 1, 32000, System, Frontier);
    0xf1 => CALL         (7, 1,    40, System, Frontier);
    0xf2 => CALLCODE     (7, 1,    40, System, Frontier);
    0xf3 => RETURN       (2, 0,     0, System, Frontier) [term()];
    0xf4 => DELEGATECALL (6, 1,    40, System, Homestead)      [eip(7)];
    0xf5 => CREATE2      (4, 1, 32000, System, Constantinople) [eip(1014)];
    0xfa => STATICCALL   (6, 1,   700, System, Byzantium)      [eip(214)];
    0xfd => REVERT       (2, 0,     0, System, Byzantium)      [eip(140).term()];
    0xfe => INVALID      (0, 0,     0, System, Frontier)       [term()];
    0xff => SELFDESTRUCT (1, 0,     0, System, Frontier);

    // -- 0xd0–0xd3: EOF Data Section Access (EIP-7480) ---------------------
    0xd0 => DATALOAD       (1, 1, 4, Eof, Fusaka) [eip(7480)];
    0xd1 => DATALOADN      (0, 1, 3, Eof, Fusaka) [eip(7480).imm(2)];
    0xd2 => DATASIZE       (0, 1, 2, Eof, Fusaka) [eip(7480)];
    0xd3 => DATACOPY       (3, 0, 3, Eof, Fusaka) [eip(7480)];

    // -- 0xe0–0xe8: EOF Control Flow & Stack (EIP-4200/4750/6206/663) ------
    0xe0 => RJUMP          (0, 0, 2, Eof, Fusaka) [eip(4200).imm(2)];
    0xe1 => RJUMPI         (1, 0, 4, Eof, Fusaka) [eip(4200).imm(2)];
    0xe2 => RJUMPV         (1, 0, 4, Eof, Fusaka) [eip(4200).imm(1)];
    0xe3 => CALLF          (0, 0, 5, Eof, Fusaka) [eip(4750).imm(2)];
    0xe4 => RETF           (0, 0, 3, Eof, Fusaka) [eip(4750).term()];
    0xe5 => JUMPF          (0, 0, 5, Eof, Fusaka) [eip(6206).imm(2).term()];
    0xe6 => DUPN           (0, 1, 3, Eof, Fusaka) [eip(663).imm(1)];
    0xe7 => SWAPN          (0, 0, 3, Eof, Fusaka) [eip(663).imm(1)];
    0xe8 => EXCHANGE       (0, 0, 3, Eof, Fusaka) [eip(663).imm(1)];

    // -- 0xec, 0xee: EOF Contract Creation (EIP-7620) ----------------------
    0xec => EOFCREATE      (4, 1, 32000, Eof, Fusaka) [eip(7620).imm(1)];
    0xee => RETURNCONTRACT (2, 0,     0, Eof, Fusaka) [eip(7620).imm(1).term()];

    // -- 0xf7–0xfb: EOF Calls & Return Data (EIP-7069) --------------------
    0xf7 => RETURNDATALOAD  (1, 1,   3, System, Fusaka) [eip(7069)];
    0xf8 => EXTCALL         (4, 1, 100, System, Fusaka) [eip(7069)];
    0xf9 => EXTDELEGATECALL (3, 1, 100, System, Fusaka) [eip(7069)];
    0xfb => EXTSTATICCALL   (3, 1, 100, System, Fusaka) [eip(7069)];
}

// Alias: after The Merge (Paris), DIFFICULTY returns the beacon chain
// PREVRANDAO value (EIP-4399). The byte 0x44 is unchanged.
impl OpCode {
    /// Alias for [`DIFFICULTY`](Self::DIFFICULTY) after The Merge (EIP-4399).
    pub const PREVRANDAO: Self = Self(0x44);
}

// ---------------------------------------------------------------------------
// OpCode — methods
// ---------------------------------------------------------------------------

impl OpCode {
    /// Creates an `OpCode` if `byte` maps to a known opcode.
    pub const fn new(byte: u8) -> Option<Self> {
        if OPCODE_TABLE[byte as usize].is_some() {
            Some(Self(byte))
        } else {
            None
        }
    }

    /// Wraps any `u8` as an `OpCode` without validation.
    ///
    /// Unknown bytes will return `None` from [`info`](Self::info).
    pub const fn from_byte(byte: u8) -> Self {
        Self(byte)
    }

    /// Returns the raw byte value.
    pub const fn byte(&self) -> u8 {
        self.0
    }

    /// Returns static metadata, or `None` for unknown opcodes.
    pub const fn info(&self) -> Option<&'static OpCodeInfo> {
        match &OPCODE_TABLE[self.0 as usize] {
            Some(info) => Some(info),
            None => None,
        }
    }

    /// Returns the mnemonic name, or `"UNKNOWN"` for unknown opcodes.
    pub fn name(&self) -> &'static str {
        match self.info() {
            Some(info) => info.name,
            None => "UNKNOWN",
        }
    }

    /// Returns `true` if this opcode existed at `fork`.
    pub fn is_valid_in(&self, fork: Fork) -> bool {
        match self.info() {
            Some(info) => info.introduced_in <= fork,
            None => false,
        }
    }

    /// Returns the static gas cost for this opcode at `fork`.
    ///
    /// This accounts for EIP-based gas repricing across forks. For opcodes
    /// with dynamic pricing (warm/cold access, memory expansion, etc.), use
    /// [`DynamicGasCalculator`].
    pub fn gas_cost(&self, fork: Fork) -> u16 {
        match self.info() {
            Some(info) => gas_cost_for_fork(self.0, info.base_gas, fork),
            None => 0,
        }
    }

    /// Returns `true` if this is a PUSH opcode (PUSH0–PUSH32).
    pub const fn is_push(&self) -> bool {
        matches!(self.0, 0x5f..=0x7f)
    }

    /// Returns `true` if this is a DUP opcode (DUP1–DUP16).
    pub const fn is_dup(&self) -> bool {
        matches!(self.0, 0x80..=0x8f)
    }

    /// Returns `true` if this is a SWAP opcode (SWAP1–SWAP16).
    pub const fn is_swap(&self) -> bool {
        matches!(self.0, 0x90..=0x9f)
    }

    /// Returns `true` if this is a LOG opcode (LOG0–LOG4).
    pub const fn is_log(&self) -> bool {
        matches!(self.0, 0xa0..=0xa4)
    }

    /// Returns `true` if this opcode terminates execution.
    pub fn terminates(&self) -> bool {
        self.info().is_some_and(|i| i.terminates)
    }

    /// Returns `true` if this opcode affects control flow.
    pub fn is_control_flow(&self) -> bool {
        matches!(
            self.0,
            0x00 | 0x56 | 0x57 | 0x5b | 0xf3 | 0xfd | 0xfe | 0xff |
            // EOF: RJUMP, RJUMPI, RJUMPV, CALLF, RETF, JUMPF
            0xe0..=0xe5
        )
    }

    /// Returns `true` if this opcode modifies persistent state.
    pub fn modifies_state(&self) -> bool {
        matches!(
            self.0,
            0x55 | 0x5d | 0xf0 | 0xf1 | 0xf2 | 0xf4 | 0xf5 | 0xff |
            // EOF: EOFCREATE, EXTCALL, EXTDELEGATECALL
            0xec | 0xf8 | 0xf9
        )
    }

    /// Returns `true` if this opcode reads or writes memory.
    pub fn affects_memory(&self) -> bool {
        matches!(
            self.0,
            0x20 | 0x37 | 0x39 | 0x3e | 0x51..=0x53 | 0x5e |
            0xa0..=0xa4 | 0xf0..=0xf5 | 0xfa | 0xfd |
            // EOF: DATACOPY, EOFCREATE, RETURNCONTRACT
            0xd3 | 0xec | 0xee
        )
    }

    /// Returns `true` if this opcode reads or writes storage.
    pub fn affects_storage(&self) -> bool {
        matches!(self.0, 0x54 | 0x55 | 0x5c | 0x5d)
    }

    /// Returns `true` if the gas cost varies with execution context.
    pub fn has_dynamic_gas(&self) -> bool {
        matches!(
            self.0,
            0x0a | 0x20 |
            0x31 | 0x37 | 0x39 | 0x3b | 0x3c | 0x3e | 0x3f |
            0x54 | 0x55 | 0x5c | 0x5d | 0x5e |
            0x51..=0x53 |
            0xa0..=0xa4 |
            0xf0..=0xf5 | 0xfa | 0xff |
            // EOF: DATACOPY, EOFCREATE, EXTCALL, EXTDELEGATECALL, EXTSTATICCALL
            0xd3 | 0xec | 0xf8 | 0xf9 | 0xfb
        )
    }

    /// Returns `true` if this is an EOF-only opcode (EIP-7692).
    pub fn is_eof(&self) -> bool {
        matches!(
            self.0,
            0xd0..=0xd3 | 0xe0..=0xe8 | 0xec | 0xee
        )
    }

    /// Returns an iterator over all known opcodes.
    pub fn iter_all() -> impl Iterator<Item = OpCode> {
        (0u16..=255).filter_map(|b| Self::new(b as u8))
    }

    /// Returns the number of opcodes valid at `fork`.
    pub fn count_at(fork: Fork) -> usize {
        Self::iter_all().filter(|op| op.is_valid_in(fork)).count()
    }
}

// ---------------------------------------------------------------------------
// Fork-specific gas cost adjustments
// ---------------------------------------------------------------------------

/// Applies known EIP gas repricing for a given fork.
///
/// This encodes the historical gas changes from Tangerine Whistle (EIP-150),
/// Istanbul (EIP-1884), and Berlin (EIP-2929) in a single function.
fn gas_cost_for_fork(byte: u8, base: u16, fork: Fork) -> u16 {
    use Fork::*;
    match byte {
        // BALANCE: 20 → 400 (TW/EIP-150) → 700 (Istanbul/EIP-1884) → 2600 (Berlin/EIP-2929)
        0x31 => {
            if fork >= Berlin {
                2600
            } else if fork >= Istanbul {
                700
            } else if fork >= TangerineWhistle {
                400
            } else {
                base
            }
        }
        // EXTCODESIZE: 20 → 700 (TW) → 2600 (Berlin)
        0x3b => {
            if fork >= Berlin {
                2600
            } else if fork >= TangerineWhistle {
                700
            } else {
                base
            }
        }
        // EXTCODECOPY: 20 → 700 (TW) → 2600 (Berlin)
        0x3c => {
            if fork >= Berlin {
                2600
            } else if fork >= TangerineWhistle {
                700
            } else {
                base
            }
        }
        // EXTCODEHASH: 400 (Constantinople) → 700 (Istanbul) → 2600 (Berlin)
        0x3f => {
            if fork >= Berlin {
                2600
            } else if fork >= Istanbul {
                700
            } else {
                base
            }
        }
        // SLOAD: 50 → 200 (TW) → 800 (Istanbul) → 2100 (Berlin, cold)
        0x54 => {
            if fork >= Berlin {
                2100
            } else if fork >= Istanbul {
                800
            } else if fork >= TangerineWhistle {
                200
            } else {
                base
            }
        }
        // CALL, CALLCODE: 40 → 700 (TW) → 100 (Berlin, warm)
        0xf1 | 0xf2 => {
            if fork >= Berlin {
                100
            } else if fork >= TangerineWhistle {
                700
            } else {
                base
            }
        }
        // DELEGATECALL: 40 (Homestead) → 700 (TW) → 100 (Berlin, warm)
        0xf4 => {
            if fork >= Berlin {
                100
            } else if fork >= TangerineWhistle {
                700
            } else {
                base
            }
        }
        // STATICCALL: 700 (Byzantium) → 100 (Berlin, warm)
        0xfa => {
            if fork >= Berlin {
                100
            } else {
                base
            }
        }
        // SELFDESTRUCT: 0 → 5000 (TW/EIP-150)
        0xff => {
            if fork >= TangerineWhistle {
                5000
            } else {
                base
            }
        }
        _ => base,
    }
}

// ---------------------------------------------------------------------------
// Trait implementations
// ---------------------------------------------------------------------------

impl fmt::Debug for OpCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.info() {
            Some(info) => write!(f, "{}", info.name),
            None => write!(f, "UNKNOWN(0x{:02x})", self.0),
        }
    }
}

impl fmt::Display for OpCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

impl From<u8> for OpCode {
    fn from(byte: u8) -> Self {
        Self(byte)
    }
}

impl From<OpCode> for u8 {
    fn from(op: OpCode) -> Self {
        op.0
    }
}

impl FromStr for OpCode {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Linear scan — fine for a parse utility, not a hot path.
        for (i, entry) in OPCODE_TABLE.iter().enumerate() {
            if let Some(info) = entry {
                if info.name.eq_ignore_ascii_case(s) {
                    return Ok(Self(i as u8));
                }
            }
        }
        // Aliases
        match s {
            "SHA3" => Ok(Self::KECCAK256),
            "PREVRANDAO" => Ok(Self::DIFFICULTY),
            _ => Err(format!("unknown opcode: {s}")),
        }
    }
}

// ---------------------------------------------------------------------------
// Serde support
// ---------------------------------------------------------------------------

#[cfg(feature = "serde")]
mod serde_impl {
    use super::OpCode;

    impl serde::Serialize for OpCode {
        fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
            serializer.serialize_str(self.name())
        }
    }

    impl<'de> serde::Deserialize<'de> for OpCode {
        fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
            struct Visitor;
            impl serde::de::Visitor<'_> for Visitor {
                type Value = OpCode;
                fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    f.write_str("an opcode name or byte value")
                }
                fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<OpCode, E> {
                    v.parse().map_err(E::custom)
                }
                fn visit_u64<E: serde::de::Error>(self, v: u64) -> Result<OpCode, E> {
                    if v > 255 {
                        return Err(E::custom("opcode byte out of range"));
                    }
                    Ok(OpCode::from_byte(v as u8))
                }
            }
            deserializer.deserialize_any(Visitor)
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opcode_constants() {
        assert_eq!(OpCode::STOP.byte(), 0x00);
        assert_eq!(OpCode::ADD.byte(), 0x01);
        assert_eq!(OpCode::PUSH1.byte(), 0x60);
        assert_eq!(OpCode::DUP1.byte(), 0x80);
        assert_eq!(OpCode::SWAP1.byte(), 0x90);
        assert_eq!(OpCode::SELFDESTRUCT.byte(), 0xff);
        assert_eq!(OpCode::PREVRANDAO, OpCode::DIFFICULTY);
    }

    #[test]
    fn opcode_new_valid() {
        assert!(OpCode::new(0x01).is_some()); // ADD
        assert!(OpCode::new(0x5f).is_some()); // PUSH0
        assert!(OpCode::new(0xff).is_some()); // SELFDESTRUCT
    }

    #[test]
    fn opcode_new_invalid() {
        assert!(OpCode::new(0x0c).is_none()); // gap between SIGNEXTEND and LT
        assert!(OpCode::new(0x21).is_none()); // gap after KECCAK256
        assert!(OpCode::new(0xef).is_none()); // unassigned
    }

    #[test]
    fn opcode_info() {
        let add = OpCode::ADD;
        let info = add.info().unwrap();
        assert_eq!(info.name, "ADD");
        assert_eq!(info.inputs, 2);
        assert_eq!(info.outputs, 1);
        assert_eq!(info.base_gas, 3);
        assert_eq!(info.group, Group::StopArithmetic);
        assert_eq!(info.introduced_in, Fork::Frontier);
    }

    #[test]
    fn fork_availability() {
        // ADD available since Frontier
        assert!(OpCode::ADD.is_valid_in(Fork::Frontier));
        assert!(OpCode::ADD.is_valid_in(Fork::Prague));

        // DELEGATECALL only from Homestead
        assert!(!OpCode::DELEGATECALL.is_valid_in(Fork::Frontier));
        assert!(OpCode::DELEGATECALL.is_valid_in(Fork::Homestead));

        // TLOAD only from Cancun
        assert!(!OpCode::TLOAD.is_valid_in(Fork::Shanghai));
        assert!(OpCode::TLOAD.is_valid_in(Fork::Cancun));

        // PUSH0 only from Shanghai
        assert!(!OpCode::PUSH0.is_valid_in(Fork::London));
        assert!(OpCode::PUSH0.is_valid_in(Fork::Shanghai));
    }

    #[test]
    fn gas_cost_frontier() {
        assert_eq!(OpCode::ADD.gas_cost(Fork::Frontier), 3);
        assert_eq!(OpCode::SLOAD.gas_cost(Fork::Frontier), 50);
        assert_eq!(OpCode::BALANCE.gas_cost(Fork::Frontier), 20);
        assert_eq!(OpCode::CALL.gas_cost(Fork::Frontier), 40);
    }

    #[test]
    fn gas_cost_tangerine_whistle() {
        assert_eq!(OpCode::SLOAD.gas_cost(Fork::TangerineWhistle), 200);
        assert_eq!(OpCode::BALANCE.gas_cost(Fork::TangerineWhistle), 400);
        assert_eq!(OpCode::CALL.gas_cost(Fork::TangerineWhistle), 700);
        assert_eq!(OpCode::SELFDESTRUCT.gas_cost(Fork::TangerineWhistle), 5000);
    }

    #[test]
    fn gas_cost_istanbul() {
        assert_eq!(OpCode::SLOAD.gas_cost(Fork::Istanbul), 800);
        assert_eq!(OpCode::BALANCE.gas_cost(Fork::Istanbul), 700);
    }

    #[test]
    fn gas_cost_berlin() {
        assert_eq!(OpCode::SLOAD.gas_cost(Fork::Berlin), 2100);
        assert_eq!(OpCode::BALANCE.gas_cost(Fork::Berlin), 2600);
        assert_eq!(OpCode::EXTCODESIZE.gas_cost(Fork::Berlin), 2600);
        assert_eq!(OpCode::CALL.gas_cost(Fork::Berlin), 100);
        assert_eq!(OpCode::STATICCALL.gas_cost(Fork::Berlin), 100);
    }

    #[test]
    fn classification() {
        assert!(OpCode::PUSH0.is_push());
        assert!(OpCode::PUSH1.is_push());
        assert!(OpCode::PUSH32.is_push());
        assert!(!OpCode::ADD.is_push());

        assert!(OpCode::DUP1.is_dup());
        assert!(OpCode::DUP16.is_dup());
        assert!(!OpCode::PUSH1.is_dup());

        assert!(OpCode::SWAP1.is_swap());
        assert!(OpCode::SWAP16.is_swap());
        assert!(!OpCode::DUP1.is_swap());

        assert!(OpCode::STOP.terminates());
        assert!(OpCode::RETURN.terminates());
        assert!(OpCode::REVERT.terminates());
        assert!(!OpCode::ADD.terminates());
    }

    #[test]
    fn display() {
        assert_eq!(OpCode::ADD.to_string(), "ADD");
        assert_eq!(OpCode::PUSH1.to_string(), "PUSH1");
        assert_eq!(OpCode::from_byte(0x0c).to_string(), "UNKNOWN(0x0c)");
    }

    #[test]
    fn from_str_roundtrip() {
        for op in OpCode::iter_all() {
            let name = op.name();
            let parsed: OpCode = name.parse().unwrap();
            assert_eq!(parsed, op, "roundtrip failed for {name}");
        }
    }

    #[test]
    fn from_str_aliases() {
        assert_eq!("SHA3".parse::<OpCode>().unwrap(), OpCode::KECCAK256);
        assert_eq!("PREVRANDAO".parse::<OpCode>().unwrap(), OpCode::DIFFICULTY);
    }

    #[test]
    fn byte_roundtrip() {
        for b in 0u8..=255 {
            assert_eq!(OpCode::from_byte(b).byte(), b);
        }
    }

    #[test]
    fn immediate_sizes() {
        assert_eq!(OpCode::PUSH0.info().unwrap().immediate_size, 0);
        assert_eq!(OpCode::PUSH1.info().unwrap().immediate_size, 1);
        assert_eq!(OpCode::PUSH32.info().unwrap().immediate_size, 32);
        assert_eq!(OpCode::ADD.info().unwrap().immediate_size, 0);
    }

    #[test]
    fn opcode_count_grows_with_forks() {
        let frontier = OpCode::count_at(Fork::Frontier);
        let homestead = OpCode::count_at(Fork::Homestead);
        let cancun = OpCode::count_at(Fork::Cancun);

        assert!(homestead > frontier);
        assert!(cancun > homestead);
    }

    #[test]
    fn eip_references() {
        assert_eq!(OpCode::PUSH0.info().unwrap().eip, Some(3855));
        assert_eq!(OpCode::TLOAD.info().unwrap().eip, Some(1153));
        assert_eq!(OpCode::CREATE2.info().unwrap().eip, Some(1014));
        assert_eq!(OpCode::ADD.info().unwrap().eip, None);
    }
}
