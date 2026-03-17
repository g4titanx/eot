//! Dynamic gas cost calculator for EVM opcodes.

use super::context::ExecutionContext;
use super::{GasAnalysis, TX_BASE};
use crate::{Fork, OpCode};

/// Context-aware gas calculator for a specific fork.
///
/// Handles EIP-2929 warm/cold access, memory expansion costs, complex call
/// pricing, and other dynamic gas factors that can't be captured by the
/// static [`OpCode::gas_cost`] alone.
pub struct DynamicGasCalculator {
    fork: Fork,
}

impl DynamicGasCalculator {
    /// Create a calculator for the given fork.
    pub fn new(fork: Fork) -> Self {
        Self { fork }
    }

    /// Calculate total gas cost for a single opcode.
    pub fn calculate_gas_cost(
        &self,
        opcode: u8,
        context: &ExecutionContext,
        operands: &[u64],
    ) -> Result<u64, String> {
        let op = OpCode::from_byte(opcode);
        let base = op.gas_cost(self.fork) as u64;
        let dynamic = self.dynamic_cost(opcode, context, operands)?;
        Ok(base + dynamic)
    }

    /// Calculate only the dynamic (context-dependent) portion of gas cost.
    fn dynamic_cost(&self, opcode: u8, ctx: &ExecutionContext, ops: &[u64]) -> Result<u64, String> {
        match opcode {
            0x54 => self.sload_cost(ctx, ops),
            0x55 => self.sstore_cost(ctx, ops),
            0x5c => self.tload_cost(ops),
            0x5d => self.tstore_cost(ops),
            0x51..=0x53 => self.memory_cost(opcode, ctx, ops),
            0x5e => self.mcopy_cost(ctx, ops),
            0xf1 | 0xf2 | 0xf4 | 0xfa => self.call_cost(opcode, ctx, ops),
            0x31 | 0x3b | 0x3c | 0x3f => self.account_access_cost(ctx, ops),
            0x37 | 0x39 | 0x3e => self.copy_cost(ctx, ops),
            0xf0 | 0xf5 => self.create_cost(opcode, ctx, ops),
            0x20 => self.keccak256_cost(ctx, ops),
            0xa0..=0xa4 => self.log_cost(opcode, ctx, ops),
            _ => Ok(0),
        }
    }

    // -- SLOAD / SSTORE ----------------------------------------------------

    fn sload_cost(&self, ctx: &ExecutionContext, ops: &[u64]) -> Result<u64, String> {
        if self.fork >= Fork::Berlin {
            if ops.is_empty() {
                return Err("SLOAD requires storage key operand".into());
            }
            let key = Self::key_from_operand(ops[0]);
            if ctx.is_storage_warm(&ctx.current_address, &key) {
                Ok(100) // warm
            } else {
                Ok(2100) // cold
            }
        } else {
            Ok(0) // pre-Berlin: base cost covers it
        }
    }

    fn sstore_cost(&self, ctx: &ExecutionContext, ops: &[u64]) -> Result<u64, String> {
        if ops.len() < 2 {
            return Err("SSTORE requires key and value operands".into());
        }
        if self.fork >= Fork::Berlin {
            let key = Self::key_from_operand(ops[0]);
            if !ctx.is_storage_warm(&ctx.current_address, &key) {
                Ok(2100) // cold access surcharge
            } else {
                Ok(0)
            }
        } else {
            Ok(0)
        }
    }

    fn tload_cost(&self, ops: &[u64]) -> Result<u64, String> {
        if self.fork < Fork::Cancun {
            return Err("TLOAD not available before Cancun".into());
        }
        if ops.is_empty() {
            return Err("TLOAD requires key operand".into());
        }
        Ok(0) // base cost (100) already in static table
    }

    fn tstore_cost(&self, ops: &[u64]) -> Result<u64, String> {
        if self.fork < Fork::Cancun {
            return Err("TSTORE not available before Cancun".into());
        }
        if ops.len() < 2 {
            return Err("TSTORE requires key and value operands".into());
        }
        Ok(0) // base cost (100) already in static table
    }

    // -- Memory operations -------------------------------------------------

