//! Validation and verification system for opcode consistency with gas analysis integration

use crate::{Fork, OpcodeRegistry, traits::OpcodeAnalysis, gas::GasAnalysis};
use std::collections::{HashMap, HashSet};

/// Validate the entire opcode registry for consistency
pub fn validate_registry(registry: &OpcodeRegistry) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();

    // Run all validation checks
    errors.extend(validate_opcode_uniqueness(registry));
    errors.extend(validate_fork_inheritance(registry));
    errors.extend(validate_historical_accuracy(registry));
    errors.extend(validate_gas_cost_consistency(registry));
    errors.extend(validate_stack_consistency(registry));
    errors.extend(validate_gas_analysis_integration(registry));

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

/// Ensure no opcode is defined twice in the same fork
fn validate_opcode_uniqueness(registry: &OpcodeRegistry) -> Vec<String> {
    let mut errors = Vec::new();

    for (fork, opcodes) in &registry.opcodes {
        let mut seen = HashSet::new();

        for &opcode_byte in opcodes.keys() {
            if !seen.insert(opcode_byte) {
                errors.push(format!(
                    "Duplicate opcode 0x{opcode_byte:02x} found in fork {fork:?}"
                ));
            }
        }
    }

    errors
}

/// Validate that forks properly inherit opcodes from previous forks
fn validate_fork_inheritance(registry: &OpcodeRegistry) -> Vec<String> {
    let mut errors = Vec::new();

    let fork_order = [
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

    for i in 1..fork_order.len() {
        let current_fork = fork_order[i];
        let previous_fork = fork_order[i - 1];

        let current_opcodes = registry.get_opcodes(current_fork);
        let previous_opcodes = registry.get_opcodes(previous_fork);

        // Check that all opcodes from previous fork exist in current fork
        // (unless explicitly removed, which is rare)
        for (opcode_byte, metadata) in &previous_opcodes {
            if metadata.introduced_in <= previous_fork && !current_opcodes.contains_key(opcode_byte)
            {
                errors.push(format!(
                    "Opcode 0x{:02x} ({}) missing from fork {:?} but exists in {:?}",
                    opcode_byte, metadata.name, current_fork, previous_fork
                ));
            }
        }
    }

    errors
}

/// Validate historical accuracy against known opcode introductions
fn validate_historical_accuracy(registry: &OpcodeRegistry) -> Vec<String> {
    let mut errors = Vec::new();

    // Known historical facts about opcode introductions
    let known_introductions = [
        (0xf4, Fork::Homestead, "DELEGATECALL"),
        (0x3d, Fork::Byzantium, "RETURNDATASIZE"),
        (0x3e, Fork::Byzantium, "RETURNDATACOPY"),
        (0xfa, Fork::Byzantium, "STATICCALL"),
        (0xfd, Fork::Byzantium, "REVERT"),
        (0x1b, Fork::Constantinople, "SHL"),
        (0x1c, Fork::Constantinople, "SHR"),
        (0x1d, Fork::Constantinople, "SAR"),
        (0x3f, Fork::Constantinople, "EXTCODEHASH"),
        (0xf5, Fork::Constantinople, "CREATE2"),
        (0x5f, Fork::Shanghai, "PUSH0"),
        (0x5c, Fork::Cancun, "TLOAD"),
        (0x5d, Fork::Cancun, "TSTORE"),
        (0x5e, Fork::Cancun, "MCOPY"),
        (0x49, Fork::Cancun, "BLOBHASH"),
        (0x4a, Fork::Cancun, "BLOBBASEFEE"),
    ];

    for (opcode_byte, expected_fork, name) in &known_introductions {
        let all_opcodes = registry.get_opcodes(Fork::Cancun); // Get from latest fork

        if let Some(metadata) = all_opcodes.get(opcode_byte) {
            if metadata.introduced_in != *expected_fork {
                errors.push(format!(
                    "Opcode 0x{:02x} ({}) should be introduced in {:?} but found in {:?}",
                    opcode_byte, name, expected_fork, metadata.introduced_in
                ));
            }
        } else {
            errors.push(format!(
                "Missing expected opcode 0x{opcode_byte:02x} ({name}) introduced in {expected_fork:?}"
            ));
        }
    }

    errors
}

/// Validate gas cost consistency and historical changes
fn validate_gas_cost_consistency(registry: &OpcodeRegistry) -> Vec<String> {
    let mut errors = Vec::new();

    // Check for reasonable gas costs
    for (fork, opcodes) in &registry.opcodes {
        for (opcode_byte, metadata) in opcodes {
            // Gas costs should be reasonable (not negative, not absurdly high)
            if metadata.gas_cost > 50000 {
                errors.push(format!(
                    "Unusually high gas cost {} for opcode 0x{:02x} ({}) in fork {:?}",
                    metadata.gas_cost, opcode_byte, metadata.name, fork
                ));
            }

            // Validate gas history is in chronological order
            let mut last_fork = None;
            for (gas_fork, _) in metadata.gas_history {
                if let Some(last) = last_fork {
                    if gas_fork < &last {
                        errors.push(format!(
                            "Gas history for opcode 0x{:02x} ({}) is not in chronological order",
                            opcode_byte, metadata.name
                        ));
                    }
                }
                last_fork = Some(*gas_fork);
            }
        }
    }

    errors
}

/// Validate stack input/output consistency
fn validate_stack_consistency(registry: &OpcodeRegistry) -> Vec<String> {
    let mut errors = Vec::new();

    for opcodes in registry.opcodes.values() {
        for (opcode_byte, metadata) in opcodes {
            // Basic sanity checks
            if metadata.stack_inputs > 17 {
                errors.push(format!(
                    "Opcode 0x{:02x} ({}) has more than 17 stack inputs ({}), which exceeds EVM stack limit",
                    opcode_byte, metadata.name, metadata.stack_inputs
                ));
            }

            if metadata.stack_outputs > 1 && !matches!(opcode_byte, 0x80..=0x8f) {
                // Only DUP opcodes should produce more than 1 output
                errors.push(format!(
                    "Non-DUP opcode 0x{:02x} ({}) produces {} stack outputs",
                    opcode_byte, metadata.name, metadata.stack_outputs
                ));
            }

            // Validate DUP opcodes
            if (0x80..=0x8f).contains(opcode_byte) {
                let dup_num = opcode_byte - 0x7f;
                if metadata.stack_inputs != dup_num {
                    errors.push(format!(
                        "DUP{} opcode should have {} stack inputs, found {}",
                        dup_num, dup_num, metadata.stack_inputs
                    ));
                }
                if metadata.stack_outputs != dup_num + 1 {
                    errors.push(format!(
                        "DUP{} opcode should have {} stack outputs, found {}",
                        dup_num,
                        dup_num + 1,
                        metadata.stack_outputs
                    ));
                }
            }

            // Validate SWAP opcodes
            if (0x90..=0x9f).contains(opcode_byte) {
                let swap_num = opcode_byte - 0x8f;
                if metadata.stack_inputs != swap_num + 1 {
                    errors.push(format!(
                        "SWAP{} opcode should have {} stack inputs, found {}",
                        swap_num,
                        swap_num + 1,
                        metadata.stack_inputs
                    ));
                }
                if metadata.stack_outputs != swap_num + 1 {
                    errors.push(format!(
                        "SWAP{} opcode should have {} stack outputs, found {}",
                        swap_num,
                        swap_num + 1,
                        metadata.stack_outputs
                    ));
                }
            }
        }
    }

    errors
}

/// Validate integration with gas analysis system
fn validate_gas_analysis_integration(_registry: &OpcodeRegistry) -> Vec<String> {
    let mut errors = Vec::new();

    // Test gas analysis on a simple sequence for each fork
    let test_sequence = vec![0x01, 0x02, 0x03]; // ADD, MUL, SUB

    for fork in [Fork::Frontier, Fork::Berlin, Fork::London, Fork::Shanghai, Fork::Cancun] {
        match std::panic::catch_unwind(|| {
            let analysis = OpcodeRegistry::analyze_gas_usage(&test_sequence, fork);
            
            // Basic sanity checks
            if analysis.total_gas < 21000 {
                return Err("Gas analysis returned less than base transaction cost".to_string());
            }
            
            if analysis.breakdown.len() != test_sequence.len() {
                return Err("Gas breakdown length doesn't match sequence length".to_string());
            }
            
            Ok(())
        }) {
            Ok(Ok(())) => {}, // Success
            Ok(Err(e)) => errors.push(format!("Gas analysis validation failed for {:?}: {}", fork, e)),
            Err(_) => errors.push(format!("Gas analysis panicked for fork {:?}", fork)),
        }
    }

    errors
}

/// Known gas cost changes between forks for validation
struct KnownGasChanges {
    /// Opcode byte
    opcode: u8,
    /// Fork where change occurred
    fork: Fork,
    /// Old gas cost
    old_cost: u16,
    /// New gas cost
    new_cost: u16,
    /// Reason for change
    reason: &'static str,
}

/// Validate against known historical gas cost changes
pub fn validate_known_gas_changes(registry: &OpcodeRegistry) -> Vec<String> {
    let mut errors = Vec::new();

    // Some known gas cost changes (this would be a comprehensive list)
    let known_changes = [
        KnownGasChanges {
            opcode: 0x54, // SLOAD
            fork: Fork::Berlin,
            old_cost: 800,
            new_cost: 2100,
            reason: "EIP-2929: Gas cost increases for state access opcodes",
        },
        KnownGasChanges {
            opcode: 0x31, // BALANCE
            fork: Fork::Berlin,
            old_cost: 400,
            new_cost: 2600,
            reason: "EIP-2929: Gas cost increases for state access opcodes",
        },
        KnownGasChanges {
            opcode: 0x3b, // EXTCODESIZE
            fork: Fork::Berlin,
            old_cost: 700,
            new_cost: 2600,
            reason: "EIP-2929: Gas cost increases for state access opcodes",
        },
        KnownGasChanges {
            opcode: 0x3c, // EXTCODECOPY
            fork: Fork::Berlin,
            old_cost: 700,
            new_cost: 2600,
            reason: "EIP-2929: Gas cost increases for state access opcodes",
        },
        KnownGasChanges {
            opcode: 0x3f, // EXTCODEHASH
            fork: Fork::Berlin,
            old_cost: 400,
            new_cost: 2600,
            reason: "EIP-2929: Gas cost increases for state access opcodes",
        },
    ];

    for change in &known_changes {
        // Check if the opcode exists in both the pre-change and post-change forks
        let pre_fork_opcodes = registry.get_opcodes(get_previous_fork(change.fork));
        let post_fork_opcodes = registry.get_opcodes(change.fork);

        if let (Some(pre_metadata), Some(post_metadata)) = (
            pre_fork_opcodes.get(&change.opcode),
            post_fork_opcodes.get(&change.opcode),
        ) {
            if pre_metadata.gas_cost != change.old_cost {
                errors.push(format!(
                    "Expected gas cost {} for opcode 0x{:02x} before fork {:?}, found {}",
                    change.old_cost, change.opcode, change.fork, pre_metadata.gas_cost
                ));
            }

            let actual_new_cost = post_metadata
                .gas_history
                .iter()
                .find(|(f, _)| *f == change.fork)
                .map(|(_, cost)| *cost)
                .unwrap_or(post_metadata.gas_cost);

            if actual_new_cost != change.new_cost {
                errors.push(format!(
                    "Expected gas cost {} for opcode 0x{:02x} after fork {:?}, found {}. Reason: {}",
                    change.new_cost, change.opcode, change.fork, actual_new_cost, change.reason
                ));
            }
        }
    }

    errors
}

/// Get the fork that immediately precedes the given fork
fn get_previous_fork(fork: Fork) -> Fork {
    match fork {
        Fork::IceAge => Fork::Frontier,
        Fork::Homestead => Fork::IceAge,
        Fork::DaoFork => Fork::Homestead,
        Fork::TangerineWhistle => Fork::DaoFork,
        Fork::SpuriousDragon => Fork::TangerineWhistle,
        Fork::Byzantium => Fork::SpuriousDragon,
        Fork::Constantinople => Fork::Byzantium,
        Fork::Petersburg => Fork::Constantinople,
        Fork::Istanbul => Fork::Petersburg,
        Fork::MuirGlacier => Fork::Istanbul,
        Fork::Berlin => Fork::MuirGlacier,
        Fork::London => Fork::Berlin,
        Fork::Altair => Fork::London,
        Fork::ArrowGlacier => Fork::Altair,
        Fork::GrayGlacier => Fork::ArrowGlacier,
        Fork::Bellatrix => Fork::GrayGlacier,
        Fork::Paris => Fork::Bellatrix,
        Fork::Shanghai => Fork::Paris,
        Fork::Capella => Fork::Shanghai,
        Fork::Cancun => Fork::Capella,
        Fork::Deneb => Fork::Cancun,
        Fork::Frontier => Fork::Frontier, // No previous fork
    }
}

/// Check for common validation patterns and issues
pub fn run_comprehensive_validation(registry: &OpcodeRegistry) -> ValidationReport {
    let mut report = ValidationReport::new();

    // Run all validation checks
    report.add_errors("Opcode Uniqueness", validate_opcode_uniqueness(registry));
    report.add_errors("Fork Inheritance", validate_fork_inheritance(registry));
    report.add_errors(
        "Historical Accuracy",
        validate_historical_accuracy(registry),
    );
    report.add_errors(
        "Gas Cost Consistency",
        validate_gas_cost_consistency(registry),
    );
    report.add_errors("Stack Consistency", validate_stack_consistency(registry));
    report.add_errors("Known Gas Changes", validate_known_gas_changes(registry));
    report.add_errors("Gas Analysis Integration", validate_gas_analysis_integration(registry));

    // Additional checks
    report.add_warnings("Missing EIPs", check_missing_eip_references(registry));
    report.add_info("Coverage", generate_coverage_info(registry));
    report.add_info("Gas Analysis", generate_gas_analysis_info(registry));

    report
}

/// Comprehensive validation report
#[derive(Debug, Default)]
pub struct ValidationReport {
    /// Critical errors that must be fixed
    pub errors: HashMap<String, Vec<String>>,
    /// Warnings that should be addressed
    pub warnings: HashMap<String, Vec<String>>,
    /// Informational messages
    pub info: HashMap<String, Vec<String>>,
}

impl ValidationReport {
    /// Create a new validation report
    pub fn new() -> Self {
        Self::default()
    }

    /// Add errors to a category
    pub fn add_errors(&mut self, category: &str, errors: Vec<String>) {
        if !errors.is_empty() {
            self.errors.insert(category.to_string(), errors);
        }
    }

    /// Add warnings to a category
    pub fn add_warnings(&mut self, category: &str, warnings: Vec<String>) {
        if !warnings.is_empty() {
            self.warnings.insert(category.to_string(), warnings);
        }
    }

    /// Add info to a category
    pub fn add_info(&mut self, category: &str, info: Vec<String>) {
        if !info.is_empty() {
            self.info.insert(category.to_string(), info);
        }
    }

    /// Check if report has any errors
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    /// Print summary of validation report
    pub fn print_summary(&self) {
        println!("=== EOT Validation Report ===");

        if !self.errors.is_empty() {
            println!("\n❌ ERRORS:");
            for (category, errors) in &self.errors {
                println!("  {category}:");
                for error in errors {
                    println!("    - {error}");
                }
            }
        }

        if !self.warnings.is_empty() {
            println!("\n⚠️  WARNINGS:");
            for (category, warnings) in &self.warnings {
                println!("  {category}:");
                for warning in warnings {
                    println!("    - {warning}");
                }
            }
        }

        if !self.info.is_empty() {
            println!("\nℹ️  INFO:");
            for (category, info_items) in &self.info {
                println!("  {category}:");
                for info in info_items {
                    println!("    {info}");
                }
            }
        }

        if self.errors.is_empty() && self.warnings.is_empty() {
            println!("\n✅ All validations passed!");
        }
    }
}

/// Check for opcodes missing EIP references
fn check_missing_eip_references(registry: &OpcodeRegistry) -> Vec<String> {
    let mut warnings = Vec::new();

    for opcodes in registry.opcodes.values() {
        for (opcode_byte, metadata) in opcodes {
            // Opcodes introduced after Frontier should generally have EIP references
            if metadata.introduced_in > Fork::Frontier && metadata.eip.is_none() {
                warnings.push(format!(
                    "Opcode 0x{:02x} ({}) introduced in {:?} is missing EIP reference",
                    opcode_byte, metadata.name, metadata.introduced_in
                ));
            }
        }
    }

    warnings
}

/// Generate coverage information
fn generate_coverage_info(registry: &OpcodeRegistry) -> Vec<String> {
    let mut info = Vec::new();

    let total_possible_opcodes = 256;
    let latest_opcodes = registry.get_opcodes(Fork::Cancun);
    let coverage_percentage = (latest_opcodes.len() * 100) / total_possible_opcodes;

    info.push(format!(
        "Total opcodes implemented: {} / {} ({:.1}% coverage)",
        latest_opcodes.len(),
        total_possible_opcodes,
        coverage_percentage as f32
    ));

    // Count opcodes by fork
    for fork in [
        Fork::Frontier,
        Fork::Homestead,
        Fork::Byzantium,
        Fork::Constantinople,
        Fork::Istanbul,
        Fork::Berlin,
        Fork::London,
        Fork::Shanghai,
        Fork::Cancun,
    ] {
        let opcodes = registry.get_opcodes(fork);
        info.push(format!("{:?}: {} opcodes", fork, opcodes.len()));
    }

    info
}

/// Generate gas analysis system information
fn generate_gas_analysis_info(_registry: &OpcodeRegistry) -> Vec<String> {
    let mut info = Vec::new();

    // Test gas analysis capabilities
    let test_sequences = vec![
        (vec![0x01, 0x02, 0x03], "Simple arithmetic"),
        (vec![0x54, 0x55], "Storage operations"),
        (vec![0x51, 0x52], "Memory operations"),
        (vec![0xf1], "Call operation"),
    ];

    for (sequence, description) in test_sequences {
        let analysis = OpcodeRegistry::analyze_gas_usage(&sequence, Fork::London);
        info.push(format!(
            "{}: {} gas (efficiency: {}%)",
            description,
            analysis.total_gas,
            analysis.efficiency_score()
        ));
    }

    // Test dynamic gas calculation features
    use crate::gas::{DynamicGasCalculator, ExecutionContext};
    
    let calculator = DynamicGasCalculator::new(Fork::Berlin);
    let context = ExecutionContext::new();
    
    // Test warm/cold SLOAD costs
    if let Ok(cold_cost) = calculator.calculate_gas_cost(0x54, &context, &[0x123]) {
        let mut warm_context = context.clone();
        warm_context.mark_storage_accessed(&vec![0u8; 20], &0x123u64.to_be_bytes());
        
        if let Ok(warm_cost) = calculator.calculate_gas_cost(0x54, &warm_context, &[0x123]) {
            info.push(format!(
                "SLOAD gas costs (Berlin): cold={}, warm={}",
                cold_cost, warm_cost
            ));
        }
    }

    // Test memory expansion
    if let Ok(small_memory_cost) = calculator.calculate_gas_cost(0x52, &context, &[64]) {
        if let Ok(large_memory_cost) = calculator.calculate_gas_cost(0x52, &context, &[10000]) {
            info.push(format!(
                "MSTORE gas costs: small_memory={}, large_memory={}",
                small_memory_cost, large_memory_cost
            ));
        }
    }

    info.push("Gas analysis system: ✅ Operational".to_string());

    info
}

/// Extended implementation of OpcodeAnalysis for the registry
impl OpcodeAnalysis for OpcodeRegistry {
    fn analyze_gas_usage(opcodes: &[u8], fork: Fork) -> GasAnalysis {
        use crate::gas::GasAnalyzer;
        GasAnalyzer::analyze_gas_usage(opcodes, fork)
    }

    fn validate_opcode_sequence(opcodes: &[u8], fork: Fork) -> Result<(), String> {
        use crate::gas::GasAnalyzer;
        GasAnalyzer::validate_opcode_sequence(opcodes, fork)
    }

    fn get_optimization_suggestions(opcodes: &[u8], fork: Fork) -> Vec<String> {
        let analysis = Self::analyze_gas_usage(opcodes, fork);
        let mut suggestions = analysis.get_optimization_recommendations();

        // Add fork-specific suggestions
        use crate::gas::GasOptimizationAdvisor;
        suggestions.extend(GasOptimizationAdvisor::analyze_pattern(opcodes, fork));

        suggestions
    }

    fn estimate_gas_savings(opcodes: &[u8], fork: Fork) -> u64 {
        let analysis = Self::analyze_gas_usage(opcodes, fork);
        analysis.estimate_optimization_savings()
    }
}
