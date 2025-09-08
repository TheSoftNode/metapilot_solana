use crate::state::AllocationTarget;
/// Event definitions for Yield Optimizer Program
use anchor_lang::prelude::*;

#[event]
pub struct YieldStrategyInitialized {
    pub authority: Pubkey,
    pub strategy_address: Pubkey,
    pub min_yield_threshold: u16,
    pub rebalance_frequency: i64,
    pub risk_level: u8,
    pub timestamp: i64,
}

#[event]
pub struct DepositEvent {
    pub authority: Pubkey,
    pub strategy_address: Pubkey,
    pub amount: u64,
    pub new_total: u64,
    pub timestamp: i64,
}

#[event]
pub struct DepositMade {
    pub authority: Pubkey,
    pub strategy_address: Pubkey,
    pub amount: u64,
    pub new_total: u64,
    pub timestamp: i64,
}

#[event]
pub struct WithdrawEvent {
    pub authority: Pubkey,
    pub strategy_address: Pubkey,
    pub amount: u64,
    pub penalty: u64,
    pub remaining_balance: u64,
    pub withdrawal_type: String,
    pub timestamp: i64,
}

#[event]
pub struct RebalanceEvent {
    pub authority: Pubkey,
    pub strategy_address: Pubkey,
    pub allocations: Vec<AllocationTarget>,
    pub ai_confidence: u8,
    pub rebalance_number: u64,
    pub total_value_locked: u64,
    pub estimated_apy_improvement: u16,
    pub timestamp: i64,
}

#[event]
pub struct CompoundEvent {
    pub authority: Pubkey,
    pub strategy_address: Pubkey,
    pub rewards_amount: u64,
    pub new_total_rewards: u64,
    pub compound_count: u64,
    pub next_compound_at: i64,
    pub timestamp: i64,
}

#[event]
pub struct EmergencyWithdrawEvent {
    pub authority: Pubkey,
    pub strategy_address: Pubkey,
    pub amount: u64,
    pub penalty: u64,
    pub reason: String,
    pub timestamp: i64,
}

#[event]
pub struct YieldFarmingExecuted {
    pub authority: Pubkey,
    pub strategy_address: Pubkey,
    pub protocol: String,
    pub apy: u16,
    pub amount_allocated: u64,
    pub ai_confidence: u8,
    pub estimated_rewards: u64,
    pub timestamp: i64,
}

#[event]
pub struct RewardsCompounded {
    pub authority: Pubkey,
    pub strategy_address: Pubkey,
    pub rewards_amount: u64,
    pub new_total_rewards: u64,
    pub compound_count: u64,
    pub next_compound_at: i64,
    pub timestamp: i64,
}

#[event]
pub struct PortfolioRebalanced {
    pub authority: Pubkey,
    pub strategy_address: Pubkey,
    pub allocations: Vec<AllocationTarget>,
    pub ai_confidence: u8,
    pub rebalance_number: u64,
    pub total_value_locked: u64,
    pub estimated_apy_improvement: u16,
    pub timestamp: i64,
}

#[event]
pub struct WithdrawalMade {
    pub authority: Pubkey,
    pub strategy_address: Pubkey,
    pub amount: u64,
    pub penalty: u64,
    pub remaining_balance: u64,
    pub withdrawal_type: String,
    pub timestamp: i64,
}

#[event]
pub struct StrategyStatusChanged {
    pub authority: Pubkey,
    pub strategy_address: Pubkey,
    pub is_active: bool,
    pub reason: String,
    pub timestamp: i64,
}

#[event]
pub struct ProtocolAdded {
    pub authority: Pubkey,
    pub strategy_address: Pubkey,
    pub protocol_id: u8,
    pub protocol_name: String,
    pub initial_allocation: u16,
    pub timestamp: i64,
}

#[event]
pub struct ProtocolRemoved {
    pub authority: Pubkey,
    pub strategy_address: Pubkey,
    pub protocol_id: u8,
    pub protocol_name: String,
    pub final_allocation: u16,
    pub timestamp: i64,
}

#[event]
pub struct RiskLevelUpdated {
    pub authority: Pubkey,
    pub strategy_address: Pubkey,
    pub old_risk_level: u8,
    pub new_risk_level: u8,
    pub timestamp: i64,
}

#[event]
pub struct ThresholdUpdated {
    pub authority: Pubkey,
    pub strategy_address: Pubkey,
    pub old_threshold: u16,
    pub new_threshold: u16,
    pub timestamp: i64,
}

#[event]
pub struct EmergencyPause {
    pub authority: Pubkey,
    pub strategy_address: Pubkey,
    pub reason: String,
    pub total_value_locked: u64,
    pub timestamp: i64,
}

#[event]
pub struct PerformanceReport {
    pub authority: Pubkey,
    pub strategy_address: Pubkey,
    pub period_start: i64,
    pub period_end: i64,
    pub total_deposits: u64,
    pub total_withdrawals: u64,
    pub total_rewards: u64,
    pub average_apy: u16,
    pub rebalance_count: u64,
    pub compound_count: u64,
    pub performance_vs_benchmark: i16, // Can be negative
}
