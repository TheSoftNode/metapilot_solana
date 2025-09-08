use crate::constants::*;
use crate::errors::YieldOptimizerError;
use crate::events::DepositEvent;
use crate::state::{AllocationTarget, DepositParams, ProtocolInfo, YieldStrategy};
use crate::utils::{RiskManager, ValidationUtils, YieldCalculator};
use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

#[derive(Accounts)]
pub struct Deposit<'info> {
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
        constraint = user_token_account.mint == deposit_token_mint.key()
    )]
    pub user_token_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = strategy_token_account.owner == yield_strategy.key(),
        constraint = strategy_token_account.mint == deposit_token_mint.key()
    )]
    pub strategy_token_account: Account<'info, TokenAccount>,

    /// CHECK: Token mint validation happens in constraints
    pub deposit_token_mint: AccountInfo<'info>,

    pub token_program: Program<'info, Token>,
}

pub fn handler(ctx: Context<Deposit>, params: DepositParams) -> Result<()> {
    let yield_strategy = &mut ctx.accounts.yield_strategy;
    let authority = &ctx.accounts.authority;
    let current_time = Clock::get()?.unix_timestamp;

    // Validate strategy is operational
    if !yield_strategy.is_operational() {
        return Err(YieldOptimizerError::StrategyNotActive.into());
    }

    // Validate deposit amount
    if params.amount < MIN_DEPOSIT_AMOUNT {
        return Err(YieldOptimizerError::DepositTooSmall.into());
    }

    // Check user has sufficient balance
    if ctx.accounts.user_token_account.amount < params.amount {
        return Err(YieldOptimizerError::InsufficientBalance.into());
    }

    // Transfer tokens from user to strategy
    let transfer_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.user_token_account.to_account_info(),
            to: ctx.accounts.strategy_token_account.to_account_info(),
            authority: authority.to_account_info(),
        },
    );

    token::transfer(transfer_ctx, params.amount)?;

    // Update strategy state
    yield_strategy.total_deposited = yield_strategy
        .total_deposited
        .checked_add(params.amount)
        .ok_or(YieldOptimizerError::MathOverflow)?;

    // Auto-allocation logic based on strategy configuration
    let allocation_result = if let Some(preferred_protocols) = params.preferred_protocols {
        // Use user's preferred protocols
        allocate_to_preferred_protocols(
            &preferred_protocols,
            params.amount,
            yield_strategy.strategy_config.max_risk_level,
        )
    } else {
        // Use AI-optimized allocation
        allocate_optimally(
            params.amount,
            yield_strategy.strategy_config.max_risk_level,
            yield_strategy.min_yield_threshold,
        )
    };

    match allocation_result {
        Ok(allocations) => {
            msg!(
                "Successfully allocated {} tokens across {} protocols",
                params.amount,
                allocations.len()
            );

            // Emit deposit event
            emit!(DepositEvent {
                authority: authority.key(),
                strategy_address: yield_strategy.key(),
                amount: params.amount,
                new_total: yield_strategy.total_deposited,
                timestamp: current_time,
            });
        }
        Err(e) => {
            msg!(
                "Warning: Could not allocate immediately: {:?}. Funds held in strategy account.",
                e
            );
        }
    }

    msg!(
        "Deposited {} tokens. Total strategy value: {}",
        params.amount,
        yield_strategy.total_deposited
    );

    Ok(())
}

fn allocate_to_preferred_protocols(
    preferred_protocols: &[u8],
    amount: u64,
    max_risk: u8,
) -> Result<Vec<AllocationTarget>> {
    // In a real implementation, this would:
    // 1. Fetch current protocol data from oracles or APIs
    // 2. Validate preferred protocols are available and within risk tolerance
    // 3. Calculate optimal allocation among preferred protocols
    // 4. Execute the allocation transactions

    // For now, create a mock allocation
    if preferred_protocols.is_empty() {
        return Err(YieldOptimizerError::NoProtocolsSelected.into());
    }

    let allocation_per_protocol = 10000 / preferred_protocols.len() as u16;
    let mut allocations = Vec::new();

    for (i, &protocol_id) in preferred_protocols.iter().enumerate() {
        let percentage = if i == preferred_protocols.len() - 1 {
            // Last protocol gets remaining allocation
            10000 - (allocation_per_protocol * (preferred_protocols.len() - 1) as u16)
        } else {
            allocation_per_protocol
        };

        allocations.push(AllocationTarget {
            protocol_id,
            percentage,
            target_apy: 500,              // 5% default
            risk_weight: max_risk.min(5), // Conservative default
        });
    }

    Ok(allocations)
}

fn allocate_optimally(amount: u64, max_risk: u8, min_yield: u16) -> Result<Vec<AllocationTarget>> {
    // Mock protocol data - in production this would come from oracles
    let mock_protocols = vec![
        ProtocolInfo {
            name: "Marinade".to_string(),
            protocol_id: 1,
            apy: 650, // 6.5%
            risk_level: 3,
            allocation_amount: 0,
            total_value_locked: 1_000_000_000, // 1B
            supported_tokens: vec![],
            is_active: true,
        },
        ProtocolInfo {
            name: "Orca".to_string(),
            protocol_id: 2,
            apy: 800, // 8%
            risk_level: 5,
            allocation_amount: 0,
            total_value_locked: 500_000_000, // 500M
            supported_tokens: vec![],
            is_active: true,
        },
        ProtocolInfo {
            name: "Raydium".to_string(),
            protocol_id: 3,
            apy: 1200, // 12%
            risk_level: 7,
            allocation_amount: 0,
            total_value_locked: 300_000_000, // 300M
            supported_tokens: vec![],
            is_active: true,
        },
    ];

    YieldCalculator::calculate_optimal_allocation(&mock_protocols, max_risk, min_yield)
}
