//! Registry and validation example for EOT
//!
//! Run with: cargo run --example registry_usage

use eot::{validation, Fork, OpcodeRegistry};

fn main() {
    println!("🗃️ EOT Registry and Validation Example\n");

    // Example 1: Basic registry usage
    basic_registry_usage();

    // Example 2: Cross-fork analysis
    cross_fork_analysis();

    // Example 3: Validation system
    validation_system_example();

    // Example 4: Historical analysis
    historical_analysis();
}

fn basic_registry_usage() {
    println!("📚 Example 1: Basic Registry Usage");
    println!("==================================");

    let registry = OpcodeRegistry::new();

    // Check opcode availability across forks
    let test_opcodes = [
        (0x01, "ADD"),
        (0xf4, "DELEGATECALL"),
        (0xfd, "REVERT"),
        (0x5f, "PUSH0"),
        (0x5c, "TLOAD"),
    ];

    println!("Opcode availability check:");
    for (opcode, name) in test_opcodes {
        println!("\n{} (0x{:02x}):", name, opcode);

        let forks_to_check = [
            Fork::Frontier,
            Fork::Homestead,
            Fork::Byzantium,
            Fork::Shanghai,
            Fork::Cancun,
        ];

        for fork in forks_to_check {
            let available = registry.is_opcode_available(fork, opcode);
            println!("  {:?}: {}", fork, if available { "✅" } else { "❌" });
        }
    }
    println!();
}

fn cross_fork_analysis() {
    println!("🔄 Example 2: Cross-Fork Analysis");
    println!("=================================");

    let registry = OpcodeRegistry::new();

    // Analyze opcode counts across forks
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

    println!("Opcode evolution across forks:");
    println!(
        "{:<15} {:>8} {:>10} {:>12}",
        "Fork", "Opcodes", "Growth", "Growth %"
    );
    println!("{}", "-".repeat(50));

    let mut previous_count = 0;
    for fork in forks {
        let opcodes = registry.get_opcodes(fork);
        let count = opcodes.len();
        let growth = if previous_count == 0 {
            0
        } else {
            count - previous_count
        };
        let growth_percent = if previous_count == 0 {
            0.0
        } else {
            (growth as f64 / previous_count as f64) * 100.0
        };

        println!(
            "{:<15} {:>8} {:>10} {:>11.1}%",
            format!("{:?}", fork),
            count,
            if growth > 0 {
                format!("+{}", growth)
            } else {
                "0".to_string()
            },
            growth_percent
        );

        previous_count = count;
    }

    // Find unique opcodes in latest fork
    println!("\nNew opcodes in Cancun fork:");
    let cancun_opcodes = registry.get_opcodes(Fork::Cancun);
    let shanghai_opcodes = registry.get_opcodes(Fork::Shanghai);

    let mut new_opcodes: Vec<_> = cancun_opcodes
        .keys()
        .filter(|opcode| !shanghai_opcodes.contains_key(opcode))
        .collect();
    new_opcodes.sort();

    for &opcode in new_opcodes {
        if let Some(metadata) = cancun_opcodes.get(&opcode) {
            println!(
                "  0x{:02x} - {} ({})",
                opcode, metadata.name, metadata.description
            );
        }
    }
    println!();
}

fn validation_system_example() {
    println!("✅ Example 3: Validation System");
    println!("==============================");

    let registry = OpcodeRegistry::new();

    println!("Running basic validation...");
    match registry.validate() {
        Ok(()) => println!("✅ Basic validation passed!"),
        Err(errors) => {
            println!("❌ Basic validation failed:");
            for error in errors {
                println!("  • {}", error);
            }
        }
    }

    println!("\nRunning comprehensive validation...");
    let report = validation::run_comprehensive_validation(&registry);

    if report.has_errors() {
        println!("❌ Comprehensive validation found issues:");
        report.print_summary();
    } else {
        println!("✅ All comprehensive validations passed!");

        // Show some info from the report
        if let Some(info) = report.info.get("Coverage") {
            println!("\n📊 Coverage Information:");
            for item in info {
                println!("  {}", item);
            }
        }

        if let Some(warnings) = report.warnings.get("Missing EIPs") {
            if !warnings.is_empty() {
                println!("\n⚠️  Missing EIP References ({} items):", warnings.len());
                for warning in warnings.iter().take(3) {
                    println!("  {}", warning);
                }
                if warnings.len() > 3 {
                    println!("  ... and {} more", warnings.len() - 3);
                }
            }
        }
    }
    println!();
}

