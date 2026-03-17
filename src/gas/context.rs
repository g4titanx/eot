//! Execution context for dynamic gas cost calculation.

use std::collections::HashSet;

/// Fixed-size address type (20 bytes).
pub type Address = [u8; 20];

/// Fixed-size storage key type (32 bytes).
pub type StorageKey = [u8; 32];

/// Execution context that affects dynamic gas pricing.
///
/// Tracks warm/cold access sets (EIP-2929), memory size, call depth,
/// and other state needed for accurate gas calculation.
#[derive(Debug, Clone, Default)]
pub struct ExecutionContext {
    /// Current memory size in bytes.
    pub memory_size: usize,
    /// Storage slots accessed in this transaction (EIP-2929).
    pub accessed_storage_keys: HashSet<(Address, StorageKey)>,
    /// Addresses accessed in this transaction (EIP-2929).
    pub accessed_addresses: HashSet<Address>,
    /// Current call depth.
    pub call_depth: u8,
    /// Whether we are in a STATICCALL context.
    pub is_static: bool,
    /// Gas price (wei).
    pub gas_price: u64,
    /// Block gas limit.
    pub gas_limit: u64,
    /// Remaining gas in this execution frame.
    pub gas_remaining: u64,
    /// Address of the executing contract.
    pub current_address: Address,
    /// Caller address.
    pub caller_address: Address,
    /// Value sent with the current call.
    pub call_value: u64,
}

impl ExecutionContext {
    /// Creates a new context with sensible defaults.
    pub fn new() -> Self {
        Self {
            gas_price: 20_000_000_000,
            gas_limit: 30_000_000,
            gas_remaining: 1_000_000,
            ..Self::default()
        }
    }

    /// Mark a storage slot as accessed (warm).
    pub fn mark_storage_accessed(&mut self, address: &Address, key: &StorageKey) {
        self.accessed_storage_keys.insert((*address, *key));
    }

    /// Mark an address as accessed (warm).
    pub fn mark_address_accessed(&mut self, address: &Address) {
        self.accessed_addresses.insert(*address);
    }

    /// Check if a storage slot is warm.
    pub fn is_storage_warm(&self, address: &Address, key: &StorageKey) -> bool {
        self.accessed_storage_keys.contains(&(*address, *key))
    }

    /// Check if an address is warm.
    pub fn is_address_warm(&self, address: &Address) -> bool {
        self.accessed_addresses.contains(address)
    }

    /// Expand memory to at least `new_size` bytes.
    pub fn expand_memory(&mut self, new_size: usize) {
        if new_size > self.memory_size {
            self.memory_size = new_size;
        }
    }

    /// Enter a new call frame.
    pub fn enter_call(&mut self) {
        self.call_depth = self.call_depth.saturating_add(1);
    }

    /// Exit a call frame.
    pub fn exit_call(&mut self) {
        self.call_depth = self.call_depth.saturating_sub(1);
    }

    /// Set static call mode.
    pub fn set_static(&mut self, is_static: bool) {
        self.is_static = is_static;
    }

    /// Consume gas. Returns `Err` if insufficient gas.
    pub fn consume_gas(&mut self, amount: u64) -> Result<(), String> {
        if self.gas_remaining < amount {
            Err(format!(
                "Out of gas: need {amount}, have {}",
                self.gas_remaining
            ))
        } else {
            self.gas_remaining -= amount;
            Ok(())
        }
    }

    /// Gas available for a sub-call (63/64 rule, EIP-150).
    pub fn available_call_gas(&self) -> u64 {
        self.gas_remaining - (self.gas_remaining / 64)
    }

    /// Reset for a new transaction.
    pub fn reset_for_new_transaction(&mut self) {
        self.accessed_storage_keys.clear();
        self.accessed_addresses.clear();
        self.call_depth = 0;
        self.is_static = false;
        self.memory_size = 0;
    }

    /// Helper: pad a byte slice into a 20-byte address.
    pub fn addr_from_slice(s: &[u8]) -> Address {
        let mut a = [0u8; 20];
        let len = s.len().min(20);
        a[..len].copy_from_slice(&s[..len]);
        a
    }

    /// Helper: pad a byte slice into a 32-byte storage key.
    pub fn key_from_slice(s: &[u8]) -> StorageKey {
        let mut k = [0u8; 32];
        let len = s.len().min(32);
        k[..len].copy_from_slice(&s[..len]);
        k
    }
}

