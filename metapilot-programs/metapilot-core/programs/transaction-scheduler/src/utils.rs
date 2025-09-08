use crate::constants::*;
use crate::errors::TransactionSchedulerError;
use crate::state::{
    ExecutionConditions, ExecutionResult, GasMarketInfo, GasTrend, OptimizationMode,
    ScheduledTransaction, TransactionScheduler, TransactionType,
};
/// Utility functions for Transaction Scheduler Program
use anchor_lang::prelude::*;

pub struct GasOptimizer;

impl GasOptimizer {
    /// Determine if current gas price is optimal for execution
    pub fn is_optimal_gas_price(
        current_gas_price: u64,
        max_gas_price: u64,
        optimization_mode: &OptimizationMode,
    ) -> bool {
        let threshold_multiplier = optimization_mode.gas_threshold_multiplier();
        let threshold = (max_gas_price * threshold_multiplier as u64) / 100;

        current_gas_price <= threshold
    }

    /// Calculate priority gas price for immediate execution
    pub fn calculate_priority_gas_price(base_gas_price: u64) -> u64 {
        (base_gas_price * PRIORITY_GAS_MULTIPLIER as u64) / 100
    }

    /// Estimate gas savings from delaying execution
    pub fn estimate_gas_savings(
        current_gas_price: u64,
        target_gas_price: u64,
        estimated_gas_usage: u32,
    ) -> u64 {
        if current_gas_price <= target_gas_price {
            return 0;
        }

        let gas_price_diff = current_gas_price - target_gas_price;
        gas_price_diff * estimated_gas_usage as u64
    }

    /// Analyze gas market trends
    pub fn analyze_gas_trend(recent_prices: &[u64]) -> GasTrend {
        if recent_prices.len() < 2 {
            return GasTrend::Stable;
        }

        let first_half_avg = recent_prices[..recent_prices.len() / 2].iter().sum::<u64>()
            / (recent_prices.len() / 2) as u64;
        let second_half_avg = recent_prices[recent_prices.len() / 2..].iter().sum::<u64>()
            / (recent_prices.len() - recent_prices.len() / 2) as u64;

        let change_threshold = first_half_avg / 20; // 5% threshold

        if second_half_avg > first_half_avg + change_threshold {
            GasTrend::Increasing
        } else if second_half_avg < first_half_avg - change_threshold {
            GasTrend::Decreasing
        } else {
            GasTrend::Stable
        }
    }

    /// Get recommended gas price based on market conditions
    pub fn get_recommended_gas_price(
        market_info: &GasMarketInfo,
        transaction_type: &TransactionType,
        urgency: u8, // 1-10 scale
    ) -> u64 {
        let base_price = market_info.current_gas_price;

        // Adjust based on transaction type
        let type_multiplier = match transaction_type {
            TransactionType::TokenSwap => 120,  // 20% higher for swaps
            TransactionType::YieldClaim => 110, // 10% higher for yields
            TransactionType::DaoVote => 90,     // 10% lower for votes
            _ => 100,
        };

        // Adjust based on urgency
        let urgency_multiplier = 100 + (urgency as u16 - 5) * 5; // ±25% based on urgency

        // Adjust based on trend
        let trend_multiplier = match market_info.trend {
            GasTrend::Increasing => 105, // 5% higher if increasing
            GasTrend::Decreasing => 95,  // 5% lower if decreasing
            GasTrend::Stable => 100,
        };

        let adjusted_price = (base_price
            * type_multiplier as u64
            * urgency_multiplier as u64
            * trend_multiplier as u64)
            / (100 * 100 * 100);

        adjusted_price.max(MIN_GAS_PRICE).min(MAX_GAS_PRICE)
    }
}

pub struct TransactionValidator;

