//! Complete examples of using EOT's gas analysis capabilities
//!
//! This file demonstrates the gas analysis features for general EVM development:
//! - Dynamic gas cost calculation
//! - Gas optimization analysis and recommendations  
//! - Fork comparison and EIP impact analysis
//! - Bytecode efficiency analysis

use eot::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔥 EOT Gas Analysis Examples\n");

    // Example 1: Basic gas analysis
    basic_gas_analysis()?;
    println!("\n{}", "=".repeat(60));

    // Example 2: Dynamic gas calculation with context
    dynamic_gas_calculation()?;
    println!("\n{}", "=".repeat(60));

    // Example 3: ERC-20 transfer analysis
    analyze_erc20_transfer()?;
    println!("\n{}", "=".repeat(60));

    // Example 4: Fork comparison and EIP impact
    fork_comparison_analysis()?;
    println!("\n{}", "=".repeat(60));

    // Example 5: Gas optimization analysis
    gas_optimization_analysis()?;
    println!("\n{}", "=".repeat(60));

    // Example 6: Bytecode efficiency comparison
    bytecode_efficiency_comparison()?;

    Ok(())
}

/// Example 1: Basic gas analysis using the enhanced traits
fn basic_gas_analysis() -> Result<(), Box<dyn std::error::Error>> {
    println!("📊 Example 1: Basic Gas Analysis");

    // Create a simple bytecode sequence
    let opcodes = vec![
        0x60, 0x10, // PUSH1 0x10
        0x60, 0x20, // PUSH1 0x20
        0x01, // ADD
        0x60, 0x00, // PUSH1 0x00
        0x52, // MSTORE
        0x60, 0x20, // PUSH1 0x20
        0x60, 0x00, // PUSH1 0x00
        0xf3, // RETURN
    ];

    // Analyze gas usage using the registry
    use crate::traits::OpcodeAnalysis;
    let analysis = OpcodeRegistry::analyze_gas_usage(&opcodes, Fork::London);

    println!("Bytecode analysis:");
    println!("  Total gas: {} gas", analysis.total_gas);
    println!("  Efficiency score: {}/100", analysis.efficiency_score());
    println!("  Opcodes analyzed: {}", analysis.breakdown.len());

    // Show gas breakdown
    println!("\nGas breakdown:");
    for (opcode, gas_cost) in &analysis.breakdown {
        println!("  0x{:02x}: {} gas", opcode, gas_cost);
    }

    // Get optimization suggestions
    let suggestions = OpcodeRegistry::get_optimization_suggestions(&opcodes, Fork::Shanghai);
    if !suggestions.is_empty() {
        println!("\nOptimization suggestions:");
        for (i, suggestion) in suggestions.iter().enumerate() {
            println!("  {}. {}", i + 1, suggestion);
        }
    }

    // Validate the sequence
    match OpcodeRegistry::validate_opcode_sequence(&opcodes, Fork::London) {
        Ok(()) => println!("\n✅ Opcode sequence is valid"),
        Err(e) => println!("\n❌ Validation error: {}", e),
    }

    Ok(())
}

/// Example 2: Dynamic gas calculation with execution context
fn dynamic_gas_calculation() -> Result<(), Box<dyn std::error::Error>> {
    println!("⚡ Example 2: Dynamic Gas Calculation");

    let calculator = DynamicGasCalculator::new(Fork::Berlin);

    // Create different execution contexts
    let cold_context = ExecutionContext::new();
    let mut warm_context = ExecutionContext::new();

    // Pre-warm a storage slot - use fixed-size arrays
    let address = [0x12u8; 20]; // Fixed-size array for address
    let storage_key = {
        let mut key = [0u8; 32]; // Fixed-size array for storage key
        key[24..32].copy_from_slice(&0x123u64.to_be_bytes()); // Put the value in the last 8 bytes
        key
    };
    warm_context.mark_storage_accessed(&address, &storage_key);

    // Test SLOAD with cold vs warm access
    println!("SLOAD gas costs (EIP-2929 impact):");

    let cold_cost = calculator.calculate_gas_cost(0x54, &cold_context, &[0x123])?;
    let warm_cost = calculator.calculate_gas_cost(0x54, &warm_context, &[0x123])?;

    println!("  Cold access: {} gas", cold_cost);
    println!("  Warm access: {} gas", warm_cost);
    println!(
        "  Savings from warming: {} gas ({:.1}%)",
        cold_cost - warm_cost,
        (cold_cost - warm_cost) as f64 / cold_cost as f64 * 100.0
    );

    // Test memory expansion costs
    println!("\nMemory expansion costs:");
    let small_memory = calculator.calculate_gas_cost(0x52, &cold_context, &[32])?; // MSTORE at 32
    let large_memory = calculator.calculate_gas_cost(0x52, &cold_context, &[10000])?; // MSTORE at 10000

    println!("  Small memory access: {} gas", small_memory);
    println!("  Large memory access: {} gas", large_memory);
    println!(
        "  Memory expansion overhead: {} gas",
        large_memory - small_memory
    );

    Ok(())
}

