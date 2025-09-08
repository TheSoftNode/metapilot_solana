/// State and account structures for Transaction Scheduler Program
use anchor_lang::prelude::*;
use crate::constants::*;

#[account]
pub struct TransactionScheduler {
    pub authority: Pubkey,                              // 32 bytes
    pub max_gas_price: u64,                            // 8 bytes
    pub optimization_mode: OptimizationMode,           // 1 byte
    pub is_active: bool,                               // 1 byte
    pub created_at: i64,                               // 8 bytes
    pub total_transactions_scheduled: u64,             // 8 bytes
    pub total_transactions_executed: u64,              // 8 bytes
    pub total_gas_saved: u64,                          // 8 bytes
    pub total_fees_collected: u64,                     // 8 bytes
    pub pending_transactions: Vec<ScheduledTransaction>, // Variable size
    pub bump: u8,                                      // 1 byte
}

impl TransactionScheduler {
    pub const SIZE: usize = TRANSACTION_SCHEDULER_SIZE;

    pub fn is_operational(&self) -> bool {
        self.is_active
    }

    pub fn can_schedule_transaction(&self) -> bool {
        self.is_active && self.pending_transactions.len() < MAX_PENDING_TRANSACTIONS
    }

    pub fn get_success_rate(&self) -> u16 {
        if self.total_transactions_scheduled == 0 {
            return 0;
        }
        ((self.total_transactions_executed * 10000) / self.total_transactions_scheduled) as u16
    }

    pub fn get_pending_count(&self) -> usize {
        self.pending_transactions.len()
    }

    pub fn find_transaction_index(&self, transaction_id: u64) -> Option<usize> {
        self.pending_transactions
            .iter()
            .position(|tx| tx.id == transaction_id)
    }

    pub fn remove_transaction(&mut self, index: usize) -> ScheduledTransaction {
        self.pending_transactions.remove(index)
    }