impl TransactionValidator {
    /// Validate transaction execution conditions
    pub fn validate_execution_conditions(
        conditions: &ExecutionConditions,
        current_time: i64,
    ) -> Result<()> {
        if !conditions.validate(current_time) {
            return Err(TransactionSchedulerError::InvalidExecutionTime.into());
        }

        // Check minimum delay
        if conditions.earliest_execution_time < current_time + MIN_EXECUTION_DELAY {
            return Err(TransactionSchedulerError::MinimumDelayNotMet.into());
        }

        // Check maximum delay
        if conditions.earliest_execution_time > current_time + MAX_EXECUTION_DELAY {
            return Err(TransactionSchedulerError::MaximumDelayExceeded.into());
        }

        // Validate gas price range
        if conditions.max_gas_price < MIN_GAS_PRICE {
            return Err(TransactionSchedulerError::GasPriceTooLow.into());
        }

        if conditions.max_gas_price > MAX_GAS_PRICE {
            return Err(TransactionSchedulerError::GasPriceTooHigh.into());
        }

        Ok(())
    }

    /// Check if transaction can be executed
    pub fn can_execute_transaction(
        transaction: &ScheduledTransaction,
        current_time: i64,
        current_gas_price: u64,
    ) -> Result<bool> {
        // Check if expired
        if transaction.is_expired(current_time) {
            return Ok(false);
        }

        // Check status
        if transaction.status.is_final() {
            return Err(TransactionSchedulerError::TransactionAlreadyExecuted.into());
        }

        // Check timing
        if current_time < transaction.execution_conditions.earliest_execution_time {
            return Ok(false);
        }

        // Check gas price
        if current_gas_price > transaction.execution_conditions.max_gas_price {
            return Ok(false);
        }

        // Check retry limit
        if transaction.attempts >= transaction.execution_conditions.retry_attempts {
            return Err(TransactionSchedulerError::RetryLimitExceeded.into());
        }

        Ok(true)
    }

    /// Validate transaction data size and structure
    pub fn validate_transaction_data(
        transaction_data: &crate::state::ScheduledTransactionData,
    ) -> Result<()> {
        if !transaction_data.validate() {
            return Err(TransactionSchedulerError::InvalidTransactionData.into());
        }

        if transaction_data.instruction_data.len() > MAX_INSTRUCTION_DATA_SIZE {
            return Err(TransactionSchedulerError::InstructionDataTooLarge.into());
        }

        if transaction_data.accounts.len() > MAX_ACCOUNTS_PER_TX {
            return Err(TransactionSchedulerError::TooManyAccounts.into());
        }

        Ok(())
    }
}

pub struct BatchOptimizer;

impl BatchOptimizer {
    /// Find optimal transactions for batch execution
    pub fn find_batchable_transactions(
        scheduler: &TransactionScheduler,
        current_time: i64,
        current_gas_price: u64,
    ) -> Vec<u64> {
        let mut batchable = Vec::new();

        for transaction in &scheduler.pending_transactions {
            if transaction.can_execute(current_time, current_gas_price) {
                batchable.push(transaction.id);

                if batchable.len() >= MAX_BATCH_SIZE {
                    break;
                }
            }
        }

        batchable
    }

    /// Calculate batch execution efficiency
    pub fn calculate_batch_efficiency(
        transactions: &[ScheduledTransaction],
        gas_price: u64,
    ) -> u16 {
        if transactions.is_empty() {
            return 0;
        }

        let total_individual_cost: u64 = transactions
            .iter()
            .map(|tx| tx.transaction_data.estimated_compute_units() as u64 * gas_price)
            .sum();

        let batch_overhead = 1000; // Fixed batch overhead in compute units
        let batch_cost = (transactions
            .iter()
            .map(|tx| tx.transaction_data.estimated_compute_units())
            .sum::<u32>()
            + batch_overhead) as u64
            * gas_price;

        if total_individual_cost == 0 {
            return 0;
        }

        let savings = total_individual_cost.saturating_sub(batch_cost);
        ((savings * 10000) / total_individual_cost) as u16
    }

