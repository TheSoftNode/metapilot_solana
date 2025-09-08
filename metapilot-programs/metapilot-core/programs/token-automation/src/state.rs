use crate::constants::*;
use anchor_lang::prelude::*;

/// Order side enum
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq)]
pub enum OrderSide {
    Buy,
    Sell,
}

/// Main token automation account that manages automated trading strategies
#[account]
pub struct TokenAutomation {
    /// Authority that can modify settings
    pub authority: Pubkey,

    /// Treasury account for collecting fees
    pub treasury: Pubkey,

    /// Supported trading strategies
    pub supported_strategies: Vec<StrategyType>,

    /// Maximum slippage allowed (in basis points)
    pub max_slippage: u16,

    /// Gas limit for automated operations
    pub gas_limit: u64,

    /// Whether automation is active
    pub is_active: bool,

    /// Whether operations are paused (emergency)
    pub is_paused: bool,

    /// When this automation was created
    pub created_at: i64,

    /// Last time settings were updated
    pub last_updated: i64,

    /// Total number of executions
    pub total_executions: u64,

    /// Total trading volume
    pub total_volume: u64,

    /// Total fees collected
    pub total_fees_collected: u64,

    /// Emergency authority for critical situations
    pub emergency_authority: Pubkey,

    /// Whether AI automation is enabled
    pub ai_automation_enabled: bool,
}

impl TokenAutomation {
    pub const SIZE: usize = TOKEN_AUTOMATION_SIZE;
    pub const LEN: usize = TOKEN_AUTOMATION_SIZE;
}

/// Limit order for buying/selling at specific price
#[account]
pub struct LimitOrder {
    /// Unique order ID
    pub id: u64,

    /// Order owner
    pub owner: Pubkey,

    /// Associated token automation account
    pub token_automation: Pubkey,

    /// Input token mint
    pub token_mint_a: Pubkey,

    /// Output token mint
    pub token_mint_b: Pubkey,

    /// Amount of input tokens
    pub amount_in: u64,

    /// Minimum amount of output tokens
    pub min_amount_out: u64,

    /// Price threshold for execution
    pub price_threshold: u64,

    /// When order was created
    pub created_at: i64,

    /// When order expires
    pub expires_at: i64,

    /// Buy or sell order
    pub order_side: OrderSide,

    /// Current order status
    pub status: OrderStatus,

    /// Maximum slippage for this order
    pub max_slippage: u16,

    /// Whether this is a fill-or-kill order
    pub fill_or_kill: bool,
}

impl LimitOrder {
    pub const SIZE: usize = LIMIT_ORDER_SIZE;

    /// Check if order is still valid
    pub fn is_valid(&self) -> bool {
        let now = Clock::get().unwrap().unix_timestamp;
        self.status == OrderStatus::Active && now <= self.expires_at
    }

    /// Check if price condition is met
    pub fn price_condition_met(&self, current_price: u64) -> bool {
        match self.order_side {
            OrderSide::Buy => current_price <= self.price_threshold,
            OrderSide::Sell => current_price >= self.price_threshold,
        }
    }
}

/// Stop loss order for risk management
#[account]
pub struct StopLoss {
    /// Unique order ID
    pub id: u64,

    /// Order owner
    pub owner: Pubkey,

    /// Associated token automation account
    pub token_automation: Pubkey,

    /// Token mint to sell
    pub token_mint: Pubkey,

    /// Amount to sell when triggered
    pub amount: u64,

    /// Price at which to trigger the stop loss
    pub stop_price: u64,

    /// When order was created
    pub created_at: i64,

    /// When order expires
    pub expires_at: i64,

    /// Current order status
    pub status: OrderStatus,

    /// Maximum slippage for this order
    pub max_slippage: u16,
}

impl StopLoss {
    pub const SIZE: usize = STOP_LOSS_SIZE;

    /// Check if stop loss should be triggered
    pub fn should_trigger(&self, current_price: u64) -> bool {
        self.status == OrderStatus::Active && current_price <= self.stop_price
    }

    /// Check if order is still valid
    pub fn is_valid(&self) -> bool {
        let now = Clock::get().unwrap().unix_timestamp;
        self.status == OrderStatus::Active && now <= self.expires_at
    }
}

/// Dollar Cost Averaging strategy
#[account]
pub struct DcaStrategy {
    /// Unique strategy ID
    pub id: u64,

    /// Strategy owner
    pub owner: Pubkey,

    /// Associated token automation account
    pub token_automation: Pubkey,

    /// Input token mint (usually stablecoin)
    pub token_mint_in: Pubkey,

    /// Output token mint (target token)
    pub token_mint_out: Pubkey,

    /// Amount to spend per execution
    pub amount_per_execution: u64,

    /// Time interval between executions (seconds)
    pub interval_seconds: i64,

    /// Total number of executions planned
    pub total_executions: u64,

    /// Number of executions completed
    pub executed_count: u64,

    /// When next execution should happen
    pub next_execution_at: i64,

    /// When strategy was created
    pub created_at: i64,

    /// Current strategy status
    pub status: StrategyStatus,

    /// Maximum slippage for each execution
    pub max_slippage: u16,
}

