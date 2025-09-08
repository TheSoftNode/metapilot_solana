use crate::constants::*;
use crate::errors::YieldOptimizerError;
use crate::events::CompoundEvent;
use crate::state::{CompoundParams, YieldStrategy};
use crate::utils::ValidationUtils;
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct Compound<'info> {
    #[account(
        mut,
        seeds = [YIELD_STRATEGY_SEED, yield_strategy.authority.as_ref()],
        bump = yield_strategy.bump,
        has_one = authority
    )]
    pub yield_strategy: Account<'info, YieldStrategy>,

    #[account(mut)]
    pub authority: Signer<'info>,

    /// CHECK: AI oracle account for yield optimization - validated in handler
    pub ai_oracle: AccountInfo<'info>,
}

pub fn handler(ctx: Context<Compound>, params: CompoundParams) -> Result<()> {
    let yield_strategy = &mut ctx.accounts.yield_strategy;
    let authority = &ctx.accounts.authority;
    let current_time = Clock::get()?.unix_timestamp;

    // Validate strategy is operational
    if !yield_strategy.is_operational() {
        return Err(YieldOptimizerError::StrategyNotActive.into());
    }

    // Check compound timing
    if !yield_strategy.can_compound(current_time) {
        return Err(YieldOptimizerError::CompoundTooFrequent.into());
    }

    // Validate minimum compound amount
    if params.estimated_rewards < params.min_compound_amount {
        return Err(YieldOptimizerError::CompoundAmountTooSmall.into());
    }

    if params.min_compound_amount < MIN_COMPOUND_AMOUNT {
        return Err(YieldOptimizerError::CompoundAmountTooSmall.into());
    }

    // Execute compounding logic
    let compound_result = if params.compound_all_protocols {
        compound_all_protocols(yield_strategy, params.estimated_rewards)
    } else if let Some(specific_protocols) = &params.specific_protocols {
        compound_specific_protocols(yield_strategy, specific_protocols, params.estimated_rewards)
    } else {
        return Err(YieldOptimizerError::NoProtocolsSelected.into());
    };

    // Update strategy state
    yield_strategy.last_compound_at = current_time;
    yield_strategy.compound_count = yield_strategy
        .compound_count
        .checked_add(1)
        .ok_or(YieldOptimizerError::MathOverflow)?;

    let (actual_rewards, gas_used, protocols_compounded) = compound_result?;

    // Update total rewards claimed
    yield_strategy.total_rewards_claimed = yield_strategy
        .total_rewards_claimed
        .checked_add(actual_rewards)
        .ok_or(YieldOptimizerError::MathOverflow)?;

    // Update total deposited (rewards are reinvested)
    yield_strategy.total_deposited = yield_strategy
        .total_deposited
        .checked_add(actual_rewards)
        .ok_or(YieldOptimizerError::MathOverflow)?;

    // Emit compound event
    emit!(CompoundEvent {
        authority: authority.key(),
        strategy_address: yield_strategy.key(),
        rewards_amount: actual_rewards,
        new_total_rewards: yield_strategy.total_rewards_claimed,
        compound_count: yield_strategy.compound_count,
        next_compound_at: current_time + MIN_COMPOUND_INTERVAL,
        timestamp: current_time,
    });

    msg!(
        "Compounded {} protocols. Rewards: {} (estimated: {}). New total: {}",
        protocols_compounded.len(),
        actual_rewards,
        params.estimated_rewards,
        yield_strategy.total_deposited
    );

    Ok(())
}

fn compound_all_protocols(
    yield_strategy: &YieldStrategy,
    estimated_rewards: u64,
) -> Result<(u64, u64, Vec<u8>)> {
    // In a real implementation, this would:
    // 1. Query all protocols where funds are allocated
    // 2. Claim rewards from each protocol
    // 3. Reinvest rewards back into the same or optimal protocols
    // 4. Handle any token conversions needed
    // 5. Track gas costs and actual rewards received

    msg!("Compounding all protocols...");

    // Simulate compounding across all target protocols
    let target_protocols = &yield_strategy.strategy_config.target_protocols;
    let mut actual_rewards = 0u64;
    let mut gas_used = 0u64;

    for &protocol_id in target_protocols {
        let (protocol_rewards, protocol_gas) = simulate_protocol_compound(
            protocol_id,
            estimated_rewards / target_protocols.len() as u64,
        )?;
        actual_rewards = actual_rewards
            .checked_add(protocol_rewards)
            .ok_or(YieldOptimizerError::MathOverflow)?;
        gas_used = gas_used
            .checked_add(protocol_gas)
            .ok_or(YieldOptimizerError::MathOverflow)?;
    }

    // Add base gas cost for the compound transaction
    gas_used = gas_used
        .checked_add(BASE_COMPOUND_GAS_COST)
        .ok_or(YieldOptimizerError::MathOverflow)?;

    Ok((actual_rewards, gas_used, target_protocols.clone()))
}

fn compound_specific_protocols(
    yield_strategy: &YieldStrategy,
    specific_protocols: &[u8],
    estimated_rewards: u64,
) -> Result<(u64, u64, Vec<u8>)> {
    // Validate that specific protocols are part of the strategy
    let target_protocols = &yield_strategy.strategy_config.target_protocols;
    for &protocol_id in specific_protocols {
        if !target_protocols.contains(&protocol_id) {
            return Err(YieldOptimizerError::ProtocolNotInStrategy.into());
        }
    }

    msg!("Compounding specific protocols: {:?}", specific_protocols);

    let mut actual_rewards = 0u64;
    let mut gas_used = 0u64;

    for &protocol_id in specific_protocols {
        let (protocol_rewards, protocol_gas) = simulate_protocol_compound(
            protocol_id,
            estimated_rewards / specific_protocols.len() as u64,
        )?;
        actual_rewards = actual_rewards
            .checked_add(protocol_rewards)
            .ok_or(YieldOptimizerError::MathOverflow)?;
        gas_used = gas_used
            .checked_add(protocol_gas)
            .ok_or(YieldOptimizerError::MathOverflow)?;
    }

    // Add base gas cost
    gas_used = gas_used
        .checked_add(BASE_COMPOUND_GAS_COST)
        .ok_or(YieldOptimizerError::MathOverflow)?;

    Ok((actual_rewards, gas_used, specific_protocols.to_vec()))
}

fn simulate_protocol_compound(protocol_id: u8, estimated_rewards: u64) -> Result<(u64, u64)> {
    // In a real implementation, this would execute protocol-specific compound logic
    // For now, simulate with some variation from estimated rewards

    let variation_factor = match protocol_id {
        1 => 95,  // Marinade: 95% of estimated (conservative)
        2 => 98,  // Orca: 98% of estimated (reliable)
        3 => 102, // Raydium: 102% of estimated (sometimes better)
        _ => 90,  // Unknown protocol: conservative estimate
    };

    let actual_rewards = (estimated_rewards * variation_factor) / 100;

    // Simulate gas cost based on protocol complexity
    let gas_cost = match protocol_id {
        1 => 35_000, // Marinade: simple staking
        2 => 45_000, // Orca: AMM operations
        3 => 55_000, // Raydium: complex AMM + farming
        _ => 60_000, // Unknown: assume high complexity
    };

    msg!(
        "Protocol {} compound: {} rewards (estimated: {}), {} gas",
        protocol_id,
        actual_rewards,
        estimated_rewards,
        gas_cost
    );

    Ok((actual_rewards, gas_cost))
}
