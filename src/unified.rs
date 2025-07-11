//! Unified opcode interface for bytecode manipulation tools
//!
//! This module provides a simplified, fork-agnostic interface to EVM opcodes
//! that's perfect for bytecode analysis and manipulation tools like obfuscators,
//! analyzers, and parsers.

use crate::{Fork, OpcodeRegistry};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use std::fmt;
use std::str::FromStr;

/// Unified opcode enum that abstracts away fork-specific details
///
/// This enum provides a simple interface for bytecode manipulation tools
/// while still allowing access to the rich metadata from the underlying
/// fork-specific implementations when needed.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum UnifiedOpcode {
    // 0x00-0x0f: Stop and Arithmetic Operations
    /// Halts execution (0x00)
    STOP,
    /// Addition operation (0x01)
    ADD,
    /// Multiplication operation (0x02)
    MUL,
    /// Subtraction operation (0x03)
    SUB,
    /// Integer division operation (0x04)
    DIV,
    /// Signed integer division operation (0x05)
    SDIV,
    /// Modulo remainder operation (0x06)
    MOD,
    /// Signed modulo remainder operation (0x07)
    SMOD,
    /// Modulo addition operation (0x08)
    ADDMOD,
    /// Modulo multiplication operation (0x09)
    MULMOD,
    /// Exponential operation (0x0a)
    EXP,
    /// Extend length of two's complement signed integer (0x0b)
    SIGNEXTEND,

    // 0x10-0x1f: Comparison & Bitwise Logic Operations
    /// Less-than comparison (0x10)
    LT,
    /// Greater-than comparison (0x11)
    GT,
    /// Signed less-than comparison (0x12)
    SLT,
    /// Signed greater-than comparison (0x13)
    SGT,
    /// Equality comparison (0x14)
    EQ,
    /// Simple not operator (0x15)
    ISZERO,
    /// Bitwise AND operation (0x16)
    AND,
    /// Bitwise OR operation (0x17)
    OR,
    /// Bitwise XOR operation (0x18)
    XOR,
    /// Bitwise NOT operation (0x19)
    NOT,
    /// Retrieve single byte from word (0x1a)
    BYTE,
    /// Left shift operation (0x1b)
    SHL,
    /// Logical right shift operation (0x1c)
    SHR,
    /// Arithmetic right shift operation (0x1d)
    SAR,

    // 0x20: SHA3
    /// Compute Keccak-256 hash (0x20)
    KECCAK256,

    // 0x30-0x3f: Environmental Information
    /// Get address of currently executing account (0x30)
    ADDRESS,
    /// Get balance of the given account (0x31)
    BALANCE,
    /// Get execution origination address (0x32)
    ORIGIN,
    /// Get caller address (0x33)
    CALLER,
    /// Get deposited value by the instruction/transaction responsible for this execution (0x34)
    CALLVALUE,
    /// Get input data in current environment (0x35)
    CALLDATALOAD,
    /// Get size of input data in current environment (0x36)
    CALLDATASIZE,
    /// Copy input data in current environment to memory (0x37)
    CALLDATACOPY,
    /// Get size of code running in current environment (0x38)
    CODESIZE,
    /// Copy code running in current environment to memory (0x39)
    CODECOPY,
    /// Get price of gas in current environment (0x3a)
    GASPRICE,
    /// Get size of an account's code (0x3b)
    EXTCODESIZE,
    /// Copy an account's code to memory (0x3c)
    EXTCODECOPY,
    /// Get size of output data from the previous call from the current environment (0x3d)
    RETURNDATASIZE,
    /// Copy output data from the previous call to memory (0x3e)
    RETURNDATACOPY,
    /// Get hash of an account's code (0x3f)
    EXTCODEHASH,

    // 0x40-0x4f: Block Information
    /// Get the hash of one of the 256 most recent complete blocks (0x40)
    BLOCKHASH,
    /// Get the block's beneficiary address (0x41)
    COINBASE,
    /// Get the block's timestamp (0x42)
    TIMESTAMP,
    /// Get the block's number (0x43)
    NUMBER,
    /// Get the block's difficulty (pre-Merge) / previous beacon chain randomness (post-Merge) (0x44)
    DIFFICULTY,
    /// Get the block's gas limit (0x45)
    GASLIMIT,
    /// Get the chain ID (0x46)
    CHAINID,
    /// Get balance of currently executing account (0x47)
    SELFBALANCE,
    /// Get the base fee (0x48)
    BASEFEE,
    /// Get the versioned hash of the i-th blob (0x49, Cancun)
    BLOBHASH,
    /// Get the blob base fee (0x4a, Cancun)
    BLOBBASEFEE,

    // 0x50-0x5f: Stack, Memory, Storage and Flow Operations
    /// Remove item from stack (0x50)
    POP,
    /// Load word from memory (0x51)
    MLOAD,
    /// Save word to memory (0x52)
    MSTORE,
    /// Save byte to memory (0x53)
    MSTORE8,
    /// Load word from storage (0x54)
    SLOAD,
    /// Save word to storage (0x55)
    SSTORE,
    /// Alter the program counter (0x56)
    JUMP,
    /// Conditionally alter the program counter (0x57)
    JUMPI,
    /// Get the value of the program counter prior to the increment (0x58)
    PC,
    /// Get the size of active memory in bytes (0x59)
    MSIZE,
    /// Get the amount of available gas (0x5a)
    GAS,
    /// Mark a valid destination for jumps (0x5b)
    JUMPDEST,
    /// Load word from transient storage (0x5c, Cancun)
    TLOAD,
    /// Save word to transient storage (0x5d, Cancun)
    TSTORE,
    /// Copy memory from one location to another (0x5e, Cancun)
    MCOPY,
    /// Place 0 byte item on stack (0x5f, Shanghai)
    PUSH0,

    // 0x60-0x7f: Push Operations
    /// Place n-byte item on stack (PUSH1-PUSH32, 1-32 bytes)
    PUSH(u8),

    // 0x80-0x8f: Duplication Operations
    /// Duplicate nth stack item (DUP1-DUP16)
    DUP(u8),

    // 0x90-0x9f: Exchange Operations
    /// Exchange 1st and nth stack items (SWAP1-SWAP16)
    SWAP(u8),

    // 0xa0-0xa4: Logging Operations
    /// Append log record with no topics (0xa0)
    LOG0,
    /// Append log record with one topic (0xa1)
    LOG1,
    /// Append log record with two topics (0xa2)
    LOG2,
    /// Append log record with three topics (0xa3)
    LOG3,
    /// Append log record with four topics (0xa4)
    LOG4,

    // 0xf0-0xff: System Operations
    /// Create a new account with associated code (0xf0)
    CREATE,
    /// Message-call into an account (0xf1)
    CALL,
    /// Message-call into this account with alternative account's code (0xf2)
    CALLCODE,
    /// Halt execution returning output data (0xf3)
    RETURN,
    /// Message-call into an account with caller's code (0xf4)
    DELEGATECALL,
    /// Create a new account with associated code at a predictable address (0xf5)
    CREATE2,
    /// Static message-call into an account (0xfa)
    STATICCALL,
    /// Halt execution reverting state changes (0xfd)
    REVERT,
    /// Designated invalid instruction (0xfe)
    INVALID,
    /// Halt execution and register account for later deletion (0xff)
    SELFDESTRUCT,

    /// Catch-all for unknown or unsupported opcodes
    UNKNOWN(u8),
}

