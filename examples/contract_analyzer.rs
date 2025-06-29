//! Contract analyzer example using EOT
//!
//! This example shows how to build a basic contract analyzer using EOT
//! Run with: cargo run --example contract_analyzer

use eot::{forks::*, Fork, Group, OpCode, OpcodeRegistry};
use std::collections::HashMap;

fn main() {
    println!("🔍 EOT Contract Analyzer Example\n");

    // Example 1: Analyze a simple ERC20-like contract
    analyze_erc20_contract();

    // Example 2: Gas profiling
    gas_profiling_example();

    // Example 3: Security analysis
    security_analysis_example();

    // Example 4: Fork compatibility checker
    fork_compatibility_checker();
}

fn analyze_erc20_contract() {
    println!("📊 Example 1: ERC20 Contract Analysis");
    println!("=====================================");

    // Simplified ERC20 transfer function bytecode pattern
    let transfer_function = vec![
        0x60, 0x40, // PUSH1 0x40 - Free memory pointer
        0x52, // MSTORE
        0x60, 0x04, // PUSH1 0x04 - Start of calldata
        0x35, // CALLDATALOAD - Load recipient
        0x60, 0x24, // PUSH1 0x24 - Next calldata position
        0x35, // CALLDATALOAD - Load amount
        0x33, // CALLER - Get sender
        0x60, 0x00, // PUSH1 0x00 - Storage slot base
        0x81, // DUP2 - Duplicate sender
        0x52, // MSTORE - Store sender in memory
        0x60, 0x20, // PUSH1 0x20 - Hash input size
        0x60, 0x00, // PUSH1 0x00 - Hash input offset
        0x20, // KECCAK256 - Hash to get storage slot
        0x54, // SLOAD - Load sender balance
        0x82, // DUP3 - Duplicate amount
        0x11, // GT - Check if balance >= amount
        0x61, 0x00, 0x5a, // PUSH2 0x005a - Revert jump target
        0x57, // JUMPI - Jump if insufficient balance
        0x81, // DUP2 - Duplicate amount
        0x90, // SWAP1 - Rearrange stack
        0x03, // SUB - Subtract amount from balance
        0x81, // DUP2 - Duplicate storage slot
        0x55, // SSTORE - Store new sender balance
        // ... more opcodes for recipient balance update
        0x5b, // JUMPDEST - Continue execution
        0xf3, // RETURN
    ];

    let analyzer = ContractAnalyzer::new();
    let analysis = analyzer.analyze_bytecode(&transfer_function);

    println!("📈 Analysis Results:");
    println!("  Total opcodes: {}", analysis.total_opcodes);
    println!(
        "  Estimated gas: {} (base costs only)",
        analysis.estimated_gas
    );
    println!("  Storage operations: {}", analysis.storage_ops);
    println!("  Memory operations: {}", analysis.memory_ops);
    println!("  Arithmetic operations: {}", analysis.arithmetic_ops);
    println!(
        "  Uses revert: {}",
        if analysis.uses_revert { "✅" } else { "❌" }
    );
    println!(
        "  Uses external calls: {}",
        if analysis.uses_external_calls {
            "✅"
        } else {
            "❌"
        }
    );
    println!("  Minimum fork required: {:?}", analysis.min_fork_required);

    println!("\n🔍 Detailed breakdown:");
    for (group, count) in analysis.opcode_groups {
        if count > 0 {
            println!("  {:?}: {} opcodes", group, count);
        }
    }
    println!();
}

