//! Trading State Machine
//!
//! This module implements a state machine for managing trading logic.
//! The state machine coordinates between market data, technical indicators,
//! and trading strategies to make entry/exit decisions.
//!
//! # Architecture
//!
//! The state machine follows a three-state model:
//! - **Idle**: Waiting for trading opportunities
//! - **Analyzing**: Opportunity detected, analyzing for entry
//! - **InPosition**: Currently holding a position
//!
//! # Example
//!
//! ```
//! use trading_engine::state_machine::{StateMachine, State};
//!
//! let mut sm = StateMachine::new("BTCUSDT".to_string());
//! assert_eq!(sm.current_state(), &State::Idle);
//! ```

pub mod state;
pub mod context;
pub mod action;
pub mod position;

pub use state::State;
pub use context::Context;
pub use action::{Action, Side};
pub use position::Position;

use crate::{MarketData, Result};
use std::collections::VecDeque;

/// Maximum number of state transitions to keep in history
const MAX_TRANSITION_HISTORY: usize = 100;

/// Represents a state transition event
#[derive(Debug, Clone)]
pub struct Transition {
    pub from: State,
    pub to: State,
    pub timestamp: i64,
    pub reason: String,
}

/// The main state machine for trading logic
///
/// Manages the current state, context, and position for a single trading symbol.
/// This is a generic, reusable component that will be wrapped by SymbolRunner in Phase 5.
///
/// # Examples
///
/// ```
/// use trading_engine::state_machine::StateMachine;
///
/// let mut sm = StateMachine::new("BTCUSDT".to_string());
/// println!("Initial state: {:?}", sm.current_state());
/// ```
pub struct StateMachine {
    /// Trading symbol this state machine manages
    symbol: String,

    /// Current state
    state: State,

    /// Context data for decision making
    context: Context,

    /// Current position (if any)
    position: Option<Position>,

    /// History of state transitions
    transition_history: VecDeque<Transition>,
}

impl StateMachine {
    /// Create a new state machine for a symbol
    ///
    /// Starts in the Idle state with empty context.
    ///
    /// # Examples
    ///
    /// ```
    /// use trading_engine::state_machine::{StateMachine, State};
    ///
    /// let sm = StateMachine::new("BTCUSDT".to_string());
    /// assert_eq!(sm.current_state(), &State::Idle);
    /// assert_eq!(sm.symbol(), "BTCUSDT");
    /// ```
    pub fn new(symbol: String) -> Self {
        Self {
            symbol,
            state: State::Idle,
            context: Context::new(),
            position: None,
            transition_history: VecDeque::new(),
        }
    }

    /// Get the current state
    pub fn current_state(&self) -> &State {
        &self.state
    }

    /// Get the trading symbol
    pub fn symbol(&self) -> &str {
        &self.symbol
    }

    /// Get a reference to the context
    pub fn context(&self) -> &Context {
        &self.context
    }

    /// Get a mutable reference to the context
    pub fn context_mut(&mut self) -> &mut Context {
        &mut self.context
    }

    /// Get the current position (if any)
    pub fn position(&self) -> Option<&Position> {
        self.position.as_ref()
    }

    /// Get a mutable reference to the current position (if any)
    pub fn position_mut(&mut self) -> Option<&mut Position> {
        self.position.as_mut()
    }

    /// Get transition history
    pub fn transition_history(&self) -> &VecDeque<Transition> {
        &self.transition_history
    }

    /// Transition to a new state
    ///
    /// Records the transition in history and updates the current state.
    ///
    /// # Examples
    ///
    /// ```
    /// use trading_engine::state_machine::{StateMachine, State};
    ///
    /// let mut sm = StateMachine::new("BTCUSDT".to_string());
    /// sm.transition_to(State::Analyzing, "Opportunity detected".to_string());
    /// assert_eq!(sm.current_state(), &State::Analyzing);
    /// ```
    pub fn transition_to(&mut self, new_state: State, reason: String) {
        let transition = Transition {
            from: self.state,
            to: new_state,
            timestamp: chrono::Utc::now().timestamp_millis(),
            reason,
        };

        tracing::info!(
            symbol = %self.symbol,
            from = ?transition.from,
            to = ?transition.to,
            reason = %transition.reason,
            "State transition"
        );

        self.transition_history.push_back(transition);

        // Keep history bounded
        if self.transition_history.len() > MAX_TRANSITION_HISTORY {
            self.transition_history.pop_front();
        }

        self.state = new_state;
    }

