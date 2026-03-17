//! Validation of the opcode table for internal consistency.
//!
//! These checks run against the static `OPCODE_TABLE` and verify things like
//! correct fork ordering, reasonable gas costs, and stack I/O sanity.

use crate::{Fork, OpCode};
use std::collections::HashMap;

/// Run all validation checks and return any errors.
pub fn validate() -> Result<(), Vec<String>> {
    let mut errors = Vec::new();
    errors.extend(validate_fork_availability());
    errors.extend(validate_historical_accuracy());
    errors.extend(validate_gas_costs());
    errors.extend(validate_stack_consistency());
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

/// Verify that opcodes are only available from their `introduced_in` fork onward.
fn validate_fork_availability() -> Vec<String> {
    let mut errors = Vec::new();
    for op in OpCode::iter_all() {
        let info = op.info().unwrap();
        // Should be valid at its introduction fork
        if !op.is_valid_in(info.introduced_in) {
            errors.push(format!(
                "0x{:02x} ({}) not valid at its own introduction fork {:?}",
                op.byte(),
                info.name,
                info.introduced_in
            ));
        }
        // Should NOT be valid one fork before (if there is one)
        let forks = Fork::ordered();
        if let Some(pos) = forks.iter().position(|f| *f == info.introduced_in) {
            if pos > 0 {
                let prev = forks[pos - 1];
                if op.is_valid_in(prev) {
                    errors.push(format!(
                        "0x{:02x} ({}) should not be valid at {:?} (introduced in {:?})",
                        op.byte(),
                        info.name,
                        prev,
                        info.introduced_in
                    ));
                }
            }
        }
    }
    errors
}

/// Cross-check known opcode introductions against the table.
fn validate_historical_accuracy() -> Vec<String> {
    let mut errors = Vec::new();
    let known = [
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
        (0x46, Fork::Istanbul, "CHAINID"),
        (0x47, Fork::Istanbul, "SELFBALANCE"),
        (0x48, Fork::London, "BASEFEE"),
        (0x5f, Fork::Shanghai, "PUSH0"),
        (0x5c, Fork::Cancun, "TLOAD"),
        (0x5d, Fork::Cancun, "TSTORE"),
        (0x5e, Fork::Cancun, "MCOPY"),
        (0x49, Fork::Cancun, "BLOBHASH"),
        (0x4a, Fork::Cancun, "BLOBBASEFEE"),
        (0x1e, Fork::Fusaka, "CLZ"),
        (0xd0, Fork::Fusaka, "DATALOAD"),
        (0xe0, Fork::Fusaka, "RJUMP"),
        (0xe3, Fork::Fusaka, "CALLF"),
        (0xe6, Fork::Fusaka, "DUPN"),
        (0xec, Fork::Fusaka, "EOFCREATE"),
        (0xf8, Fork::Fusaka, "EXTCALL"),
    ];
    for (byte, expected_fork, name) in known {
        match OpCode::new(byte) {
            Some(op) => {
                let info = op.info().unwrap();
                if info.introduced_in != expected_fork {
                    errors.push(format!(
                        "0x{byte:02x} ({name}) should be introduced in {expected_fork:?}, found {:?}",
                        info.introduced_in
                    ));
                }
            }
            None => {
                errors.push(format!("Missing opcode 0x{byte:02x} ({name})"));
            }
        }
    }
    errors
}

/// Check that gas costs are within reasonable bounds.
fn validate_gas_costs() -> Vec<String> {
    let mut errors = Vec::new();
    for op in OpCode::iter_all() {
        let info = op.info().unwrap();
        if info.base_gas > 50000 {
            errors.push(format!(
                "0x{:02x} ({}) has unusually high base gas: {}",
                op.byte(),
                info.name,
                info.base_gas
            ));
        }
    }
    errors
}

/// Validate stack I/O for DUP and SWAP opcodes.
fn validate_stack_consistency() -> Vec<String> {
    let mut errors = Vec::new();
    for op in OpCode::iter_all() {
        let info = op.info().unwrap();
        let byte = op.byte();

        if info.inputs > 17 {
            errors.push(format!(
                "0x{byte:02x} ({}) has {} inputs, exceeding EVM stack reach",
                info.name, info.inputs
            ));
        }

        // DUPn: inputs = n, outputs = n+1
        if (0x80..=0x8f).contains(&byte) {
            let n = byte - 0x7f;
            if info.inputs != n {
                errors.push(format!(
                    "DUP{n}: expected {n} inputs, found {}",
                    info.inputs
                ));
            }
            if info.outputs != n + 1 {
                errors.push(format!(
                    "DUP{n}: expected {} outputs, found {}",
                    n + 1,
                    info.outputs
                ));
            }
        }

        // SWAPn: inputs = n+1, outputs = n+1
        if (0x90..=0x9f).contains(&byte) {
            let n = byte - 0x8f;
            if info.inputs != n + 1 {
                errors.push(format!(
                    "SWAP{n}: expected {} inputs, found {}",
                    n + 1,
                    info.inputs
                ));
            }
            if info.outputs != n + 1 {
                errors.push(format!(
                    "SWAP{n}: expected {} outputs, found {}",
                    n + 1,
                    info.outputs
                ));
            }
        }
    }
    errors
}

/// Comprehensive validation report.
#[derive(Debug, Default)]
pub struct ValidationReport {
    /// Critical errors.
    pub errors: HashMap<String, Vec<String>>,
    /// Non-critical warnings.
    pub warnings: HashMap<String, Vec<String>>,
    /// Informational messages.
    pub info: HashMap<String, Vec<String>>,
}

impl ValidationReport {
    /// Run comprehensive validation.
    pub fn generate() -> Self {
        let mut report = Self::default();
        report.add_errors("Fork Availability", validate_fork_availability());
        report.add_errors("Historical Accuracy", validate_historical_accuracy());
        report.add_errors("Gas Costs", validate_gas_costs());
        report.add_errors("Stack Consistency", validate_stack_consistency());
        report.add_info("Coverage", generate_coverage_info());
        report
    }

    /// Add errors under a category.
    pub fn add_errors(&mut self, category: &str, errors: Vec<String>) {
        if !errors.is_empty() {
            self.errors.insert(category.into(), errors);
        }
    }

    /// Add warnings under a category.
    pub fn add_warnings(&mut self, category: &str, warnings: Vec<String>) {
        if !warnings.is_empty() {
            self.warnings.insert(category.into(), warnings);
        }
    }

    /// Add info under a category.
    pub fn add_info(&mut self, category: &str, info: Vec<String>) {
        if !info.is_empty() {
            self.info.insert(category.into(), info);
        }
    }

    /// Whether there are any errors.
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    /// Print a human-readable summary.
    pub fn print_summary(&self) {
        println!("=== EOT Validation Report ===");
        if !self.errors.is_empty() {
            println!("\nERRORS:");
            for (cat, errs) in &self.errors {
                println!("  {cat}:");
                for e in errs {
                    println!("    - {e}");
                }
            }
        }
        if !self.warnings.is_empty() {
            println!("\nWARNINGS:");
            for (cat, ws) in &self.warnings {
                println!("  {cat}:");
                for w in ws {
                    println!("    - {w}");
                }
            }
        }
        if !self.info.is_empty() {
            println!("\nINFO:");
            for (cat, items) in &self.info {
                println!("  {cat}:");
                for i in items {
                    println!("    {i}");
                }
            }
        }
        if self.errors.is_empty() && self.warnings.is_empty() {
            println!("\nAll validations passed!");
        }
    }
}

fn generate_coverage_info() -> Vec<String> {
    let mut info = Vec::new();
    let total = OpCode::iter_all().count();
    info.push(format!("Total known opcodes: {total}/256"));
    for &fork in Fork::ordered() {
        let count = OpCode::count_at(fork);
        info.push(format!("  {fork:?}: {count} opcodes"));
    }
    info
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_validations_pass() {
        assert!(validate().is_ok(), "Validation errors: {:?}", validate());
    }

    #[test]
    fn report_has_no_errors() {
        let report = ValidationReport::generate();
        assert!(!report.has_errors(), "Report errors: {:?}", report.errors);
    }
}