fn gas_profiling_example() {
    println!("⛽ Example 2: Gas Profiling");
    println!("===========================");

    // Different contract patterns with their gas costs
    let patterns = [
        ("Simple storage write", vec![0x60, 0x60, 0x55]), // PUSH1, PUSH1, SSTORE
        (
            "Storage read + write",
            vec![0x60, 0x54, 0x60, 0x01, 0x01, 0x60, 0x55],
        ), // Complex
        ("Keccak256 hash", vec![0x60, 0x20, 0x60, 0x00, 0x20]), // PUSH1 32, PUSH1 0, KECCAK256
        (
            "External call",
            vec![0x60, 0x60, 0x60, 0x60, 0x60, 0x60, 0x60, 0xf1],
        ), // CALL setup
        ("Contract creation", vec![0x60, 0x60, 0x60, 0xf0]), // CREATE
    ];

    println!("Gas cost comparison:");
    println!(
        "{:<20} {:<10} {:<15} {:<20}",
        "Pattern", "Opcodes", "Base Gas", "Description"
    );
    println!("{}", "-".repeat(70));

    for (name, opcodes) in patterns {
        let total_gas: u32 = opcodes
            .iter()
            .map(|&byte| {
                if Cancun::has_opcode(byte) {
                    Cancun::from(byte).gas_cost() as u32
                } else {
                    0
                }
            })
            .sum();

        println!(
            "{:<20} {:<10} {:<15} {:<20}",
            name,
            opcodes.len(),
            total_gas,
            "Base cost only"
        );
    }

    println!("\n⚠️  Note: Actual gas costs include dynamic costs (memory expansion, storage refunds, etc.)");
    println!();
}

fn security_analysis_example() {
    println!("🔒 Example 3: Security Analysis");
    println!("===============================");

    // Patterns that might indicate security issues
    let suspicious_patterns = [
        ("Potential reentrancy", vec![0xf1, 0x55]), // CALL followed by SSTORE
        ("Self-destruct usage", vec![0xff]),        // SELFDESTRUCT
        ("Delegatecall usage", vec![0xf4]),         // DELEGATECALL
        ("External code access", vec![0x3c]),       // EXTCODECOPY
        ("Timestamp dependency", vec![0x42]),       // TIMESTAMP
    ];

    println!("Security pattern analysis:");
    println!(
        "{:<25} {:<15} {:<30}",
        "Pattern", "Risk Level", "Description"
    );
    println!("{}", "-".repeat(75));

    for (pattern_name, opcodes) in suspicious_patterns {
        let risk_level = assess_risk_level(&opcodes);
        let description = get_security_description(pattern_name);

        println!(
            "{:<25} {:<15} {:<30}",
            pattern_name, risk_level, description
        );
    }

    println!("\n🛡️  Security recommendations:");
    println!("  • Use checks-effects-interactions pattern");
    println!("  • Avoid timestamp dependency for critical logic");
    println!("  • Be cautious with delegatecall and external code");
    println!("  • Consider using pull payment pattern");
    println!();
}

fn fork_compatibility_checker() {
    println!("🔗 Example 4: Fork Compatibility Checker");
    println!("========================================");

    // Real-world contract patterns from different eras
    let contract_examples = [
        ("Legacy Contract (2016)", vec![0x60, 0x60, 0x01, 0x55, 0xf3]),
        ("Post-Homestead", vec![0x60, 0x60, 0xf4, 0xf3]), // Uses DELEGATECALL
        ("Post-Byzantium", vec![0x60, 0x60, 0xfa, 0x50, 0xfd]), // Uses STATICCALL, REVERT
        ("Modern Shanghai", vec![0x5f, 0x5f, 0x01]),      // Uses PUSH0
        ("Cancun Features", vec![0x5f, 0x5c, 0x5d]),      // Uses transient storage
    ];

    let registry = OpcodeRegistry::new();

    println!("Fork compatibility analysis:");
    println!(
        "{:<20} {:<12} {:<30}",
        "Contract Type", "Min Fork", "Incompatible Features"
    );
    println!("{}", "-".repeat(65));

    for (contract_name, opcodes) in contract_examples {
        let min_fork = find_minimum_fork(&opcodes, &registry);
        let incompatible = find_incompatible_features(&opcodes);

        println!(
            "{:<20} {:<12} {:<30}",
            contract_name,
            format!("{:?}", min_fork),
            incompatible
        );
    }

    println!("\n📋 Upgrade path analysis:");
    println!("  To deploy on older networks, avoid:");
    println!("    • PUSH0 (requires Shanghai+)");
    println!("    • Transient storage opcodes (requires Cancun+)");
    println!("    • Recent EIP features");
    println!("  Consider using compiler flags for target fork compatibility");

    println!("\n✅ Contract analysis examples completed!");
}

// Supporting structures and functions

struct ContractAnalysis {
    total_opcodes: usize,
    estimated_gas: u64,
    storage_ops: usize,
    memory_ops: usize,
    arithmetic_ops: usize,
    uses_revert: bool,
    uses_external_calls: bool,
    min_fork_required: Fork,
    opcode_groups: HashMap<Group, usize>,
}

