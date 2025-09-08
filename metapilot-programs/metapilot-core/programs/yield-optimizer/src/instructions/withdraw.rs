use crate::constants::*;
use crate::errors::YieldOptimizerError;
use crate::events::WithdrawEvent;
use crate::state::{WithdrawParams, WithdrawType, YieldStrategy};
use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

#[derive(Accounts)]
pub struct Withdraw<'info> {
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
        constraint = user_token_account.mint == withdraw_token_mint.key()
    )]
    pub user_token_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = strategy_token_account.owner == yield_strategy.key(),
        constraint = strategy_token_account.mint == withdraw_token_mint.key()
    )]
    pub strategy_token_account: Account<'info, TokenAccount>,

    /// CHECK: Token mint validation happens in constraints
    pub withdraw_token_mint: AccountInfo<'info>,

    pub token_program: Program<'info, Token>,
}

pub fn handler(ctx: Context<Withdraw>, params: WithdrawParams) -> Result<()> {
    let yield_strategy = &mut ctx.accounts.yield_strategy;
    let authority = &ctx.accounts.authority;
    let current_time = Clock::get()?.unix_timestamp;

    // Validate withdrawal type and conditions
    match params.withdraw_type {
        WithdrawType::Emergency => {
            // Emergency withdrawals can always proceed
            msg!(
                "Processing emergency withdrawal: {:?}",
                params.emergency_reason
            );
        }
        WithdrawType::Normal => {
            // Normal withdrawals require strategy to be operational
            if !yield_strategy.is_operational() {
                return Err(YieldOptimizerError::StrategyNotActive.into());
            }
        }
        WithdrawType::Partial { percentage } => {
            if percentage == 0 || percentage > 10000 {
                return Err(YieldOptimizerError::InvalidWithdrawPercentage.into());
            }

            // Calculate actual amount based on percentage
            let calculated_amount = (yield_strategy.total_deposited * percentage as u64) / 10000;
            if calculated_amount != params.amount {
                return Err(YieldOptimizerError::InconsistentWithdrawAmount.into());
            }
        }
    }

    // Validate withdrawal amount
    if params.amount == 0 {
        return Err(YieldOptimizerError::InvalidWithdrawAmount.into());
    }

    if params.amount > yield_strategy.total_deposited {
        return Err(YieldOptimizerError::InsufficientDeposits.into());
    }

    // Check strategy account has sufficient balance
    if ctx.accounts.strategy_token_account.amount < params.amount {
        return Err(YieldOptimizerError::InsufficientStrategyBalance.into());
    }

    // For non-emergency withdrawals, check minimum withdrawal limits
    if !matches!(params.withdraw_type, WithdrawType::Emergency) {
        if params.amount < MIN_WITHDRAW_AMOUNT {
            return Err(YieldOptimizerError::WithdrawTooSmall.into());
        }
    }

    // Calculate potential fees (for normal withdrawals)
    let (withdrawal_amount, fee_amount) = match params.withdraw_type {
        WithdrawType::Emergency => {
            // Emergency withdrawals might have higher fees
            let fee = (params.amount * EMERGENCY_WITHDRAW_FEE_BASIS_POINTS as u64) / 10000;
            (params.amount - fee, fee)
        }
        _ => {
            // Normal withdrawal fee
            let fee = (params.amount * WITHDRAW_FEE_BASIS_POINTS as u64) / 10000;
            (params.amount - fee, fee)
        }
    };

    // Create strategy seeds for PDA signing
    let authority_key = yield_strategy.authority.key();
    let seeds = &[
        YIELD_STRATEGY_SEED,
        authority_key.as_ref(),
        &[yield_strategy.bump],
    ];
    let signer_seeds = &[&seeds[..]];

    // Transfer tokens from strategy to user
    let transfer_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.strategy_token_account.to_account_info(),
            to: ctx.accounts.user_token_account.to_account_info(),
            authority: yield_strategy.to_account_info(),
        },
        signer_seeds,
    );

    token::transfer(transfer_ctx, withdrawal_amount)?;

    // Update strategy state
    yield_strategy.total_deposited = yield_strategy
        .total_deposited
        .checked_sub(params.amount)
        .ok_or(YieldOptimizerError::MathOverflow)?;

    // If emergency withdrawal, pause the strategy
    if matches!(params.withdraw_type, WithdrawType::Emergency) {
        yield_strategy.emergency_pause = true;
        msg!("Strategy paused due to emergency withdrawal");
    }

    // Emit withdrawal event
    emit!(WithdrawEvent {
        authority: authority.key(),
        strategy_address: yield_strategy.key(),
        amount: withdrawal_amount,
        penalty: fee_amount,
        remaining_balance: yield_strategy.total_deposited,
        withdrawal_type: format!("{:?}", params.withdraw_type),
        timestamp: current_time,
    });

    msg!(
        "Withdrew {} tokens (fee: {}). Remaining strategy value: {}",
        withdrawal_amount,
        fee_amount,
        yield_strategy.total_deposited
    );

    Ok(())
}