    /// Determine optimal batch size for current conditions
    pub fn optimal_batch_size(
        available_transactions: &[ScheduledTransaction],
        gas_price: u64,
        efficiency_threshold: u16,
    ) -> usize {
        let mut best_size = 1;
        let mut best_efficiency = 0;

        for size in 2..=available_transactions.len().min(MAX_BATCH_SIZE) {
            let batch = &available_transactions[..size];
            let efficiency = Self::calculate_batch_efficiency(batch, gas_price);

            if efficiency > best_efficiency && efficiency >= efficiency_threshold {
                best_efficiency = efficiency;
                best_size = size;
            }
        }

        best_size
    }
}

pub struct ExecutionEngine;

impl ExecutionEngine {
    /// Execute a single transaction
    pub fn execute_transaction(
        transaction: &ScheduledTransaction,
        remaining_accounts: &[AccountInfo],
    ) -> Result<ExecutionResult> {
        let start_time = Clock::get()?.unix_timestamp;

        match transaction.transaction_data.transaction_type {
            TransactionType::TokenTransfer => {
                Self::execute_token_transfer(remaining_accounts, &transaction.transaction_data)
            }
            TransactionType::DaoVote => Self::execute_dao_vote(&transaction.transaction_data),
            TransactionType::YieldClaim => {
                Self::execute_yield_claim(remaining_accounts, &transaction.transaction_data)
            }
            TransactionType::TokenSwap => {
                Self::execute_token_swap(remaining_accounts, &transaction.transaction_data)
            }
            TransactionType::ContractCall => {
                Self::execute_contract_call(remaining_accounts, &transaction.transaction_data)
            }
            TransactionType::NFTMint => {
                Self::execute_nft_mint(remaining_accounts, &transaction.transaction_data)
            }
        }
        .map(|gas_used| ExecutionResult {
            success: true,
            gas_used,
            error_message: None,
            execution_time: Clock::get().unwrap().unix_timestamp - start_time,
        })
        .or_else(|error| {
            Ok(ExecutionResult {
                success: false,
                gas_used: 0,
                error_message: Some(format!("{:?}", error)),
                execution_time: Clock::get().unwrap().unix_timestamp - start_time,
            })
        })
    }

    fn execute_token_transfer(
        _accounts: &[AccountInfo],
        _tx_data: &crate::state::ScheduledTransactionData,
    ) -> Result<u64> {
        // Mock implementation - would execute actual token transfer
        msg!("Executing token transfer");
        Ok(5_000) // Mock gas usage
    }

    fn execute_dao_vote(_tx_data: &crate::state::ScheduledTransactionData) -> Result<u64> {
        // Mock implementation - would execute DAO vote
        msg!("Executing DAO vote");
        Ok(15_000) // Mock gas usage
    }

    fn execute_yield_claim(
        _accounts: &[AccountInfo],
        _tx_data: &crate::state::ScheduledTransactionData,
    ) -> Result<u64> {
        // Mock implementation - would execute yield claim
        msg!("Executing yield claim");
        Ok(25_000) // Mock gas usage
    }

    fn execute_token_swap(
        _accounts: &[AccountInfo],
        _tx_data: &crate::state::ScheduledTransactionData,
    ) -> Result<u64> {
        // Mock implementation - would execute token swap
        msg!("Executing token swap");
        Ok(35_000) // Mock gas usage
    }

    fn execute_contract_call(
        _accounts: &[AccountInfo],
        _tx_data: &crate::state::ScheduledTransactionData,
    ) -> Result<u64> {
        // Mock implementation - would execute contract call
        msg!("Executing contract call");
        Ok(20_000) // Mock gas usage
    }

    fn execute_nft_mint(
        _accounts: &[AccountInfo],
        _tx_data: &crate::state::ScheduledTransactionData,
    ) -> Result<u64> {
        // Mock implementation - would execute NFT mint
        msg!("Executing NFT mint");
        Ok(30_000) // Mock gas usage
    }
}

pub struct FeeCalculator;

impl FeeCalculator {
    /// Calculate scheduler fee for a transaction
    pub fn calculate_scheduler_fee(estimated_gas: u64, gas_price: u64) -> u64 {
        let transaction_cost = estimated_gas * gas_price;
        (transaction_cost * SCHEDULER_FEE_BASIS_POINTS as u64) / 10000
    }

