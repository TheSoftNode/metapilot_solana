use crate::constants::*;
use crate::errors::*;
use crate::state::*;
use anchor_lang::prelude::*;

// Import parameter types from instruction modules
use crate::instructions::{
    arbitrage::ArbitrageParams, dca::DcaParams, limit_orders::LimitOrderParams,
    stop_loss::StopLossParams, swaps::SwapParams,
};

/// Validate token automation configuration
pub fn validate_automation_config(
    supported_strategies: &[StrategyType],
    max_slippage: u16,
    gas_limit: u64,
) -> Result<()> {
    // Validate max slippage (0-10000 basis points = 0-100%)
    require!(
        max_slippage <= MAX_SLIPPAGE_BPS,
        TokenAutomationError::InvalidSlippage
    );

    // Validate gas limit
    require!(
        gas_limit >= MIN_GAS_LIMIT && gas_limit <= MAX_GAS_LIMIT,
        TokenAutomationError::InvalidGasLimit
    );

    // Validate strategy list is not empty
    require!(
        !supported_strategies.is_empty(),
        TokenAutomationError::NoStrategiesProvided
    );

    // Validate we don't exceed maximum number of strategies
    require!(
        supported_strategies.len() <= MAX_STRATEGIES_PER_AUTOMATION,
        TokenAutomationError::TooManyStrategies
    );

    Ok(())
}

/// Validate limit order parameters
pub fn validate_limit_order(params: &LimitOrderParams) -> Result<()> {
    // Validate amount
    require!(params.amount_in > 0, TokenAutomationError::InvalidAmount);

    require!(
        params.min_amount_out > 0,
        TokenAutomationError::InvalidAmount
    );

    // Validate price threshold
    require!(
        params.price_threshold > 0,
        TokenAutomationError::InvalidPriceThreshold
    );

    // Validate expiration
    require!(
        params.expires_in_seconds > MIN_ORDER_DURATION
            && params.expires_in_seconds <= MAX_ORDER_DURATION,
        TokenAutomationError::InvalidExpirationTime
    );

    // Validate slippage
    require!(
        params.max_slippage <= MAX_SLIPPAGE_BPS,
        TokenAutomationError::InvalidSlippage
    );

    // Validate different token mints
    require!(
        params.token_mint_a != params.token_mint_b,
        TokenAutomationError::SameTokenMints
    );

    Ok(())
}

/// Validate stop loss parameters
pub fn validate_stop_loss(params: &StopLossParams) -> Result<()> {
    // Validate amount
    require!(params.amount > 0, TokenAutomationError::InvalidAmount);

    // Validate stop price
    require!(
        params.stop_price > 0,
        TokenAutomationError::InvalidStopPrice
    );

    // Validate expiration
    require!(
        params.expires_in_seconds > MIN_ORDER_DURATION
            && params.expires_in_seconds <= MAX_ORDER_DURATION,
        TokenAutomationError::InvalidExpirationTime
    );

    // Validate slippage
    require!(
        params.max_slippage <= MAX_SLIPPAGE_BPS,
        TokenAutomationError::InvalidSlippage
    );

    Ok(())
}

/// Validate DCA strategy parameters
pub fn validate_dca_strategy(params: &DcaParams) -> Result<()> {
    // Validate amount per execution
    require!(
        params.amount_per_execution > 0,
        TokenAutomationError::InvalidAmount
    );

    // Validate interval
    require!(
        params.interval_seconds >= MIN_DCA_INTERVAL && params.interval_seconds <= MAX_DCA_INTERVAL,
        TokenAutomationError::InvalidDcaInterval
    );

    // Validate total executions
    require!(
        params.total_executions > 0 && params.total_executions <= MAX_DCA_EXECUTIONS,
        TokenAutomationError::InvalidExecutionCount
    );

    // Validate slippage
    require!(
        params.max_slippage <= MAX_SLIPPAGE_BPS,
        TokenAutomationError::InvalidSlippage
    );

    // Validate different token mints
    require!(
        params.token_mint_in != params.token_mint_out,
        TokenAutomationError::SameTokenMints
    );

    Ok(())
}

