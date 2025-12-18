//! Trading states
//!
//! Defines the possible states in the trading state machine.

use serde::{Deserialize, Serialize};

/// Trading state
///
/// Represents the current state of the trading state machine.
///
/// # State Transitions
///
/// ```text
/// Idle ──────> Analyzing ──────> InPosition
///   ^                               │
///   └───────────────────────────────┘
/// ```
///
/// # Examples
///
/// ```
/// use trading_engine::state_machine::State;
///
/// let state = State::Idle;
/// assert!(!state.is_in_position());
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum State {
    /// Waiting for trading opportunities
    ///
    /// In this state, the system monitors market data for potential setups.
    Idle,

    /// Opportunity detected, analyzing for entry
    ///
    /// In this state, the system is evaluating whether conditions are right
    /// to enter a position. May transition to InPosition or back to Idle.
    Analyzing,

    /// Currently holding a position
    ///
    /// In this state, the system is managing an active trade, monitoring
    /// for exit conditions (stop loss, take profit, or strategy exit signal).
    InPosition,
}

impl State {
    /// Check if currently in a position
    ///
    /// # Examples
    ///
    /// ```
    /// use trading_engine::state_machine::State;
    ///
    /// assert!(!State::Idle.is_in_position());
    /// assert!(!State::Analyzing.is_in_position());
    /// assert!(State::InPosition.is_in_position());
    /// ```
    pub fn is_in_position(&self) -> bool {
        matches!(self, State::InPosition)
    }

    /// Check if actively analyzing
    ///
    /// # Examples
    ///
    /// ```
    /// use trading_engine::state_machine::State;
    ///
    /// assert!(State::Analyzing.is_analyzing());
    /// assert!(!State::Idle.is_analyzing());
    /// ```
    pub fn is_analyzing(&self) -> bool {
        matches!(self, State::Analyzing)
    }

    /// Check if idle
    ///
    /// # Examples
    ///
    /// ```
    /// use trading_engine::state_machine::State;
    ///
    /// assert!(State::Idle.is_idle());
    /// assert!(!State::InPosition.is_idle());
    /// ```
    pub fn is_idle(&self) -> bool {
        matches!(self, State::Idle)
    }
}

impl std::fmt::Display for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            State::Idle => write!(f, "Idle"),
            State::Analyzing => write!(f, "Analyzing"),
            State::InPosition => write!(f, "InPosition"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_checks() {
        assert!(State::Idle.is_idle());
        assert!(!State::Idle.is_analyzing());
        assert!(!State::Idle.is_in_position());

        assert!(State::Analyzing.is_analyzing());
        assert!(!State::Analyzing.is_idle());
        assert!(!State::Analyzing.is_in_position());

        assert!(State::InPosition.is_in_position());
        assert!(!State::InPosition.is_idle());
        assert!(!State::InPosition.is_analyzing());
    }

    #[test]
    fn test_state_display() {
        assert_eq!(format!("{}", State::Idle), "Idle");
        assert_eq!(format!("{}", State::Analyzing), "Analyzing");
        assert_eq!(format!("{}", State::InPosition), "InPosition");
    }

    #[test]
    fn test_state_clone_copy() {
        let state = State::Analyzing;
        let state2 = state;
        assert_eq!(state, state2);
    }
}