    fn memory_cost(&self, opcode: u8, ctx: &ExecutionContext, ops: &[u64]) -> Result<u64, String> {
        if ops.is_empty() {
            return Err("Memory operation requires offset operand".into());
        }
        let offset = ops[0] as usize;
        let size = match opcode {
            0x51 | 0x52 => 32,
            0x53 => 1,
            _ => return Err("Unknown memory opcode".into()),
        };
        let new_size = offset + size;
        if new_size > ctx.memory_size {
            Ok(Self::memory_expansion_cost(ctx.memory_size, new_size))
        } else {
            Ok(0)
        }
    }

    fn mcopy_cost(&self, ctx: &ExecutionContext, ops: &[u64]) -> Result<u64, String> {
        if self.fork < Fork::Cancun {
            return Err("MCOPY not available before Cancun".into());
        }
        if ops.len() < 3 {
            return Err("MCOPY requires dst, src, size operands".into());
        }
        let dst = ops[0] as usize;
        let size = ops[2] as usize;
        let new_size = dst + size;
        let expansion = if new_size > ctx.memory_size {
            Self::memory_expansion_cost(ctx.memory_size, new_size)
        } else {
            0
        };
        let words = size.div_ceil(32);
        Ok(expansion + words as u64 * 3)
    }

    /// Quadratic memory expansion cost.
    fn memory_expansion_cost(old: usize, new: usize) -> u64 {
        fn cost(size: usize) -> u64 {
            let w = size.div_ceil(32);
            w as u64 * 3 + (w * w) as u64 / 512
        }
        if new <= old {
            0
        } else {
            cost(new) - cost(old)
        }
    }

    // -- Call operations ---------------------------------------------------

    fn call_cost(&self, opcode: u8, ctx: &ExecutionContext, ops: &[u64]) -> Result<u64, String> {
        if ops.len() < 7 {
            return Err("CALL requires at least 7 operands".into());
        }
        let target_bytes = ops[1].to_be_bytes();
        let target = ExecutionContext::addr_from_slice(&target_bytes);
        let value = if opcode == 0xf1 { ops[2] } else { 0 };
        let mut cost = 0u64;

        if self.fork >= Fork::Berlin && !ctx.is_address_warm(&target) {
            cost += 2600;
        }
        if value > 0 {
            cost += 9000;
            if !ctx.is_address_warm(&target) {
                cost += 25000;
            }
        }
        // Memory expansion
        let args_end = ops[3] as usize + ops[4] as usize;
        let ret_end = ops[5] as usize + ops[6] as usize;
        let max_mem = args_end.max(ret_end);
        if max_mem > ctx.memory_size {
            cost += Self::memory_expansion_cost(ctx.memory_size, max_mem);
        }
        Ok(cost)
    }

    // -- Account access (BALANCE, EXTCODESIZE, etc.) -----------------------

    fn account_access_cost(&self, ctx: &ExecutionContext, ops: &[u64]) -> Result<u64, String> {
        if self.fork >= Fork::Berlin && !ops.is_empty() {
            let addr_bytes = ops[0].to_be_bytes();
            let addr = ExecutionContext::addr_from_slice(&addr_bytes);
            Ok(if ctx.is_address_warm(&addr) {
                100
            } else {
                2600
            })
        } else {
            Ok(0)
        }
    }

    // -- Copy operations ---------------------------------------------------

    fn copy_cost(&self, ctx: &ExecutionContext, ops: &[u64]) -> Result<u64, String> {
        if ops.len() < 3 {
            return Ok(0);
        }
        let dest = ops[0] as usize;
        let size = ops[2] as usize;
        let new_size = dest + size;
        let expansion = if new_size > ctx.memory_size {
            Self::memory_expansion_cost(ctx.memory_size, new_size)
        } else {
            0
        };
        let words = size.div_ceil(32);
        Ok(expansion + words as u64 * 3)
    }

    // -- CREATE / CREATE2 --------------------------------------------------

