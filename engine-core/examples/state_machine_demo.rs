//! State Machine Demo
//!
//! Demonstrates the trading state machine with a simple strategy.
//!
//! Run with: cargo run --example state_machine_demo

use trading_engine::{
    MarketDataSource, MarketDataStorage, SimulatedFeed,
    indicators::*,
    state_machine::{StateMachine, State, Action},
};

/// Simple strategy: Enter on EMA crossover, exit on opposite crossover or stop/target
struct SimpleEmaStrategy {
    fast_period: usize,
    slow_period: usize,
    stop_loss_pct: f64,  // Stop loss percentage
    take_profit_pct: f64, // Take profit percentage
}

impl SimpleEmaStrategy {
    fn new() -> Self {
        Self {
            fast_period: 10,
            slow_period: 20,
            stop_loss_pct: 2.0,     // 2% stop loss
            take_profit_pct: 5.0,   // 5% take profit
        }
    }

    /// Decide what action to take based on current state and indicators
    fn decide(&self, sm: &StateMachine, closes: &[f64]) -> Action {
        if closes.len() < self.slow_period {
            return Action::NoAction;
        }

        let fast_ema = exponential_moving_average(closes, self.fast_period);
        let slow_ema = exponential_moving_average(closes, self.slow_period);

        let fast_current = *fast_ema.last().unwrap();
        let slow_current = *slow_ema.last().unwrap();

        let fast_prev = fast_ema[fast_ema.len() - 2];
        let slow_prev = slow_ema[slow_ema.len() - 2];

        match sm.current_state() {
            State::Idle => {
                // Look for bullish crossover
                if fast_prev <= slow_prev && fast_current > slow_current {
                    Action::StartAnalyzing {
                        reason: format!(
                            "Bullish EMA crossover detected: Fast={:.2}, Slow={:.2}",
                            fast_current, slow_current
                        ),
                    }
                } else {
                    Action::NoAction
                }
            }

            State::Analyzing => {
                // Confirm the signal is still valid
                if fast_current > slow_current {
                    let price = sm.context().latest_price().unwrap();

                    // Enter long position
                    Action::EnterLong {
                        price,
                        quantity: 0.1,
                    }
                } else {
                    // Signal invalidated
                    Action::CancelAnalysis {
                        reason: "EMA crossover invalidated".to_string(),
                    }
                }
            }

            State::InPosition => {
                let pos = sm.position().unwrap();

                // Check for bearish crossover (exit signal)
                if fast_prev >= slow_prev && fast_current < slow_current {
                    let price = sm.context().latest_price().unwrap();
                    return Action::ExitPosition { price };
                }

                // Otherwise, just monitor stop/take profit (handled automatically by StateMachine)
                Action::NoAction
            }
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .init();

    tracing::info!("=== State Machine Demo ===\n");
    tracing::info!("Strategy: EMA Crossover (10/20)");
    tracing::info!("Stop Loss: 2% | Take Profit: 5%\n");

    // Create components
    let mut feed = SimulatedFeed::new("BTCUSDT".to_string(), 50000.0);
    let storage = MarketDataStorage::new(100);
    let mut state_machine = StateMachine::new("BTCUSDT".to_string());
    let strategy = SimpleEmaStrategy::new();

    feed.connect().await?;
    feed.subscribe(vec!["BTCUSDT".to_string()]).await?;

    tracing::info!("Collecting data and running strategy...\n");

    // Run for 100 ticks
    for tick in 1..=100 {
        let data = feed.next_tick().await?;
        storage.push(data.clone());

        // Update state machine with latest data
        state_machine.update(&data);

        // Get indicators
        let window = storage.get_window("BTCUSDT").unwrap();
        let closes = window.closes(50);

        if closes.len() >= 20 {
            // Strategy decides what to do
            let action = strategy.decide(&state_machine, &closes);

            // Execute action
            if !matches!(action, Action::NoAction) {
                tracing::info!(
                    tick = tick,
                    price = format!("${:.2}", data.close),
                    state = ?state_machine.current_state(),
                    action = ?action,
                    "Action taken"
                );
            }

            state_machine.execute(action)?;

            // Log position status if in position
            if let Some(pos) = state_machine.position() {
                if tick % 10 == 0 {
                    let pnl = pos.unrealized_pnl().unwrap();
                    let pnl_pct = (pnl / (pos.entry_price() * pos.quantity())) * 100.0;

                    tracing::info!(
                        tick = tick,
                        price = format!("${:.2}", data.close),
                        entry = format!("${:.2}", pos.entry_price()),
                        pnl = format!("${:.2}", pnl),
                        pnl_pct = format!("{:.2}%", pnl_pct),
                        "Position update"
                    );
                }
            }
        }

        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    }

    feed.disconnect().await?;

    // Print summary
    tracing::info!("\n=== Summary ===");
    tracing::info!("Total ticks: 100");
    tracing::info!("Final state: {:?}", state_machine.current_state());
    tracing::info!("Transitions: {}", state_machine.transition_history().len());

    // Print transition history
    tracing::info!("\n=== Transition History ===");
    for (i, transition) in state_machine.transition_history().iter().enumerate() {
        tracing::info!(
            "{}: {:?} -> {:?}: {}",
            i + 1,
            transition.from,
            transition.to,
            transition.reason
        );
    }

    if let Some(pos) = state_machine.position() {
        let pnl = pos.unrealized_pnl().unwrap();
        tracing::info!("\n=== Current Position ===");
        tracing::info!("Entry: ${:.2}", pos.entry_price());
        tracing::info!("Current: ${:.2}", pos.current_price());
        tracing::info!("Unrealized P&L: ${:.2}", pnl);
    }

    tracing::info!("\n✅ Demo complete!\n");

    Ok(())
}
