//! Gas analysis example for EOT
//!
//! Run with: cargo run --example gas_analysis

use eot::gas::{
    DynamicGasCalculator, ExecutionContext, GasAnalyzer, GasComparator, GasOptimizationAdvisor,
};
use eot::Fork;

fn main() {
    println!("EOT - Gas Analysis Example\n");

    // 1. Dynamic gas calculation
    println!("== Dynamic Gas (Berlin) ==");
    let calc = DynamicGasCalculator::new(Fork::Berlin);
    let mut ctx = ExecutionContext::new();

    let cold = calc.calculate_gas_cost(0x54, &ctx, &[0x100]).unwrap();
    println!("SLOAD cold: {cold} gas");

    // Warm the slot
    let key = {
        let mut k = [0u8; 32];
        k[24..32].copy_from_slice(&0x100u64.to_be_bytes());
        k
    };
    let addr = ctx.current_address;
    ctx.mark_storage_accessed(&addr, &key);
    let warm = calc.calculate_gas_cost(0x54, &ctx, &[0x100]).unwrap();
    println!("SLOAD warm: {warm} gas");
    println!();

    // 2. Sequence analysis
    println!("== Sequence Analysis ==");
    let seq = [0x60, 0x54, 0x54, 0x01, 0x55];
    let analysis = GasAnalyzer::analyze_gas_usage(&seq, Fork::London);
    println!("Total gas: {}", analysis.total_gas);
    println!("Efficiency: {}%", analysis.efficiency_score());
    for opt in analysis.get_optimization_recommendations() {
        println!("  Tip: {opt}");
    }
    println!();

    // 3. Fork comparison
    println!("== Istanbul -> Berlin Changes ==");
    let report = GasComparator::generate_comparison_report(Fork::Istanbul, Fork::Berlin);
    report.print_report();
    println!();

    // 4. Optimization advice
    println!("== Cancun Optimizations ==");
    for tip in GasOptimizationAdvisor::get_fork_optimizations(Fork::Cancun) {
        println!("  - {tip}");
    }
}