    fn create_cost(&self, opcode: u8, ctx: &ExecutionContext, ops: &[u64]) -> Result<u64, String> {
        if ops.len() < 3 {
            return Ok(0);
        }
        let offset = ops[1] as usize;
        let size = ops[2] as usize;
        let mut cost = 32000u64;

        if opcode == 0xf5 {
            // CREATE2: extra SHA3 cost for address derivation
            cost += size.div_ceil(32) as u64 * 6;
        }
        if self.fork >= Fork::Shanghai {
            // EIP-3860: initcode word cost
            cost += size.div_ceil(32) as u64 * 2;
        }
        let new_size = offset + size;
        if new_size > ctx.memory_size {
            cost += Self::memory_expansion_cost(ctx.memory_size, new_size);
        }
        Ok(cost)
    }

    // -- KECCAK256 ---------------------------------------------------------

    fn keccak256_cost(&self, ctx: &ExecutionContext, ops: &[u64]) -> Result<u64, String> {
        if ops.len() < 2 {
            return Ok(0);
        }
        let offset = ops[0] as usize;
        let size = ops[1] as usize;
        let new_size = offset + size;
        let expansion = if new_size > ctx.memory_size {
            Self::memory_expansion_cost(ctx.memory_size, new_size)
        } else {
            0
        };
        Ok(expansion + size.div_ceil(32) as u64 * 6)
    }

    // -- LOG ---------------------------------------------------------------

    fn log_cost(&self, opcode: u8, ctx: &ExecutionContext, ops: &[u64]) -> Result<u64, String> {
        if ops.len() < 2 {
            return Ok(0);
        }
        let offset = ops[0] as usize;
        let size = ops[1] as usize;
        let topics = (opcode - 0xa0) as u64;
        let new_size = offset + size;
        let expansion = if new_size > ctx.memory_size {
            Self::memory_expansion_cost(ctx.memory_size, new_size)
        } else {
            0
        };
        Ok(expansion + topics * 375 + size as u64 * 8)
    }

    // -- Helpers -----------------------------------------------------------

    fn key_from_operand(val: u64) -> [u8; 32] {
        let mut key = [0u8; 32];
        key[24..32].copy_from_slice(&val.to_be_bytes());
        key
    }

    // -- Sequence analysis -------------------------------------------------

    /// Analyse gas for a sequence of `(opcode, operands)` pairs.
    pub fn analyze_sequence_gas(&self, sequence: &[(u8, Vec<u64>)]) -> Result<GasAnalysis, String> {
        let mut ctx = ExecutionContext::new();
        let mut analysis = GasAnalysis {
            total_gas: TX_BASE,
            breakdown: Vec::with_capacity(sequence.len()),
            warnings: Vec::new(),
            optimizations: Vec::new(),
        };

        for (opcode, operands) in sequence {
            let cost = self.calculate_gas_cost(*opcode, &ctx, operands)?;
            analysis.total_gas += cost;
            analysis
                .breakdown
                .push((*opcode, cost.min(u16::MAX as u64) as u16));

            if cost > 10000 {
                let name = OpCode::from_byte(*opcode).name();
                analysis.warnings.push(format!(
                    "High gas: {name} (0x{opcode:02x}) costs {cost} gas"
                ));
            }

            self.update_context(&mut ctx, *opcode, operands);
        }
        self.generate_optimizations(&analysis.breakdown, &mut analysis.optimizations);
        Ok(analysis)
    }

    fn update_context(&self, ctx: &mut ExecutionContext, opcode: u8, ops: &[u64]) {
        match opcode {
            0x54 | 0x55 if !ops.is_empty() => {
                let key = Self::key_from_operand(ops[0]);
                let addr = ctx.current_address;
                ctx.mark_storage_accessed(&addr, &key);
            }
            0x31 | 0x3b | 0x3c | 0x3f if !ops.is_empty() => {
                let addr_bytes = ops[0].to_be_bytes();
                let addr = ExecutionContext::addr_from_slice(&addr_bytes);
                ctx.mark_address_accessed(&addr);
            }
            0xf1 | 0xf2 | 0xf4 | 0xfa if ops.len() >= 2 => {
                let addr_bytes = ops[1].to_be_bytes();
                let addr = ExecutionContext::addr_from_slice(&addr_bytes);
                ctx.mark_address_accessed(&addr);
                ctx.enter_call();
            }
            0x51..=0x53 if !ops.is_empty() => {
                let size = if opcode == 0x53 { 1 } else { 32 };
                ctx.expand_memory(ops[0] as usize + size);
            }
            0x5e if ops.len() >= 3 => {
                ctx.expand_memory(ops[0] as usize + ops[2] as usize);
            }
            0x37 | 0x39 | 0x3e if ops.len() >= 3 => {
                ctx.expand_memory(ops[0] as usize + ops[2] as usize);
            }
            _ => {}
        }
    }

