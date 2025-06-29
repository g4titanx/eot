//! Integration tests for real-world usage scenarios

use eot::{forks::*, Fork, OpCode, OpcodeRegistry};

#[test]
fn test_gas_cost_analysis() {
    // Simulate analyzing a simple contract
    let opcodes_to_analyze = vec![
        0x60, // PUSH1
        0x60, // PUSH1
        0x01, // ADD
        0x54, // SLOAD
        0x55, // SSTORE
        0xf3, // RETURN
    ];

    let mut total_gas = 0;
    for &byte in &opcodes_to_analyze {
        if Cancun::has_opcode(byte) {
            let opcode = Cancun::from(byte);
            total_gas += opcode.gas_cost() as u64;
        }
    }

    // Should calculate reasonable gas cost
    assert!(total_gas > 0);
    assert!(total_gas < 1000); // Reasonable for this simple sequence
}

#[test]
fn test_fork_compatibility_check() {
    // Test checking if a contract is compatible with different forks
    let modern_opcodes = vec![0x5c, 0x5d]; // TLOAD, TSTORE

    // Should not be available in older forks
    for &opcode in &modern_opcodes {
        assert!(!Frontier::has_opcode(opcode));
        assert!(!Homestead::has_opcode(opcode));
        assert!(!Byzantium::has_opcode(opcode));
        assert!(!Constantinople::has_opcode(opcode));
        assert!(!Istanbul::has_opcode(opcode));
        assert!(!Berlin::has_opcode(opcode));
        assert!(!London::has_opcode(opcode));
        assert!(!Shanghai::has_opcode(opcode));

        // Should be available in Cancun
        assert!(Cancun::has_opcode(opcode));
    }
}

#[test]
fn test_contract_analysis_workflow() {
    // Simulate a complete contract analysis workflow
    let _contract_bytecode = vec![
        0x60, 0x80, // PUSH1 0x80
        0x60, 0x40, // PUSH1 0x40
        0x52, // MSTORE
        0x34, // CALLVALUE
        0x80, // DUP1
        0x15, // ISZERO
        0x61, 0x00, 0x16, // PUSH2 0x0016
        0x57, // JUMPI
        0x60, 0x00, // PUSH1 0x00
        0x80, // DUP1
        0xfd, // REVERT
    ];

    // Filter only opcodes (skip immediate data)
    let opcodes = vec![
        0x60, 0x60, 0x52, 0x34, 0x80, 0x15, 0x61, 0x57, 0x60, 0x80, 0xfd,
    ];

    let mut analysis = ContractAnalysis::new();

    for &opcode_byte in &opcodes {
        if Cancun::has_opcode(opcode_byte) {
            let opcode = Cancun::from(opcode_byte);
            analysis.add_opcode(opcode);
        }
    }

    assert!(analysis.total_gas > 0);
    assert!(analysis.uses_revert);
    assert!(!analysis.uses_create);
    assert!(!analysis.uses_transient_storage);
}

#[test]
fn test_minimal_fork_detection() {
    // Find the minimal fork required for a set of opcodes
    let opcodes = vec![
        0x01, // ADD (Frontier)
        0xf4, // DELEGATECALL (Homestead)
        0xfd, // REVERT (Byzantium)
        0x5f, // PUSH0 (Shanghai)
    ];

    let min_fork = find_minimal_fork(&opcodes);
    assert_eq!(min_fork, Fork::Shanghai);
}

#[test]
fn test_upgrade_path_analysis() {
    // Test analyzing what opcodes become available when upgrading forks
    let london_count = London::all_opcodes().len();
    let shanghai_count = Shanghai::all_opcodes().len();
    let cancun_count = Cancun::all_opcodes().len();

    assert!(shanghai_count > london_count);
    assert!(cancun_count > shanghai_count);

    // Check specific additions
    assert!(!London::has_opcode(0x5f)); // PUSH0
    assert!(Shanghai::has_opcode(0x5f)); // PUSH0

    assert!(!Shanghai::has_opcode(0x5c)); // TLOAD
    assert!(Cancun::has_opcode(0x5c)); // TLOAD
}

#[test]
fn test_registry_comprehensive() {
    let registry = OpcodeRegistry::new();

    // Test that registry contains opcodes for all forks
    let frontier_opcodes = registry.get_opcodes(Fork::Frontier);
    let cancun_opcodes = registry.get_opcodes(Fork::Cancun);

    assert!(!frontier_opcodes.is_empty());
    assert!(!cancun_opcodes.is_empty());
    assert!(cancun_opcodes.len() > frontier_opcodes.len());

    // Test specific opcode availability
    assert!(registry.is_opcode_available(Fork::Frontier, 0x01)); // ADD
    assert!(!registry.is_opcode_available(Fork::Frontier, 0xf4)); // DELEGATECALL
    assert!(registry.is_opcode_available(Fork::Homestead, 0xf4)); // DELEGATECALL
    assert!(registry.is_opcode_available(Fork::Cancun, 0x5c)); // TLOAD
}

struct ContractAnalysis {
    total_gas: u64,
    uses_revert: bool,
    uses_create: bool,
    uses_transient_storage: bool,
    opcode_count: usize,
}

impl ContractAnalysis {
    fn new() -> Self {
        Self {
            total_gas: 0,
            uses_revert: false,
            uses_create: false,
            uses_transient_storage: false,
            opcode_count: 0,
        }
    }

    fn add_opcode<T: OpCode>(&mut self, opcode: T) {
        self.total_gas += opcode.gas_cost() as u64;
        self.opcode_count += 1;

        let byte_val: u8 = opcode.into();
        match byte_val {
            0xfd => self.uses_revert = true,                   // REVERT
            0xf0 | 0xf5 => self.uses_create = true,            // CREATE, CREATE2
            0x5c | 0x5d => self.uses_transient_storage = true, // TLOAD, TSTORE
            _ => {}
        }
    }
}

fn find_minimal_fork(opcodes: &[u8]) -> Fork {
    let forks = [
        Fork::Frontier,
        Fork::Homestead,
        Fork::Byzantium,
        Fork::Constantinople,
        Fork::Istanbul,
        Fork::Berlin,
        Fork::London,
        Fork::Shanghai,
        Fork::Cancun,
    ];

    let registry = OpcodeRegistry::new();

    for &fork in &forks {
        let available = opcodes
            .iter()
            .all(|&opcode| registry.is_opcode_available(fork, opcode));

        if available {
            return fork;
        }
    }

    Fork::Cancun // Fallback to latest
}
