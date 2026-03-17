//! Pattern-matchable opcode enum that wraps [`OpCode`].
//!
//! Enable with the `unified-opcodes` feature. This provides a `UnifiedOpcode`
//! enum with variants for every opcode — including parameterised `PUSH(n)`,
//! `DUP(n)`, `SWAP(n)`, and `UNKNOWN(u8)` — so that downstream crates can
//! `match` on opcodes directly.
//!
//! Conversion to/from [`OpCode`] is zero-cost via `From` impls.

use crate::{Fork, OpCode};
use std::fmt;
use std::str::FromStr;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Pattern-matchable enum covering all EVM opcodes.
///
/// This is a thin wrapper over [`OpCode`] for crates that need `match`
/// ergonomics (bytecode transforms, disassemblers, etc.). Use [`OpCode`]
/// directly when you only need metadata lookups.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum UnifiedOpcode {
    // 0x00–0x0b: Stop and Arithmetic
    /// Halts execution (0x00)
    STOP,
    /// Addition (0x01)
    ADD,
    /// Multiplication (0x02)
    MUL,
    /// Subtraction (0x03)
    SUB,
    /// Integer division (0x04)
    DIV,
    /// Signed integer division (0x05)
    SDIV,
    /// Modulo (0x06)
    MOD,
    /// Signed modulo (0x07)
    SMOD,
    /// Modulo addition (0x08)
    ADDMOD,
    /// Modulo multiplication (0x09)
    MULMOD,
    /// Exponential (0x0a)
    EXP,
    /// Sign-extend (0x0b)
    SIGNEXTEND,

    // 0x10–0x1e: Comparison & Bitwise Logic
    /// Less-than (0x10)
    LT,
    /// Greater-than (0x11)
    GT,
    /// Signed less-than (0x12)
    SLT,
    /// Signed greater-than (0x13)
    SGT,
    /// Equality (0x14)
    EQ,
    /// Is zero (0x15)
    ISZERO,
    /// Bitwise AND (0x16)
    AND,
    /// Bitwise OR (0x17)
    OR,
    /// Bitwise XOR (0x18)
    XOR,
    /// Bitwise NOT (0x19)
    NOT,
    /// Retrieve single byte (0x1a)
    BYTE,
    /// Left shift (0x1b)
    SHL,
    /// Logical right shift (0x1c)
    SHR,
    /// Arithmetic right shift (0x1d)
    SAR,
    /// Count leading zeros (0x1e)
    CLZ,

    // 0x20: SHA3
    /// Keccak-256 hash (0x20)
    KECCAK256,

    // 0x30–0x3f: Environmental Information
    /// Current address (0x30)
    ADDRESS,
    /// Account balance (0x31)
    BALANCE,
    /// Transaction origin (0x32)
    ORIGIN,
    /// Caller address (0x33)
    CALLER,
    /// Call value (0x34)
    CALLVALUE,
    /// Load calldata (0x35)
    CALLDATALOAD,
    /// Calldata size (0x36)
    CALLDATASIZE,
    /// Copy calldata to memory (0x37)
    CALLDATACOPY,
    /// Code size (0x38)
    CODESIZE,
    /// Copy code to memory (0x39)
    CODECOPY,
    /// Gas price (0x3a)
    GASPRICE,
    /// External code size (0x3b)
    EXTCODESIZE,
    /// Copy external code (0x3c)
    EXTCODECOPY,
    /// Return data size (0x3d)
    RETURNDATASIZE,
    /// Copy return data (0x3e)
    RETURNDATACOPY,
    /// External code hash (0x3f)
    EXTCODEHASH,

    // 0x40–0x4a: Block Information
    /// Block hash (0x40)
    BLOCKHASH,
    /// Coinbase address (0x41)
    COINBASE,
    /// Block timestamp (0x42)
    TIMESTAMP,
    /// Block number (0x43)
    NUMBER,
    /// Difficulty / PREVRANDAO (0x44)
    DIFFICULTY,
    /// Block gas limit (0x45)
    GASLIMIT,
    /// Chain ID (0x46)
    CHAINID,
    /// Self balance (0x47)
    SELFBALANCE,
    /// Base fee (0x48)
    BASEFEE,
    /// Blob hash (0x49)
    BLOBHASH,
    /// Blob base fee (0x4a)
    BLOBBASEFEE,

    // 0x50–0x5f: Stack, Memory, Storage and Flow
    /// Remove top of stack (0x50)
    POP,
    /// Load from memory (0x51)
    MLOAD,
    /// Store to memory (0x52)
    MSTORE,
    /// Store byte to memory (0x53)
    MSTORE8,
    /// Load from storage (0x54)
    SLOAD,
    /// Store to storage (0x55)
    SSTORE,
    /// Unconditional jump (0x56)
    JUMP,
    /// Conditional jump (0x57)
    JUMPI,
    /// Program counter (0x58)
    PC,
    /// Memory size (0x59)
    MSIZE,
    /// Available gas (0x5a)
    GAS,
    /// Jump destination marker (0x5b)
    JUMPDEST,
    /// Transient storage load (0x5c)
    TLOAD,
    /// Transient storage store (0x5d)
    TSTORE,
    /// Memory copy (0x5e)
    MCOPY,
    /// Push zero (0x5f)
    PUSH0,

    // 0x60–0x7f: Push Operations (parameterised)
    /// Push 1–32 bytes onto stack
    PUSH(u8),

    // 0x80–0x8f: Duplication Operations (parameterised)
    /// Duplicate nth stack item (1–16)
    DUP(u8),

    // 0x90–0x9f: Exchange Operations (parameterised)
    /// Swap top with nth stack item (1–16)
    SWAP(u8),

    // 0xa0–0xa4: Logging
    /// Log with 0 topics (0xa0)
    LOG0,
    /// Log with 1 topic (0xa1)
    LOG1,
    /// Log with 2 topics (0xa2)
    LOG2,
    /// Log with 3 topics (0xa3)
    LOG3,
    /// Log with 4 topics (0xa4)
    LOG4,

    // 0xd0–0xd3: EOF Data Section Access
    /// Load from data section (0xd0)
    DATALOAD,
    /// Load from data section, immediate index (0xd1)
    DATALOADN,
    /// Data section size (0xd2)
    DATASIZE,
    /// Copy from data section (0xd3)
    DATACOPY,

    // 0xe0–0xe8: EOF Control Flow & Stack
    /// Relative jump (0xe0)
    RJUMP,
    /// Relative conditional jump (0xe1)
    RJUMPI,
    /// Relative jump via table (0xe2)
    RJUMPV,
    /// Call function (0xe3)
    CALLF,
    /// Return from function (0xe4)
    RETF,
    /// Jump to function (0xe5)
    JUMPF,
    /// Duplicate nth stack item — EOF extended (0xe6)
    DUPN,
    /// Swap with nth stack item — EOF extended (0xe7)
    SWAPN,
    /// Exchange stack items (0xe8)
    EXCHANGE,

    // 0xec, 0xee: EOF Contract Creation
    /// Create EOF contract (0xec)
    EOFCREATE,
    /// Return new contract (0xee)
    RETURNCONTRACT,

    // 0xf0–0xff: System Operations
    /// Create contract (0xf0)
    CREATE,
    /// Message call (0xf1)
    CALL,
    /// Call with alternative code (0xf2)
    CALLCODE,
    /// Halt and return data (0xf3)
    RETURN,
    /// Delegate call (0xf4)
    DELEGATECALL,
    /// Create at deterministic address (0xf5)
    CREATE2,
    /// Load word from return data (0xf7)
    RETURNDATALOAD,
    /// EOF external call (0xf8)
    EXTCALL,
    /// EOF external delegate call (0xf9)
    EXTDELEGATECALL,
    /// Static message call (0xfa)
    STATICCALL,
    /// EOF external static call (0xfb)
    EXTSTATICCALL,
    /// Halt and revert state (0xfd)
    REVERT,
    /// Designated invalid (0xfe)
    INVALID,
    /// Halt and destroy account (0xff)
    SELFDESTRUCT,

    /// Unknown / unrecognised opcode byte.
    UNKNOWN(u8),
}