/// Builder for [`ExecutionContext`].
pub struct ExecutionContextBuilder {
    ctx: ExecutionContext,
}

impl ExecutionContextBuilder {
    /// Create a new builder.
    pub fn new() -> Self {
        Self {
            ctx: ExecutionContext::new(),
        }
    }

    /// Set the current contract address.
    pub fn with_address(mut self, address: Address) -> Self {
        self.ctx.current_address = address;
        self
    }

    /// Set the caller address.
    pub fn with_caller(mut self, caller: Address) -> Self {
        self.ctx.caller_address = caller;
        self
    }

    /// Set the call value.
    pub fn with_value(mut self, value: u64) -> Self {
        self.ctx.call_value = value;
        self
    }

    /// Set gas parameters.
    pub fn with_gas(mut self, remaining: u64, price: u64, limit: u64) -> Self {
        self.ctx.gas_remaining = remaining;
        self.ctx.gas_price = price;
        self.ctx.gas_limit = limit;
        self
    }

    /// Pre-warm storage slots.
    pub fn with_warm_storage(mut self, slots: Vec<(Address, StorageKey)>) -> Self {
        for (a, k) in slots {
            self.ctx.accessed_storage_keys.insert((a, k));
        }
        self
    }

    /// Pre-warm addresses.
    pub fn with_warm_addresses(mut self, addrs: Vec<Address>) -> Self {
        for a in addrs {
            self.ctx.accessed_addresses.insert(a);
        }
        self
    }

    /// Set static call mode.
    pub fn with_static(mut self, is_static: bool) -> Self {
        self.ctx.is_static = is_static;
        self
    }

    /// Build the context.
    pub fn build(self) -> ExecutionContext {
        self.ctx
    }
}

impl Default for ExecutionContextBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn storage_warming() {
        let mut ctx = ExecutionContext::new();
        let addr = [1u8; 20];
        let key = [2u8; 32];
        assert!(!ctx.is_storage_warm(&addr, &key));
        ctx.mark_storage_accessed(&addr, &key);
        assert!(ctx.is_storage_warm(&addr, &key));
    }

    #[test]
    fn address_warming() {
        let mut ctx = ExecutionContext::new();
        let addr = [1u8; 20];
        assert!(!ctx.is_address_warm(&addr));
        ctx.mark_address_accessed(&addr);
        assert!(ctx.is_address_warm(&addr));
    }

    #[test]
    fn memory_expansion() {
        let mut ctx = ExecutionContext::new();
        ctx.expand_memory(64);
        assert_eq!(ctx.memory_size, 64);
        ctx.expand_memory(32);
        assert_eq!(ctx.memory_size, 64); // no shrink
        ctx.expand_memory(128);
        assert_eq!(ctx.memory_size, 128);
    }

    #[test]
    fn call_depth() {
        let mut ctx = ExecutionContext::new();
        ctx.enter_call();
        assert_eq!(ctx.call_depth, 1);
        ctx.enter_call();
        assert_eq!(ctx.call_depth, 2);
        ctx.exit_call();
        assert_eq!(ctx.call_depth, 1);
        ctx.exit_call();
        assert_eq!(ctx.call_depth, 0);
        ctx.exit_call();
        assert_eq!(ctx.call_depth, 0); // saturates
    }

    #[test]
    fn gas_consumption() {
        let mut ctx = ExecutionContext::new();
        ctx.gas_remaining = 1000;
        assert!(ctx.consume_gas(500).is_ok());
        assert_eq!(ctx.gas_remaining, 500);
        assert!(ctx.consume_gas(600).is_err());
        assert_eq!(ctx.gas_remaining, 500);
    }

    #[test]
    fn available_call_gas_eip150() {
        let ctx = ExecutionContext {
            gas_remaining: 64000,
            ..ExecutionContext::new()
        };
        assert_eq!(ctx.available_call_gas(), 63000);
    }

    #[test]
    fn builder() {
        let addr = [1u8; 20];
        let caller = [2u8; 20];
        let ctx = ExecutionContextBuilder::new()
            .with_address(addr)
            .with_caller(caller)
            .with_value(1000)
            .with_gas(500_000, 20_000_000_000, 30_000_000)
            .with_warm_addresses(vec![addr])
            .with_static(true)
            .build();

        assert_eq!(ctx.current_address, addr);
        assert_eq!(ctx.caller_address, caller);
        assert_eq!(ctx.call_value, 1000);
        assert!(ctx.is_static);
        assert!(ctx.is_address_warm(&addr));
    }
}
