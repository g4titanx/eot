//! Basic usage example for EOT
//!
//! Run with: cargo run --example basic_usage

use eot::{forks::*, Fork, OpCode};

fn main() {
    println!("🚀 EOT (EVM Opcode Table) - Basic Usage Example\n");

    // Example 1: Basic opcode usage
    basic_opcode_operations();

    // Example 2: Working with different forks
    fork_comparison();

    // Example 3: Opcode metadata exploration
    explore_metadata();

    // Example 4: Gas cost analysis
    gas_cost_analysis();

    // Example 5: Opcode conversion
    opcode_conversion();
}

fn basic_opcode_operations() {
    println!("📋 Example 1: Basic Opcode Operations");
    println!("=====================================");

    // Get an opcode from the latest fork (Cancun)
    let tload = Cancun::TLOAD;

    println!("Opcode: {}", tload);
    println!("Byte value: 0x{:02x}", u8::from(tload));
    println!("Gas cost: {}", tload.gas_cost());
    println!("Stack inputs: {}", tload.stack_inputs());
    println!("Stack outputs: {}", tload.stack_outputs());
    println!("Introduced in: {:?}", tload.introduced_in());
    println!("Group: {:?}", tload.group());
    println!("EIP: {:?}", tload.eip());
    println!();
}

fn fork_comparison() {
    println!("🔄 Example 2: Fork Comparison");
    println!("=============================");

    // Compare opcode availability across forks
    let opcodes_to_check = [
        (0x01, "ADD"),
        (0xf4, "DELEGATECALL"),
        (0xfd, "REVERT"),
        (0x5f, "PUSH0"),
        (0x5c, "TLOAD"),
    ];

    let _forks = [
        ("Frontier", Fork::Frontier),
        ("Homestead", Fork::Homestead),
        ("Byzantium", Fork::Byzantium),
        ("Shanghai", Fork::Shanghai),
        ("Cancun", Fork::Cancun),
    ];

    println!("Opcode availability across forks:");
    println!(
        "{:<12} {:<12} {:<10} {:<8} {:<8} {:<6}",
        "Opcode", "Frontier", "Homestead", "Byzantium", "Shanghai", "Cancun"
    );
    println!("{}", "-".repeat(70));

    for (opcode_byte, name) in opcodes_to_check {
        print!("{:<12}", name);

        // Check Frontier
        print!(
            " {:<12}",
            if Frontier::has_opcode(opcode_byte) {
                "✅"
            } else {
                "❌"
            }
        );

        // Check Homestead
        print!(
            " {:<10}",
            if Homestead::has_opcode(opcode_byte) {
                "✅"
            } else {
                "❌"
            }
        );

        // Check Byzantium
        print!(
            " {:<8}",
            if Byzantium::has_opcode(opcode_byte) {
                "✅"
            } else {
                "❌"
            }
        );

        // Check Shanghai
        print!(
            " {:<8}",
            if Shanghai::has_opcode(opcode_byte) {
                "✅"
            } else {
                "❌"
            }
        );

        // Check Cancun
        print!(
            " {:<6}",
            if Cancun::has_opcode(opcode_byte) {
                "✅"
            } else {
                "❌"
            }
        );

        println!();
    }
    println!();
}

fn explore_metadata() {
    println!("🔍 Example 3: Exploring Opcode Metadata");
    println!("=======================================");

    // Explore different types of opcodes
    let opcodes = [
        Cancun::ADD,
        Cancun::PUSH1,
        Cancun::DUP1,
        Cancun::SSTORE,
        Cancun::CREATE2,
        Cancun::TLOAD,
    ];

    for opcode in opcodes {
        let metadata = opcode.metadata();
        println!("📊 {} (0x{:02x})", metadata.name, metadata.opcode);
        println!("   Description: {}", metadata.description);
        println!("   Group: {:?}", metadata.group);
        println!(
            "   Gas: {}, Stack: {} → {}",
            metadata.gas_cost, metadata.stack_inputs, metadata.stack_outputs
        );
        println!("   Introduced: {:?}", metadata.introduced_in);
        if let Some(eip) = metadata.eip {
            println!("   EIP: {}", eip);
        }
        println!();
    }
}

fn gas_cost_analysis() {
    println!("⛽ Example 4: Gas Cost Analysis");
    println!("==============================");

    // Analyze a simple contract sequence
    let contract_opcodes = vec![
        0x60, // PUSH1
        0x60, // PUSH1
        0x01, // ADD
        0x60, // PUSH1
        0x55, // SSTORE
        0x60, // PUSH1
        0x54, // SLOAD
        0xf3, // RETURN
    ];

    println!("Analyzing contract opcode sequence:");
    let mut total_gas = 0u64;

    for (i, &byte) in contract_opcodes.iter().enumerate() {
        if Cancun::has_opcode(byte) {
            let opcode = Cancun::from(byte);
            let gas = opcode.gas_cost();
            total_gas += gas as u64;

            println!("  {}: {} (0x{:02x}) - {} gas", i + 1, opcode, byte, gas);
        }
    }

    println!("📈 Total estimated gas: {} (base costs only)", total_gas);
    println!("⚠️  Note: This is simplified - actual gas costs depend on execution context");
    println!();
}

fn opcode_conversion() {
    println!("🔄 Example 5: Opcode Conversion");
    println!("==============================");

    // Convert between different representations
    let add_opcode = Cancun::ADD;

    println!("Original opcode: {}", add_opcode);

    // Convert to byte
    let byte_val: u8 = add_opcode.into();
    println!("As byte: 0x{:02x}", byte_val);

    // Convert back from byte
    let back_to_opcode = Cancun::from(byte_val);
    println!("Back to opcode: {}", back_to_opcode);

    // Verify they're the same
    println!("Are they equal? {}", add_opcode == back_to_opcode);

    // Show opcode count for different forks
    println!("\nOpcode counts by fork:");
    println!("  Frontier: {} opcodes", Frontier::all_opcodes().len());
    println!("  Homestead: {} opcodes", Homestead::all_opcodes().len());
    println!("  Byzantium: {} opcodes", Byzantium::all_opcodes().len());
    println!(
        "  Constantinople: {} opcodes",
        Constantinople::all_opcodes().len()
    );
    println!("  Istanbul: {} opcodes", Istanbul::all_opcodes().len());
    println!("  Berlin: {} opcodes", Berlin::all_opcodes().len());
    println!("  London: {} opcodes", London::all_opcodes().len());
    println!("  Shanghai: {} opcodes", Shanghai::all_opcodes().len());
    println!("  Cancun: {} opcodes", Cancun::all_opcodes().len());

    println!("\n✅ Basic usage example completed!");
}
