//! Position tracking
//!
//! Manages active trading positions with P&L calculation.

use serde::{Deserialize, Serialize};

pub use super::action::Side;

/// Represents an active or closed trading position
///
/// Tracks entry, current price, and P&L for a position.
///
/// # Examples
///
/// ```
/// use trading_engine::state_machine::{Position, position::Side};
///
/// let mut pos = Position::new(50000.0, 0.1, Side::Long, 1234567890);
/// pos.update_current_price(51000.0);
///
/// assert!(pos.unrealized_pnl().unwrap() > 0.0);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    /// Entry price
    entry_price: f64,

    /// Position size
    quantity: f64,

    /// Position side (Long or Short)
    side: Side,

    /// Entry timestamp (milliseconds)
    entry_timestamp: i64,

    /// Current price (updated on each tick)
    current_price: f64,

    /// Stop loss price (optional)
    stop_loss: Option<f64>,

    /// Take profit price (optional)
    take_profit: Option<f64>,

    /// Exit price (if closed)
    exit_price: Option<f64>,

    /// Exit timestamp (if closed)
    exit_timestamp: Option<i64>,
}

impl Position {
    /// Create a new position
    ///
    /// # Arguments
    ///
    /// * `entry_price` - Price at which position was entered
    /// * `quantity` - Position size
    /// * `side` - Long or Short
    /// * `entry_timestamp` - Entry time in milliseconds
    ///
    /// # Examples
    ///
    /// ```
    /// use trading_engine::state_machine::{Position, position::Side};
    ///
    /// let pos = Position::new(50000.0, 0.1, Side::Long, 1234567890);
    /// assert_eq!(pos.entry_price(), 50000.0);
    /// assert_eq!(pos.quantity(), 0.1);
    /// ```
    pub fn new(entry_price: f64, quantity: f64, side: Side, entry_timestamp: i64) -> Self {
        Self {
            entry_price,
            quantity,
            side,
            entry_timestamp,
            current_price: entry_price,
            stop_loss: None,
            take_profit: None,
            exit_price: None,
            exit_timestamp: None,
        }
    }

    /// Get entry price
    pub fn entry_price(&self) -> f64 {
        self.entry_price
    }

    /// Get position quantity
    pub fn quantity(&self) -> f64 {
        self.quantity
    }

    /// Get position side
    pub fn side(&self) -> Side {
        self.side
    }

    /// Get entry timestamp
    pub fn entry_timestamp(&self) -> i64 {
        self.entry_timestamp
    }

    /// Get current price
    pub fn current_price(&self) -> f64 {
        self.current_price
    }

    /// Update current price
    ///
    /// # Examples
    ///
    /// ```
    /// use trading_engine::state_machine::{Position, position::Side};
    ///
    /// let mut pos = Position::new(50000.0, 0.1, Side::Long, 1234567890);
    /// pos.update_current_price(51000.0);
    /// assert_eq!(pos.current_price(), 51000.0);
    /// ```
    pub fn update_current_price(&mut self, price: f64) {
        self.current_price = price;
    }

    /// Set stop loss
    ///
    /// # Examples
    ///
    /// ```
    /// use trading_engine::state_machine::{Position, position::Side};
    ///
    /// let mut pos = Position::new(50000.0, 0.1, Side::Long, 1234567890);
    /// pos.set_stop_loss(49000.0);
    /// assert_eq!(pos.stop_loss(), Some(49000.0));
    /// ```
    pub fn set_stop_loss(&mut self, stop: f64) {
        self.stop_loss = Some(stop);
    }

    /// Get stop loss
    pub fn stop_loss(&self) -> Option<f64> {
        self.stop_loss
    }

    /// Set take profit
    ///
    /// # Examples
    ///
    /// ```
    /// use trading_engine::state_machine::{Position, position::Side};
    ///
    /// let mut pos = Position::new(50000.0, 0.1, Side::Long, 1234567890);
    /// pos.set_take_profit(52000.0);
    /// assert_eq!(pos.take_profit(), Some(52000.0));
    /// ```
    pub fn set_take_profit(&mut self, target: f64) {
        self.take_profit = Some(target);
    }

    /// Get take profit
    pub fn take_profit(&self) -> Option<f64> {
        self.take_profit
    }

    /// Calculate unrealized P&L
    ///
    /// Returns P&L in dollars (not percentage).
    ///
    /// # Examples
    ///
    /// ```
    /// use trading_engine::state_machine::{Position, position::Side};
    ///
    /// let mut pos = Position::new(50000.0, 0.1, Side::Long, 1234567890);
    /// pos.update_current_price(51000.0);
    ///
    /// // Long: profit when price goes up
    /// // (51000 - 50000) * 0.1 = 100
    /// assert!((pos.unrealized_pnl().unwrap() - 100.0).abs() < 0.01);
    /// ```
    pub fn unrealized_pnl(&self) -> Option<f64> {
        if self.is_closed() {
            return None;
        }

        let price_diff = match self.side {
            Side::Long => self.current_price - self.entry_price,
            Side::Short => self.entry_price - self.current_price,
        };

        Some(price_diff * self.quantity)
    }

