use anchor_lang::prelude::*;
use crate::state::{TransactionScheduler, BatchExecuteParams, BatchExecutionSummary};
use crate::errors::TransactionSchedulerError;
use crate::constants::*;
use crate::utils::{TransactionValidator, ExecutionEngine, BatchOptimizer, FeeCalculator};
use crate::events::BatchTransactionsExecuted;

#[derive(Accounts)]
pub struct BatchExecute<'info> {
    #[account(
        mut,
        seeds = [TRANSACTION_SCHEDULER_SEED, transaction_scheduler.authority.as_ref()],
        bump = transaction_scheduler.bump
    )]
    pub transaction_scheduler: Account<'info, TransactionScheduler>,
    
    #[account(mut)]
    pub executor: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

pub fn handler(
    ctx: Context<BatchExecute>,
    params: BatchExecuteParams,
) -> Result<()> {
    let scheduler = &mut ctx.accounts.transaction_scheduler;
    let _executor = &ctx.accounts.executor;
    let current_time = Clock::get()?.unix_timestamp;

    // Validate scheduler is operational
    if !scheduler.is_operational() {
        return Err(TransactionSchedulerError::SchedulerInactive.into());
    }

    // Validate batch size
    if params.transaction_ids.is_empty() {
        return Err(TransactionSchedulerError::InvalidTransactionData.into());
    }

    if params.transaction_ids.len() > MAX_BATCH_SIZE {
        return Err(TransactionSchedulerError::BatchTooLarge.into());
    }

    // Collect valid transactions for batch execution
    let mut executable_transactions = Vec::new();
    let mut invalid_transaction_ids = Vec::new();

    for &transaction_id in &params.transaction_ids {
        if let Some(tx_index) = scheduler.find_transaction_index(transaction_id) {
            let transaction = &scheduler.pending_transactions[tx_index];
            
            // Check if transaction can be executed
            match TransactionValidator::can_execute_transaction(
                transaction,
                current_time,
                params.current_gas_price,
            ) {
                Ok(true) => {
                    if !transaction.is_expired(current_time) {
                        executable_transactions.push((tx_index, transaction.clone()));
                    } else {
                        invalid_transaction_ids.push(transaction_id);
                    }
                }
                _ => {
                    invalid_transaction_ids.push(transaction_id);
                }
            }
        } else {
            invalid_transaction_ids.push(transaction_id);
        }
    }

    // Check if we have enough valid transactions
    if executable_transactions.is_empty() {
        return Err(TransactionSchedulerError::ConditionsNotMet.into());
    }

    // Calculate batch efficiency
    let transactions: Vec<_> = executable_transactions.iter().map(|(_, tx)| tx.clone()).collect();
    let batch_efficiency = BatchOptimizer::calculate_batch_efficiency(
        &transactions,
        params.current_gas_price,
    );

    // Require minimum efficiency for batch execution
    if batch_efficiency < 500 { // 5% minimum efficiency
        return Err(TransactionSchedulerError::BatchThresholdNotMet.into());
    }

    // Execute transactions in batch
    let mut successful_executions = 0u8;
    let mut failed_executions = 0u8;
    let mut total_gas_used = 0u64;
    let mut executed_transaction_ids = Vec::new();
    let mut removal_indices = Vec::new();

    // Sort indices in descending order for safe removal
    let mut sorted_executable = executable_transactions;
    sorted_executable.sort_by(|a, b| b.0.cmp(&a.0));

    for (tx_index, transaction) in sorted_executable {
        if failed_executions >= params.max_failures {
            msg!("Maximum failures reached, stopping batch execution");
            break;
        }

        // Execute the transaction
        match ExecutionEngine::execute_transaction(&transaction, ctx.remaining_accounts) {
            Ok(execution_result) => {
                if execution_result.success {
                    successful_executions += 1;
                    total_gas_used += execution_result.gas_used;
                    executed_transaction_ids.push(transaction.id);
                    removal_indices.push(tx_index);
                    
                    msg!("Batch transaction {} executed successfully", transaction.id);
                } else {
                    failed_executions += 1;
                    msg!("Batch transaction {} failed: {:?}", transaction.id, execution_result.error_message);
                }
            }
            Err(_) => {
                failed_executions += 1;
                msg!("Batch transaction {} failed during execution", transaction.id);
            }
        }
    }

    // Remove successfully executed transactions
    for &index in &removal_indices {
        scheduler.remove_transaction(index);
    }

    // Calculate total gas savings
    let individual_gas_cost: u64 = transactions.iter()
        .map(|tx| tx.transaction_data.estimated_compute_units() as u64 * params.current_gas_price)
        .sum();

    let batch_gas_cost = total_gas_used * params.current_gas_price;
    let total_gas_saved = individual_gas_cost.saturating_sub(batch_gas_cost);

    // Calculate batch discount
    let total_fee = transactions.iter()
        .map(|tx| FeeCalculator::calculate_scheduler_fee(
            tx.transaction_data.estimated_compute_units() as u64,
            params.current_gas_price,
        ))
        .sum::<u64>();

    let batch_discount = FeeCalculator::calculate_batch_discount(
        successful_executions as usize,
        total_fee,
    );

    // Update scheduler state
    scheduler.total_transactions_executed = scheduler.total_transactions_executed
        .checked_add(successful_executions as u64)
        .ok_or(TransactionSchedulerError::MathOverflow)?;

    scheduler.total_gas_saved = scheduler.total_gas_saved
        .checked_add(total_gas_saved)
        .ok_or(TransactionSchedulerError::MathOverflow)?;

    // Generate batch ID for tracking
    let batch_id = current_time as u64 + successful_executions as u64;

    // Emit batch execution event
    emit!(BatchTransactionsExecuted {
        batch_id,
        authority: scheduler.authority,
        transaction_ids: executed_transaction_ids,
        executed_count: successful_executions,
        failed_count: failed_executions,
        total_gas_used,
        total_gas_saved,
        batch_discount,
        timestamp: current_time,
    });

    msg!(
        "Batch execution completed: {}/{} successful, {} gas used, {} gas saved, {:.2}% efficiency",
        successful_executions,
        successful_executions + failed_executions,
        total_gas_used,
        total_gas_saved,
        batch_efficiency as f64 / 100.0
    );

    // Return error if no transactions were executed
    if successful_executions == 0 {
        return Err(TransactionSchedulerError::ExecutionFailed.into());
    }

    Ok(())
}
