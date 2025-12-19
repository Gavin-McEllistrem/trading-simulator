--[[
    EMA Crossover Strategy

    This strategy uses two Exponential Moving Averages (fast and slow) to generate
    trading signals. When the fast EMA crosses above the slow EMA, it signals a
    bullish trend (potential buy). When the fast EMA crosses below the slow EMA,
    it signals a bearish trend (potential exit).

    Parameters:
    - Fast EMA: 10 periods
    - Slow EMA: 20 periods
    - Stop Loss: 2% below entry
    - Take Profit: 5% above entry

    State Flow:
    1. Idle: Monitor for EMA crossovers
    2. Analyzing: Confirm the crossover signal
    3. InPosition: Manage the active trade
]]

-- Configuration
local FAST_EMA = 10
local SLOW_EMA = 20
local STOP_LOSS_PCT = 0.02  -- 2%
local TAKE_PROFIT_PCT = 0.05  -- 5%
local POSITION_SIZE = 0.1  -- 10% of capital

--[[
    Called in Idle state to detect trading opportunities

    Returns:
    - nil: No opportunity detected
    - table: Opportunity details to store in context
]]
function detect_opportunity(market_data, context, indicators)
    -- Need enough data for indicators
    local ema_10 = indicators.ema(FAST_EMA)
    local ema_20 = indicators.ema(SLOW_EMA)

    if ema_10 == nil or ema_20 == nil then
        return nil
    end

    -- Get previous values from context (if available)
    local prev_ema_10 = context.prev_ema_10
    local prev_ema_20 = context.prev_ema_20

    -- Update context with current values
    context.prev_ema_10 = ema_10
    context.prev_ema_20 = ema_20

    -- Check for bullish crossover (fast crosses above slow)
    if prev_ema_10 and prev_ema_20 then
        if prev_ema_10 <= prev_ema_20 and ema_10 > ema_20 then
            return {
                signal = "bullish",
                ema_10 = ema_10,
                ema_20 = ema_20,
                confidence = 0.8
            }
        end

        -- Check for bearish crossover (fast crosses below slow)
        if prev_ema_10 >= prev_ema_20 and ema_10 < ema_20 then
            return {
                signal = "bearish",
                ema_10 = ema_10,
                ema_20 = ema_20,
                confidence = 0.7
            }
        end
    end

    return nil
end

--[[
    Called in Analyzing state to decide on trade entry

    Returns:
    - nil: Not ready to enter (cancel analysis)
    - action table: Enter the trade
]]
function filter_commitment(market_data, context, indicators)
    -- Only enter on bullish signals for this long-only strategy
    if context.signal == "bullish" then
        local entry_price = market_data.close
        local stop_loss = entry_price * (1 - STOP_LOSS_PCT)
        local take_profit = entry_price * (1 + TAKE_PROFIT_PCT)

        -- Check volume confirmation (optional)
        local avg_vol = indicators.avg_volume
        if avg_vol and avg_vol > 0 and market_data.volume < avg_vol * 0.5 then
            -- Low volume, cancel analysis
            return {
                action = "cancel_analysis",
                reason = "Low volume - insufficient confirmation"
            }
        end

        -- Enter long position
        return {
            action = "enter_long",
            price = entry_price,
            quantity = POSITION_SIZE,
            stop_loss = stop_loss,
            take_profit = take_profit
        }
    end

    -- Cancel analysis if signal is not bullish
    return {
        action = "cancel_analysis",
        reason = "No bullish signal"
    }
end

--[[
    Called in InPosition state to manage the active trade

    Returns:
    - nil: No action needed
    - action table: Modify or exit the position
]]
function manage_position(market_data, context, indicators)
    -- Get current EMAs
    local ema_10 = indicators.ema(FAST_EMA)
    local ema_20 = indicators.ema(SLOW_EMA)

    if ema_10 == nil or ema_20 == nil then
        return nil
    end

    -- Exit on bearish crossover
    if ema_10 < ema_20 then
        return {
            action = "exit",
            price = market_data.close,
            reason = "Bearish crossover signal"
        }
    end

    -- Optional: Implement trailing stop
    -- This would require tracking the highest price since entry

    return nil
end