/// Example 3: Analyze an ERC-20 transfer function
fn analyze_erc20_transfer() -> Result<(), Box<dyn std::error::Error>> {
    println!("💰 Example 3: ERC-20 Transfer Analysis");

    let calculator = DynamicGasCalculator::new(Fork::London);

    // Simplified ERC-20 transfer sequence
    let transfer_sequence = vec![
        (0x54, vec![0x1001]),        // SLOAD - sender balance
        (0x54, vec![0x1002]),        // SLOAD - receiver balance
        (0x03, vec![]),              // SUB - subtract from sender
        (0x01, vec![]),              // ADD - add to receiver
        (0x55, vec![0x1001, 0x100]), // SSTORE - update sender balance (key, value)
        (0x55, vec![0x1002, 0x200]), // SSTORE - update receiver balance (key, value)
        (0xa1, vec![0x40, 0x20]),    // LOG1 - Transfer event (offset, size)
    ];

    let analysis = calculator.analyze_sequence_gas(&transfer_sequence)?;

    println!("ERC-20 Transfer Gas Analysis:");
    println!("  Total gas: {} gas", analysis.total_gas);
    println!("  Base transaction: 21,000 gas");
    println!("  Transfer logic: {} gas", analysis.total_gas - 21000);

    // Analyze the most expensive operations
    let expensive_ops = analysis.top_expensive_operations(3);
    println!("\nMost expensive operations:");
    for (i, (opcode, cost)) in expensive_ops.iter().enumerate() {
        let name = match *opcode {
            0x54 => "SLOAD",
            0x55 => "SSTORE",
            0xa1 => "LOG1",
            _ => "OTHER",
        };
        println!("  {}. {}: {} gas", i + 1, name, cost);
    }

    // Show optimization opportunities
    if !analysis.optimizations.is_empty() {
        println!("\nOptimization opportunities:");
        for (i, opt) in analysis.optimizations.iter().enumerate() {
            println!("  {}. {}", i + 1, opt);
        }
    }

    Ok(())
}

/// Example 4: Compare gas costs across forks and analyze EIP impact
fn fork_comparison_analysis() -> Result<(), Box<dyn std::error::Error>> {
    println!("🍴 Example 4: Fork Comparison & EIP Impact");

    // Compare specific opcodes across forks
    let opcodes_to_compare = vec![
        (0x54, "SLOAD"),
        (0x31, "BALANCE"),
        (0x3b, "EXTCODESIZE"),
        (0xf1, "CALL"),
    ];

    let forks = vec![
        Fork::Istanbul,
        Fork::Berlin,
        Fork::London,
        Fork::Shanghai,
        Fork::Cancun,
    ];

    println!("Gas cost evolution across forks:");
    println!(
        "{:<12} {:<8} {:<8} {:<8} {:<8} {:<8}",
        "Opcode", "Istanbul", "Berlin", "London", "Shanghai", "Cancun"
    );
    println!("{}", "-".repeat(60));

    for (opcode, name) in opcodes_to_compare {
        print!("{:<12}", name);
        for fork in &forks {
            let calculator = DynamicGasCalculator::new(*fork);
            let context = ExecutionContext::new();

            match calculator.calculate_gas_cost(opcode, &context, &[0x123]) {
                Ok(cost) => print!(" {:<8}", cost),
                Err(_) => print!(" {:<8}", "N/A"),
            }
        }
        println!();
    }

    // Analyze changes between Berlin and pre-Berlin (EIP-2929 impact)
    println!("\nEIP-2929 Impact Analysis (Istanbul → Berlin):");
    use crate::gas::GasComparator;
    let changes = GasComparator::get_changes_between_forks(Fork::Istanbul, Fork::Berlin);

    for change in changes.iter().take(5) {
        if let (Some(old), Some(new)) = (change.old_value, change.new_value) {
            println!(
                "  0x{:02x}: {} → {} gas ({:+} gas)",
                change.opcode,
                old,
                new,
                new as i32 - old as i32
            );
        }
    }

    // Generate comparison report
    let report = GasComparator::generate_comparison_report(Fork::Istanbul, Fork::Berlin);
    println!("\nFork comparison summary:");
    println!(
        "  Opcodes with gas changes: {}",
        report.summary.gas_cost_changes
    );
    println!("  Gas increases: {}", report.summary.gas_increases);
    println!("  Gas decreases: {}", report.summary.gas_decreases);

    Ok(())
}