impl UnifiedOpcode {
    /// Parse a byte into a unified opcode with immediate data size
    /// Uses the latest fork (Cancun) by default for maximum compatibility
    ///
    /// # Returns
    /// A tuple of (opcode, immediate_data_size)
    ///
    /// # Examples
    /// ```
    /// use eot::UnifiedOpcode;
    ///
    /// let (opcode, imm_size) = UnifiedOpcode::parse(0x60);
    /// assert_eq!(opcode, UnifiedOpcode::PUSH(1));
    /// assert_eq!(imm_size, 1);
    /// ```
    pub fn parse(byte: u8) -> (Self, usize) {
        Self::parse_with_fork(byte, Fork::Cancun)
    }

    /// Parse a byte into a unified opcode for a specific fork
    pub fn parse_with_fork(byte: u8, fork: Fork) -> (Self, usize) {
        let registry = OpcodeRegistry::new();

        if registry.is_opcode_available(fork, byte) {
            let unified = Self::from_byte(byte);
            let imm_size = Self::immediate_size(&unified);
            (unified, imm_size)
        } else {
            (Self::UNKNOWN(byte), 0)
        }
    }

    /// Convert a byte directly to a unified opcode (no fork checking)
    /// This is faster but doesn't validate fork compatibility
    pub fn from_byte(byte: u8) -> Self {
        match byte {
            0x00 => Self::STOP,
            0x01 => Self::ADD,
            0x02 => Self::MUL,
            0x03 => Self::SUB,
            0x04 => Self::DIV,
            0x05 => Self::SDIV,
            0x06 => Self::MOD,
            0x07 => Self::SMOD,
            0x08 => Self::ADDMOD,
            0x09 => Self::MULMOD,
            0x0a => Self::EXP,
            0x0b => Self::SIGNEXTEND,

            0x10 => Self::LT,
            0x11 => Self::GT,
            0x12 => Self::SLT,
            0x13 => Self::SGT,
            0x14 => Self::EQ,
            0x15 => Self::ISZERO,
            0x16 => Self::AND,
            0x17 => Self::OR,
            0x18 => Self::XOR,
            0x19 => Self::NOT,
            0x1a => Self::BYTE,
            0x1b => Self::SHL,
            0x1c => Self::SHR,
            0x1d => Self::SAR,

            0x20 => Self::KECCAK256,

            0x30 => Self::ADDRESS,
            0x31 => Self::BALANCE,
            0x32 => Self::ORIGIN,
            0x33 => Self::CALLER,
            0x34 => Self::CALLVALUE,
            0x35 => Self::CALLDATALOAD,
            0x36 => Self::CALLDATASIZE,
            0x37 => Self::CALLDATACOPY,
            0x38 => Self::CODESIZE,
            0x39 => Self::CODECOPY,
            0x3a => Self::GASPRICE,
            0x3b => Self::EXTCODESIZE,
            0x3c => Self::EXTCODECOPY,
            0x3d => Self::RETURNDATASIZE,
            0x3e => Self::RETURNDATACOPY,
            0x3f => Self::EXTCODEHASH,

            0x40 => Self::BLOCKHASH,
            0x41 => Self::COINBASE,
            0x42 => Self::TIMESTAMP,
            0x43 => Self::NUMBER,
            0x44 => Self::DIFFICULTY,
            0x45 => Self::GASLIMIT,
            0x46 => Self::CHAINID,
            0x47 => Self::SELFBALANCE,
            0x48 => Self::BASEFEE,
            0x49 => Self::BLOBHASH,
            0x4a => Self::BLOBBASEFEE,

            0x50 => Self::POP,
            0x51 => Self::MLOAD,
            0x52 => Self::MSTORE,
            0x53 => Self::MSTORE8,
            0x54 => Self::SLOAD,
            0x55 => Self::SSTORE,
            0x56 => Self::JUMP,
            0x57 => Self::JUMPI,
            0x58 => Self::PC,
            0x59 => Self::MSIZE,
            0x5a => Self::GAS,
            0x5b => Self::JUMPDEST,
            0x5c => Self::TLOAD,
            0x5d => Self::TSTORE,
            0x5e => Self::MCOPY,
            0x5f => Self::PUSH0,

            0x60..=0x7f => Self::PUSH(byte - 0x5f), // PUSH1 to PUSH32
            0x80..=0x8f => Self::DUP(byte - 0x7f),  // DUP1 to DUP16
            0x90..=0x9f => Self::SWAP(byte - 0x8f), // SWAP1 to SWAP16

            0xa0 => Self::LOG0,
            0xa1 => Self::LOG1,
            0xa2 => Self::LOG2,
            0xa3 => Self::LOG3,
            0xa4 => Self::LOG4,

            0xf0 => Self::CREATE,
            0xf1 => Self::CALL,
            0xf2 => Self::CALLCODE,
            0xf3 => Self::RETURN,
            0xf4 => Self::DELEGATECALL,
            0xf5 => Self::CREATE2,
            0xfa => Self::STATICCALL,
            0xfd => Self::REVERT,
            0xfe => Self::INVALID,
            0xff => Self::SELFDESTRUCT,

            _ => Self::UNKNOWN(byte),
        }
    }

