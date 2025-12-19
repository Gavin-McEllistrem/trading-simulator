use trading_engine::{
    market_data::{MarketData, MarketDataWindow},
    sources::{MarketDataSource, SimulatedFeed},
    state_machine::{Action, Context, State, StateMachine},
    strategy::{IndicatorApi, LuaStrategy},
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("=== Lua Strategy Demo ===\n");
    println!("This demo shows a Lua strategy driving the state machine.\n");

    // Load the EMA crossover strategy
    let strategy = LuaStrategy::new("../lua-strategies/examples/ema_crossover.lua")?;
    println!("✓ Loaded strategy: {}\n", strategy.name());

    // Create a simulated market data feed
    let mut feed = SimulatedFeed::new("BTCUSDT".to_string(), 50000.0);
    feed.connect().await?;
    feed.subscribe(vec!["BTCUSDT".to_string()]).await?;

    // Create state machine and data window
    let mut state_machine = StateMachine::new("BTCUSDT".to_string());
    let mut window = MarketDataWindow::new(50);

    println!("Strategy: EMA Crossover (10/20 periods)");
    println!("Starting simulation with 100 ticks...\n");

    let mut tick = 0;
    let max_ticks = 100;

    while tick < max_ticks {
        // Get next market data
        let market_data = feed.next_tick().await?;
        window.push(market_data.clone());

        // Update context with latest price
        let context = state_machine.context_mut();
        context.set("latest_price", market_data.close);
        context.set("latest_timestamp", market_data.timestamp);

        // Create indicator API
        let indicator_api = IndicatorApi::new(window.clone());

        // Execute strategy based on current state
        let action = match state_machine.current_state() {
            State::Idle => {
                // Look for opportunities
                let opportunity =
                    strategy.detect_opportunity(&market_data, state_machine.context(), &indicator_api)?;

                if let Some(opp_table) = opportunity {
                    // Update context with opportunity data
                    let signal: Option<String> = opp_table.get("signal").ok();
                    if let Some(sig) = signal {
                        state_machine.context_mut().set("signal", sig);
                        println!(
                            "[tick={:3}] Opportunity detected! Transitioning to Analyzing",
                            tick
                        );
                        Some(Action::StartAnalyzing {
                            reason: "Strategy signal detected".to_string(),
                        })
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            State::Analyzing => {
                // Decide whether to enter
                strategy.filter_commitment(&market_data, state_machine.context(), &indicator_api)?
            }
            State::InPosition => {
                // Manage the position
                strategy.manage_position(&market_data, state_machine.context(), &indicator_api)?
            }
        };

        // Execute action if strategy returned one
        if let Some(act) = action {
            println!("[tick={:3}] Action: {:?}", tick, act);
            state_machine.execute(act)?;
        }

        // Update state machine with market data (handles auto-exits)
        state_machine.update(&market_data);

        // Print position status
        if let Some(position) = state_machine.position() {
            if let Some(pnl) = position.unrealized_pnl() {
                let pnl_pct = (pnl / (position.entry_price() * position.quantity())) * 100.0;
                println!(
                    "[tick={:3}] Position: {} @ ${:.2}, P&L: ${:.2} ({:.2}%)",
                    tick,
                    position.side(),
                    position.entry_price(),
                    pnl,
                    pnl_pct
                );
            }

            if position.is_stop_loss_hit() {
                println!("[tick={:3}] ⚠️  Stop loss hit!", tick);
            } else if position.is_take_profit_hit() {
                println!("[tick={:3}] 🎯 Take profit hit!", tick);
            }
        }

        tick += 1;

        // Add a small delay for readability
        if tick % 10 == 0 {
            println!();
        }
    }

    println!("\n=== Simulation Complete ===\n");

    // Print final summary
    println!("Total ticks processed: {}", tick);
    println!("Final state: {:?}", state_machine.current_state());

    if let Some(position) = state_machine.position() {
        println!("\nFinal Position:");
        println!("  Side: {}", position.side());
        println!("  Entry: ${:.2}", position.entry_price());
        println!("  Current: ${:.2}", position.current_price());
        if let Some(pnl) = position.unrealized_pnl() {
            let pnl_pct = (pnl / (position.entry_price() * position.quantity())) * 100.0;
            println!("  P&L: ${:.2} ({:.2}%)", pnl, pnl_pct);
        }
    } else {
        println!("\nNo open position");
    }

    // Print state transition history
    println!("\n=== Transition History ===");
    for (i, transition) in state_machine.transition_history().iter().enumerate() {
        println!(
            "{}. {} -> {} ({})",
            i + 1,
            transition.from,
            transition.to,
            transition.reason
        );
    }

    feed.disconnect().await?;

    Ok(())
}
