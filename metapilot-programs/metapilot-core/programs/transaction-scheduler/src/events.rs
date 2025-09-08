/// Events for Transaction Scheduler Program
use anchor_lang::prelude::*;
use crate::state::{TransactionType, OptimizationMode, TransactionStatus, ExecutionConditions};

#[event]
pub struct SchedulerInitialized {
    pub authority: Pubkey,
    pub max_gas_price: u64,
    pub optimization_mode: OptimizationMode,
    pub timestamp: i64,
}

#[event]
pub struct TransactionScheduled {
    pub transaction_id: u64,
    pub authority: Pubkey,
    pub transaction_type: TransactionType,
    pub target_program: Pubkey,
    pub execution_conditions: ExecutionConditions,
    pub estimated_gas: u64,
    pub scheduled_fee: u64,
    pub earliest_execution: i64,
    pub expiry_time: Option<i64>,
    pub timestamp: i64,
}

#[event]
pub struct TransactionExecuted {
    pub transaction_id: u64,
    pub authority: Pubkey,
    pub transaction_type: TransactionType,
    pub execution_gas_price: u64,
    pub actual_gas_used: u64,
    pub gas_saved: u64,
    pub execution_fee: u64,
    pub execution_delay: i64, // How long it was delayed
    pub attempts: u8,
    pub timestamp: i64,
}

#[event]
pub struct BatchTransactionsExecuted {
    pub batch_id: u64,
    pub authority: Pubkey,
    pub transaction_ids: Vec<u64>,
    pub executed_count: u8,
    pub failed_count: u8,
    pub total_gas_used: u64,
    pub total_gas_saved: u64,
    pub batch_discount: u64,
    pub timestamp: i64,
}

#[event]
pub struct TransactionCancelled {
    pub transaction_id: u64,
    pub authority: Pubkey,
    pub reason: String,
    pub refund_amount: u64,
    pub timestamp: i64,
}

#[event]
pub struct TransactionFailed {
    pub transaction_id: u64,
    pub authority: Pubkey,
    pub failure_reason: String,
    pub gas_consumed: u64,
    pub retry_scheduled: bool,
    pub attempt_number: u8,
    pub timestamp: i64,
}

#[event]
pub struct TransactionExpired {
    pub transaction_id: u64,
    pub authority: Pubkey,
    pub expiry_time: i64,
    pub refund_amount: u64,
    pub timestamp: i64,
}

#[event]
pub struct GasOptimizationTriggered {
    pub current_gas_price: u64,
    pub threshold_gas_price: u64,
    pub transactions_affected: u16,
    pub estimated_savings: u64,
    pub timestamp: i64,
}

#[event]
pub struct PriorityExecutionRequested {
    pub transaction_id: u64,
    pub authority: Pubkey,
    pub priority_fee: u64,
    pub original_gas_price: u64,
    pub priority_gas_price: u64,
    pub timestamp: i64,
}

#[event]
pub struct SchedulerPaused {
    pub authority: Pubkey,
    pub reason: String,
    pub pending_transactions: u16,
    pub timestamp: i64,
}

#[event]
pub struct SchedulerResumed {
    pub authority: Pubkey,
    pub pending_transactions: u16,
    pub timestamp: i64,
}

#[event]
pub struct TransactionRetried {
    pub transaction_id: u64,
    pub authority: Pubkey,
    pub attempt_number: u8,
    pub new_gas_price: u64,
    pub retry_reason: String,
    pub timestamp: i64,
}

#[event]
pub struct NetworkCongestionDetected {
    pub current_gas_price: u64,
    pub congestion_level: u8, // 1-10 scale
    pub affected_transactions: u16,
    pub recommended_delay: i64,
    pub timestamp: i64,
}

#[event]
pub struct OpportunisticExecutionTriggered {
    pub gas_price_drop: u64,
    pub transactions_executed: u16,
    pub total_savings: u64,
    pub execution_window: i64,
    pub timestamp: i64,
}

#[event]
pub struct SchedulerStatsUpdated {
    pub total_transactions_scheduled: u64,
    pub total_transactions_executed: u64,
    pub total_gas_saved: u64,
    pub total_fees_collected: u64,
    pub success_rate: u16, // basis points
    pub average_delay: i64,
    pub timestamp: i64,
}

#[event]
pub struct TransactionDependencyResolved {
    pub transaction_id: u64,
    pub dependency_transaction_id: u64,
    pub authority: Pubkey,
    pub ready_for_execution: bool,
    pub timestamp: i64,
}

#[event]
pub struct BatchOptimizationCalculated {
    pub potential_batch_size: u8,
    pub estimated_gas_savings: u64,
    pub batch_efficiency_score: u16, // 0-10000 (100%)
    pub recommended_execution_time: i64,
    pub timestamp: i64,
}