// ---------------------------------------------------------------------------
// Conversion: byte → UnifiedOpcode
// ---------------------------------------------------------------------------

impl UnifiedOpcode {
    /// Convert a raw byte to a `UnifiedOpcode`.
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
            0x1e => Self::CLZ,

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

            0x60..=0x7f => Self::PUSH(byte - 0x5f),
            0x80..=0x8f => Self::DUP(byte - 0x7f),
            0x90..=0x9f => Self::SWAP(byte - 0x8f),

            0xa0 => Self::LOG0,
            0xa1 => Self::LOG1,
            0xa2 => Self::LOG2,
            0xa3 => Self::LOG3,
            0xa4 => Self::LOG4,

            0xd0 => Self::DATALOAD,
            0xd1 => Self::DATALOADN,
            0xd2 => Self::DATASIZE,
            0xd3 => Self::DATACOPY,

            0xe0 => Self::RJUMP,
            0xe1 => Self::RJUMPI,
            0xe2 => Self::RJUMPV,
            0xe3 => Self::CALLF,
            0xe4 => Self::RETF,
            0xe5 => Self::JUMPF,
            0xe6 => Self::DUPN,
            0xe7 => Self::SWAPN,
            0xe8 => Self::EXCHANGE,

            0xec => Self::EOFCREATE,
            0xee => Self::RETURNCONTRACT,

