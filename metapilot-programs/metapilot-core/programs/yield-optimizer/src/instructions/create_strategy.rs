use anchor_lang::prelude::*;
use crate::state::{YieldStrategy, YieldStrategyConfig, StrategyType};
use crate::errors::YieldOptimizerError;
use crate::constants::*;
use crate::utils::ValidationUtils;

#[derive(Accounts)]
#[instruction(config: YieldStrategyConfig, strategy_type: StrategyType)]
pub struct CreateStrategy<'info> {
    #[account(
        init,
        payer = authority,
        space = YieldStrategy::SIZE,
        seeds = [YIELD_STRATEGY_SEED, authority.key().as_ref()],
        bump
    )]
    pub yield_strategy: Account<'info, YieldStrategy>,
    
    #[account(mut)]
    pub authority: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

pub fn handler(
    ctx: Context<CreateStrategy>,
    config: YieldStrategyConfig,
    strategy_type: StrategyType,
) -> Result<()> {
    let yield_strategy = &mut ctx.accounts.yield_strategy;
    let authority = &ctx.accounts.authority;
    let current_time = Clock::get()?.unix_timestamp;

    // Validate strategy configuration
    ValidationUtils::validate_strategy_config(&config)?;

    // Validate risk level for strategy type
    let risk_level = strategy_type.get_risk_level();
    if risk_level > config.max_risk_level {
        return Err(YieldOptimizerError::RiskToleranceExceeded.into());
    }

    // Initialize the yield strategy
    yield_strategy.authority = authority.key();
    yield_strategy.strategy_config = config;
    yield_strategy.min_yield_threshold = strategy_type.get_min_yield_threshold();
    yield_strategy.rebalance_frequency = DEFAULT_REBALANCE_FREQUENCY;
    yield_strategy.is_active = true;
    yield_strategy.created_at = current_time;
    yield_strategy.last_rebalance_at = current_time;
    yield_strategy.last_compound_at = current_time;
    yield_strategy.total_deposited = 0;
    yield_strategy.total_rewards_claimed = 0;
    yield_strategy.rebalance_count = 0;
    yield_strategy.compound_count = 0;
    yield_strategy.emergency_pause = false;
    yield_strategy.bump = ctx.bumps.yield_strategy;

    msg!(
        "Created yield strategy for authority: {}, risk level: {}, min yield: {}%",
        authority.key(),
        risk_level,
        yield_strategy.min_yield_threshold as f64 / 100.0
    );

    Ok(())
}