    /// Convert back to byte representation
    pub fn to_byte(&self) -> u8 {
        match self {
            Self::STOP => 0x00,
            Self::ADD => 0x01,
            Self::MUL => 0x02,
            Self::SUB => 0x03,
            Self::DIV => 0x04,
            Self::SDIV => 0x05,
            Self::MOD => 0x06,
            Self::SMOD => 0x07,
            Self::ADDMOD => 0x08,
            Self::MULMOD => 0x09,
            Self::EXP => 0x0a,
            Self::SIGNEXTEND => 0x0b,

            Self::LT => 0x10,
            Self::GT => 0x11,
            Self::SLT => 0x12,
            Self::SGT => 0x13,
            Self::EQ => 0x14,
            Self::ISZERO => 0x15,
            Self::AND => 0x16,
            Self::OR => 0x17,
            Self::XOR => 0x18,
            Self::NOT => 0x19,
            Self::BYTE => 0x1a,
            Self::SHL => 0x1b,
            Self::SHR => 0x1c,
            Self::SAR => 0x1d,

            Self::KECCAK256 => 0x20,

            Self::ADDRESS => 0x30,
            Self::BALANCE => 0x31,
            Self::ORIGIN => 0x32,
            Self::CALLER => 0x33,
            Self::CALLVALUE => 0x34,
            Self::CALLDATALOAD => 0x35,
            Self::CALLDATASIZE => 0x36,
            Self::CALLDATACOPY => 0x37,
            Self::CODESIZE => 0x38,
            Self::CODECOPY => 0x39,
            Self::GASPRICE => 0x3a,
            Self::EXTCODESIZE => 0x3b,
            Self::EXTCODECOPY => 0x3c,
            Self::RETURNDATASIZE => 0x3d,
            Self::RETURNDATACOPY => 0x3e,
            Self::EXTCODEHASH => 0x3f,

            Self::BLOCKHASH => 0x40,
            Self::COINBASE => 0x41,
            Self::TIMESTAMP => 0x42,
            Self::NUMBER => 0x43,
            Self::DIFFICULTY => 0x44,
            Self::GASLIMIT => 0x45,
            Self::CHAINID => 0x46,
            Self::SELFBALANCE => 0x47,
            Self::BASEFEE => 0x48,
            Self::BLOBHASH => 0x49,
            Self::BLOBBASEFEE => 0x4a,

            Self::POP => 0x50,
            Self::MLOAD => 0x51,
            Self::MSTORE => 0x52,
            Self::MSTORE8 => 0x53,
            Self::SLOAD => 0x54,
            Self::SSTORE => 0x55,
            Self::JUMP => 0x56,
            Self::JUMPI => 0x57,
            Self::PC => 0x58,
            Self::MSIZE => 0x59,
            Self::GAS => 0x5a,
            Self::JUMPDEST => 0x5b,
            Self::TLOAD => 0x5c,
            Self::TSTORE => 0x5d,
            Self::MCOPY => 0x5e,
            Self::PUSH0 => 0x5f,

            Self::PUSH(n) => 0x5f + n,
            Self::DUP(n) => 0x7f + n,
            Self::SWAP(n) => 0x8f + n,

            Self::LOG0 => 0xa0,
            Self::LOG1 => 0xa1,
            Self::LOG2 => 0xa2,
            Self::LOG3 => 0xa3,
            Self::LOG4 => 0xa4,

            Self::CREATE => 0xf0,
            Self::CALL => 0xf1,
            Self::CALLCODE => 0xf2,
            Self::RETURN => 0xf3,
            Self::DELEGATECALL => 0xf4,
            Self::CREATE2 => 0xf5,
            Self::STATICCALL => 0xfa,
            Self::REVERT => 0xfd,
            Self::INVALID => 0xfe,
            Self::SELFDESTRUCT => 0xff,

            Self::UNKNOWN(byte) => *byte,
        }
    }