            0xf0 => Self::CREATE,
            0xf1 => Self::CALL,
            0xf2 => Self::CALLCODE,
            0xf3 => Self::RETURN,
            0xf4 => Self::DELEGATECALL,
            0xf5 => Self::CREATE2,
            0xf7 => Self::RETURNDATALOAD,
            0xf8 => Self::EXTCALL,
            0xf9 => Self::EXTDELEGATECALL,
            0xfa => Self::STATICCALL,
            0xfb => Self::EXTSTATICCALL,
            0xfd => Self::REVERT,
            0xfe => Self::INVALID,
            0xff => Self::SELFDESTRUCT,

            _ => Self::UNKNOWN(byte),
        }
    }

    /// Convert back to a raw byte.
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
            Self::CLZ => 0x1e,

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

            Self::DATALOAD => 0xd0,
            Self::DATALOADN => 0xd1,
            Self::DATASIZE => 0xd2,
            Self::DATACOPY => 0xd3,

            Self::RJUMP => 0xe0,
            Self::RJUMPI => 0xe1,
            Self::RJUMPV => 0xe2,
            Self::CALLF => 0xe3,
            Self::RETF => 0xe4,
            Self::JUMPF => 0xe5,
            Self::DUPN => 0xe6,
            Self::SWAPN => 0xe7,
            Self::EXCHANGE => 0xe8,

            Self::EOFCREATE => 0xec,
            Self::RETURNCONTRACT => 0xee,

            Self::CREATE => 0xf0,
            Self::CALL => 0xf1,
            Self::CALLCODE => 0xf2,
            Self::RETURN => 0xf3,
            Self::DELEGATECALL => 0xf4,
            Self::CREATE2 => 0xf5,
            Self::RETURNDATALOAD => 0xf7,
            Self::EXTCALL => 0xf8,
            Self::EXTDELEGATECALL => 0xf9,
            Self::STATICCALL => 0xfa,
            Self::EXTSTATICCALL => 0xfb,
            Self::REVERT => 0xfd,
            Self::INVALID => 0xfe,
            Self::SELFDESTRUCT => 0xff,

            Self::UNKNOWN(b) => *b,
        }
    }

    /// Parse a byte, returning `(opcode, immediate_data_size)`.
    pub fn parse(byte: u8) -> (Self, usize) {
        let op = Self::from_byte(byte);
        let imm = match &op {
            Self::PUSH(n) => *n as usize,
            _ => {
                // fall back to OpCode table for EOF immediates etc.
                OpCode::from_byte(byte)
                    .info()
                    .map(|i| i.immediate_size as usize)
                    .unwrap_or(0)
            }
        };
        (op, imm)
    }

    /// Parse with fork validation — returns `UNKNOWN` if not valid at `fork`.
    pub fn parse_with_fork(byte: u8, fork: Fork) -> (Self, usize) {
        let op = OpCode::from_byte(byte);
        if op.is_valid_in(fork) {
            Self::parse(byte)
        } else {
            (Self::UNKNOWN(byte), 0)
        }
    }

    /// Returns `true` if this is an unknown / unrecognised opcode.
    pub fn is_unknown(&self) -> bool {
        matches!(self, Self::UNKNOWN(_))
    }

    /// Returns `true` if this opcode affects control flow.
    pub fn is_control_flow(&self) -> bool {
        OpCode::from_byte(self.to_byte()).is_control_flow()
    }

    /// Get the underlying [`OpCode`] for metadata lookups.
    pub fn as_opcode(&self) -> OpCode {
        OpCode::from_byte(self.to_byte())
    }

    /// Get opcode name as a string.
    pub fn name(&self) -> String {
        match self {
            Self::PUSH0 => "PUSH0".into(),
            Self::PUSH(n) => format!("PUSH{n}"),
            Self::DUP(n) => format!("DUP{n}"),
            Self::SWAP(n) => format!("SWAP{n}"),
            Self::UNKNOWN(b) => format!("UNKNOWN(0x{b:02x})"),
            other => format!("{other:?}"),
        }
    }
}

// ---------------------------------------------------------------------------
// Conversions: OpCode ↔ UnifiedOpcode
// ---------------------------------------------------------------------------

impl From<OpCode> for UnifiedOpcode {
    fn from(op: OpCode) -> Self {
        Self::from_byte(op.byte())
    }
}

