//! Integration tests for real-world usage scenarios.

use eot::{Fork, OpCode};

#[test]
fn gas_cost_analysis() {
    let opcodes = [0x01, 0x02]; // ADD, MUL
    let total: u64 = opcodes
        .iter()
        .map(|&b| OpCode::from_byte(b).gas_cost(Fork::London) as u64)
        .sum();
    assert!(
        total < 1000,
        "simple arithmetic should be cheap, got {total}"
    );
    assert!(total > 0);
}

#[test]
fn fork_compatibility_check() {
    // TLOAD/TSTORE only from Cancun
    for &byte in &[0x5c, 0x5d] {
        let op = OpCode::new(byte).unwrap();
        assert!(!op.is_valid_in(Fork::Frontier));
        assert!(!op.is_valid_in(Fork::Shanghai));
        assert!(op.is_valid_in(Fork::Cancun));
        assert!(op.is_valid_in(Fork::Prague));
    }
}

#[test]
fn contract_analysis_workflow() {
    let opcodes = [
        0x60u8, 0x60, 0x52, 0x34, 0x80, 0x15, 0x61, 0x57, 0x60, 0x80, 0xfd,
    ];

    let mut total_gas = 0u64;
    let mut uses_revert = false;
    let mut uses_create = false;
    let mut uses_transient = false;

    for &b in &opcodes {
        if let Some(op) = OpCode::new(b) {
            total_gas += op.gas_cost(Fork::Cancun) as u64;
            match b {
                0xfd => uses_revert = true,
                0xf0 | 0xf5 => uses_create = true,
                0x5c | 0x5d => uses_transient = true,
                _ => {}
            }
        }
    }

    assert!(total_gas > 0);
    assert!(uses_revert);
    assert!(!uses_create);
    assert!(!uses_transient);
}

#[test]
fn minimal_fork_detection() {
    let opcodes = [
        0x01, // ADD (Frontier)
        0xf4, // DELEGATECALL (Homestead)
        0xfd, // REVERT (Byzantium)
        0x5f, // PUSH0 (Shanghai)
    ];

    let min_fork = find_minimal_fork(&opcodes);
    assert_eq!(min_fork, Fork::Shanghai);
}

#[test]
fn upgrade_path_analysis() {
    let london = OpCode::count_at(Fork::London);
    let shanghai = OpCode::count_at(Fork::Shanghai);
    let cancun = OpCode::count_at(Fork::Cancun);

    assert!(shanghai > london, "Shanghai should add PUSH0");
    assert!(
        cancun > shanghai,
        "Cancun should add TLOAD/TSTORE/MCOPY/etc."
    );

    assert!(!OpCode::PUSH0.is_valid_in(Fork::London));
    assert!(OpCode::PUSH0.is_valid_in(Fork::Shanghai));

    assert!(!OpCode::TLOAD.is_valid_in(Fork::Shanghai));
    assert!(OpCode::TLOAD.is_valid_in(Fork::Cancun));
}

#[test]
fn validation_passes() {
    assert!(eot::validation::validate().is_ok());
}

fn find_minimal_fork(opcodes: &[u8]) -> Fork {
    for &fork in Fork::ordered() {
        if opcodes
            .iter()
            .all(|&b| OpCode::from_byte(b).is_valid_in(fork))
        {
            return fork;
        }
    }
    Fork::latest()
}
