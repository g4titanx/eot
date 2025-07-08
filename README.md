# eot - EVM Opcode Table

EVM opcodes library for all Ethereum forks, with complete fork inheritance, validation, and metadata

[![Crates.io](https://img.shields.io/crates/v/eot.svg)](https://crates.io/crates/eot)
[![Documentation](https://docs.rs/eot/badge.svg)](https://docs.rs/eot)
[![Build Status](https://github.com/g4titanx/eot/workflows/CI/badge.svg)](https://github.com/g4titanx/eot/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## What can it do?
`eot` is an EVM opcode table library that provides complete opcode metadata, fork inheritance, and validation for all Ethereum hard forks from Frontier to Cancun. It offers both a unified interface for simple opcode lookup and fork-specific implementations that accurately reflect the evolution of the EVM instruction set. You can query opcode properties like gas costs, stack behavior, and descriptions, check opcode availability across different forks, validate bytecode sequences, and build EVM analysis tools with confidence that the data matches each fork's specifications exactly.

See the `examples/` directory for practical demonstrations of opcode queries, fork compatibility checking, and gas analysis workflows.

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
cargo tt
```

## Acknowledgments

- [clearloop](github.com/clearloop) for [evm-opcodes](https://crates.io/crates/evm-opcodes)
