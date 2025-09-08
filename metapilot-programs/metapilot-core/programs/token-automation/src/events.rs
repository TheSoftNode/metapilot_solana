use crate::state::*;
use anchor_lang::prelude::*;

#[event]
pub struct AutomationInitialized {
    pub automation: Pubkey,
    pub authority: Pubkey,
    pub treasury: Pubkey,
    pub supported_strategies: Vec<StrategyType>,
    pub max_slippage: u16,
    pub timestamp: i64,
}

// Alias for backwards compatibility
pub type TokenAutomationInitialized = AutomationInitialized;

#[event]
pub struct AutomationUpdated {
    pub automation: Pubkey,
    pub authority: Pubkey,
    pub timestamp: i64,
}

// Alias for backwards compatibility
pub type AutomationSettingsUpdated = AutomationUpdated;

#[event]
pub struct AutomationStateChanged {
    pub automation: Pubkey,
    pub is_active: bool,
    pub timestamp: i64,
}

#[event]
pub struct EmergencyPaused {
    pub automation: Pubkey,
    pub emergency_authority: Pubkey,
    pub timestamp: i64,
}

// Alias for backwards compatibility
pub type EmergencyPauseActivated = EmergencyPaused;

#[event]
pub struct AutomationResumed {
    pub automation: Pubkey,
    pub resumed_by: Pubkey,
    pub timestamp: i64,
}

// Alias for backwards compatibility
pub type EmergencyPauseDeactivated = AutomationResumed;

#[event]
pub struct AuthorityTransferred {
    pub automation: Pubkey,
    pub old_authority: Pubkey,
    pub new_authority: Pubkey,
    pub timestamp: i64,
}

#[event]
pub struct AutomationClosed {
    pub automation: Pubkey,
    pub authority: Pubkey,
    pub timestamp: i64,
}

#[event]
pub struct LimitOrderCreated {
    pub order_id: u64,
    pub owner: Pubkey,
    pub automation: Pubkey,
    pub token_mint_a: Pubkey,
    pub token_mint_b: Pubkey,
    pub amount_in: u64,
    pub price_threshold: u64,
    pub order_side: OrderSide,
    pub expires_at: i64,
    pub timestamp: i64,
}

#[event]
pub struct LimitOrderCancelled {
    pub order_id: u64,
    pub owner: Pubkey,
    pub automation: Pubkey,
    pub timestamp: i64,
}

#[event]
pub struct LimitOrderExecuted {
    pub order_id: u64,
    pub owner: Pubkey,
    pub automation: Pubkey,
    pub amount_in: u64,
    pub amount_out: u64,
    pub execution_price: u64,
    pub execution_fee: u64,
    pub timestamp: i64,
}

#[event]
pub struct LimitOrderUpdated {
    pub order_id: u64,
    pub owner: Pubkey,
    pub new_price_threshold: Option<u64>,
    pub new_expires_at: Option<i64>,
    pub new_max_slippage: Option<u16>,
    pub timestamp: i64,
}

#[event]
pub struct StopLossCreated {
    pub order_id: u64,
    pub owner: Pubkey,
    pub automation: Pubkey,
    pub token_mint: Pubkey,
    pub amount: u64,
    pub stop_price: u64,
    pub expires_at: i64,
    pub timestamp: i64,
}

#[event]
pub struct StopLossCancelled {
    pub order_id: u64,
    pub owner: Pubkey,
    pub automation: Pubkey,
    pub timestamp: i64,
}

#[event]
pub struct StopLossExecuted {
    pub order_id: u64,
    pub owner: Pubkey,
    pub automation: Pubkey,
    pub amount_in: u64,
    pub amount_out: u64,
    pub trigger_price: u64,
    pub execution_fee: u64,
    pub timestamp: i64,
}

#[event]
pub struct StopLossUpdated {
    pub order_id: u64,
    pub owner: Pubkey,
    pub new_stop_price: Option<u64>,
    pub new_expires_at: Option<i64>,
    pub new_max_slippage: Option<u16>,
    pub timestamp: i64,
}