/// Validate swap parameters
pub fn validate_swap_params(params: &SwapParams) -> Result<()> {
    // Validate amounts
    require!(params.amount_in > 0, TokenAutomationError::InvalidAmount);

    require!(
        params.minimum_amount_out > 0,
        TokenAutomationError::InvalidAmount
    );

    // Validate AI confidence level
    require!(
        params.ai_confidence <= 100,
        TokenAutomationError::InvalidConfidenceLevel
    );

    // Validate price data freshness (should be within last hour)
    let now = Clock::get()?.unix_timestamp;
    let price_age = now - params.price_data.timestamp;
    require!(
        price_age <= MAX_PRICE_AGE_SECONDS,
        TokenAutomationError::StalePrice
    );

    // Validate price confidence
    require!(
        params.price_data.confidence >= MIN_PRICE_CONFIDENCE,
        TokenAutomationError::LowPriceConfidence
    );

    Ok(())
}

/// Validate arbitrage parameters
pub fn validate_arbitrage_params(params: &ArbitrageParams) -> Result<()> {
    // Validate amount
    require!(params.amount > 0, TokenAutomationError::InvalidAmount);

    // Validate expected profit
    require!(
        params.expected_profit_bps >= MIN_ARBITRAGE_PROFIT_BPS.into(),
        TokenAutomationError::InsufficientProfit
    );

    // Validate different DEXs
    require!(
        params.dex_a != params.dex_b,
        TokenAutomationError::SameDexes
    );

    // Validate different token mints
    require!(
        params.token_mint_a != params.token_mint_b,
        TokenAutomationError::SameTokenMints
    );

    Ok(())
}

/// Calculate execution fee based on gas and complexity
pub fn calculate_execution_fee(gas_used: u64, complexity_multiplier: u64, base_fee: u64) -> u64 {
    let gas_fee = gas_used.saturating_mul(DEFAULT_GAS_PRICE);
    let complexity_fee = base_fee.saturating_mul(complexity_multiplier);
    gas_fee.saturating_add(complexity_fee)
}

/// Calculate slippage based on market conditions
pub fn calculate_dynamic_slippage(
    base_slippage: u16,
    volatility: u16,
    volume: u64,
    market_impact: u16,
) -> u16 {
    let volatility_adjustment = volatility.saturating_div(100); // Convert from basis points
    let volume_adjustment = if volume < LOW_VOLUME_THRESHOLD {
        100
    } else {
        0
    }; // 1% extra for low volume
    let impact_adjustment = market_impact.saturating_div(10);

    base_slippage
        .saturating_add(volatility_adjustment)
        .saturating_add(volume_adjustment)
        .saturating_add(impact_adjustment)
        .min(MAX_SLIPPAGE_BPS)
}

/// Calculate price impact based on amount and liquidity
pub fn calculate_price_impact(amount: u64, liquidity: u64, base_price: u64) -> Result<u64> {
    require!(liquidity > 0, TokenAutomationError::InsufficientLiquidity);

    // Simple price impact calculation: impact = (amount / liquidity) * base_price
    let impact_ratio = amount
        .checked_div(liquidity)
        .ok_or(TokenAutomationError::MathOverflow)?;

    let price_impact = base_price
        .checked_mul(impact_ratio)
        .ok_or(TokenAutomationError::MathOverflow)?;

    Ok(price_impact)
}

/// Check if price meets threshold conditions
pub fn check_price_threshold(
    current_price: u64,
    threshold_price: u64,
    order_side: OrderSide,
) -> bool {
    match order_side {
        OrderSide::Buy => current_price <= threshold_price,
        OrderSide::Sell => current_price >= threshold_price,
    }
}

/// Calculate moving average for technical analysis
pub fn calculate_moving_average(prices: &[u64], period: usize) -> Result<u64> {
    require!(
        prices.len() >= period && period > 0,
        TokenAutomationError::InsufficientPriceData
    );

    let sum = prices[prices.len() - period..]
        .iter()
        .try_fold(0u64, |acc, &price| acc.checked_add(price))
        .ok_or(TokenAutomationError::MathOverflow)?;

    Ok(sum / period as u64)
}

