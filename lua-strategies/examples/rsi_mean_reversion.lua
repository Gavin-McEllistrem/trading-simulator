--[[
    RSI Mean Reversion Strategy

    This strategy uses the Relative Strength Index (RSI) to identify oversold
    and overbought conditions for mean reversion trading. It enters long when
    RSI is oversold (<30) and exits when RSI returns to neutral or overbought.

    Parameters:
    - RSI Period: 14
    - Oversold Level: 30
    - Overbought Level: 70
    - Stop Loss: 3% below entry
    - Take Profit: 4% above entry

    State Flow:
    1. Idle: Monitor RSI for oversold conditions
    2. Analyzing: Wait for RSI confirmation
    3. InPosition: Exit on RSI returning to neutral/overbought
]]

-- Configuration
local RSI_PERIOD = 14
local OVERSOLD_LEVEL = 30
local OVERBOUGHT_LEVEL = 70
local NEUTRAL_LEVEL = 50
local STOP_LOSS_PCT = 0.03  -- 3%
local TAKE_PROFIT_PCT = 0.04  -- 4%
local POSITION_SIZE = 0.15  -- 15% of capital

--[[
    Detect oversold/overbought conditions
]]
function detect_opportunity(market_data, context, indicators)
    local rsi = indicators.rsi(RSI_PERIOD)

    if rsi == nil then
        return nil
    end

    -- Update context
    context.current_rsi = rsi

    -- Detect oversold (potential buy)
    if rsi < OVERSOLD_LEVEL then
        return {
            signal = "oversold",
            rsi = rsi,
            confidence = (OVERSOLD_LEVEL - rsi) / OVERSOLD_LEVEL
        }
    end

    -- Detect overbought (not used for long-only, but tracked)
    if rsi > OVERBOUGHT_LEVEL then
        return {
            signal = "overbought",
            rsi = rsi,
            confidence = (rsi - OVERBOUGHT_LEVEL) / (100 - OVERBOUGHT_LEVEL)
        }
    end

    return nil
end

--[[
    Decide whether to enter a mean reversion trade
]]
function filter_commitment(market_data, context, indicators)
    -- Only enter on oversold signals
    if context.signal ~= "oversold" then
        return {
            action = "cancel_analysis",
            reason = "Not oversold"
        }
    end

    local rsi = indicators.rsi(RSI_PERIOD)
    if rsi == nil then
        return {
            action = "cancel_analysis",
            reason = "RSI not available"
        }
    end

    -- Confirm oversold condition still exists
    if rsi >= OVERSOLD_LEVEL then
        return {
            action = "cancel_analysis",
            reason = "RSI no longer oversold"
        }
    end

    -- Additional confirmation: Check if price is near recent low
    local low = indicators.low
    local current_price = market_data.close

    if low and current_price > low * 1.02 then
        -- Price has bounced more than 2% from low, wait
        return {
            action = "cancel_analysis",
            reason = "Price already bouncing"
        }
    end

    -- Enter long position
    local entry_price = market_data.close
    local stop_loss = entry_price * (1 - STOP_LOSS_PCT)
    local take_profit = entry_price * (1 + TAKE_PROFIT_PCT)

    return {
        action = "enter_long",
        price = entry_price,
        quantity = POSITION_SIZE,
        stop_loss = stop_loss,
        take_profit = take_profit
    }
end

--[[
    Manage the mean reversion position
]]
function manage_position(market_data, context, indicators)
    local rsi = indicators.rsi(RSI_PERIOD)

    if rsi == nil then
        return nil
    end

    -- Exit when RSI returns to neutral or becomes overbought
    if rsi >= NEUTRAL_LEVEL then
        return {
            action = "exit",
            price = market_data.close,
            reason = string.format("RSI returned to neutral/overbought: %.2f", rsi)
        }
    end

    -- Optional: Tighten stop loss if RSI improves significantly
    if rsi > 40 and rsi < NEUTRAL_LEVEL then
        local current_price = market_data.close
        local tight_stop = current_price * 0.99  -- 1% stop

        return {
            action = "update_stop_loss",
            new_stop = tight_stop
        }
    end

    return nil
end