fn historical_analysis() {
    println!("📜 Example 4: Historical Analysis");
    println!("=================================");

    let registry = OpcodeRegistry::new();

    // Analyze when each major opcode category was introduced
    let categories = [
        ("Core arithmetic", vec![0x01, 0x02, 0x03]), // ADD, MUL, SUB
        ("Comparison ops", vec![0x10, 0x11, 0x14]),  // LT, GT, EQ
        ("Hash operations", vec![0x20]),             // KECCAK256
        ("Environment info", vec![0x30, 0x31, 0x32]), // ADDRESS, BALANCE, ORIGIN
        ("Block info", vec![0x40, 0x41, 0x42]),      // BLOCKHASH, COINBASE, TIMESTAMP
        ("Stack operations", vec![0x50, 0x80, 0x90]), // POP, DUP1, SWAP1
        ("Memory operations", vec![0x51, 0x52, 0x53]), // MLOAD, MSTORE, MSTORE8
        ("Storage operations", vec![0x54, 0x55]),    // SLOAD, SSTORE
        ("Control flow", vec![0x56, 0x57, 0x5b]),    // JUMP, JUMPI, JUMPDEST
        ("System operations", vec![0xf0, 0xf1, 0xf3]), // CREATE, CALL, RETURN
    ];

    println!("Historical introduction of opcode categories:");
    println!(
        "{:<20} {:<12} {:<30}",
        "Category", "Fork", "Representative Opcodes"
    );
    println!("{}", "-".repeat(65));

    for (category_name, opcodes) in categories {
        // Find the earliest fork that has all opcodes in this category
        let mut earliest_fork = Fork::Cancun;
        let forks_to_check = [
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

        for fork in forks_to_check {
            let all_available = opcodes
                .iter()
                .all(|&opcode| registry.is_opcode_available(fork, opcode));

            if all_available {
                earliest_fork = fork;
                break;
            }
        }

        let opcode_names: Vec<String> = opcodes
            .iter()
            .take(3)
            .map(|&opcode| {
                let all_opcodes = registry.get_opcodes(Fork::Cancun);
                if let Some(metadata) = all_opcodes.get(&opcode) {
                    metadata.name.to_string()
                } else {
                    format!("0x{:02x}", opcode)
                }
            })
            .collect();

        println!(
            "{:<20} {:<12} {:<30}",
            category_name,
            format!("{:?}", earliest_fork),
            opcode_names.join(", ")
        );
    }

    // Show major milestones
    println!("\n🏛️ Major EVM milestones:");
    println!("  Frontier (2015): Foundation - basic EVM functionality");
    println!("  Homestead (2016): DELEGATECALL - improved contract interactions");
    println!("  Byzantium (2017): REVERT, STATICCALL - better error handling");
    println!(
        "  Constantinople (2019): Bitwise shifts, CREATE2 - optimization & deterministic addresses"
    );
    println!("  Istanbul (2019): CHAINID, SELFBALANCE - network identification & gas optimization");
    println!("  Berlin (2021): Gas repricing - security improvements");
    println!("  London (2021): BASEFEE - EIP-1559 fee market");
    println!("  Shanghai (2023): PUSH0 - gas optimization");
    println!("  Cancun (2024): Transient storage, blobs - L2 scaling & temporary data");

    println!("\n✅ Registry and validation examples completed!");
}
