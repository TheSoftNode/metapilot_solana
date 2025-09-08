use crate::constants::*;
use crate::errors::TransactionSchedulerError;
use crate::events::{SchedulerPaused, SchedulerResumed};
use crate::state::{OptimizationMode, TransactionScheduler};
use anchor_lang::prelude::*;

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct UpdateGasSettingsParams {
    pub new_max_gas_price: Option<u64>,
    pub new_optimization_mode: Option<OptimizationMode>,
}

#[derive(Accounts)]
pub struct UpdateGasSettings<'info> {
    #[account(
        mut,
        seeds = [TRANSACTION_SCHEDULER_SEED, transaction_scheduler.authority.as_ref()],
        bump = transaction_scheduler.bump,
        has_one = authority
    )]
    pub transaction_scheduler: Account<'info, TransactionScheduler>,

    #[account(mut)]
    pub authority: Signer<'info>,
}

pub fn handler(ctx: Context<UpdateGasSettings>, params: UpdateGasSettingsParams) -> Result<()> {
    let scheduler = &mut ctx.accounts.transaction_scheduler;
    let authority = &ctx.accounts.authority;

    // Validate new gas price if provided
    if let Some(gas_price) = params.new_max_gas_price {
        if gas_price < MIN_GAS_PRICE || gas_price > MAX_GAS_PRICE {
            return Err(TransactionSchedulerError::InvalidGasPrice.into());
        }

        let old_price = scheduler.max_gas_price;
        scheduler.max_gas_price = gas_price;

        msg!(
            "Max gas price updated from {} to {} by {}",
            old_price,
            gas_price,
            authority.key()
        );
    }

    // Update optimization mode if provided
    if let Some(mode) = params.new_optimization_mode {
        let old_mode = scheduler.optimization_mode.clone();
        scheduler.optimization_mode = mode.clone();

        msg!(
            "Optimization mode updated from {:?} to {:?} by {}",
            old_mode,
            mode,
            authority.key()
        );
    }

    Ok(())
}

pub fn pause_scheduler_handler(ctx: Context<UpdateGasSettings>, reason: String) -> Result<()> {
    let scheduler = &mut ctx.accounts.transaction_scheduler;
    let authority = &ctx.accounts.authority;
    let current_time = Clock::get()?.unix_timestamp;

    if !scheduler.is_active {
        return Err(TransactionSchedulerError::SchedulerInactive.into());
    }

    scheduler.is_active = false;
    let pending_count = scheduler.get_pending_count();

    emit!(SchedulerPaused {
        authority: authority.key(),
        reason: reason.clone(),
        pending_transactions: pending_count as u16,
        timestamp: current_time,
    });

    msg!(
        "Scheduler paused by {} with reason: '{}'. {} transactions pending.",
        authority.key(),
        reason,
        pending_count
    );

    Ok(())
}

pub fn resume_scheduler_handler(ctx: Context<UpdateGasSettings>) -> Result<()> {
    let scheduler = &mut ctx.accounts.transaction_scheduler;
    let authority = &ctx.accounts.authority;
    let current_time = Clock::get()?.unix_timestamp;

    if scheduler.is_active {
        return Err(TransactionSchedulerError::SchedulerInactive.into());
    }

    scheduler.is_active = true;
    let pending_count = scheduler.get_pending_count();

    emit!(SchedulerResumed {
        authority: authority.key(),
        pending_transactions: pending_count as u16,
        timestamp: current_time,
    });

    msg!(
        "Scheduler resumed by {}. {} transactions ready for execution.",
        authority.key(),
        pending_count
    );

    Ok(())
}

#[derive(Accounts)]
pub struct UpdateSchedulerSettings<'info> {
    #[account(
        mut,
        seeds = [TRANSACTION_SCHEDULER_SEED, transaction_scheduler.authority.as_ref()],
        bump = transaction_scheduler.bump,
        has_one = authority
    )]
    pub transaction_scheduler: Account<'info, TransactionScheduler>,

    #[account(mut)]
    pub authority: Signer<'info>,
}

pub fn update_advanced_settings_handler(
    ctx: Context<UpdateSchedulerSettings>,
    enable_priority_execution: Option<bool>,
    max_batch_size: Option<u8>,
    emergency_pause: Option<bool>,
) -> Result<()> {
    let scheduler = &mut ctx.accounts.transaction_scheduler;
    let authority = &ctx.accounts.authority;

    // In a production implementation, these would be additional fields in the scheduler state
    // For now, we'll just log the changes

    if let Some(enable_priority) = enable_priority_execution {
        msg!(
            "Priority execution {} by {}",
            if enable_priority {
                "enabled"
            } else {
                "disabled"
            },
            authority.key()
        );
    }

    if let Some(batch_size) = max_batch_size {
        if batch_size == 0 || batch_size as usize > MAX_BATCH_SIZE {
            return Err(TransactionSchedulerError::InvalidTransactionData.into());
        }

        msg!(
            "Max batch size updated to {} by {}",
            batch_size,
            authority.key()
        );
    }

    if let Some(emergency) = emergency_pause {
        if emergency && scheduler.is_active {
            scheduler.is_active = false;
            msg!("Emergency pause activated by {}", authority.key());
        } else if !emergency && !scheduler.is_active {
            scheduler.is_active = true;
            msg!("Emergency pause deactivated by {}", authority.key());
        }
    }

    Ok(())
}
