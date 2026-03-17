//! Tests for opcode parsing, roundtrip, and classification.

use eot::{Fork, OpCode};
use std::str::FromStr;

#[test]
fn parse_basic_opcodes() {
    assert_eq!(OpCode::new(0x00), Some(OpCode::STOP));
    assert_eq!(OpCode::new(0x60), Some(OpCode::PUSH1));
    assert_eq!(OpCode::new(0x7f), Some(OpCode::PUSH32));
}

#[test]
fn from_str_known() {
    assert_eq!(OpCode::from_str("PUSH1").unwrap(), OpCode::PUSH1);
    assert_eq!(OpCode::from_str("DUP1").unwrap(), OpCode::DUP1);
    assert_eq!(OpCode::from_str("SWAP1").unwrap(), OpCode::SWAP1);
    assert_eq!(OpCode::from_str("STOP").unwrap(), OpCode::STOP);
    assert!(OpCode::from_str("INVALID_OP").is_err());
}

#[test]
fn control_flow() {
    assert!(OpCode::JUMP.is_control_flow());
    assert!(OpCode::JUMPI.is_control_flow());
    assert!(OpCode::RETURN.is_control_flow());
    assert!(!OpCode::ADD.is_control_flow());
    assert!(!OpCode::PUSH1.is_control_flow());
}

#[test]
fn byte_roundtrip() {
    for byte in 0u8..=255 {
        assert_eq!(OpCode::from_byte(byte).byte(), byte);
    }
}

#[test]
fn push_variants() {
    assert_eq!(OpCode::PUSH0.byte(), 0x5f);
    assert!(OpCode::PUSH0.is_push());
    assert_eq!(OpCode::PUSH0.info().unwrap().immediate_size, 0);

    for i in 1u8..=32 {
        let op = OpCode::new(0x5f + i).unwrap();
        assert!(op.is_push());
        assert_eq!(op.info().unwrap().immediate_size, i);
    }
}

#[test]
fn dup_and_swap() {
    for i in 0u8..16 {
        let dup = OpCode::new(0x80 + i).unwrap();
        assert!(dup.is_dup());
        assert!(!dup.is_swap());

        let swap = OpCode::new(0x90 + i).unwrap();
        assert!(swap.is_swap());
        assert!(!swap.is_dup());
    }
}

#[test]
fn edge_cases() {
    // Unknown bytes
    assert!(OpCode::new(0x0c).is_none());
    assert!(OpCode::new(0xef).is_none());

    // from_byte always works
    let unknown = OpCode::from_byte(0x0c);
    assert!(unknown.info().is_none());
    assert_eq!(unknown.name(), "UNKNOWN");
}

#[test]
fn display_formatting() {
    assert_eq!(OpCode::STOP.to_string(), "STOP");
    assert_eq!(OpCode::PUSH1.to_string(), "PUSH1");
    assert_eq!(OpCode::DUP5.to_string(), "DUP5");
    assert_eq!(OpCode::SWAP10.to_string(), "SWAP10");
    assert!(OpCode::from_byte(0x0c).to_string().contains("UNKNOWN"));
}

#[test]
fn metadata_access() {
    let info = OpCode::ADD.info().unwrap();
    assert_eq!(info.name, "ADD");
    assert_eq!(info.inputs, 2);
    assert_eq!(info.outputs, 1);
    assert_eq!(info.base_gas, 3);
    assert_eq!(info.introduced_in, Fork::Frontier);
}

#[test]
fn from_str_aliases() {
    assert_eq!(OpCode::from_str("SHA3").unwrap(), OpCode::KECCAK256);
    assert_eq!(OpCode::from_str("PREVRANDAO").unwrap(), OpCode::DIFFICULTY);
}

#[test]
fn from_str_roundtrip_all() {
    for op in OpCode::iter_all() {
        let name = op.name();
        let parsed: OpCode = name.parse().unwrap();
        assert_eq!(parsed, op, "roundtrip failed for {name}");
    }
}
