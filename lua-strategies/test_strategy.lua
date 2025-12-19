-- Simple test strategy for integration tests

function detect_opportunity(market_data, context, indicators)
    local ema_10 = indicators.ema(10)

    if ema_10 == nil then
        return nil
    end

    -- Trigger on price above EMA
    if market_data.close > ema_10 then
        return {
            signal = "bullish",
            ema = ema_10
        }
    end

    return nil
end

function filter_commitment(market_data, context, indicators)
    if context.signal == "bullish" then
        return {
            action = "enter_long",
            price = market_data.close,
            quantity = 0.1
        }
    end

    return {
        action = "cancel_analysis",
        reason = "No signal"
    }
end

function manage_position(market_data, context, indicators)
    -- Simple exit on price drop
    if market_data.close < 45000 then
        return {
            action = "exit",
            price = market_data.close
        }
    end

    return nil
end