    pub fn add_transaction(&mut self, transaction: ScheduledTransaction) {
        self.pending_transactions.push(transaction);
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct ScheduledTransaction {
    pub id: u64,                                      // 8 bytes
    pub authority: Pubkey,                            // 32 bytes
    pub transaction_data: ScheduledTransactionData,   // Variable size
    pub execution_conditions: ExecutionConditions,    // ~40 bytes
    pub created_at: i64,                             // 8 bytes
    pub status: TransactionStatus,                    // 1 byte
    pub attempts: u8,                                // 1 byte
}

impl ScheduledTransaction {
    pub fn is_expired(&self, current_time: i64) -> bool {
        if let Some(expiry) = self.execution_conditions.expiry_time {
            current_time > expiry
        } else {
            false
        }
    }

    pub fn can_execute(&self, current_time: i64, current_gas_price: u64) -> bool {
        !self.is_expired(current_time) &&
        current_time >= self.execution_conditions.earliest_execution_time &&
        current_gas_price <= self.execution_conditions.max_gas_price &&
        self.attempts < self.execution_conditions.retry_attempts &&
        self.status == TransactionStatus::Pending
    }

    pub fn should_retry(&self) -> bool {
        self.status == TransactionStatus::Failed && 
        self.attempts < self.execution_conditions.retry_attempts
    }

    pub fn mark_executed(&mut self) {
        self.status = TransactionStatus::Executed;
    }

    pub fn mark_failed(&mut self) {
        self.status = TransactionStatus::Failed;
        self.attempts += 1;
    }

    pub fn mark_expired(&mut self) {
        self.status = TransactionStatus::Expired;
    }

    pub fn mark_cancelled(&mut self) {
        self.status = TransactionStatus::Cancelled;
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct ScheduledTransactionData {
    pub transaction_type: TransactionType,   // 1 byte
    pub target_program: Pubkey,             // 32 bytes
    pub instruction_data: Vec<u8>,          // Variable size (max 1000 bytes)
    pub accounts: Vec<Pubkey>,              // Variable size (max 20 accounts)
}

impl ScheduledTransactionData {
    pub fn validate(&self) -> bool {
        self.instruction_data.len() <= MAX_INSTRUCTION_DATA_SIZE &&
        self.accounts.len() <= MAX_ACCOUNTS_PER_TX
    }

    pub fn estimated_compute_units(&self) -> u32 {
        // Estimate based on transaction type and complexity
        let base_units = match self.transaction_type {
            TransactionType::TokenTransfer => 5_000,
            TransactionType::DaoVote => 15_000,
            TransactionType::YieldClaim => 25_000,
            TransactionType::TokenSwap => 35_000,
            TransactionType::ContractCall => 20_000,
            TransactionType::NFTMint => 30_000,
        };

        // Add complexity based on accounts and instruction data
        let account_complexity = (self.accounts.len() as u32) * 1_000;
        let data_complexity = (self.instruction_data.len() as u32) / 10;

        base_units + account_complexity + data_complexity
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct ExecutionConditions {
    pub earliest_execution_time: i64,    // 8 bytes
    pub max_gas_price: u64,              // 8 bytes
    pub expiry_time: Option<i64>,        // 9 bytes (1 + 8)
    pub retry_attempts: u8,              // 1 byte
    pub priority_fee: u64,               // 8 bytes
    pub slippage_tolerance: u16,         // 2 bytes (basis points)
}

impl ExecutionConditions {
    pub fn validate(&self, current_time: i64) -> bool {
        // Earliest execution time must be in the future (with some tolerance)
        self.earliest_execution_time >= current_time - 60 &&
        // Max gas price must be reasonable
        self.max_gas_price >= MIN_GAS_PRICE &&
        self.max_gas_price <= MAX_GAS_PRICE &&
        // Retry attempts must be within limits
        self.retry_attempts <= MAX_RETRY_ATTEMPTS &&
        // If expiry is set, it must be after earliest execution
        self.expiry_time.map_or(true, |expiry| expiry > self.earliest_execution_time) &&
        // Slippage tolerance must be reasonable (max 50%)
        self.slippage_tolerance <= 5000
    }

    pub fn is_priority(&self) -> bool {
        self.priority_fee > 0
    }

    pub fn total_max_cost(&self) -> u64 {
        self.max_gas_price + self.priority_fee
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq, Eq)]
pub enum OptimizationMode {
    GasOptimized,  // Prioritize lowest gas prices
    TimeOptimized, // Execute as soon as conditions are met
    Balanced,      // Balance between gas and time
}

impl OptimizationMode {
    pub fn gas_threshold_multiplier(&self) -> u16 {
        match self {
            OptimizationMode::GasOptimized => 50,  // Very aggressive gas savings
            OptimizationMode::Balanced => 75,      // Moderate gas savings
            OptimizationMode::TimeOptimized => 95, // Minimal gas savings requirement
        }
    }

    pub fn max_delay_tolerance(&self) -> i64 {
        match self {
            OptimizationMode::GasOptimized => 3600,  // 1 hour
            OptimizationMode::Balanced => 1800,      // 30 minutes
            OptimizationMode::TimeOptimized => 300,  // 5 minutes
        }
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, Hash, PartialEq, Eq)]
pub enum TransactionType {
    TokenTransfer,
    DaoVote,
    YieldClaim,
    TokenSwap,
    ContractCall,
    NFTMint,
}

impl TransactionType {
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            TX_TYPE_TOKEN_TRANSFER => Some(TransactionType::TokenTransfer),
            TX_TYPE_DAO_VOTE => Some(TransactionType::DaoVote),
            TX_TYPE_YIELD_CLAIM => Some(TransactionType::YieldClaim),
            TX_TYPE_TOKEN_SWAP => Some(TransactionType::TokenSwap),
            TX_TYPE_CONTRACT_CALL => Some(TransactionType::ContractCall),
            TX_TYPE_NFT_MINT => Some(TransactionType::NFTMint),
            _ => None,
        }
    }

    pub fn to_u8(&self) -> u8 {
        match self {
            TransactionType::TokenTransfer => TX_TYPE_TOKEN_TRANSFER,
            TransactionType::DaoVote => TX_TYPE_DAO_VOTE,
            TransactionType::YieldClaim => TX_TYPE_YIELD_CLAIM,
            TransactionType::TokenSwap => TX_TYPE_TOKEN_SWAP,
            TransactionType::ContractCall => TX_TYPE_CONTRACT_CALL,
            TransactionType::NFTMint => TX_TYPE_NFT_MINT,
        }
    }

    pub fn requires_priority(&self) -> bool {
        matches!(self, TransactionType::TokenSwap | TransactionType::YieldClaim)
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq, Eq)]
pub enum TransactionStatus {
    Pending,
    Executed,
    Failed,
    Expired,
    Cancelled,
}

impl TransactionStatus {
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            STATUS_PENDING => Some(TransactionStatus::Pending),
            STATUS_EXECUTED => Some(TransactionStatus::Executed),
            STATUS_FAILED => Some(TransactionStatus::Failed),
            STATUS_EXPIRED => Some(TransactionStatus::Expired),
            STATUS_CANCELLED => Some(TransactionStatus::Cancelled),
            _ => None,
        }
    }

    pub fn to_u8(&self) -> u8 {
        match self {
            TransactionStatus::Pending => STATUS_PENDING,
            TransactionStatus::Executed => STATUS_EXECUTED,
            TransactionStatus::Failed => STATUS_FAILED,
            TransactionStatus::Expired => STATUS_EXPIRED,
            TransactionStatus::Cancelled => STATUS_CANCELLED,
        }
    }

    pub fn is_final(&self) -> bool {
        matches!(
            self,
            TransactionStatus::Executed
                | TransactionStatus::Expired
                | TransactionStatus::Cancelled
        )
    }
}

/// Parameters for scheduling a new transaction
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct ScheduleTransactionParams {
    pub transaction_data: ScheduledTransactionData,
    pub execution_conditions: ExecutionConditions,
    pub pay_priority_fee: bool,
}

/// Parameters for executing scheduled transaction
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct ExecuteTransactionParams {
    pub transaction_id: u64,
    pub current_gas_price: u64,
    pub force_execution: bool,
}

/// Parameters for batch execution
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct BatchExecuteParams {
    pub transaction_ids: Vec<u64>,
    pub current_gas_price: u64,
    pub max_failures: u8,
}

/// Parameters for cancelling a transaction
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct CancelTransactionParams {
    pub transaction_id: u64,
    pub reason: String,
}

/// Gas market information
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct GasMarketInfo {
    pub current_gas_price: u64,
    pub congestion_level: u8,
    pub recommended_gas_price: u64,
    pub trend: GasTrend,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub enum GasTrend {
    Increasing,
    Stable,
    Decreasing,
}

/// Execution result information
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct ExecutionResult {
    pub success: bool,
    pub gas_used: u64,
    pub error_message: Option<String>,
    pub execution_time: i64,
}

/// Batch execution summary
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct BatchExecutionSummary {
    pub total_transactions: u8,
    pub successful_executions: u8,
    pub failed_executions: u8,
    pub total_gas_used: u64,
    pub total_gas_saved: u64,
    pub execution_time: i64,
}
