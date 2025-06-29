# EOT - EVM Opcode Table

A comprehensive Rust implementation of EVM opcodes for all Ethereum forks, with complete fork inheritance, validation, and metadata.

[![Crates.io](https://img.shields.io/crates/v/eot.svg)](https://crates.io/crates/eot)
[![Documentation](https://docs.rs/eot/badge.svg)](https://docs.rs/eot)
[![Build Status](https://github.com/g4titanx/eot/workflows/CI/badge.svg)](https://github.com/g4titanx/eot/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## Features

- ✅ **Complete Fork Coverage**: Frontier → Cancun (and beyond)
- ✅ **Smart Inheritance**: Forks automatically inherit opcodes from previous forks
- ✅ **Rich Metadata**: Gas costs, stack effects, EIP references, descriptions
- ✅ **Historical Gas Tracking**: Track gas cost changes across forks
- ✅ **Built-in Validation**: Verify consistency and historical accuracy
- ✅ **Type Safety**: Each fork is a separate type with compile-time guarantees
- ✅ **Zero Dependencies**: Lightweight with no external dependencies
- ✅ **Data-Driven**: Generated from verified CSV data for accuracy

## Quick Start

Add this to your `Cargo.toml`:

```toml
[dependencies]
eot = "0.1"
```

Basic usage:

```rust
use eot::{Cancun, OpCode, Fork};

// Use the latest fork (Cancun)
let tload = Cancun::TLOAD;
println!("Gas cost: {}", tload.gas_cost());           // 100
println!("Introduced in: {:?}", tload.introduced_in()); // Fork::Cancun
println!("EIP: {:?}", tload.eip());                    // Some(1153)

// Check if an opcode exists in a fork
if Cancun::has_opcode(0x5c) {
    println!("TLOAD exists in Cancun!");
}

// Get all opcodes for a fork
let all_opcodes = Cancun::all_opcodes();
println!("Cancun has {} opcodes", all_opcodes.len());

// Convert between opcode and byte value
let byte_val: u8 = tload.into();        // 0x5c
let back_to_opcode = Cancun::from(byte_val);
assert_eq!(tload, back_to_opcode);
```

## Architecture

### Smart Fork System

Instead of manually copying opcodes between forks, we use automatic inheritance:

```
Frontier (Base) → Homestead → Byzantium → Constantinople → Istanbul → Berlin → London → Shanghai → Cancun
```

Each fork automatically includes all opcodes from previous forks plus its own additions.

### Rich Metadata

Every opcode includes complete information:

```rust
use eot::{Cancun, OpCode};

let tload = Cancun::TLOAD;
let metadata = tload.metadata();

assert_eq!(metadata.opcode, 0x5c);
assert_eq!(metadata.name, "TLOAD");
assert_eq!(metadata.gas_cost, 100);
assert_eq!(metadata.stack_inputs, 1);
assert_eq!(metadata.stack_outputs, 1);
assert_eq!(metadata.introduced_in, Fork::Cancun);
assert_eq!(metadata.group, Group::StackMemoryStorageFlow);
assert_eq!(metadata.eip, Some(1153));
```

### Advanced Features

```rust
use eot::{Cancun, traits::OpcodeExt};

// State modification analysis
let sstore = Cancun::SSTORE;
println!("Modifies state: {}", sstore.modifies_state()); // true
println!("Can revert: {}", sstore.can_revert());         // false

// Push opcode analysis
let push1 = Cancun::PUSH1;
println!("Is push opcode: {}", push1.is_push());         // true
println!("Push size: {:?}", push1.push_size());          // Some(1)

// Stack depth requirements
let dup5 = Cancun::DUP5;
println!("Min stack depth: {}", dup5.min_stack_depth()); // 5

// Opcode groups
let add = Cancun::ADD;
println!("Group: {:?}", add.group()); // Group::StopArithmetic
```

## Supported Forks

| Fork | Block | Date | New Opcodes | Status |
|------|-------|------|-------------|---------|
| Frontier | 0 | Jul 2015 | Base set (140+ opcodes) | ✅ |
| Homestead | 1,150,000 | Mar 2016 | `DELEGATECALL` | ✅ |
| Byzantium | 4,370,000 | Oct 2017 | `REVERT`, `RETURNDATASIZE`, `RETURNDATACOPY`, `STATICCALL` | ✅ |
| Constantinople | 7,280,000 | Feb 2019 | `SHL`, `SHR`, `SAR`, `CREATE2`, `EXTCODEHASH` | ✅ |
| Istanbul | 9,069,000 | Dec 2019 | `CHAINID`, `SELFBALANCE` | ✅ |
| Berlin | 12,244,000 | Apr 2021 | Gas cost changes | ✅ |
| London | 12,965,000 | Aug 2021 | `BASEFEE` | ✅ |
| Shanghai | 17,034,870 | Apr 2023 | `PUSH0` | ✅ |
| Cancun | 19,426,587 | Mar 2024 | `TLOAD`, `TSTORE`, `MCOPY`, `BLOBHASH`, `BLOBBASEFEE` | ✅ |

## Examples

### Fork Inheritance

```rust
use eot::{Frontier, Cancun, OpCode};

// Cancun includes all Frontier opcodes
assert!(Cancun::has_opcode(0x01)); // ADD from Frontier
assert!(Cancun::has_opcode(0x5c)); // TLOAD from Cancun

// But Frontier doesn't have newer opcodes
assert!(Frontier::has_opcode(0x01));  // ADD ✅
assert!(!Frontier::has_opcode(0x5c)); // TLOAD ❌
```

### Gas Cost Analysis

```rust
use eot::{Cancun, OpCode};

let opcodes = vec![0x60, 0x60, 0x01]; // PUSH1, PUSH1, ADD
let total_gas: u16 = opcodes.iter()
    .map(|&byte| Cancun::from(byte).gas_cost())
    .sum();

println!("Total gas cost: {}", total_gas); // 9 (3 + 3 + 3)
```

### Contract Fork Compatibility

```rust
use eot::{Shanghai, Cancun, OpCode};

let contract_opcodes = vec![0x5f, 0x5c]; // PUSH0, TLOAD

// Check compatibility with different forks
let shanghai_compatible = contract_opcodes.iter()
    .all(|&opcode| Shanghai::has_opcode(opcode));
println!("Shanghai compatible: {}", shanghai_compatible); // false (no TLOAD)

let cancun_compatible = contract_opcodes.iter()
    .all(|&opcode| Cancun::has_opcode(opcode));
println!("Cancun compatible: {}", cancun_compatible); // true
```

### Opcode Registry

```rust
use eot::{OpcodeRegistry, Fork};

let registry = OpcodeRegistry::new();

// Get all opcodes for a specific fork
let cancun_opcodes = registry.get_opcodes(Fork::Cancun);
println!("Cancun has {} opcodes", cancun_opcodes.len());

// Check opcode availability across forks
println!("TLOAD in Frontier: {}", 
    registry.is_opcode_available(Fork::Frontier, 0x5c)); // false
println!("TLOAD in Cancun: {}", 
    registry.is_opcode_available(Fork::Cancun, 0x5c));   // true
```

## Building the Project

### Prerequisites
- Rust 1.70+ (for proper trait support)
- Python 3.8+ (for code generation, optional)

### Building

```bash
git clone https://github.com/g4titanx/eot
cd eot
cargo bb && cargo tt
```

### Regenerating Fork Files (Optional)

If you need to modify opcode data:

```bash
# Run the Python generator
python3 generate_forks.py

# Then rebuild
cargo build
```

## Validation

Built-in validation ensures historical accuracy:

```rust
use eot::{OpcodeRegistry, validation};

let registry = OpcodeRegistry::new();
let report = validation::run_comprehensive_validation(&registry);

if report.has_errors() {
    eprintln!("❌ Validation failed:");
    report.print_summary();
} else {
    println!("✅ All validations passed!");
}
```

The validation system checks:
- ✅ Opcode uniqueness within forks
- ✅ Proper fork inheritance
- ✅ Historical accuracy of opcode introductions
- ✅ Gas cost consistency
- ✅ Stack input/output validation
- ✅ EIP reference accuracy

## Contributing

1. **Adding a new fork**: 
   - Update the CSV data in the generator script
   - Add the fork to the `Fork` enum in `lib.rs`
   - Regenerate files with `python3 generate_forks.py`

2. **Fixing opcode data**: 
   - Update the relevant data in `generate_forks.py`
   - Regenerate and test

3. **Adding features**: 
   - Extend the trait system in `traits.rs`
   - Add comprehensive tests

### Example: Adding a New Fork

```python
# In generate_forks.py, add to get_historical_additions():
'prague': """0x61,NEWOP,5,1,1,New operation,StackMemoryStorageFlow,Prague,9999"""
```

Then:
```bash
python3 generate_forks.py
cargo test
```

## License

MIT License - see [LICENSE](LICENSE) file for details.

## Acknowledgments

- clearloop for [evm-opcodes](https://crates.io/crates/evm-opcodes)
- Ethereum Foundation for EVM specification
- EIP authors for comprehensive opcode documentation