impl DcaStrategy {
    pub const SIZE: usize = DCA_STRATEGY_SIZE;

    /// Check if strategy is ready for next execution
    pub fn ready_for_execution(&self) -> bool {
        let now = Clock::get().unwrap().unix_timestamp;
        self.status == StrategyStatus::Active
            && now >= self.next_execution_at
            && self.executed_count < self.total_executions
    }

    /// Check if strategy is completed
    pub fn is_completed(&self) -> bool {
        self.executed_count >= self.total_executions || self.status == StrategyStatus::Completed
    }
}

/// Arbitrage opportunity
#[account]
pub struct ArbitrageOpportunity {
    /// Unique opportunity ID
    pub id: u64,

    /// Who discovered this opportunity
    pub discoverer: Pubkey,

    /// First token in the pair
    pub token_mint_a: Pubkey,

    /// Second token in the pair
    pub token_mint_b: Pubkey,

    /// First DEX for the arbitrage
    pub dex_a: Pubkey,

    /// Second DEX for the arbitrage
    pub dex_b: Pubkey,

    /// Price difference in basis points
    pub price_diff_bps: u64,

    /// Maximum amount that can be arbitraged
    pub max_amount: u64,

    /// When opportunity was discovered
    pub discovered_at: i64,

    /// When opportunity expires
    pub expires_at: i64,

    /// Current opportunity status
    pub status: OpportunityStatus,
}

impl ArbitrageOpportunity {
    pub const SIZE: usize = ARBITRAGE_OPPORTUNITY_SIZE;

    /// Check if opportunity is still valid
    pub fn is_valid(&self) -> bool {
        let now = Clock::get().unwrap().unix_timestamp;
        self.status == OpportunityStatus::Active && now <= self.expires_at
    }

    /// Check if opportunity is profitable enough
    pub fn is_profitable(&self, min_profit_bps: u64) -> bool {
        self.price_diff_bps >= min_profit_bps
    }
}

/// Trading strategy types
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq, Debug)]
pub enum StrategyType {
    DollarCostAveraging,
    LimitOrder,
    StopLoss,
    TrendFollowing,
    Arbitrage,
    MeanReversion,
    Momentum,
    GridTrading,
}

/// Order status
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq, Debug)]
pub enum OrderStatus {
    Active,
    Executed,
    Cancelled,
    Expired,
    PartiallyFilled,
}

/// Strategy status
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq, Debug)]
pub enum StrategyStatus {
    Active,
    Paused,
    Completed,
    Cancelled,
}

/// Arbitrage opportunity status
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq, Debug)]
pub enum OpportunityStatus {
    Active,
    Executed,
    Expired,
    Invalidated,
}

/// Order type enumeration
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq, Debug)]
pub enum OrderType {
    LimitOrder,
    StopLoss,
    DcaStrategy,
}

/// Trigger conditions for automated execution
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct TriggerConditions {
    /// Price threshold for execution
    pub price_threshold: u64,

    /// Time interval for DCA (in seconds)
    pub time_interval: i64,

    /// Volume threshold
    pub volume_threshold: u64,

    /// Market condition indicator
    pub market_condition: u8,

    /// RSI thresholds for technical analysis
    pub rsi_upper: u8,
    pub rsi_lower: u8,

    /// Moving average period
    pub ma_period: u16,

    /// Volatility threshold
    pub volatility_threshold: u16,
}

impl Default for TriggerConditions {
    fn default() -> Self {
        Self {
            price_threshold: 0,
            time_interval: DEFAULT_DCA_INTERVAL,
            volume_threshold: 0,
            market_condition: 0,
            rsi_upper: 70,
            rsi_lower: 30,
            ma_period: 20,
            volatility_threshold: 500, // 5%
        }
    }
}

/// Price data from oracles
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct PriceData {
    /// Exchange rate between tokens
    pub exchange_rate: u64,

    /// Decimal precision
    pub decimals: u8,

    /// When price was last updated
    pub timestamp: i64,

    /// Confidence level of the price
    pub confidence: u8,

    /// Volume in the last period
    pub volume_24h: u64,

    /// Price change percentage (in basis points)
    pub price_change_24h: i16,
}

/// Treasury account for fee collection
#[account]
pub struct Treasury {
    pub authority: Pubkey,
    pub balance: u64,
}

impl Treasury {
    pub const LEN: usize = 8 + 32 + 8;
}

/// DCA execution status
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq)]
pub enum DcaStatus {
    Active,
    Paused,
    Completed,
    Cancelled,
}

/// Arbitrage opportunity status
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq)]
pub enum ArbitrageStatus {
    Active,
    Executed,
    Expired,
    Cancelled,
}

/// Market data for price feeds
#[account]
#[derive(Default)]
pub struct MarketData {
    pub symbol: String,
    pub price: u64,
    pub confidence: u8,
    pub volume_24h: u64,
    pub price_change_24h: i16,
    pub exchange_rate: u64,
    pub decimals: u8,
    pub timestamp: i64,
}

/// Execution type for different automation methods
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum ExecutionType {
    Manual,
    Automated,
    AiDriven,
    Scheduled,
    Triggered,
}
