//! Basic usage example for EOT
//!
//! Run with: cargo run --example basic_usage

use eot::{Fork, OpCode};

fn main() {
    println!("EOT (EVM Opcode Table) - Basic Usage\n");

    // 1. Opcode metadata
    let tload = OpCode::TLOAD;
    let info = tload.info().unwrap();
    println!("Opcode: {tload}");
    println!("  Byte: 0x{:02x}", tload.byte());
    println!("  Gas: {}", tload.gas_cost(Fork::Cancun));
    println!("  Stack: {} in, {} out", info.inputs, info.outputs);
    println!("  Introduced: {:?}", info.introduced_in);
    println!("  EIP: {:?}", info.eip);
    println!();

    // 2. Fork availability
    println!("Fork availability:");
    let checks = [
        (0x01, "ADD"),
        (0xf4, "DELEGATECALL"),
        (0xfd, "REVERT"),
        (0x5f, "PUSH0"),
        (0x5c, "TLOAD"),
    ];
    let forks = [
        Fork::Frontier,
        Fork::Homestead,
        Fork::Byzantium,
        Fork::Shanghai,
        Fork::Cancun,
    ];
    print!("{:<14}", "Opcode");
    for f in &forks {
        print!("{:<14}", format!("{f:?}"));
    }
    println!();
    for (byte, name) in checks {
        let op = OpCode::from_byte(byte);
        print!("{:<14}", name);
        for &f in &forks {
            print!("{:<14}", if op.is_valid_in(f) { "yes" } else { "-" });
        }
        println!();
    }
    println!();

    // 3. Gas cost evolution
    println!("SLOAD gas cost across forks:");
    for &fork in Fork::ordered() {
        if OpCode::SLOAD.is_valid_in(fork) {
            println!("  {:?}: {} gas", fork, OpCode::SLOAD.gas_cost(fork));
        }
    }
    println!();

    // 4. Opcode counts
    println!("Opcodes per fork:");
    for &fork in Fork::ordered() {
        println!("  {:?}: {}", fork, OpCode::count_at(fork));
    }
}