impl From<UnifiedOpcode> for OpCode {
    fn from(op: UnifiedOpcode) -> Self {
        OpCode::from_byte(op.to_byte())
    }
}

impl From<u8> for UnifiedOpcode {
    fn from(byte: u8) -> Self {
        Self::from_byte(byte)
    }
}

impl From<UnifiedOpcode> for u8 {
    fn from(op: UnifiedOpcode) -> Self {
        op.to_byte()
    }
}

// ---------------------------------------------------------------------------
// Display / FromStr
// ---------------------------------------------------------------------------

impl fmt::Display for UnifiedOpcode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl FromStr for UnifiedOpcode {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Try the core OpCode parser first (handles all known names + aliases)
        if let Ok(op) = s.parse::<OpCode>() {
            return Ok(Self::from(op));
        }

        // Handle parameterised names: PUSH1–PUSH32, DUP1–DUP16, SWAP1–SWAP16
        if let Some(rest) = s.strip_prefix("PUSH") {
            if rest == "0" {
                return Ok(Self::PUSH0);
            }
            return rest
                .parse::<u8>()
                .ok()
                .filter(|&n| (1..=32).contains(&n))
                .map(Self::PUSH)
                .ok_or_else(|| format!("invalid PUSH opcode: {s}"));
        }
        if let Some(rest) = s.strip_prefix("DUP") {
            return rest
                .parse::<u8>()
                .ok()
                .filter(|&n| (1..=16).contains(&n))
                .map(Self::DUP)
                .ok_or_else(|| format!("invalid DUP opcode: {s}"));
        }
        if let Some(rest) = s.strip_prefix("SWAP") {
            return rest
                .parse::<u8>()
                .ok()
                .filter(|&n| (1..=16).contains(&n))
                .map(Self::SWAP)
                .ok_or_else(|| format!("invalid SWAP opcode: {s}"));
        }

        Err(format!("unknown opcode: {s}"))
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn byte_roundtrip() {
        for b in 0u8..=255 {
            let op = UnifiedOpcode::from_byte(b);
            assert_eq!(op.to_byte(), b, "roundtrip failed for 0x{b:02x}");
        }
    }

    #[test]
    fn unknown_detection() {
        assert!(!UnifiedOpcode::ADD.is_unknown());
        assert!(UnifiedOpcode::UNKNOWN(0xcc).is_unknown());
        assert!(UnifiedOpcode::from_byte(0x0c).is_unknown());
    }

    #[test]
    fn push_parameterised() {
        assert_eq!(UnifiedOpcode::from_byte(0x60), UnifiedOpcode::PUSH(1));
        assert_eq!(UnifiedOpcode::from_byte(0x7f), UnifiedOpcode::PUSH(32));
        assert_eq!(UnifiedOpcode::PUSH(1).to_byte(), 0x60);
        assert_eq!(UnifiedOpcode::PUSH(32).to_byte(), 0x7f);
    }

    #[test]
    fn parse_immediate_size() {
        let (op, imm) = UnifiedOpcode::parse(0x60);
        assert_eq!(op, UnifiedOpcode::PUSH(1));
        assert_eq!(imm, 1);

        let (op, imm) = UnifiedOpcode::parse(0x7f);
        assert_eq!(op, UnifiedOpcode::PUSH(32));
        assert_eq!(imm, 32);

        let (op, imm) = UnifiedOpcode::parse(0x01);
        assert_eq!(op, UnifiedOpcode::ADD);
        assert_eq!(imm, 0);
    }

    #[test]
    fn from_str_roundtrip() {
        let cases = ["PUSH1", "PUSH32", "PUSH0", "DUP1", "DUP16", "SWAP1", "SWAP16", "ADD", "STOP"];
        for name in cases {
            let op: UnifiedOpcode = name.parse().unwrap();
            assert_eq!(op.name(), name, "roundtrip failed for {name}");
        }
    }

    #[test]
    fn opcode_conversion() {
        let unified = UnifiedOpcode::ADD;
        let core: OpCode = unified.into();
        assert_eq!(core, OpCode::ADD);

        let back: UnifiedOpcode = core.into();
        assert_eq!(back, UnifiedOpcode::ADD);
    }

    #[test]
    fn display() {
        assert_eq!(UnifiedOpcode::STOP.to_string(), "STOP");
        assert_eq!(UnifiedOpcode::PUSH(1).to_string(), "PUSH1");
        assert_eq!(UnifiedOpcode::DUP(5).to_string(), "DUP5");
        assert_eq!(UnifiedOpcode::UNKNOWN(0xcc).to_string(), "UNKNOWN(0xcc)");
    }
}