    /// Check if this opcode affects control flow (for CFG construction)
    pub fn is_control_flow(&self) -> bool {
        matches!(
            self,
            Self::STOP
                | Self::JUMP
                | Self::JUMPI
                | Self::JUMPDEST
                | Self::RETURN
                | Self::REVERT
                | Self::INVALID
                | Self::SELFDESTRUCT
                | Self::CREATE
                | Self::CREATE2
                | Self::CALL
                | Self::CALLCODE
                | Self::DELEGATECALL
                | Self::STATICCALL
        )
    }

    /// Get the name of this opcode as a string
    pub fn name(&self) -> String {
        match self {
            Self::PUSH0 => "PUSH0".to_string(),
            Self::PUSH(n) => format!("PUSH{n}"),
            Self::DUP(n) => format!("DUP{n}"),
            Self::SWAP(n) => format!("SWAP{n}"),
            Self::UNKNOWN(byte) => format!("UNKNOWN{byte:02x}"),
            _ => {
                // For known opcodes, use debug formatting and extract the name
                format!("{self:?}")
            }
        }
    }

    /// Get immediate data size for this opcode
    fn immediate_size(opcode: &Self) -> usize {
        match opcode {
            Self::PUSH(n) => *n as usize,
            Self::PUSH0 => 0,
            _ => 0,
        }
    }

