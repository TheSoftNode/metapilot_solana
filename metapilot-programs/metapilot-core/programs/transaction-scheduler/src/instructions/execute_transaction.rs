use anchor_lang::prelude::*;
use crate::state::{TransactionScheduler, ExecuteTransactionParams, TransactionStatus};
use crate::errors::TransactionSchedulerError;
use crate::constants::*;
use crate::utils::{TransactionValidator, ExecutionEngine, GasOptimizer};
use crate::events::{TransactionExecuted, TransactionFailed, TransactionExpired};

#[derive(Accounts)]
pub struct ExecuteTransaction<'info> {
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
    ctx: Context<ExecuteTransaction>,
    params: ExecuteTransactionParams,
) -> Result<()> {
    let scheduler = &mut ctx.accounts.transaction_scheduler;
    let _executor = &ctx.accounts.executor;
    let current_time = Clock::get()?.unix_timestamp;

    // Validate scheduler is operational
    if !scheduler.is_operational() {
        return Err(TransactionSchedulerError::SchedulerInactive.into());
    }

    // Find the transaction
    let tx_index = scheduler
        .find_transaction_index(params.transaction_id)
        .ok_or(TransactionSchedulerError::TransactionNotFound)?;

    let mut transaction = scheduler.pending_transactions[tx_index].clone();

    // Check if transaction has expired
    if transaction.is_expired(current_time) {
        transaction.mark_expired();
        scheduler.pending_transactions[tx_index] = transaction.clone();
        
        emit!(TransactionExpired {
            transaction_id: params.transaction_id,
            authority: transaction.authority,
            expiry_time: transaction.execution_conditions.expiry_time.unwrap_or(0),
            refund_amount: 0, // Calculate actual refund in production
            timestamp: current_time,
        });

        scheduler.remove_transaction(tx_index);
        return Err(TransactionSchedulerError::TransactionExpired.into());
    }

    // Validate execution conditions unless forced
    if !params.force_execution {
        let can_execute = TransactionValidator::can_execute_transaction(
            &transaction,
            current_time,
            params.current_gas_price,
        )?;

        if !can_execute {
            return Err(TransactionSchedulerError::ConditionsNotMet.into());
        }

        // Additional gas optimization check
        if !GasOptimizer::is_optimal_gas_price(
            params.current_gas_price,
            transaction.execution_conditions.max_gas_price,
            &scheduler.optimization_mode,
        ) {
            msg!(
                "Gas price {} not optimal for transaction {}, waiting for better conditions",
                params.current_gas_price,
                params.transaction_id
            );
            return Err(TransactionSchedulerError::ConditionsNotMet.into());
        }
    }

    // Execute the transaction
    let execution_result = ExecutionEngine::execute_transaction(
        &transaction,
        ctx.remaining_accounts,
    )?;

    if execution_result.success {
        // Calculate gas savings
        let gas_saved = transaction.execution_conditions.max_gas_price
            .saturating_sub(params.current_gas_price)
            .saturating_mul(execution_result.gas_used);

        // Update scheduler state
        scheduler.total_transactions_executed = scheduler.total_transactions_executed
            .checked_add(1)
            .ok_or(TransactionSchedulerError::MathOverflow)?;

        scheduler.total_gas_saved = scheduler.total_gas_saved
            .checked_add(gas_saved)
            .ok_or(TransactionSchedulerError::MathOverflow)?;

        // Remove transaction from pending
        scheduler.remove_transaction(tx_index);

        // Calculate execution delay
        let execution_delay = current_time - transaction.execution_conditions.earliest_execution_time;

        // Emit success event
        emit!(TransactionExecuted {
            transaction_id: params.transaction_id,
            authority: transaction.authority,
            transaction_type: transaction.transaction_data.transaction_type,
            execution_gas_price: params.current_gas_price,
            actual_gas_used: execution_result.gas_used,
            gas_saved,
            execution_fee: 0, // Calculate actual execution fee
            execution_delay: execution_delay.max(0),
            attempts: transaction.attempts + 1,
            timestamp: current_time,
        });

        msg!(
            "Transaction {} executed successfully. Gas used: {}, Gas saved: {}",
            params.transaction_id,
            execution_result.gas_used,
            gas_saved
        );
    } else {
        // Handle execution failure
        transaction.mark_failed();

        // Check if we should retry
        if transaction.should_retry() {
            // Update the transaction in pending list for retry
            scheduler.pending_transactions[tx_index] = transaction.clone();
            
            msg!(
                "Transaction {} failed, attempt {}/{}. Scheduled for retry.",
                params.transaction_id,
                transaction.attempts,
                transaction.execution_conditions.retry_attempts
            );
        } else {
            // Remove from pending if no more retries
            scheduler.remove_transaction(tx_index);
        }

        // Emit failure event
        emit!(TransactionFailed {
            transaction_id: params.transaction_id,
            authority: transaction.authority,
            failure_reason: execution_result.error_message.unwrap_or_else(|| "Unknown error".to_string()),
            gas_consumed: execution_result.gas_used,
            retry_scheduled: transaction.should_retry(),
            attempt_number: transaction.attempts,
            timestamp: current_time,
        });

        return Err(TransactionSchedulerError::ExecutionFailed.into());
    }

    Ok(())
}