    /// Calculate batch discount
    pub fn calculate_batch_discount(batch_size: usize, total_fee: u64) -> u64 {
        if batch_size < 2 {
            return 0;
        }

        let discount_rate = (batch_size as u16).min(10) * BATCH_DISCOUNT_BASIS_POINTS / 10;
        (total_fee * discount_rate as u64) / 10000
    }

    /// Calculate priority fee
    pub fn calculate_priority_fee(base_fee: u64) -> u64 {
        (base_fee * PRIORITY_FEE_BASIS_POINTS as u64) / 10000
    }

    /// Calculate total transaction cost including all fees
    pub fn calculate_total_cost(
        estimated_gas: u64,
        gas_price: u64,
        is_priority: bool,
        batch_size: usize,
    ) -> u64 {
        let base_cost = estimated_gas * gas_price;
        let scheduler_fee = Self::calculate_scheduler_fee(estimated_gas, gas_price);

        let mut total_fee = scheduler_fee;

        if is_priority {
            total_fee += Self::calculate_priority_fee(scheduler_fee);
        }

        let discount = Self::calculate_batch_discount(batch_size, total_fee);

        base_cost + total_fee - discount
    }
}

pub struct SchedulerAnalytics;

impl SchedulerAnalytics {
    /// Calculate average execution delay
    pub fn calculate_average_delay(_scheduler: &TransactionScheduler) -> i64 {
        // This would typically be calculated from historical data
        // For now, return a mock value
        1800 // 30 minutes average
    }

    /// Calculate success rate
    pub fn calculate_success_rate(scheduler: &TransactionScheduler) -> u16 {
        scheduler.get_success_rate()
    }

    /// Generate scheduler performance metrics
    pub fn generate_performance_report(scheduler: &TransactionScheduler) -> String {
        format!(
            "Scheduler Performance: {} total scheduled, {} executed, {}% success rate, {} gas saved",
            scheduler.total_transactions_scheduled,
            scheduler.total_transactions_executed,
            scheduler.get_success_rate() as f64 / 100.0,
            scheduler.total_gas_saved
        )
    }
}

/// Helper function to check if scheduler is healthy
pub fn is_scheduler_healthy(scheduler: &TransactionScheduler) -> bool {
    scheduler.is_operational() && scheduler.get_success_rate() >= MIN_SUCCESS_RATE as u16 * 100
}

/// Helper function to format gas price for display
pub fn format_gas_price(gas_price: u64) -> String {
    if gas_price >= 1_000_000 {
        format!("{:.2}M lamports", gas_price as f64 / 1_000_000.0)
    } else if gas_price >= 1_000 {
        format!("{:.2}K lamports", gas_price as f64 / 1_000.0)
    } else {
        format!("{} lamports", gas_price)
    }
}

/// Helper function to estimate transaction priority based on conditions
pub fn calculate_transaction_priority(transaction: &ScheduledTransaction, current_time: i64) -> u8 {
    let time_until_execution =
        transaction.execution_conditions.earliest_execution_time - current_time;
    let time_until_expiry = transaction
        .execution_conditions
        .expiry_time
        .map(|expiry| expiry - current_time)
        .unwrap_or(i64::MAX);

    // Priority calculation based on urgency and type
    let time_urgency = if time_until_execution <= 300 {
        10
    }
    // 5 minutes
    else if time_until_execution <= 1800 {
        8
    }
    // 30 minutes
    else if time_until_execution <= 3600 {
        6
    }
    // 1 hour
    else {
        4
    };

    let expiry_urgency = if time_until_expiry <= 1800 {
        3
    }
    // 30 minutes to expiry
    else if time_until_expiry <= 3600 {
        2
    }
    // 1 hour to expiry
    else {
        1
    };

    let type_priority = if transaction
        .transaction_data
        .transaction_type
        .requires_priority()
    {
        2
    } else {
        1
    };

    (time_urgency + expiry_urgency + type_priority).min(10)
}