    /// Execute an action
    ///
    /// Processes an action from a strategy and updates the state machine accordingly.
    ///
    /// # Arguments
    ///
    /// * `action` - The action to execute
    ///
    /// # Returns
    ///
    /// Result indicating success or error
    ///
    /// # Examples
    ///
    /// ```
    /// use trading_engine::state_machine::{StateMachine, Action};
    ///
    /// let mut sm = StateMachine::new("BTCUSDT".to_string());
    ///
    /// let action = Action::StartAnalyzing {
    ///     reason: "EMA crossover detected".to_string(),
    /// };
    ///
    /// sm.execute(action).unwrap();
    /// ```
    pub fn execute(&mut self, action: Action) -> Result<()> {
        match action {
            Action::EnterLong { price, quantity } => {
                self.enter_position(price, quantity, Side::Long);
            }

            Action::EnterShort { price, quantity } => {
                self.enter_position(price, quantity, Side::Short);
            }

            Action::ExitPosition { price } => {
                self.exit_position(price);
            }

            Action::UpdateStopLoss { new_stop } => {
                if let Some(pos) = self.position_mut() {
                    pos.set_stop_loss(new_stop);
                    tracing::info!(
                        symbol = %self.symbol,
                        new_stop = %new_stop,
                        "Updated stop loss"
                    );
                }
            }

            Action::UpdateTakeProfit { new_target } => {
                if let Some(pos) = self.position_mut() {
                    pos.set_take_profit(new_target);
                    tracing::info!(
                        symbol = %self.symbol,
                        new_target = %new_target,
                        "Updated take profit"
                    );
                }
            }

            Action::StartAnalyzing { reason } => {
                if self.state.is_idle() {
                    self.transition_to(State::Analyzing, reason);
                }
            }

            Action::CancelAnalysis { reason } => {
                if self.state.is_analyzing() {
                    self.transition_to(State::Idle, reason);
                }
            }

            Action::NoAction => {
                // Do nothing
            }
        }

        Ok(())
    }

    /// Update the state machine with new market data
    ///
    /// This updates the context and position with the latest price.
    /// Note: Strategies should call this and then execute actions based on their logic.
    ///
    /// # Arguments
    ///
    /// * `data` - New market data
    pub fn update(&mut self, data: &MarketData) {
        // Update context with latest data
        self.context.set_latest_price(data.close);
        self.context.set_latest_timestamp(data.timestamp);

        // Update position if we have one
        if let Some(ref mut pos) = self.position {
            pos.update_current_price(data.close);

            // Auto-exit on stop loss or take profit
            if pos.is_stop_loss_hit() {
                tracing::warn!(
                    symbol = %self.symbol,
                    price = %data.close,
                    stop = %pos.stop_loss().unwrap(),
                    "Stop loss hit"
                );
                self.exit_position(data.close);
            } else if pos.is_take_profit_hit() {
                tracing::info!(
                    symbol = %self.symbol,
                    price = %data.close,
                    target = %pos.take_profit().unwrap(),
                    "Take profit hit"
                );
                self.exit_position(data.close);
            }
        }
    }

    /// Enter a position
    ///
    /// Transitions to InPosition state and creates a Position.
    ///
    /// # Arguments
    ///
    /// * `entry_price` - Price at which position was entered
    /// * `quantity` - Position size
    /// * `side` - Long or Short
    fn enter_position(&mut self, entry_price: f64, quantity: f64, side: Side) {
        let position = Position::new(
            entry_price,
            quantity,
            side,
            chrono::Utc::now().timestamp_millis(),
        );

        self.position = Some(position);
        self.transition_to(
            State::InPosition,
            format!(
                "Entered {} position at ${:.2}, qty: {:.4}",
                side, entry_price, quantity
            ),
        );
    }

    /// Exit the current position
    ///
    /// Transitions back to Idle state and clears the position.
    ///
    /// # Arguments
    ///
    /// * `exit_price` - Price at which position was exited
    ///
    /// # Returns
    ///
    /// The closed position (if any)
    fn exit_position(&mut self, exit_price: f64) -> Option<Position> {
        if let Some(mut pos) = self.position.take() {
            pos.close(exit_price, chrono::Utc::now().timestamp_millis());

            let pnl = pos.realized_pnl().unwrap_or(0.0);
            let pnl_pct = (pnl / (pos.entry_price() * pos.quantity())) * 100.0;

            self.transition_to(
                State::Idle,
                format!(
                    "Exited position at ${:.2}, PnL: ${:.2} ({:.2}%)",
                    exit_price, pnl, pnl_pct
                ),
            );

            Some(pos)
        } else {
            None
        }
    }

