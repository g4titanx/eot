# eot - EVM Opcode Table

EVM opcodes library with fork-aware gas costs, static metadata, and bytecode analysis.

[![Crates.io](https://img.shields.io/crates/v/eot.svg)](https://crates.io/crates/eot)
[![Documentation](https://docs.rs/eot/badge.svg)](https://docs.rs/eot)
[![Build Status](https://github.com/g4titanx/eot/workflows/CI/badge.svg)](https://github.com/g4titanx/eot/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## Overview

`eot` provides a single `OpCode(u8)` type backed by a static `[Option<OpCodeInfo>; 256]` lookup table for O(1) opcode lookups with zero heap allocation. Every opcode carries its name, stack I/O, gas cost, group, introduction fork, EIP reference, immediate size, and termination flag.

Fork-specific gas costs (EIP-150, EIP-1884, EIP-2929) are resolved at query time via `OpCode::gas_cost(fork)`, so there's no need for per-fork enum types. A `DynamicGasCalculator` handles warm/cold access, memory expansion, and complex call pricing for more precise analysis.

## Quick start

```rust
use eot::{OpCode, Fork};

// O(1) lookup
let add = OpCode::ADD;
assert_eq!(add.gas_cost(Fork::Frontier), 3);
assert!(add.is_valid_in(Fork::Frontier));

// Parse from byte
let op = OpCode::new(0x60).unwrap();
assert_eq!(op, OpCode::PUSH1);
assert_eq!(op.info().unwrap().immediate_size, 1);

// Parse from string
let op: OpCode = "KECCAK256".parse().unwrap();
assert_eq!(op.byte(), 0x20);

// Gas cost evolution
assert_eq!(OpCode::SLOAD.gas_cost(Fork::Frontier), 50);
assert_eq!(OpCode::SLOAD.gas_cost(Fork::TangerineWhistle), 200);
assert_eq!(OpCode::SLOAD.gas_cost(Fork::Istanbul), 800);
assert_eq!(OpCode::SLOAD.gas_cost(Fork::Berlin), 2100); // cold access
```

## Features

- **149 opcodes** from Frontier through Prague with full metadata
- **Zero-allocation lookups** via a static 256-element array
- **Fork-aware gas costs** encoding EIP-150, EIP-1884, EIP-2929 repricing
- **Dynamic gas calculator** for warm/cold access, memory expansion, call pricing
- **Sequence analysis** with optimization suggestions
- **Fork comparison** to see what changed between any two forks
- **Validation** of the opcode table against known historical facts
- **Optional serde** support behind the `serde` feature flag

## Supported forks

| Fork | Date | Key changes |
|------|------|-------------|
| Frontier | Jul 2015 | Base set (130+ opcodes) |
| Homestead | Mar 2016 | `DELEGATECALL` |
| Tangerine Whistle | Oct 2016 | EIP-150 gas repricing |
| Spurious Dragon | Nov 2016 | EIP-161/170 |
| Byzantium | Oct 2017 | `REVERT`, `RETURNDATASIZE`, `RETURNDATACOPY`, `STATICCALL` |
| Constantinople | Feb 2019 | `SHL`, `SHR`, `SAR`, `CREATE2`, `EXTCODEHASH` |
| Petersburg | Feb 2019 | Reverted EIP-1283 |
| Istanbul | Dec 2019 | `CHAINID`, `SELFBALANCE`, EIP-1884 repricing |
| Berlin | Apr 2021 | EIP-2929 warm/cold access costs |
| London | Aug 2021 | `BASEFEE` (EIP-1559) |
| Paris | Sep 2022 | `DIFFICULTY` becomes `PREVRANDAO` |
| Shanghai | Apr 2023 | `PUSH0` (EIP-3855) |
| Cancun | Mar 2024 | `TLOAD`, `TSTORE`, `MCOPY`, `BLOBHASH`, `BLOBBASEFEE` |
| Prague | May 2025 | — |
| Fusaka | Dec 2025 | EOF (EIP-7692), `CLZ`, `RJUMP`, `CALLF`, `DUPN`, `SWAPN`, `EXTCALL`, +13 more |

## Gas analysis

```rust
use eot::gas::{DynamicGasCalculator, ExecutionContext};
use eot::Fork;

let calc = DynamicGasCalculator::new(Fork::Berlin);
let mut ctx = ExecutionContext::new();

// Cold SLOAD
let cold = calc.calculate_gas_cost(0x54, &ctx, &[0x100]).unwrap();

// Warm the slot, then re-read
let key = {
    let mut k = [0u8; 32];
    k[24..32].copy_from_slice(&0x100u64.to_be_bytes());
    k
};
let addr = ctx.current_address;
ctx.mark_storage_accessed(&addr, &key);
let warm = calc.calculate_gas_cost(0x54, &ctx, &[0x100]).unwrap();

assert!(warm < cold);
```

## Architecture

```
src/
  lib.rs              OpCode, OpCodeInfo, Fork, Group, OPCODE_TABLE, gas_cost_for_fork()
  gas.rs              Gas constants, GasCostCategory, GasAnalysis
  gas/calculator.rs   DynamicGasCalculator (warm/cold, memory, calls)
  gas/context.rs      ExecutionContext, Address/StorageKey types
  gas/analysis.rs     GasAnalyzer, GasComparator, GasOptimizationAdvisor
  validation.rs       Static table validation
```

The entire opcode table is a single `static [Option<OpCodeInfo>; 256]` initialized at compile time. No `HashMap`, no heap allocation, no per-fork enum duplication.

## Contributing

1. **Adding an opcode or fork**: edit the `opcodes!` invocation in `src/lib.rs` and add the fork variant to the `Fork` enum. Run `cargo tt` to verify.
2. **Fixing gas costs**: update `gas_cost_for_fork()` in `src/lib.rs` for historical changes, or the `base_gas` in the opcode table for base costs.
3. **Adding analysis features**: extend `gas/analysis.rs` or `gas/calculator.rs`.
