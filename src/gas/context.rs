//! Execution context for gas cost calculation

use std::collections::HashSet;

/// Execution context that affects gas costs
/// 
/// This tracks the state that influences dynamic gas pricing,
/// particularly for EIP-2929 warm/cold access patterns.
#[derive(Debug, Clone, Default)]
pub struct ExecutionContext {
    /// Current memory size in bytes
    pub memory_size: usize,
    
    /// Storage slots that have been accessed in this transaction (EIP-2929)
    /// Format: (address, storage_key)
    pub accessed_storage_keys: HashSet<(Vec<u8>, Vec<u8>)>,
    
    /// Addresses that have been accessed in this transaction (EIP-2929)
    pub accessed_addresses: HashSet<Vec<u8>>,
    
    /// Current call depth (affects gas availability)
    pub call_depth: u8,
    
    /// Whether we're in a static call context (affects state modifications)
    pub is_static: bool,
    
    /// Current gas price (for GASPRICE opcode)
    pub gas_price: u64,
    
    /// Current block gas limit
    pub gas_limit: u64,
    
    /// Available gas remaining in this execution
    pub gas_remaining: u64,
    
    /// Current contract address (20 bytes)
    pub current_address: Vec<u8>,
    
    /// Caller address (20 bytes)  
    pub caller_address: Vec<u8>,
    
    /// Value sent with the current call
    pub call_value: u64,
}

impl ExecutionContext {
    /// Create a new execution context with default values
    pub fn new() -> Self {
        Self {
            memory_size: 0,
            accessed_storage_keys: HashSet::new(),
            accessed_addresses: HashSet::new(),
            call_depth: 0,
            is_static: false,
            gas_price: 20_000_000_000, // 20 gwei default
            gas_limit: 30_000_000,     // 30M gas default block limit
            gas_remaining: 1_000_000,  // 1M gas default for call
            current_address: vec![0u8; 20],
            caller_address: vec![0u8; 20],
            call_value: 0,
        }
    }

    /// Mark a storage slot as accessed (warm)
    pub fn mark_storage_accessed(&mut self, address: &[u8], key: &[u8]) {
        self.accessed_storage_keys.insert((address.to_vec(), key.to_vec()));
    }

    /// Mark an address as accessed (warm)
    pub fn mark_address_accessed(&mut self, address: &[u8]) {
        self.accessed_addresses.insert(address.to_vec());
    }

    /// Check if a storage slot has been accessed (is warm)
    pub fn is_storage_warm(&self, address: &[u8], key: &[u8]) -> bool {
        self.accessed_storage_keys.contains(&(address.to_vec(), key.to_vec()))
    }

    /// Check if an address has been accessed (is warm)  
    pub fn is_address_warm(&self, address: &[u8]) -> bool {
        self.accessed_addresses.contains(&address.to_vec())
    }

    /// Update memory size if the new size is larger
    pub fn expand_memory(&mut self, new_size: usize) {
        if new_size > self.memory_size {
            self.memory_size = new_size;
        }
    }

    /// Enter a new call frame (increment depth)
    pub fn enter_call(&mut self) {
        self.call_depth += 1;
    }

    /// Exit a call frame (decrement depth)
    pub fn exit_call(&mut self) {
        if self.call_depth > 0 {
            self.call_depth -= 1;
        }
    }

    /// Set static call mode
    pub fn set_static(&mut self, is_static: bool) {
        self.is_static = is_static;
    }

    /// Consume gas from remaining amount
    pub fn consume_gas(&mut self, amount: u64) -> Result<(), String> {
        if self.gas_remaining < amount {
            Err(format!("Out of gas: need {}, have {}", amount, self.gas_remaining))
        } else {
            self.gas_remaining -= amount;
            Ok(())
        }
    }

    /// Get the gas available for a sub-call (63/64 rule)
    pub fn available_call_gas(&self) -> u64 {
        // EIP-150: Reserve 1/64 of remaining gas
        self.gas_remaining - (self.gas_remaining / 64)
    }

    /// Reset context for a new transaction
    pub fn reset_for_new_transaction(&mut self) {
        self.accessed_storage_keys.clear();
        self.accessed_addresses.clear();
        self.call_depth = 0;
        self.is_static = false;
        self.memory_size = 0;
    }

    /// Clone context for simulation (doesn't affect original state)
    pub fn simulate(&self) -> Self {
        self.clone()
    }
}

/// Builder pattern for creating execution contexts
pub struct ExecutionContextBuilder {
    context: ExecutionContext,
}