/// Example 5: Gas optimization analysis
fn gas_optimization_analysis() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔧 Example 5: Gas Optimization Analysis");

    // Original inefficient contract that reads the same storage slot multiple times
    let original_contract = vec![
        (0x54, vec![0x100]),       // SLOAD slot 0x100
        (0x01, vec![]),            // ADD with something
        (0x54, vec![0x100]),       // SLOAD same slot again (inefficient!)
        (0x02, vec![]),            // MUL
        (0x54, vec![0x100]),       // SLOAD same slot third time!
        (0x03, vec![]),            // SUB
        (0x55, vec![0x200, 0x42]), // SSTORE result (key, value)
    ];

    // Optimized version that caches the storage value
    let optimized_contract = vec![
        (0x54, vec![0x100]),       // SLOAD slot 0x100 once
        (0x80, vec![]),            // DUP1 - duplicate the value
        (0x01, vec![]),            // ADD
        (0x81, vec![]),            // DUP2 - use cached value
        (0x02, vec![]),            // MUL
        (0x82, vec![]),            // DUP3 - use cached value again
        (0x03, vec![]),            // SUB
        (0x55, vec![0x200, 0x42]), // SSTORE result (key, value)
    ];

    let calculator = DynamicGasCalculator::new(Fork::London);

    let original_analysis = calculator.analyze_sequence_gas(&original_contract)?;
    let optimized_analysis = calculator.analyze_sequence_gas(&optimized_contract)?;

    println!("Original contract:");
    println!("  Total gas: {}", original_analysis.total_gas);
    println!(
        "  Efficiency score: {}",
        original_analysis.efficiency_score()
    );

    println!("\nOptimized contract:");
    println!("  Total gas: {}", optimized_analysis.total_gas);
    println!(
        "  Efficiency score: {}",
        optimized_analysis.efficiency_score()
    );

    let savings = original_analysis.total_gas - optimized_analysis.total_gas;
    let savings_percent = (savings as f64 / original_analysis.total_gas as f64) * 100.0;

    println!("\nOptimization results:");
    println!("  Gas saved: {} ({:.1}%)", savings, savings_percent);

    if savings > 0 {
        println!("  ✅ Optimization successful!");
    } else {
        println!("  ❌ Optimization made things worse!");
    }

    Ok(())
}

/// Example 6: Compare bytecode efficiency between different implementations
fn bytecode_efficiency_comparison() -> Result<(), Box<dyn std::error::Error>> {
    println!("⚖️  Example 6: Bytecode Efficiency Comparison");

    let calculator = DynamicGasCalculator::new(Fork::Shanghai);

    // Different ways to push zero onto the stack
    let push_zero_old = vec![(0x60, vec![0])]; // PUSH1 0x00
    let push_zero_new = vec![(0x5f, vec![])]; // PUSH0 (Shanghai+)

    // Different ways to check if value is zero
    let iszero_simple = vec![
        (0x15, vec![]), // ISZERO
    ];
    let iszero_complex = vec![
        (0x80, vec![]), // DUP1
        (0x80, vec![]), // DUP1
        (0x14, vec![]), // EQ (compare with itself)
        (0x15, vec![]), // ISZERO
    ];

    println!("Efficiency comparison results:");

    // Compare PUSH implementations
    let old_push_analysis = calculator.analyze_sequence_gas(&push_zero_old)?;
    let new_push_analysis = calculator.analyze_sequence_gas(&push_zero_new)?;

    println!("\nPush zero implementations:");
    println!("  PUSH1 0x00: {} gas", old_push_analysis.total_gas);
    println!("  PUSH0:      {} gas", new_push_analysis.total_gas);
    println!(
        "  Savings:    {} gas",
        old_push_analysis.total_gas - new_push_analysis.total_gas
    );

    // Compare zero-check implementations
    let simple_analysis = calculator.analyze_sequence_gas(&iszero_simple)?;
    let complex_analysis = calculator.analyze_sequence_gas(&iszero_complex)?;

    println!("\nZero-check implementations:");
    println!(
        "  Simple ISZERO:  {} gas (efficiency: {}%)",
        simple_analysis.total_gas,
        simple_analysis.efficiency_score()
    );
    println!(
        "  Complex check:  {} gas (efficiency: {}%)",
        complex_analysis.total_gas,
        complex_analysis.efficiency_score()
    );

    // General recommendations
    println!("\n📋 General Optimization Recommendations:");
    println!("  1. Use PUSH0 instead of PUSH1 0x00 (Shanghai+)");
    println!("  2. Avoid redundant DUP operations");
    println!("  3. Cache storage reads when accessing the same slot multiple times");
    println!("  4. Use events instead of storage for data that doesn't need querying");
    println!("  5. Consider using newer opcodes for better efficiency");

    Ok(())
}
