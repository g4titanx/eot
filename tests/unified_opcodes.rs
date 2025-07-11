//! Integration tests for unified opcodes feature
//! These tests run when the unified-opcodes feature is enabled

#![cfg(feature = "unified-opcodes")]

use eot::UnifiedOpcode;
use std::str::FromStr;

#[test]
fn test_parse_basic_opcodes() {
    let (opcode, size) = UnifiedOpcode::parse(0x00);
    assert_eq!(opcode, UnifiedOpcode::STOP);
    assert_eq!(size, 0);

    let (opcode, size) = UnifiedOpcode::parse(0x60);
    assert_eq!(opcode, UnifiedOpcode::PUSH(1));
    assert_eq!(size, 1);

    let (opcode, size) = UnifiedOpcode::parse(0x7f);
    assert_eq!(opcode, UnifiedOpcode::PUSH(32));
    assert_eq!(size, 32);
}

#[test]
fn test_from_str() {
    assert_eq!(
        UnifiedOpcode::from_str("PUSH1").unwrap(),
        UnifiedOpcode::PUSH(1)
    );
    assert_eq!(
        UnifiedOpcode::from_str("DUP1").unwrap(),
        UnifiedOpcode::DUP(1)
    );
    assert_eq!(
        UnifiedOpcode::from_str("SWAP1").unwrap(),
        UnifiedOpcode::SWAP(1)
    );
    assert_eq!(
        UnifiedOpcode::from_str("STOP").unwrap(),
        UnifiedOpcode::STOP
    );
    assert!(UnifiedOpcode::from_str("INVALID_OP").is_err());
}

#[test]
fn test_control_flow() {
    assert!(UnifiedOpcode::JUMP.is_control_flow());
    assert!(UnifiedOpcode::JUMPI.is_control_flow());
    assert!(UnifiedOpcode::RETURN.is_control_flow());
    assert!(!UnifiedOpcode::ADD.is_control_flow());
    assert!(!UnifiedOpcode::PUSH(1).is_control_flow());
}

#[test]
fn test_roundtrip() {
    for byte in 0u8..=255u8 {
        let opcode = UnifiedOpcode::from_byte(byte);
        assert_eq!(opcode.to_byte(), byte);
    }
}

#[test]
fn test_all_push_variants() {
    // Test PUSH0
    assert_eq!(
        UnifiedOpcode::from_str("PUSH0").unwrap(),
        UnifiedOpcode::PUSH0
    );
    assert_eq!(UnifiedOpcode::PUSH0.to_byte(), 0x5f);

    // Test PUSH1-PUSH32
    for i in 1..=32 {
        let opcode_str = format!("PUSH{}", i);
        let expected = UnifiedOpcode::PUSH(i);
        assert_eq!(UnifiedOpcode::from_str(&opcode_str).unwrap(), expected);
        assert_eq!(expected.to_byte(), 0x5f + i);
    }
}

#[test]
fn test_dup_and_swap() {
    // Test DUP1-DUP16
    for i in 1..=16 {
        let opcode_str = format!("DUP{}", i);
        let expected = UnifiedOpcode::DUP(i);
        assert_eq!(UnifiedOpcode::from_str(&opcode_str).unwrap(), expected);
        assert_eq!(expected.to_byte(), 0x7f + i);
    }

    // Test SWAP1-SWAP16
    for i in 1..=16 {
        let opcode_str = format!("SWAP{}", i);
        let expected = UnifiedOpcode::SWAP(i);
        assert_eq!(UnifiedOpcode::from_str(&opcode_str).unwrap(), expected);
        assert_eq!(expected.to_byte(), 0x8f + i);
    }
}

#[test]
fn test_edge_cases() {
    // Test invalid PUSH values
    assert!(UnifiedOpcode::from_str("PUSH0").is_ok());
    assert!(UnifiedOpcode::from_str("PUSH33").is_err());
    assert!(UnifiedOpcode::from_str("PUSH").is_err());

    // Test invalid DUP values
    assert!(UnifiedOpcode::from_str("DUP17").is_err());
    assert!(UnifiedOpcode::from_str("DUP0").is_err());

    // Test invalid SWAP values
    assert!(UnifiedOpcode::from_str("SWAP17").is_err());
    assert!(UnifiedOpcode::from_str("SWAP0").is_err());
}

#[test]
fn test_display_formatting() {
    assert_eq!(UnifiedOpcode::STOP.to_string(), "STOP");
    assert_eq!(UnifiedOpcode::PUSH(1).to_string(), "PUSH1");
    assert_eq!(UnifiedOpcode::DUP(5).to_string(), "DUP5");
    assert_eq!(UnifiedOpcode::SWAP(10).to_string(), "SWAP10");
    assert_eq!(UnifiedOpcode::UNKNOWN(0xff).to_string(), "UNKNOWNff");
}

#[test]
fn test_metadata_access() {
    use eot::Fork;

    let add_opcode = UnifiedOpcode::ADD;
    if let Some(metadata) = add_opcode.metadata(Fork::Cancun) {
        assert_eq!(metadata.name, "ADD");
        assert_eq!(metadata.opcode, 0x01);
        assert!(metadata.gas_cost > 0);
    }

    if let Some(metadata) = add_opcode.metadata_latest() {
        assert_eq!(metadata.name, "ADD");
    }
}
