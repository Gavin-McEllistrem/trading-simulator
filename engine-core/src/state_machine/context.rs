//! State machine context
//!
//! Provides flexible storage for state-specific data.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Context for storing state-specific data
///
/// Provides a flexible key-value store for maintaining state data
/// that persists across state transitions.
///
/// # Examples
///
/// ```
/// use trading_engine::state_machine::Context;
///
/// let mut ctx = Context::new();
/// ctx.set("entry_price", 50000.0);
/// assert_eq!(ctx.get::<f64>("entry_price"), Some(&50000.0));
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Context {
    /// String storage
    pub strings: HashMap<String, String>,

    /// Numeric storage
    pub numbers: HashMap<String, f64>,

    /// Integer storage
    pub integers: HashMap<String, i64>,

    /// Boolean storage
    pub booleans: HashMap<String, bool>,
}

impl Context {
    /// Create a new empty context
    ///
    /// # Examples
    ///
    /// ```
    /// use trading_engine::state_machine::Context;
    ///
    /// let ctx = Context::new();
    /// assert!(ctx.is_empty());
    /// ```
    pub fn new() -> Self {
        Self {
            strings: HashMap::new(),
            numbers: HashMap::new(),
            integers: HashMap::new(),
            booleans: HashMap::new(),
        }
    }

    /// Set a value in the context
    ///
    /// # Examples
    ///
    /// ```
    /// use trading_engine::state_machine::Context;
    ///
    /// let mut ctx = Context::new();
    /// ctx.set("price", 50000.0);
    /// ctx.set("symbol", "BTCUSDT".to_string());
    /// ctx.set("count", 42i64);
    /// ctx.set("active", true);
    /// ```
    pub fn set<T: ContextValue>(&mut self, key: &str, value: T) {
        value.insert_into(key, self);
    }

    /// Get a value from the context
    ///
    /// # Examples
    ///
    /// ```
    /// use trading_engine::state_machine::Context;
    ///
    /// let mut ctx = Context::new();
    /// ctx.set("price", 50000.0);
    /// assert_eq!(ctx.get::<f64>("price"), Some(&50000.0));
    /// assert_eq!(ctx.get::<f64>("missing"), None);
    /// ```
    pub fn get<T: ContextValue>(&self, key: &str) -> Option<&T> {
        T::get_from(key, self)
    }

    /// Remove a value from the context
    ///
    /// # Examples
    ///
    /// ```
    /// use trading_engine::state_machine::Context;
    ///
    /// let mut ctx = Context::new();
    /// ctx.set("price", 50000.0);
    /// ctx.remove::<f64>("price");
    /// assert_eq!(ctx.get::<f64>("price"), None);
    /// ```
    pub fn remove<T: ContextValue>(&mut self, key: &str) -> Option<T> {
        T::remove_from(key, self)
    }

    /// Check if context is empty
    pub fn is_empty(&self) -> bool {
        self.strings.is_empty()
            && self.numbers.is_empty()
            && self.integers.is_empty()
            && self.booleans.is_empty()
    }

    /// Clear all context data
    pub fn clear(&mut self) {
        self.strings.clear();
        self.numbers.clear();
        self.integers.clear();
        self.booleans.clear();
    }

    /// Convenience method: Set latest price
    pub fn set_latest_price(&mut self, price: f64) {
        self.set("latest_price", price);
    }

    /// Convenience method: Get latest price
    pub fn latest_price(&self) -> Option<f64> {
        self.get::<f64>("latest_price").copied()
    }

    /// Convenience method: Set latest timestamp
    pub fn set_latest_timestamp(&mut self, timestamp: i64) {
        self.set("latest_timestamp", timestamp);
    }

    /// Convenience method: Get latest timestamp
    pub fn latest_timestamp(&self) -> Option<i64> {
        self.get::<i64>("latest_timestamp").copied()
    }

    /// Iterate over all number entries
    pub fn iter_numbers(&self) -> impl Iterator<Item = (&String, &f64)> {
        self.numbers.iter()
    }

    /// Iterate over all string entries
    pub fn iter_strings(&self) -> impl Iterator<Item = (&String, &String)> {
        self.strings.iter()
    }

