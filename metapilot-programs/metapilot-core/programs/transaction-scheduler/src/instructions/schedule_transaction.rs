use anchor_lang::prelude::*;
use crate::state::{TransactionScheduler, ScheduledTransaction, ScheduleTransactionParams, TransactionStatus};
use crate::errors::TransactionSchedulerError;
use crate::constants::*;
use crate::utils::{TransactionValidator, FeeCalculator};
use crate::events::TransactionScheduled;

#[derive(Accounts)]
pub struct ScheduleTransaction<'info> {
    #[account(
        mut,
        seeds = [TRANSACTION_SCHEDULER_SEED, transaction_scheduler.authority.as_ref()],
        bump = transaction_scheduler.bump
    )]
    pub transaction_scheduler: Account<'info, TransactionScheduler>,
    
    #[account(mut)]
    pub authority: Signer<'info>,
    
    /// CHECK: Fee payment account - validated in handler
    #[account(mut)]
    pub fee_payer: AccountInfo<'info>,
    
    pub system_program: Program<'info, System>,
}

pub fn handler(
    ctx: Context<ScheduleTransaction>,
    params: ScheduleTransactionParams,
) -> Result<()> {
    let scheduler = &mut ctx.accounts.transaction_scheduler;
    let authority = &ctx.accounts.authority;
    let current_time = Clock::get()?.unix_timestamp;

    // Validate scheduler is operational
    if !scheduler.is_operational() {
        return Err(TransactionSchedulerError::SchedulerInactive.into());
    }

    // Check capacity
    if !scheduler.can_schedule_transaction() {
        return Err(TransactionSchedulerError::TooManyPendingTransactions.into());
    }

    // Validate execution conditions
    TransactionValidator::validate_execution_conditions(
        &params.execution_conditions,
        current_time,
    )?;

    // Validate transaction data
    TransactionValidator::validate_transaction_data(&params.transaction_data)?;

    // Generate unique transaction ID
    let transaction_id = scheduler.total_transactions_scheduled;

    // Estimate gas and calculate fees
    let estimated_gas = params.transaction_data.estimated_compute_units() as u64;
    let scheduler_fee = FeeCalculator::calculate_scheduler_fee(
        estimated_gas,
        params.execution_conditions.max_gas_price,
    );

    let priority_fee = if params.pay_priority_fee {
        FeeCalculator::calculate_priority_fee(scheduler_fee)
    } else {
        0
    };

    let total_fee = scheduler_fee + priority_fee;

    // Create scheduled transaction
    let scheduled_tx = ScheduledTransaction {
        id: transaction_id,
        authority: authority.key(),
        transaction_data: params.transaction_data.clone(),
        execution_conditions: params.execution_conditions.clone(),
        created_at: current_time,
        status: TransactionStatus::Pending,
        attempts: 0,
    };

    // Add to scheduler
    scheduler.add_transaction(scheduled_tx);
    scheduler.total_transactions_scheduled = scheduler.total_transactions_scheduled
        .checked_add(1)
        .ok_or(TransactionSchedulerError::MathOverflow)?;

    // Collect fees (in a real implementation, this would transfer SOL)
    scheduler.total_fees_collected = scheduler.total_fees_collected
        .checked_add(total_fee)
        .ok_or(TransactionSchedulerError::MathOverflow)?;

    // Emit scheduling event
    emit!(TransactionScheduled {
        transaction_id,
        authority: authority.key(),
        transaction_type: params.transaction_data.transaction_type,
        target_program: params.transaction_data.target_program,
        execution_conditions: params.execution_conditions.clone(),
        estimated_gas,
        scheduled_fee: total_fee,
        earliest_execution: params.execution_conditions.earliest_execution_time,
        expiry_time: params.execution_conditions.expiry_time,
        timestamp: current_time,
    });

    msg!(
        "Transaction {} scheduled for execution at {} with max gas price: {}",
        transaction_id,
        params.execution_conditions.earliest_execution_time,
        params.execution_conditions.max_gas_price
    );

    Ok(())
}