#[event]
pub struct DcaStrategyCreated {
    pub strategy_id: u64,
    pub owner: Pubkey,
    pub automation: Pubkey,
    pub token_mint_in: Pubkey,
    pub token_mint_out: Pubkey,
    pub amount_per_execution: u64,
    pub interval_seconds: i64,
    pub total_executions: u64,
    pub next_execution_at: i64,
    pub timestamp: i64,
}

#[event]
pub struct DcaOrderExecuted {
    pub strategy_id: u64,
    pub owner: Pubkey,
    pub automation: Pubkey,
    pub execution_number: u64,
    pub amount_in: u64,
    pub amount_out: u64,
    pub execution_price: u64,
    pub execution_fee: u64,
    pub next_execution_at: i64,
    pub is_completed: bool,
    pub timestamp: i64,
}

#[event]
pub struct DcaStrategyPaused {
    pub strategy_id: u64,
    pub owner: Pubkey,
    pub timestamp: i64,
}

#[event]
pub struct DcaStrategyResumed {
    pub strategy_id: u64,
    pub owner: Pubkey,
    pub next_execution_at: i64,
    pub timestamp: i64,
}

#[event]
pub struct DcaStrategyCancelled {
    pub strategy_id: u64,
    pub owner: Pubkey,
    pub executions_completed: u64,
    pub timestamp: i64,
}

#[event]
pub struct DcaStrategyUpdated {
    pub strategy_id: u64,
    pub owner: Pubkey,
    pub new_interval_seconds: Option<i64>,
    pub new_max_slippage: Option<u16>,
    pub timestamp: i64,
}

#[event]
pub struct AiSwapExecuted {
    pub automation: Pubkey,
    pub owner: Pubkey,
    pub token_mint_a: Pubkey,
    pub token_mint_b: Pubkey,
    pub amount_in: u64,
    pub amount_out: u64,
    pub ai_confidence: u8,
    pub execution_price: u64,
    pub execution_fee: u64,
    pub execution_type: ExecutionType,
    pub timestamp: i64,
}

#[event]
pub struct ManualSwapExecuted {
    pub automation: Pubkey,
    pub owner: Pubkey,
    pub token_mint_a: Pubkey,
    pub token_mint_b: Pubkey,
    pub amount_in: u64,
    pub minimum_amount_out: u64,
    pub max_slippage: u16,
    pub execution_fee: u64,
    pub timestamp: i64,
}

#[event]
pub struct BatchSwapExecuted {
    pub automation: Pubkey,
    pub owner: Pubkey,
    pub swap_count: u32,
    pub total_volume: u64,
    pub total_fees: u64,
    pub timestamp: i64,
}

#[event]
pub struct SwapsEmergencyStopped {
    pub automation: Pubkey,
    pub emergency_authority: Pubkey,
    pub timestamp: i64,
}

#[event]
pub struct ArbitrageOpportunityCreated {
    pub opportunity_id: u64,
    pub discoverer: Pubkey,
    pub automation: Pubkey,
    pub token_mint_a: Pubkey,
    pub token_mint_b: Pubkey,
    pub dex_a: Pubkey,
    pub dex_b: Pubkey,
    pub expected_profit_bps: u64,
    pub max_amount: u64,
    pub expires_at: i64,
    pub timestamp: i64,
}

#[event]
pub struct ArbitrageExecuted {
    pub token_automation: Pubkey,
    pub token_mint_a: Pubkey,
    pub token_mint_b: Pubkey,
    pub amount_in: u64,
    pub profit: u64,
    pub executor: Pubkey,
    pub dex_a: Pubkey,
    pub dex_b: Pubkey,
    pub timestamp: i64,
}

#[event]
pub struct ArbitrageOpportunityCancelled {
    pub opportunity_id: u64,
    pub discoverer: Pubkey,
    pub timestamp: i64,
}

#[event]
pub struct ArbitrageScanCompleted {
    pub automation: Pubkey,
    pub scanner: Pubkey,
    pub pairs_scanned: u32,
    pub opportunities_found: u32,
    pub timestamp: i64,
}

/// Order cancellation reasons
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum CancellationReason {
    UserCancelled,
    Expired,
    InsufficientBalance,
    PriceMoved,
    SystemError,
    EmergencyPause,
}

/// Price alert types
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum PriceAlertType {
    PriceAbove,
    PriceBelow,
    VolatilityHigh,
    VolumeSpike,
    LiquidityLow,
}