impl ExecutionContextBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self {
            context: ExecutionContext::new(),
        }
    }

    /// Set the current contract address
    pub fn with_address(mut self, address: Vec<u8>) -> Self {
        self.context.current_address = address;
        self
    }

    /// Set the caller address
    pub fn with_caller(mut self, caller: Vec<u8>) -> Self {
        self.context.caller_address = caller;
        self
    }

    /// Set the call value
    pub fn with_value(mut self, value: u64) -> Self {
        self.context.call_value = value;
        self
    }

    /// Set gas parameters
    pub fn with_gas(mut self, gas_remaining: u64, gas_price: u64, gas_limit: u64) -> Self {
        self.context.gas_remaining = gas_remaining;
        self.context.gas_price = gas_price;
        self.context.gas_limit = gas_limit;
        self
    }

    /// Pre-warm storage slots
    pub fn with_warm_storage(mut self, slots: Vec<(Vec<u8>, Vec<u8>)>) -> Self {
        for (addr, key) in slots {
            self.context.accessed_storage_keys.insert((addr, key));
        }
        self
    }

    /// Pre-warm addresses
    pub fn with_warm_addresses(mut self, addresses: Vec<Vec<u8>>) -> Self {
        for addr in addresses {
            self.context.accessed_addresses.insert(addr);
        }
        self
    }

    /// Set static call mode
    pub fn with_static(mut self, is_static: bool) -> Self {
        self.context.is_static = is_static;
        self
    }

    /// Build the execution context
    pub fn build(self) -> ExecutionContext {
        self.context
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
    fn test_storage_warming() {
        let mut context = ExecutionContext::new();
        let addr = vec![1u8; 20];
        let key = vec![2u8; 32];

        assert!(!context.is_storage_warm(&addr, &key));
        
        context.mark_storage_accessed(&addr, &key);
        assert!(context.is_storage_warm(&addr, &key));
    }

    #[test]
    fn test_address_warming() {
        let mut context = ExecutionContext::new();
        let addr = vec![1u8; 20];

        assert!(!context.is_address_warm(&addr));
        
        context.mark_address_accessed(&addr);
        assert!(context.is_address_warm(&addr));
    }

    #[test]
    fn test_memory_expansion() {
        let mut context = ExecutionContext::new();
        assert_eq!(context.memory_size, 0);

        context.expand_memory(64);
        assert_eq!(context.memory_size, 64);

        // Should not shrink
        context.expand_memory(32);
        assert_eq!(context.memory_size, 64);

        // Should expand further
        context.expand_memory(128);
        assert_eq!(context.memory_size, 128);
    }

    #[test]
    fn test_call_depth_tracking() {
        let mut context = ExecutionContext::new();
        assert_eq!(context.call_depth, 0);

        context.enter_call();
        assert_eq!(context.call_depth, 1);

        context.enter_call();
        assert_eq!(context.call_depth, 2);

        context.exit_call();
        assert_eq!(context.call_depth, 1);

        context.exit_call();
        assert_eq!(context.call_depth, 0);

        // Should not go below 0
        context.exit_call();
        assert_eq!(context.call_depth, 0);
    }

    #[test]
    fn test_gas_consumption() {
        let mut context = ExecutionContext::new();
        context.gas_remaining = 1000;

        assert!(context.consume_gas(500).is_ok());
        assert_eq!(context.gas_remaining, 500);

        assert!(context.consume_gas(600).is_err()); // Should fail - not enough gas
        assert_eq!(context.gas_remaining, 500); // Should not change on failure
    }

    #[test]
    fn test_available_call_gas() {
        let context = ExecutionContext {
            gas_remaining: 64000,
            ..ExecutionContext::new()
        };

        let available = context.available_call_gas();
        assert_eq!(available, 63000); // 64000 - 64000/64 = 64000 - 1000
    }

    #[test]
    fn test_context_builder() {
        let addr = vec![1u8; 20];
        let caller = vec![2u8; 20];
        let storage_slots = vec![(addr.clone(), vec![3u8; 32])];
        let warm_addresses = vec![addr.clone()];

        let context = ExecutionContextBuilder::new()
            .with_address(addr.clone())
            .with_caller(caller.clone())
            .with_value(1000)
            .with_gas(500000, 20_000_000_000, 30_000_000)
            .with_warm_storage(storage_slots)
            .with_warm_addresses(warm_addresses)
            .with_static(true)
            .build();

        assert_eq!(context.current_address, addr);
        assert_eq!(context.caller_address, caller);
        assert_eq!(context.call_value, 1000);
        assert_eq!(context.gas_remaining, 500000);
        assert!(context.is_static);
        assert!(context.is_address_warm(&addr));
    }
}
