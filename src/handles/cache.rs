//! Implements a cache around a HashMap to store handles related to a transaction.

use std::collections::HashMap;

use tracing::{info, warn};

use crate::compute::SolidityValue;

/// Stores and manage a cache of handles and their associated [`SolidityValue`].
pub struct HandlesCache {
    hash_map: HashMap<String, SolidityValue>,
}

impl HandlesCache {
    pub fn new() -> Self {
        Self {
            hash_map: HashMap::new(),
        }
    }

    /// clears cache.
    pub fn clear(&mut self) {
        self.hash_map.clear();
    }

    /// Creates a new entry in the cache.
    pub fn add_handle(&mut self, handle: &str, entry: SolidityValue) {
        match self.hash_map.insert(handle.to_string(), entry) {
            Some(_) => warn!(handle = handle, "Did not expect to find an entry"),
            None => info!(handle = handle, "New cache entry"),
        }
    }

    /// Checks a list of handles against the cache and returns those which are not present.
    pub fn find_handles_not_in_cache(&self, operand_handles: Vec<String>) -> Vec<String> {
        operand_handles
            .into_iter()
            .filter(|handle| !self.hash_map.contains_key(handle))
            .collect()
    }

    /// Fetches a list of handles from the cache.
    ///
    /// The implementation does not guarantee that all values will be found.
    pub fn read_handles(&self, operand_handles: Vec<String>) -> Vec<SolidityValue> {
        operand_handles
            .iter()
            .filter_map(|handle| self.hash_map.get(handle).cloned())
            .collect()
    }
}
