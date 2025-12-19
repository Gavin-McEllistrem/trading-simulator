--[[
    Range Breakout Strategy

    This strategy identifies price ranges and enters trades when price breaks
    out of the range with volume confirmation. It tracks the high and low over
    a lookback period and enters when price exceeds these levels.

    Parameters:
    - Lookback Period: 20 bars
    - Breakout Threshold: Price must exceed range by 0.5%
    - Volume Confirmation: Volume must be 1.5x average
    - Stop Loss: Recent low (for long) or high (for short)
    - Take Profit: 2x the range size

    State Flow:
    1. Idle: Monitor for range breakouts
    2. Analyzing: Confirm breakout with volume
    3. InPosition: Manage based on price action
]]

-- Configuration
local LOOKBACK_PERIOD = 20
local BREAKOUT_THRESHOLD = 0.005  -- 0.5%
local VOLUME_MULTIPLIER = 1.5
local POSITION_SIZE = 0.12  -- 12% of capital

--[[
    Detect range breakouts
]]
function detect_opportunity(market_data, context, indicators)
    local high = indicators.high
    local low = indicators.low

    if high == nil or low == nil then
        return nil
    end

    local current_price = market_data.close
    local range_size = high - low

    -- Store range for later use
    context.range_high = high
    context.range_low = low
    context.range_size = range_size

    -- Detect upward breakout
    if current_price > high * (1 + BREAKOUT_THRESHOLD) then
        return {
            signal = "breakout_up",
            breakout_level = high,
            current_price = current_price,
            range_size = range_size,
            confidence = 0.75
        }
    end

    -- Detect downward breakout (for short positions)
    if current_price < low * (1 - BREAKOUT_THRESHOLD) then
        return {
            signal = "breakout_down",
            breakout_level = low,
            current_price = current_price,
            range_size = range_size,
            confidence = 0.70
        }
    end

    return nil
end

--[[
    Confirm breakout with volume and enter trade
]]
function filter_commitment(market_data, context, indicators)
    -- Get volume confirmation
    local avg_volume = indicators.avg_volume

    if avg_volume == nil or avg_volume == 0 then
        return {
            action = "cancel_analysis",
            reason = "Cannot calculate average volume"
        }
    end

    local current_volume = market_data.volume
    local volume_ratio = current_volume / avg_volume

    -- Require strong volume for breakout confirmation
    if volume_ratio < VOLUME_MULTIPLIER then
        return {
            action = "cancel_analysis",
            reason = string.format("Insufficient volume: %.2fx (need %.2fx)",
                volume_ratio, VOLUME_MULTIPLIER)
        }
    end

    -- Handle upward breakout (long)
    if context.signal == "breakout_up" then
        local entry_price = market_data.close
        local stop_loss = context.range_low
        local range_size = context.range_size
        local take_profit = entry_price + (range_size * 2)

        return {
            action = "enter_long",
            price = entry_price,
            quantity = POSITION_SIZE,
            stop_loss = stop_loss,
            take_profit = take_profit
        }
    end

    -- Handle downward breakout (short) - not implemented for long-only strategies
    if context.signal == "breakout_down" then
        return {
            action = "cancel_analysis",
            reason = "Short positions not enabled"
        }
    end

    return {
        action = "cancel_analysis",
        reason = "Unknown signal type"
    }
end

--[[
    Manage breakout position
]]
function manage_position(market_data, context, indicators)
    -- Get current price
    local current_price = market_data.close

    -- Retrieve entry information
    local range_low = context.range_low
    local range_high = context.range_high

    if range_low == nil or range_high == nil then
        return nil
    end

    -- Exit if price falls back into the range (failed breakout)
    if current_price < range_high then
        return {
            action = "exit",
            price = current_price,
            reason = "Price fell back into range (failed breakout)"
        }
    end

    -- Implement trailing stop: Move stop to breakeven once profit >= 50% of range
    local profit_target = range_high + (context.range_size * 0.5)

    if current_price >= profit_target then
        -- Move stop to breakeven (range_high)
        return {
            action = "update_stop_loss",
            new_stop = range_high
        }
    end

    return nil
end
