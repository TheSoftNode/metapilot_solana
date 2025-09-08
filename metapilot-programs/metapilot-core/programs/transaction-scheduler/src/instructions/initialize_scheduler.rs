use crate::constants::*;
use crate::errors::TransactionSchedulerError;
use crate::events::SchedulerInitialized;
use crate::state::{OptimizationMode, TransactionScheduler};
use anchor_lang::prelude::*;

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct InitializeSchedulerParams {
    pub max_gas_price: u64,
    pub optimization_mode: OptimizationMode,
}

#[derive(Accounts)]
pub struct InitializeScheduler<'info> {
    #[account(
        init,
        payer = authority,
        space = TransactionScheduler::SIZE,
        seeds = [TRANSACTION_SCHEDULER_SEED, authority.key().as_ref()],
        bump
    )]
    pub transaction_scheduler: Account<'info, TransactionScheduler>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<InitializeScheduler>, params: InitializeSchedulerParams) -> Result<()> {
    let scheduler = &mut ctx.accounts.transaction_scheduler;
    let authority = &ctx.accounts.authority;
    let current_time = Clock::get()?.unix_timestamp;

    // Validate gas price parameters
    if params.max_gas_price < MIN_GAS_PRICE || params.max_gas_price > MAX_GAS_PRICE {
        return Err(TransactionSchedulerError::InvalidGasPrice.into());
    }

    // Initialize the scheduler
    scheduler.authority = authority.key();
    scheduler.max_gas_price = params.max_gas_price;
    scheduler.optimization_mode = params.optimization_mode.clone();
    scheduler.is_active = true;
    scheduler.created_at = current_time;
    scheduler.total_transactions_scheduled = 0;
    scheduler.total_transactions_executed = 0;
    scheduler.total_gas_saved = 0;
    scheduler.total_fees_collected = 0;
    scheduler.pending_transactions = Vec::new();
    scheduler.bump = ctx.bumps.transaction_scheduler;

    // Emit initialization event
    emit!(SchedulerInitialized {
        authority: authority.key(),
        max_gas_price: params.max_gas_price,
        optimization_mode: params.optimization_mode.clone(),
        timestamp: current_time,
    });

    msg!(
        "Transaction scheduler initialized for authority: {} with max gas price: {} and optimization mode: {:?}",
        authority.key(),
        params.max_gas_price,
        params.optimization_mode
    );

    Ok(())
}
