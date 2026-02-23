//! Implements a cache around a HashMap to store handles related to a transaction.

use std::collections::HashMap;

use tracing::{info, warn};
use zeroize::Zeroize;

use crate::compute::SolidityValue;

/// Stores and manage a cache of handles and their associated [`SolidityValue`].
pub struct HandlesCache {
    inner: HashMap<String, SolidityValue>,
}

/// Implements Zeroize trait to clean memory.
impl Zeroize for HandlesCache {
    fn zeroize(&mut self) {
        info!("zeroize");
        for value in self.inner.values_mut() {
            value.zeroize();
        }
        self.inner.clear();
    }
}

/// Forces memory cleanup before dropping values.
impl Drop for HandlesCache {
    fn drop(&mut self) {
        info!("drop");
        self.zeroize();
    }
}

impl HandlesCache {
    pub fn new() -> Self {
        Self {
            inner: HashMap::new(),
        }
    }

    /// Creates a new entry in the cache.
    pub fn add_handle(&mut self, handle: &str, entry: SolidityValue) {
        match self.inner.insert(handle.to_string(), entry) {
            Some(_) => warn!(handle = handle, "Did not expect to find an entry"),
            None => info!(handle = handle, "New cache entry"),
        }
    }

    /// Checks a list of handles against the cache and returns those which are not present.
    pub fn find_handles_not_in_cache(&self, operand_handles: Vec<String>) -> Vec<String> {
        operand_handles
            .into_iter()
            .filter(|handle| !self.inner.contains_key(handle))
            .collect()
    }

    /// Fetches a list of handles from the cache.
    ///
    /// The implementation does not guarantee that all values will be found.
    pub fn read_handles(&self, operand_handles: Vec<String>) -> Vec<SolidityValue> {
        operand_handles
            .iter()
            .filter_map(|handle| self.inner.get(handle).cloned())
            .collect()
    }
}
