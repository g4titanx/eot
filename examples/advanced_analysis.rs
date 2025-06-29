//! Advanced analysis example for EOT
//!
//! Run with: cargo run --example advanced_analysis

use eot::{forks::*, traits::OpcodeExt, Group, OpCode};
use std::collections::HashMap;

fn main() {
    println!("🔬 EOT - Advanced Analysis Example\n");

    // Example 1: Advanced opcode analysis
    advanced_opcode_analysis();

    // Example 2: Contract compatibility analysis
    contract_compatibility_analysis();

    // Example 3: Fork evolution analysis
    fork_evolution_analysis();

    // Example 4: Opcode categorization
    opcode_categorization();

    // Example 5: Gas optimization analysis
    gas_optimization_analysis();
}

fn advanced_opcode_analysis() {
    println!("🧬 Example 1: Advanced Opcode Analysis");
    println!("======================================");

    let opcodes = [
        Cancun::SSTORE,
        Cancun::CALL,
        Cancun::STATICCALL,
        Cancun::PUSH1,
        Cancun::DUP5,
        Cancun::SWAP3,
        Cancun::TLOAD,
        Cancun::CREATE2,
    ];

    println!("Detailed opcode analysis:");
    println!(
        "{:<12} {:<8} {:<8} {:<6} {:<6} {:<6} {:<8}",
        "Opcode", "State", "Revert", "Push", "Dup", "Swap", "MinStack"
    );
    println!("{}", "-".repeat(70));

    for opcode in opcodes {
        println!(
            "{:<12} {:<8} {:<8} {:<6} {:<6} {:<6} {:<8}",
            format!("{}", opcode),
            if opcode.modifies_state() {
                "✅"
            } else {
                "❌"
            },
            if opcode.can_revert() { "✅" } else { "❌" },
            if opcode.is_push() { "✅" } else { "❌" },
            if opcode.is_dup() { "✅" } else { "❌" },
            if opcode.is_swap() { "✅" } else { "❌" },
            opcode.min_stack_depth()
        );
    }

    println!("\nLegend:");
    println!("  State: Modifies blockchain state");
    println!("  Revert: Can cause transaction revert");
    println!("  Push: Push operation");
    println!("  Dup: Duplication operation");
    println!("  Swap: Swap operation");
    println!("  MinStack: Minimum required stack depth");
    println!();
}

fn contract_compatibility_analysis() {
    println!("🔗 Example 2: Contract Compatibility Analysis");
    println!("=============================================");

    // Simulate different contract patterns
    let contracts = [
        ("Simple Transfer", vec![0x60, 0x60, 0x01, 0xf3]), // PUSH1, PUSH1, ADD, RETURN
        ("With Delegatecall", vec![0x60, 0xf4, 0xf3]),     // PUSH1, DELEGATECALL, RETURN
        ("With Revert", vec![0x60, 0x60, 0xfd]),           // PUSH1, PUSH1, REVERT
        ("Modern with PUSH0", vec![0x5f, 0x5f, 0x01]),     // PUSH0, PUSH0, ADD
        ("Transient Storage", vec![0x5f, 0x5c, 0x5d]),     // PUSH0, TLOAD, TSTORE
    ];

    println!("Contract compatibility matrix:");
    println!(
        "{:<20} {:<10} {:<10} {:<10} {:<10} {:<10}",
        "Contract", "Frontier", "Homestead", "Byzantium", "Shanghai", "Cancun"
    );
    println!("{}", "-".repeat(80));

    for (contract_name, opcodes) in contracts {
        print!("{:<20}", contract_name);

        // Check each fork manually
        print!(
            " {:<10}",
            if check_frontier_compatibility(&opcodes) {
                "✅"
            } else {
                "❌"
            }
        );
        print!(
            " {:<10}",
            if check_homestead_compatibility(&opcodes) {
                "✅"
            } else {
                "❌"
            }
        );
        print!(
            " {:<10}",
            if check_byzantium_compatibility(&opcodes) {
                "✅"
            } else {
                "❌"
            }
        );
        print!(
            " {:<10}",
            if check_shanghai_compatibility(&opcodes) {
                "✅"
            } else {
                "❌"
            }
        );
        print!(
            " {:<10}",
            if check_cancun_compatibility(&opcodes) {
                "✅"
            } else {
                "❌"
            }
        );

        println!();
    }
    println!();
}

