use anchor_lang::prelude::*;
use crate::state::{TransactionScheduler, CancelTransactionParams, TransactionStatus};
use crate::errors::TransactionSchedulerError;
use crate::constants::*;
use crate::utils::FeeCalculator;
use crate::events::TransactionCancelled;

#[derive(Accounts)]
pub struct CancelTransaction<'info> {
    #[account(
        mut,
        seeds = [TRANSACTION_SCHEDULER_SEED, transaction_scheduler.authority.as_ref()],
        bump = transaction_scheduler.bump
    )]
    pub transaction_scheduler: Account<'info, TransactionScheduler>,
    
    #[account(mut)]
    pub authority: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

pub fn handler(
    ctx: Context<CancelTransaction>,
    params: CancelTransactionParams,
) -> Result<()> {
    let scheduler = &mut ctx.accounts.transaction_scheduler;
    let authority = &ctx.accounts.authority;
    let current_time = Clock::get()?.unix_timestamp;

    // Find the transaction
    let tx_index = scheduler
        .find_transaction_index(params.transaction_id)
        .ok_or(TransactionSchedulerError::TransactionNotFound)?;

    let mut transaction = scheduler.pending_transactions[tx_index].clone();

    // Verify authority - either transaction owner or scheduler owner
    if transaction.authority != authority.key() && scheduler.authority != authority.key() {
        return Err(TransactionSchedulerError::UnauthorizedAccess.into());
    }

    // Check if transaction can be cancelled
    if transaction.status.is_final() {
        return Err(TransactionSchedulerError::TransactionAlreadyExecuted.into());
    }

    // Mark transaction as cancelled
    transaction.mark_cancelled();

    // Calculate refund amount
    let estimated_gas = transaction.transaction_data.estimated_compute_units() as u64;
    let scheduled_fee = FeeCalculator::calculate_scheduler_fee(
        estimated_gas,
        transaction.execution_conditions.max_gas_price,
    );

    // Calculate refund based on how early the cancellation is
    let time_until_execution = transaction.execution_conditions.earliest_execution_time - current_time;
    let refund_percentage = if time_until_execution > 3600 {
        90 // 90% refund if cancelled more than 1 hour before execution
    } else if time_until_execution > 1800 {
        75 // 75% refund if cancelled more than 30 minutes before
    } else if time_until_execution > 300 {
        50 // 50% refund if cancelled more than 5 minutes before
    } else {
        25 // 25% refund if cancelled less than 5 minutes before
    };

    let refund_amount = (scheduled_fee * refund_percentage) / 100;

    // Remove transaction from pending list
    scheduler.remove_transaction(tx_index);

    // Update scheduler fees (subtract the refund)
    scheduler.total_fees_collected = scheduler.total_fees_collected
        .saturating_sub(refund_amount);

    // Emit cancellation event
    emit!(TransactionCancelled {
        transaction_id: params.transaction_id,
        authority: transaction.authority,
        reason: params.reason.clone(),
        refund_amount,
        timestamp: current_time,
    });

    msg!(
        "Transaction {} cancelled by {}. Reason: '{}'. Refund: {} ({}%)",
        params.transaction_id,
        authority.key(),
        params.reason,
        refund_amount,
        refund_percentage
    );

    Ok(())
}

#[derive(Accounts)]
pub struct CancelExpiredTransactions<'info> {
    #[account(
        mut,
        seeds = [TRANSACTION_SCHEDULER_SEED, transaction_scheduler.authority.as_ref()],
        bump = transaction_scheduler.bump
    )]
    pub transaction_scheduler: Account<'info, TransactionScheduler>,
    
    #[account(mut)]
    pub caller: Signer<'info>,
}

pub fn cleanup_expired_handler(ctx: Context<CancelExpiredTransactions>) -> Result<()> {
    let scheduler = &mut ctx.accounts.transaction_scheduler;
    let current_time = Clock::get()?.unix_timestamp;

    let mut expired_count = 0u16;
    let mut total_refund = 0u64;

    // Find expired transactions (iterate in reverse to safely remove)
    let mut i = scheduler.pending_transactions.len();
    while i > 0 {
        i -= 1;
        
        let transaction = &scheduler.pending_transactions[i];
        
        if transaction.is_expired(current_time) {
            let estimated_gas = transaction.transaction_data.estimated_compute_units() as u64;
            let scheduled_fee = FeeCalculator::calculate_scheduler_fee(
                estimated_gas,
                transaction.execution_conditions.max_gas_price,
            );
            
            // Full refund for expired transactions
            total_refund += scheduled_fee;
            expired_count += 1;

            // Emit expiry event for this transaction
            emit!(crate::events::TransactionExpired {
                transaction_id: transaction.id,
                authority: transaction.authority,
                expiry_time: transaction.execution_conditions.expiry_time.unwrap_or(0),
                refund_amount: scheduled_fee,
                timestamp: current_time,
            });

            // Remove the expired transaction
            scheduler.remove_transaction(i);
        }
    }

    // Update scheduler state
    if total_refund > 0 {
        scheduler.total_fees_collected = scheduler.total_fees_collected
            .saturating_sub(total_refund);
    }

    msg!(
        "Cleanup completed: {} expired transactions removed, {} total refund",
        expired_count,
        total_refund
    );

    Ok(())
}