    /// Reset the state machine
    ///
    /// Returns to Idle state, clears context and position.
    pub fn reset(&mut self) {
        self.state = State::Idle;
        self.context = Context::new();
        self.position = None;
        self.transition_history.clear();

        tracing::info!(symbol = %self.symbol, "State machine reset");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_data(price: f64) -> MarketData {
        MarketData {
            symbol: "BTCUSDT".to_string(),
            timestamp: chrono::Utc::now().timestamp_millis(),
            open: price,
            high: price,
            low: price,
            close: price,
            volume: 100,
            bid: price - 1.0,
            ask: price + 1.0,
        }
    }

    #[test]
    fn test_new_state_machine() {
        let sm = StateMachine::new("BTCUSDT".to_string());
        assert_eq!(sm.symbol(), "BTCUSDT");
        assert_eq!(sm.current_state(), &State::Idle);
        assert!(sm.position().is_none());
    }

    #[test]
    fn test_state_transition() {
        let mut sm = StateMachine::new("BTCUSDT".to_string());

        sm.transition_to(State::Analyzing, "Test transition".to_string());
        assert_eq!(sm.current_state(), &State::Analyzing);
        assert_eq!(sm.transition_history().len(), 1);
    }

    #[test]
    fn test_execute_enter_long() {
        let mut sm = StateMachine::new("BTCUSDT".to_string());

        let action = Action::EnterLong {
            price: 50000.0,
            quantity: 0.1,
        };

        sm.execute(action).unwrap();

        assert_eq!(sm.current_state(), &State::InPosition);
        assert!(sm.position().is_some());

        let pos = sm.position().unwrap();
        assert_eq!(pos.side(), Side::Long);
        assert_eq!(pos.entry_price(), 50000.0);
    }

    #[test]
    fn test_execute_exit() {
        let mut sm = StateMachine::new("BTCUSDT".to_string());

        // Enter position
        sm.execute(Action::EnterLong {
            price: 50000.0,
            quantity: 0.1,
        })
        .unwrap();

        // Exit position
        sm.execute(Action::ExitPosition { price: 51000.0 })
            .unwrap();

        assert_eq!(sm.current_state(), &State::Idle);
        assert!(sm.position().is_none());
    }

    #[test]
    fn test_update_with_data() {
        let mut sm = StateMachine::new("BTCUSDT".to_string());
        let data = create_test_data(50000.0);

        sm.update(&data);

        assert_eq!(sm.context().latest_price(), Some(50000.0));
    }

    #[test]
    fn test_stop_loss_auto_exit() {
        let mut sm = StateMachine::new("BTCUSDT".to_string());

        // Enter long position
        sm.execute(Action::EnterLong {
            price: 50000.0,
            quantity: 0.1,
        })
        .unwrap();

        // Set stop loss
        sm.execute(Action::UpdateStopLoss { new_stop: 49000.0 })
            .unwrap();

        // Update with price below stop
        let data = create_test_data(48500.0);
        sm.update(&data);

        // Should auto-exit
        assert_eq!(sm.current_state(), &State::Idle);
        assert!(sm.position().is_none());
    }

    #[test]
    fn test_take_profit_auto_exit() {
        let mut sm = StateMachine::new("BTCUSDT".to_string());

        // Enter long position
        sm.execute(Action::EnterLong {
            price: 50000.0,
            quantity: 0.1,
        })
        .unwrap();

        // Set take profit
        sm.execute(Action::UpdateTakeProfit { new_target: 52000.0 })
            .unwrap();

        // Update with price above target
        let data = create_test_data(52500.0);
        sm.update(&data);

        // Should auto-exit
        assert_eq!(sm.current_state(), &State::Idle);
        assert!(sm.position().is_none());
    }

    #[test]
    fn test_reset() {
        let mut sm = StateMachine::new("BTCUSDT".to_string());

        sm.transition_to(State::Analyzing, "Test".to_string());
        sm.context_mut().set("test_key", "test_value".to_string());

        sm.reset();

        assert_eq!(sm.current_state(), &State::Idle);
        assert!(sm.transition_history().is_empty());
    }
}