fn fork_evolution_analysis() {
    println!("📈 Example 3: Fork Evolution Analysis");
    println!("====================================");

    let forks = [
        ("Frontier", Frontier::all_opcodes().len()),
        ("Homestead", Homestead::all_opcodes().len()),
        ("Byzantium", Byzantium::all_opcodes().len()),
        ("Constantinople", Constantinople::all_opcodes().len()),
        ("Istanbul", Istanbul::all_opcodes().len()),
        ("Berlin", Berlin::all_opcodes().len()),
        ("London", London::all_opcodes().len()),
        ("Shanghai", Shanghai::all_opcodes().len()),
        ("Cancun", Cancun::all_opcodes().len()),
    ];

    println!("EVM evolution - opcode count growth:");

    let mut previous_count = 0;
    for (fork_name, count) in forks {
        let growth = if previous_count == 0 {
            0
        } else {
            count - previous_count
        };
        let percentage = if previous_count == 0 {
            0.0
        } else {
            (growth as f64 / previous_count as f64) * 100.0
        };

        println!(
            "{:<15} {:>3} opcodes (+{:>2}) {:>6.1}% growth",
            fork_name, count, growth, percentage
        );
        previous_count = count;
    }

    // Analyze major additions
    println!("\nMajor additions by fork:");
    println!("  Homestead: DELEGATECALL (0xf4)");
    println!("  Byzantium: REVERT, RETURNDATASIZE, RETURNDATACOPY, STATICCALL");
    println!("  Constantinople: Shift operations (SHL, SHR, SAR), CREATE2, EXTCODEHASH");
    println!("  Istanbul: CHAINID, SELFBALANCE");
    println!("  London: BASEFEE");
    println!("  Shanghai: PUSH0");
    println!("  Cancun: Transient storage (TLOAD, TSTORE), MCOPY, Blob operations");
    println!();
}

fn opcode_categorization() {
    println!("📂 Example 4: Opcode Categorization");
    println!("===================================");

    let all_opcodes = Cancun::all_opcodes();
    let mut categories: HashMap<Group, Vec<String>> = HashMap::new();

    // Categorize all opcodes
    for opcode in all_opcodes {
        let group = opcode.group();
        let name = format!("{}", opcode);
        categories.entry(group).or_insert_with(Vec::new).push(name);
    }

    // Sort categories by group name for consistent output
    let mut sorted_categories: Vec<_> = categories.into_iter().collect();
    sorted_categories.sort_by_key(|(group, _)| format!("{:?}", group));

    for (group, mut opcodes) in sorted_categories {
        opcodes.sort();
        println!("{:?} ({} opcodes):", group, opcodes.len());

        // Print opcodes in rows of 8
        for chunk in opcodes.chunks(8) {
            println!("  {}", chunk.join(", "));
        }
        println!();
    }
}

fn gas_optimization_analysis() {
    println!("⛽ Example 5: Gas Optimization Analysis");
    println!("=======================================");

    // Compare different ways to achieve the same result
    let optimizations = [
        ("Push zero (old)", vec![0x60], "PUSH1 0x00"),
        ("Push zero (new)", vec![0x5f], "PUSH0"),
        (
            "Clear storage (SSTORE)",
            vec![0x60, 0x55],
            "PUSH1 0x00, SSTORE",
        ),
        (
            "Clear storage (optimized)",
            vec![0x5f, 0x55],
            "PUSH0, SSTORE",
        ),
    ];

    println!("Gas optimization opportunities:");
    println!(
        "{:<25} {:<15} {:<20} {:<15}",
        "Pattern", "Total Gas", "Opcodes", "Description"
    );
    println!("{}", "-".repeat(80));

    for (pattern_name, opcodes, description) in optimizations {
        let total_gas: u16 = opcodes
            .iter()
            .filter_map(|&byte| {
                if Cancun::has_opcode(byte) {
                    Some(Cancun::from(byte).gas_cost())
                } else {
                    None
                }
            })
            .sum();

        println!(
            "{:<25} {:<15} {:<20} {:<15}",
            pattern_name,
            total_gas,
            opcodes.len(),
            description
        );
    }

    // Analyze expensive operations
    println!("\nMost expensive basic operations:");
    let expensive_ops = [
        Cancun::CREATE,
        Cancun::CREATE2,
        Cancun::SELFDESTRUCT,
        Cancun::LOG4,
        Cancun::LOG3,
        Cancun::KECCAK256,
    ];

    for op in expensive_ops {
        println!("  {:<15} {:>6} gas", format!("{}", op), op.gas_cost());
    }

    println!("\n✅ Advanced analysis completed!");
}

// Helper functions for compatibility checking
fn check_frontier_compatibility(opcodes: &[u8]) -> bool {
    opcodes.iter().all(|&opcode| Frontier::has_opcode(opcode))
}

fn check_homestead_compatibility(opcodes: &[u8]) -> bool {
    opcodes.iter().all(|&opcode| Homestead::has_opcode(opcode))
}

fn check_byzantium_compatibility(opcodes: &[u8]) -> bool {
    opcodes.iter().all(|&opcode| Byzantium::has_opcode(opcode))
}

fn check_shanghai_compatibility(opcodes: &[u8]) -> bool {
    opcodes.iter().all(|&opcode| Shanghai::has_opcode(opcode))
}

fn check_cancun_compatibility(opcodes: &[u8]) -> bool {
    opcodes.iter().all(|&opcode| Cancun::has_opcode(opcode))
}
