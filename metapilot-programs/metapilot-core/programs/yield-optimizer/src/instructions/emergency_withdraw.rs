use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};
use crate::state::{YieldStrategy, WithdrawType};
use crate::errors::YieldOptimizerError;
use crate::constants::*;
use crate::events::EmergencyWithdrawEvent;

#[derive(Accounts)]
pub struct EmergencyWithdraw<'info> {
    #[account(
        mut,
        seeds = [YIELD_STRATEGY_SEED, yield_strategy.authority.as_ref()],
        bump = yield_strategy.bump,
        has_one = authority
    )]
    pub yield_strategy: Account<'info, YieldStrategy>,
    
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(
        mut,
        constraint = user_token_account.owner == authority.key(),
        constraint = user_token_account.mint == emergency_token_mint.key()
    )]
    pub user_token_account: Account<'info, TokenAccount>,
    
    #[account(
        mut,
        constraint = strategy_token_account.owner == yield_strategy.key(),
        constraint = strategy_token_account.mint == emergency_token_mint.key()
    )]
    pub strategy_token_account: Account<'info, TokenAccount>,
    
    /// CHECK: Token mint validation happens in constraints
    pub emergency_token_mint: AccountInfo<'info>,
    
    pub token_program: Program<'info, Token>,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct EmergencyWithdrawParams {
    pub reason: String,
    pub withdraw_all: bool,
    pub specific_amount: Option<u64>,
}

pub fn handler(
    ctx: Context<EmergencyWithdraw>,
    params: EmergencyWithdrawParams,
) -> Result<()> {
    let yield_strategy = &mut ctx.accounts.yield_strategy;
    let authority = &ctx.accounts.authority;
    let current_time = Clock::get()?.unix_timestamp;

    // Validate emergency reason
    if params.reason.is_empty() || params.reason.len() > 200 {
        return Err(YieldOptimizerError::InvalidEmergencyReason.into());
    }

    // Calculate withdrawal amount
    let withdrawal_amount = if params.withdraw_all {
        yield_strategy.total_deposited
    } else if let Some(amount) = params.specific_amount {
        if amount == 0 || amount > yield_strategy.total_deposited {
            return Err(YieldOptimizerError::InvalidWithdrawAmount.into());
        }
        amount
    } else {
        return Err(YieldOptimizerError::InvalidWithdrawAmount.into());
    };

    // Check strategy account has sufficient balance
    let available_balance = ctx.accounts.strategy_token_account.amount;
    if available_balance < withdrawal_amount {
        msg!(
            "Warning: Strategy balance ({}) less than requested ({}). Withdrawing available balance.",
            available_balance,
            withdrawal_amount
        );
    }

    let actual_withdrawal = withdrawal_amount.min(available_balance);

    // Calculate emergency fee (higher than normal withdrawal)
    let fee_amount = (actual_withdrawal * EMERGENCY_WITHDRAW_FEE_BASIS_POINTS as u64) / 10000;
    let net_withdrawal = actual_withdrawal - fee_amount;

    // Create strategy seeds for PDA signing
    let authority_key = yield_strategy.authority.key();
    let seeds = &[
        YIELD_STRATEGY_SEED,
        authority_key.as_ref(),
        &[yield_strategy.bump],
    ];
    let signer_seeds = &[&seeds[..]];

    // Transfer tokens from strategy to user
    if net_withdrawal > 0 {
        let transfer_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.strategy_token_account.to_account_info(),
                to: ctx.accounts.user_token_account.to_account_info(),
                authority: yield_strategy.to_account_info(),
            },
            signer_seeds,
        );

        token::transfer(transfer_ctx, net_withdrawal)?;
    }

    // Update strategy state
    yield_strategy.total_deposited = yield_strategy.total_deposited
        .checked_sub(actual_withdrawal)
        .ok_or(YieldOptimizerError::MathOverflow)?;

    // Activate emergency pause
    yield_strategy.emergency_pause = true;
    yield_strategy.is_active = false;

    // Emit emergency withdrawal event
    emit!(EmergencyWithdrawEvent {
        authority: authority.key(),
        strategy_address: yield_strategy.key(),
        amount: net_withdrawal,
        penalty: fee_amount,
        reason: params.reason.clone(),
        timestamp: current_time,
    });

    msg!(
        "EMERGENCY WITHDRAWAL executed. Reason: '{}'. Amount: {} (fee: {}). Strategy PAUSED.",
        params.reason,
        net_withdrawal,
        fee_amount
    );

    // If there are remaining funds, suggest next steps
    if yield_strategy.total_deposited > 0 {
        msg!(
            "Remaining funds: {}. Manual intervention required to recover from other protocols.",
            yield_strategy.total_deposited
        );
    }

    Ok(())
}

#[derive(Accounts)]
pub struct ReactivateStrategy<'info> {
    #[account(
        mut,
        seeds = [YIELD_STRATEGY_SEED, yield_strategy.authority.as_ref()],
        bump = yield_strategy.bump,
        has_one = authority
    )]
    pub yield_strategy: Account<'info, YieldStrategy>,
    
    #[account(mut)]
    pub authority: Signer<'info>,
}

pub fn reactivate_strategy_handler(ctx: Context<ReactivateStrategy>) -> Result<()> {
    let yield_strategy = &mut ctx.accounts.yield_strategy;
    let current_time = Clock::get()?.unix_timestamp;

    // Only allow reactivation if strategy was emergency paused
    if !yield_strategy.emergency_pause {
        return Err(YieldOptimizerError::StrategyNotPaused.into());
    }

    // Require a cooling-off period after emergency withdrawal
    const EMERGENCY_COOLDOWN: i64 = 24 * 60 * 60; // 24 hours
    if current_time < yield_strategy.last_compound_at + EMERGENCY_COOLDOWN {
        return Err(YieldOptimizerError::EmergencyCooldownActive.into());
    }

    // Reactivate the strategy
    yield_strategy.emergency_pause = false;
    yield_strategy.is_active = true;

    msg!(
        "Strategy reactivated after emergency pause. Cooldown period completed at {}",
        current_time
    );

    Ok(())
}