    /// Iterate over all integer entries
    pub fn iter_integers(&self) -> impl Iterator<Item = (&String, &i64)> {
        self.integers.iter()
    }

    /// Iterate over all boolean entries
    pub fn iter_booleans(&self) -> impl Iterator<Item = (&String, &bool)> {
        self.booleans.iter()
    }
}

impl Default for Context {
    fn default() -> Self {
        Self::new()
    }
}

/// Trait for types that can be stored in Context
pub trait ContextValue: Sized {
    fn insert_into(&self, key: &str, ctx: &mut Context);
    fn get_from<'a>(key: &str, ctx: &'a Context) -> Option<&'a Self>;
    fn remove_from(key: &str, ctx: &mut Context) -> Option<Self>;
}

impl ContextValue for String {
    fn insert_into(&self, key: &str, ctx: &mut Context) {
        ctx.strings.insert(key.to_string(), self.clone());
    }

    fn get_from<'a>(key: &str, ctx: &'a Context) -> Option<&'a Self> {
        ctx.strings.get(key)
    }

    fn remove_from(key: &str, ctx: &mut Context) -> Option<Self> {
        ctx.strings.remove(key)
    }
}

impl ContextValue for f64 {
    fn insert_into(&self, key: &str, ctx: &mut Context) {
        ctx.numbers.insert(key.to_string(), *self);
    }

    fn get_from<'a>(key: &str, ctx: &'a Context) -> Option<&'a Self> {
        ctx.numbers.get(key)
    }

    fn remove_from(key: &str, ctx: &mut Context) -> Option<Self> {
        ctx.numbers.remove(key)
    }
}

impl ContextValue for i64 {
    fn insert_into(&self, key: &str, ctx: &mut Context) {
        ctx.integers.insert(key.to_string(), *self);
    }

    fn get_from<'a>(key: &str, ctx: &'a Context) -> Option<&'a Self> {
        ctx.integers.get(key)
    }

    fn remove_from(key: &str, ctx: &mut Context) -> Option<Self> {
        ctx.integers.remove(key)
    }
}

impl ContextValue for bool {
    fn insert_into(&self, key: &str, ctx: &mut Context) {
        ctx.booleans.insert(key.to_string(), *self);
    }

    fn get_from<'a>(key: &str, ctx: &'a Context) -> Option<&'a Self> {
        ctx.booleans.get(key)
    }

    fn remove_from(key: &str, ctx: &mut Context) -> Option<Self> {
        ctx.booleans.remove(key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_strings() {
        let mut ctx = Context::new();
        ctx.set("symbol", "BTCUSDT".to_string());
        assert_eq!(ctx.get::<String>("symbol"), Some(&"BTCUSDT".to_string()));
    }

    #[test]
    fn test_context_numbers() {
        let mut ctx = Context::new();
        ctx.set("price", 50000.0);
        assert_eq!(ctx.get::<f64>("price"), Some(&50000.0));
    }

    #[test]
    fn test_context_integers() {
        let mut ctx = Context::new();
        ctx.set("count", 42i64);
        assert_eq!(ctx.get::<i64>("count"), Some(&42i64));
    }

    #[test]
    fn test_context_booleans() {
        let mut ctx = Context::new();
        ctx.set("active", true);
        assert_eq!(ctx.get::<bool>("active"), Some(&true));
    }

    #[test]
    fn test_context_remove() {
        let mut ctx = Context::new();
        ctx.set("price", 50000.0);

        let removed = ctx.remove::<f64>("price");
        assert_eq!(removed, Some(50000.0));
        assert_eq!(ctx.get::<f64>("price"), None);
    }

    #[test]
    fn test_context_clear() {
        let mut ctx = Context::new();
        ctx.set("price", 50000.0);
        ctx.set("symbol", "BTCUSDT".to_string());

        ctx.clear();
        assert!(ctx.is_empty());
    }

    #[test]
    fn test_convenience_methods() {
        let mut ctx = Context::new();

        ctx.set_latest_price(50000.0);
        ctx.set_latest_timestamp(1234567890);

        assert_eq!(ctx.latest_price(), Some(50000.0));
        assert_eq!(ctx.latest_timestamp(), Some(1234567890));
    }
}