/// Calculate RSI (Relative Strength Index)
pub fn calculate_rsi(prices: &[u64], period: usize) -> Result<u8> {
    require!(
        prices.len() > period && period > 0,
        TokenAutomationError::InsufficientPriceData
    );

    let mut gains = 0u64;
    let mut losses = 0u64;

    // Calculate gains and losses over the period
    for i in 1..=period {
        let current = prices[prices.len() - i];
        let previous = prices[prices.len() - i - 1];

        if current > previous {
            gains = gains.saturating_add(current - previous);
        } else if previous > current {
            losses = losses.saturating_add(previous - current);
        }
    }

    if losses == 0 {
        return Ok(100);
    }

    let rs = gains
        .checked_div(losses)
        .ok_or(TokenAutomationError::MathOverflow)?;

    let rsi = 100 - (100 / (1 + rs));
    Ok(rsi.min(100) as u8)
}

/// Check if market conditions are suitable for trading
pub fn check_market_conditions(
    volatility: u16,
    volume: u64,
    rsi: u8,
    strategy_type: &StrategyType,
) -> bool {
    match strategy_type {
        StrategyType::DollarCostAveraging => {
            // DCA can work in any market condition
            true
        }
        StrategyType::LimitOrder => {
            // Limit orders work better in stable markets
            volatility <= HIGH_VOLATILITY_THRESHOLD
        }
        StrategyType::StopLoss => {
            // Stop losses should work in any condition
            true
        }
        StrategyType::TrendFollowing => {
            // Trend following works in trending markets (avoid choppy markets)
            volume >= MEDIUM_VOLUME_THRESHOLD && volatility >= LOW_VOLATILITY_THRESHOLD
        }
        StrategyType::Arbitrage => {
            // Arbitrage needs sufficient volume
            volume >= HIGH_VOLUME_THRESHOLD
        }
        StrategyType::MeanReversion => {
            // Mean reversion works in overbought/oversold conditions
            rsi <= 30 || rsi >= 70
        }
        StrategyType::Momentum => {
            // Momentum strategies need strong trends
            volume >= HIGH_VOLUME_THRESHOLD && volatility >= MEDIUM_VOLATILITY_THRESHOLD
        }
        StrategyType::GridTrading => {
            // Grid trading works in ranging markets
            volatility >= LOW_VOLATILITY_THRESHOLD && volatility <= MEDIUM_VOLATILITY_THRESHOLD
        }
    }
}

/// Generate unique order ID
pub fn generate_order_id(
    automation_account: &Pubkey,
    timestamp: i64,
    order_type: OrderType,
) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    use std::hash::Hasher;

    hasher.write(automation_account.as_ref());
    hasher.write(&timestamp.to_le_bytes());
    hasher.write(&(order_type as u8).to_le_bytes());

    hasher.finish()
}

/// Validate token account ownership
pub fn validate_token_account_owner(
    token_account: &Account<anchor_spl::token::TokenAccount>,
    expected_owner: &Pubkey,
) -> Result<()> {
    require!(
        token_account.owner == *expected_owner,
        TokenAutomationError::InvalidTokenAccountOwner
    );
    Ok(())
}

/// Check if automation is in valid state for operations
pub fn check_automation_state(automation: &Account<TokenAutomation>) -> Result<()> {
    require!(
        automation.is_active,
        TokenAutomationError::AutomationInactive
    );
    require!(
        !automation.is_paused,
        TokenAutomationError::AutomationPaused
    );
    Ok(())
}

/// Format amount with decimals for display
pub fn format_amount_with_decimals(amount: u64, decimals: u8) -> String {
    let divisor = 10u64.pow(decimals as u32);
    let whole = amount / divisor;
    let fractional = amount % divisor;

    if fractional == 0 {
        whole.to_string()
    } else {
        format!(
            "{}.{:0width$}",
            whole,
            fractional,
            width = decimals as usize
        )
    }
}

/// Calculate total strategy value in USD
pub fn calculate_strategy_value_usd(
    token_amount: u64,
    token_price_usd: u64,
    token_decimals: u8,
) -> Result<u64> {
    let normalized_amount = token_amount
        .checked_div(10u64.pow(token_decimals as u32))
        .ok_or(TokenAutomationError::MathOverflow)?;

    normalized_amount
        .checked_mul(token_price_usd)
        .ok_or(TokenAutomationError::MathOverflow.into())
}

/// Validate strategy execution timing
pub fn validate_execution_timing(
    last_execution: i64,
    interval: i64,
    buffer_seconds: i64,
) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;
    let time_since_last = now - last_execution;
    let required_interval = interval - buffer_seconds;

    require!(
        time_since_last >= required_interval,
        TokenAutomationError::ExecutionTooEarly
    );

    Ok(())
}