    /// Get metadata for this opcode from the registry for a specific fork
    pub fn metadata(&self, fork: Fork) -> Option<crate::OpcodeMetadata> {
        let registry = OpcodeRegistry::new();
        let opcodes = registry.get_opcodes(fork);
        opcodes.get(&self.to_byte()).cloned()
    }

    /// Get metadata using the latest fork (Cancun)
    pub fn metadata_latest(&self) -> Option<crate::OpcodeMetadata> {
        self.metadata(Fork::Cancun)
    }
}

impl fmt::Display for UnifiedOpcode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl From<u8> for UnifiedOpcode {
    fn from(byte: u8) -> Self {
        Self::from_byte(byte)
    }
}

impl From<UnifiedOpcode> for u8 {
    fn from(opcode: UnifiedOpcode) -> Self {
        opcode.to_byte()
    }
}

impl FromStr for UnifiedOpcode {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "STOP" => Ok(Self::STOP),
            "ADD" => Ok(Self::ADD),
            "MUL" => Ok(Self::MUL),
            "SUB" => Ok(Self::SUB),
            "DIV" => Ok(Self::DIV),
            "SDIV" => Ok(Self::SDIV),
            "MOD" => Ok(Self::MOD),
            "SMOD" => Ok(Self::SMOD),
            "ADDMOD" => Ok(Self::ADDMOD),
            "MULMOD" => Ok(Self::MULMOD),
            "EXP" => Ok(Self::EXP),
            "SIGNEXTEND" => Ok(Self::SIGNEXTEND),

            "LT" => Ok(Self::LT),
            "GT" => Ok(Self::GT),
            "SLT" => Ok(Self::SLT),
            "SGT" => Ok(Self::SGT),
            "EQ" => Ok(Self::EQ),
            "ISZERO" => Ok(Self::ISZERO),
            "AND" => Ok(Self::AND),
            "OR" => Ok(Self::OR),
            "XOR" => Ok(Self::XOR),
            "NOT" => Ok(Self::NOT),
            "BYTE" => Ok(Self::BYTE),
            "SHL" => Ok(Self::SHL),
            "SHR" => Ok(Self::SHR),
            "SAR" => Ok(Self::SAR),

            "SHA3" => Ok(Self::KECCAK256), // Accept both names
            "KECCAK256" => Ok(Self::KECCAK256),

            "ADDRESS" => Ok(Self::ADDRESS),
            "BALANCE" => Ok(Self::BALANCE),
            "ORIGIN" => Ok(Self::ORIGIN),
            "CALLER" => Ok(Self::CALLER),
            "CALLVALUE" => Ok(Self::CALLVALUE),
            "CALLDATALOAD" => Ok(Self::CALLDATALOAD),
            "CALLDATASIZE" => Ok(Self::CALLDATASIZE),
            "CALLDATACOPY" => Ok(Self::CALLDATACOPY),
            "CODESIZE" => Ok(Self::CODESIZE),
            "CODECOPY" => Ok(Self::CODECOPY),
            "GASPRICE" => Ok(Self::GASPRICE),
            "EXTCODESIZE" => Ok(Self::EXTCODESIZE),
            "EXTCODECOPY" => Ok(Self::EXTCODECOPY),
            "RETURNDATASIZE" => Ok(Self::RETURNDATASIZE),
            "RETURNDATACOPY" => Ok(Self::RETURNDATACOPY),
            "EXTCODEHASH" => Ok(Self::EXTCODEHASH),