    /// Calculate realized P&L (for closed positions)
    ///
    /// # Examples
    ///
    /// ```
    /// use trading_engine::state_machine::{Position, position::Side};
    ///
    /// let mut pos = Position::new(50000.0, 0.1, Side::Long, 1234567890);
    /// pos.close(51000.0, 1234567900);
    ///
    /// assert!((pos.realized_pnl().unwrap() - 100.0).abs() < 0.01);
    /// ```
    pub fn realized_pnl(&self) -> Option<f64> {
        let exit_price = self.exit_price?;

        let price_diff = match self.side {
            Side::Long => exit_price - self.entry_price,
            Side::Short => self.entry_price - exit_price,
        };

        Some(price_diff * self.quantity)
    }

    /// Check if position is closed
    pub fn is_closed(&self) -> bool {
        self.exit_price.is_some()
    }

    /// Close the position
    ///
    /// # Arguments
    ///
    /// * `exit_price` - Price at which position was closed
    /// * `exit_timestamp` - Exit time in milliseconds
    pub fn close(&mut self, exit_price: f64, exit_timestamp: i64) {
        self.exit_price = Some(exit_price);
        self.exit_timestamp = Some(exit_timestamp);
    }

    /// Check if stop loss is hit
    ///
    /// # Examples
    ///
    /// ```
    /// use trading_engine::state_machine::{Position, position::Side};
    ///
    /// let mut pos = Position::new(50000.0, 0.1, Side::Long, 1234567890);
    /// pos.set_stop_loss(49000.0);
    /// pos.update_current_price(48500.0);
    ///
    /// assert!(pos.is_stop_loss_hit());
    /// ```
    pub fn is_stop_loss_hit(&self) -> bool {
        if let Some(stop) = self.stop_loss {
            match self.side {
                Side::Long => self.current_price <= stop,
                Side::Short => self.current_price >= stop,
            }
        } else {
            false
        }
    }

    /// Check if take profit is hit
    ///
    /// # Examples
    ///
    /// ```
    /// use trading_engine::state_machine::{Position, position::Side};
    ///
    /// let mut pos = Position::new(50000.0, 0.1, Side::Long, 1234567890);
    /// pos.set_take_profit(52000.0);
    /// pos.update_current_price(52500.0);
    ///
    /// assert!(pos.is_take_profit_hit());
    /// ```
    pub fn is_take_profit_hit(&self) -> bool {
        if let Some(target) = self.take_profit {
            match self.side {
                Side::Long => self.current_price >= target,
                Side::Short => self.current_price <= target,
            }
        } else {
            false
        }
    }

    /// Get position age in milliseconds
    pub fn age_ms(&self) -> Option<i64> {
        if self.is_closed() {
            self.exit_timestamp
                .map(|exit| exit - self.entry_timestamp)
        } else {
            Some(chrono::Utc::now().timestamp_millis() - self.entry_timestamp)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_long_position_profit() {
        let mut pos = Position::new(50000.0, 0.1, Side::Long, 1234567890);
        pos.update_current_price(51000.0);

        let pnl = pos.unrealized_pnl().unwrap();
        assert!((pnl - 100.0).abs() < 0.01);
    }

    #[test]
    fn test_long_position_loss() {
        let mut pos = Position::new(50000.0, 0.1, Side::Long, 1234567890);
        pos.update_current_price(49000.0);

        let pnl = pos.unrealized_pnl().unwrap();
        assert!((pnl + 100.0).abs() < 0.01); // Negative P&L
    }

    #[test]
    fn test_short_position_profit() {
        let mut pos = Position::new(50000.0, 0.1, Side::Short, 1234567890);
        pos.update_current_price(49000.0);

        let pnl = pos.unrealized_pnl().unwrap();
        assert!((pnl - 100.0).abs() < 0.01);
    }

    #[test]
    fn test_short_position_loss() {
        let mut pos = Position::new(50000.0, 0.1, Side::Short, 1234567890);
        pos.update_current_price(51000.0);

        let pnl = pos.unrealized_pnl().unwrap();
        assert!((pnl + 100.0).abs() < 0.01); // Negative P&L
    }

    #[test]
    fn test_stop_loss_long() {
        let mut pos = Position::new(50000.0, 0.1, Side::Long, 1234567890);
        pos.set_stop_loss(49000.0);

        pos.update_current_price(49500.0);
        assert!(!pos.is_stop_loss_hit());

        pos.update_current_price(48500.0);
        assert!(pos.is_stop_loss_hit());
    }

    #[test]
    fn test_take_profit_long() {
        let mut pos = Position::new(50000.0, 0.1, Side::Long, 1234567890);
        pos.set_take_profit(52000.0);

        pos.update_current_price(51500.0);
        assert!(!pos.is_take_profit_hit());

        pos.update_current_price(52500.0);
        assert!(pos.is_take_profit_hit());
    }

    #[test]
    fn test_close_position() {
        let mut pos = Position::new(50000.0, 0.1, Side::Long, 1234567890);
        assert!(!pos.is_closed());

        pos.close(51000.0, 1234567900);
        assert!(pos.is_closed());

        let realized = pos.realized_pnl().unwrap();
        assert!((realized - 100.0).abs() < 0.01);

        // Unrealized P&L should be None for closed positions
        assert!(pos.unrealized_pnl().is_none());
    }
}