struct ContractAnalyzer {
    registry: OpcodeRegistry,
}

impl ContractAnalyzer {
    fn new() -> Self {
        Self {
            registry: OpcodeRegistry::new(),
        }
    }

    fn analyze_bytecode(&self, bytecode: &[u8]) -> ContractAnalysis {
        let mut analysis = ContractAnalysis {
            total_opcodes: 0,
            estimated_gas: 0,
            storage_ops: 0,
            memory_ops: 0,
            arithmetic_ops: 0,
            uses_revert: false,
            uses_external_calls: false,
            min_fork_required: Fork::Frontier,
            opcode_groups: HashMap::new(),
        };

        for &byte in bytecode {
            if Cancun::has_opcode(byte) {
                let opcode = Cancun::from(byte);
                self.analyze_opcode(opcode, &mut analysis);
            }
        }

        analysis.min_fork_required = find_minimum_fork(bytecode, &self.registry);
        analysis
    }

    fn analyze_opcode<T: OpCode>(&self, opcode: T, analysis: &mut ContractAnalysis) {
        let metadata = opcode.metadata();

        analysis.total_opcodes += 1;
        analysis.estimated_gas += metadata.gas_cost as u64;

        // Count by group
        *analysis.opcode_groups.entry(metadata.group).or_insert(0) += 1;

        // Categorize by functionality
        match metadata.opcode {
            // Storage operations
            0x54 | 0x55 => analysis.storage_ops += 1, // SLOAD, SSTORE
            0x5c | 0x5d => analysis.storage_ops += 1, // TLOAD, TSTORE

            // Memory operations
            0x51 | 0x52 | 0x53 => analysis.memory_ops += 1, // MLOAD, MSTORE, MSTORE8

            // Arithmetic operations
            0x01..=0x0b => analysis.arithmetic_ops += 1, // ADD through SIGNEXTEND

            // Special behaviors
            0xfd => analysis.uses_revert = true, // REVERT
            0xf1 | 0xf2 | 0xf4 | 0xfa => analysis.uses_external_calls = true, // CALL variants

            _ => {}
        }
    }
}

fn assess_risk_level(opcodes: &[u8]) -> &'static str {
    for &opcode in opcodes {
        match opcode {
            0xff => return "🔴 HIGH",   // SELFDESTRUCT
            0xf4 => return "🟡 MEDIUM", // DELEGATECALL
            0xf1 => return "🟡 MEDIUM", // CALL
            0x42 => return "🟡 MEDIUM", // TIMESTAMP
            _ => {}
        }
    }
    "🟢 LOW"
}

fn get_security_description(pattern: &str) -> &'static str {
    match pattern {
        "Potential reentrancy" => "Check CEI pattern",
        "Self-destruct usage" => "Permanent contract destruction",
        "Delegatecall usage" => "Context preservation risk",
        "External code access" => "Code injection risk",
        "Timestamp dependency" => "Miner manipulation risk",
        _ => "Unknown pattern",
    }
}

fn find_minimum_fork(opcodes: &[u8], registry: &OpcodeRegistry) -> Fork {
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

    for &fork in &forks {
        let all_available = opcodes
            .iter()
            .all(|&opcode| registry.is_opcode_available(fork, opcode));

        if all_available {
            return fork;
        }
    }

    Fork::Cancun // Fallback to latest
}

fn find_incompatible_features(opcodes: &[u8]) -> String {
    let mut incompatible = Vec::new();

    for &opcode in opcodes {
        match opcode {
            0x5f => incompatible.push("PUSH0 (Shanghai+)"),
            0x5c | 0x5d => incompatible.push("Transient storage (Cancun+)"),
            0x5e => incompatible.push("MCOPY (Cancun+)"),
            0x49 | 0x4a => incompatible.push("Blob opcodes (Cancun+)"),
            0xf4 => incompatible.push("DELEGATECALL (Homestead+)"),
            0xfd | 0xfa | 0x3d | 0x3e => incompatible.push("Byzantium features"),
            _ => {}
        }
    }

    if incompatible.is_empty() {
        "None".to_string()
    } else {
        incompatible.join(", ")
    }
}