            "BLOCKHASH" => Ok(Self::BLOCKHASH),
            "COINBASE" => Ok(Self::COINBASE),
            "TIMESTAMP" => Ok(Self::TIMESTAMP),
            "NUMBER" => Ok(Self::NUMBER),
            "DIFFICULTY" => Ok(Self::DIFFICULTY),
            "PREVRANDAO" => Ok(Self::DIFFICULTY), // EIP-4399: Same opcode, different semantic meaning
            "GASLIMIT" => Ok(Self::GASLIMIT),
            "CHAINID" => Ok(Self::CHAINID),
            "SELFBALANCE" => Ok(Self::SELFBALANCE),
            "BASEFEE" => Ok(Self::BASEFEE),
            "BLOBHASH" => Ok(Self::BLOBHASH),
            "BLOBBASEFEE" => Ok(Self::BLOBBASEFEE),

            "POP" => Ok(Self::POP),
            "MLOAD" => Ok(Self::MLOAD),
            "MSTORE" => Ok(Self::MSTORE),
            "MSTORE8" => Ok(Self::MSTORE8),
            "SLOAD" => Ok(Self::SLOAD),
            "SSTORE" => Ok(Self::SSTORE),
            "JUMP" => Ok(Self::JUMP),
            "JUMPI" => Ok(Self::JUMPI),
            "PC" => Ok(Self::PC),
            "MSIZE" => Ok(Self::MSIZE),
            "GAS" => Ok(Self::GAS),
            "JUMPDEST" => Ok(Self::JUMPDEST),
            "TLOAD" => Ok(Self::TLOAD),
            "TSTORE" => Ok(Self::TSTORE),
            "MCOPY" => Ok(Self::MCOPY),
            "PUSH0" => Ok(Self::PUSH0),

            "LOG0" => Ok(Self::LOG0),
            "LOG1" => Ok(Self::LOG1),
            "LOG2" => Ok(Self::LOG2),
            "LOG3" => Ok(Self::LOG3),
            "LOG4" => Ok(Self::LOG4),

            "CREATE" => Ok(Self::CREATE),
            "CALL" => Ok(Self::CALL),
            "CALLCODE" => Ok(Self::CALLCODE),
            "RETURN" => Ok(Self::RETURN),
            "DELEGATECALL" => Ok(Self::DELEGATECALL),
            "CREATE2" => Ok(Self::CREATE2),
            "STATICCALL" => Ok(Self::STATICCALL),
            "REVERT" => Ok(Self::REVERT),
            "INVALID" => Ok(Self::INVALID),
            "SELFDESTRUCT" => Ok(Self::SELFDESTRUCT),

            // Handle PUSH, DUP, SWAP with numbers
            s if s.starts_with("PUSH") => {
                if s == "PUSH0" {
                    return Ok(Self::PUSH0);
                }
                s.strip_prefix("PUSH")
                    .and_then(|n_str| n_str.parse::<u8>().ok())
                    .filter(|&n| (1..=32).contains(&n))
                    .map(Self::PUSH)
                    .ok_or_else(|| format!("Invalid PUSH opcode: {s}"))
            }
            s if s.starts_with("DUP") => s
                .strip_prefix("DUP")
                .and_then(|n_str| n_str.parse::<u8>().ok())
                .filter(|&n| (1..=16).contains(&n))
                .map(Self::DUP)
                .ok_or_else(|| format!("Invalid DUP opcode: {s}")),
            s if s.starts_with("SWAP") => s
                .strip_prefix("SWAP")
                .and_then(|n_str| n_str.parse::<u8>().ok())
                .filter(|&n| (1..=16).contains(&n))
                .map(Self::SWAP)
                .ok_or_else(|| format!("Invalid SWAP opcode: {s}")),

            _ => Err(format!("Unknown opcode: {s}")),
        }
    }
}