    fn generate_optimizations(&self, breakdown: &[(u8, u16)], out: &mut Vec<String>) {
        let mut sload_count = 0u32;
        let mut sstore_count = 0u32;
        let mut prev = None;

        for &(op, _) in breakdown {
            match op {
                0x54 => sload_count += 1,
                0x55 => sstore_count += 1,
                _ => {}
            }
            if let Some(p) = prev {
                match (p, op) {
                    (0x80..=0x8f, 0x50) => {
                        out.push("DUP followed by POP — redundant operation".into());
                    }
                    (0x54, 0x54) | (0x55, 0x55) => {
                        out.push("Consecutive storage operations — consider batching".into());
                    }
                    _ => {}
                }
            }
            prev = Some(op);
        }
        if sload_count > 3 {
            out.push(format!(
                "{sload_count} SLOAD operations — consider caching in memory"
            ));
        }
        if sstore_count > 2 {
            out.push(format!(
                "{sstore_count} SSTORE operations — consider batching or transient storage"
            ));
        }
        if self.fork >= Fork::Shanghai {
            let has_push0 = breakdown.iter().any(|&(op, _)| op == 0x5f);
            if !has_push0 {
                out.push("Consider PUSH0 instead of PUSH1 0x00 (saves 1 gas)".into());
            }
        }
        if self.fork >= Fork::Cancun && sstore_count > 0 {
            let has_tstore = breakdown.iter().any(|&(op, _)| op == 0x5d);
            if !has_tstore {
                out.push("Consider TSTORE for temporary values (100 gas vs SSTORE)".into());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn static_gas() {
        let calc = DynamicGasCalculator::new(Fork::London);
        let ctx = ExecutionContext::new();
        let cost = calc.calculate_gas_cost(0x01, &ctx, &[]).unwrap();
        assert_eq!(cost, 3);
    }

    #[test]
    fn sload_warm_cold() {
        let calc = DynamicGasCalculator::new(Fork::Berlin);
        let mut ctx = ExecutionContext::new();

        let cold = calc.calculate_gas_cost(0x54, &ctx, &[0x123]).unwrap();

        let key = DynamicGasCalculator::key_from_operand(0x123);
        let addr = ctx.current_address;
        ctx.mark_storage_accessed(&addr, &key);

        let warm = calc.calculate_gas_cost(0x54, &ctx, &[0x123]).unwrap();
        assert!(warm < cold, "warm ({warm}) should be < cold ({cold})");
    }

    #[test]
    fn memory_expansion() {
        let calc = DynamicGasCalculator::new(Fork::London);
        let ctx = ExecutionContext::new();
        let cost = calc.calculate_gas_cost(0x52, &ctx, &[1000]).unwrap();
        assert!(cost > 3); // base MSTORE + expansion
    }

    #[test]
    fn sequence_analysis() {
        let calc = DynamicGasCalculator::new(Fork::London);
        let seq = vec![(0x01, vec![]), (0x02, vec![]), (0x54, vec![0x123])];
        let result = calc.analyze_sequence_gas(&seq).unwrap();
        assert!(result.total_gas > TX_BASE);
        assert_eq!(result.breakdown.len(), 3);
    }

    #[test]
    fn create_cost() {
        let calc = DynamicGasCalculator::new(Fork::Shanghai);
        let ctx = ExecutionContext::new();
        let cost = calc.calculate_gas_cost(0xf0, &ctx, &[0, 0, 100]).unwrap();
        assert!(cost >= 32000);
    }
}
