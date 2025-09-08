use anchor_lang::prelude::*;
use crate::state::{YieldStrategy, RebalanceParams, AllocationTarget, ProtocolInfo};
use crate::errors::YieldOptimizerError;
use crate::constants::*;
use crate::utils::{YieldCalculator, ValidationUtils, RiskManager};
use crate::events::RebalanceEvent;

#[derive(Accounts)]
pub struct Rebalance<'info> {
    #[account(
        mut,
        seeds = [YIELD_STRATEGY_SEED, yield_strategy.authority.as_ref()],
        bump = yield_strategy.bump,
        has_one = authority
    )]
    pub yield_strategy: Account<'info, YieldStrategy>,
    
    #[account(mut)]
    pub authority: Signer<'info>,
    
    /// CHECK: AI oracle account - validated in handler
    pub ai_oracle: AccountInfo<'info>,
}

pub fn handler(
    ctx: Context<Rebalance>,
    params: RebalanceParams,
) -> Result<()> {
    let yield_strategy = &mut ctx.accounts.yield_strategy;
    let authority = &ctx.accounts.authority;
    let current_time = Clock::get()?.unix_timestamp;

    // Validate strategy is operational
    if !yield_strategy.is_operational() {
        return Err(YieldOptimizerError::StrategyNotActive.into());
    }

    // Check rebalance timing
    ValidationUtils::validate_timing(
        yield_strategy.last_rebalance_at,
        yield_strategy.rebalance_frequency,
        current_time,
    )?;

    // Validate rebalance parameters
    if !params.validate() {
        return Err(YieldOptimizerError::InvalidRebalanceParams.into());
    }

    // Check deadline
    if current_time > params.deadline {
        return Err(YieldOptimizerError::RebalanceDeadlineExceeded.into());
    }

    // Validate AI confidence threshold
    if params.ai_confidence < MIN_AI_CONFIDENCE {
        return Err(YieldOptimizerError::InsufficientAIConfidence.into());
    }

    // Validate risk tolerance
    RiskManager::validate_portfolio_risk(
        &params.new_allocations,
        yield_strategy.strategy_config.max_risk_level,
    )?;

    // Check concentration risk
    RiskManager::check_concentration_risk(&params.new_allocations)?;

    // Validate allocation amounts
    ValidationUtils::validate_allocation_amounts(
        &params.new_allocations,
        yield_strategy.total_deposited,
    )?;

    // Check if rebalancing is actually needed
    let current_allocations = get_current_allocations(yield_strategy)?;
    if !YieldCalculator::should_rebalance(
        &current_allocations,
        &params.new_allocations,
        REBALANCE_THRESHOLD_BASIS_POINTS,
    ) {
        return Err(YieldOptimizerError::RebalanceNotNeeded.into());
    }

    // Execute rebalancing logic
    let rebalance_result = execute_rebalance(
        yield_strategy,
        &current_allocations,
        &params.new_allocations,
        params.max_slippage,
    );

    // Update strategy state
    yield_strategy.last_rebalance_at = current_time;
    yield_strategy.rebalance_count = yield_strategy.rebalance_count
        .checked_add(1)
        .ok_or(YieldOptimizerError::MathOverflow)?;

    let success = rebalance_result.is_ok();
    let (actual_slippage, gas_used) = rebalance_result.unwrap_or((0, 0));

    // Emit rebalance event
    emit!(RebalanceEvent {
        authority: authority.key(),
        strategy_address: yield_strategy.key(),
        allocations: params.new_allocations.clone(),
        ai_confidence: params.ai_confidence,
        rebalance_number: yield_strategy.rebalance_count,
        total_value_locked: yield_strategy.total_deposited,
        estimated_apy_improvement: 0, // Would calculate based on new allocations
        timestamp: current_time,
    });

    if success {
        msg!(
            "Rebalanced strategy with {} allocations. AI confidence: {}%, Slippage: {:.2}%",
            params.new_allocations.len(),
            params.ai_confidence,
            actual_slippage as f64 / 100.0
        );
    } else {
        msg!("Rebalance failed - strategy state preserved");
        return Err(YieldOptimizerError::RebalanceFailed.into());
    }

    Ok(())
}

fn get_current_allocations(yield_strategy: &YieldStrategy) -> Result<Vec<AllocationTarget>> {
    // In a real implementation, this would:
    // 1. Query current balances across all protocols
    // 2. Calculate current allocation percentages
    // 3. Return the current allocation state
    
    // Mock current allocation for demonstration
    Ok(vec![
        AllocationTarget {
            protocol_id: 1,
            percentage: 4000, // 40%
            target_apy: 650,
            risk_weight: 3,
        },
        AllocationTarget {
            protocol_id: 2,
            percentage: 3500, // 35%
            target_apy: 800,
            risk_weight: 5,
        },
        AllocationTarget {
            protocol_id: 3,
            percentage: 2500, // 25%
            target_apy: 1200,
            risk_weight: 7,
        },
    ])
}

fn execute_rebalance(
    yield_strategy: &YieldStrategy,
    current_allocations: &[AllocationTarget],
    target_allocations: &[AllocationTarget],
    max_slippage: u16,
) -> Result<(u16, u64)> {
    // In a real implementation, this would:
    // 1. Calculate required token movements between protocols
    // 2. Execute withdrawal transactions from over-allocated protocols
    // 3. Execute deposit transactions to under-allocated protocols
    // 4. Handle DEX swaps if needed for token conversions
    // 5. Monitor slippage and gas costs
    // 6. Implement safety checks and rollback mechanisms
    
    msg!("Executing rebalance simulation...");
    
    // Simulate rebalancing logic
    let total_value = yield_strategy.total_deposited;
    let mut total_movements = 0u64;
    
    for target in target_allocations {
        let target_amount = (total_value * target.percentage as u64) / 10000;
        
        let current_amount = current_allocations
            .iter()
            .find(|c| c.protocol_id == target.protocol_id)
            .map(|c| (total_value * c.percentage as u64) / 10000)
            .unwrap_or(0);
        
        if target_amount != current_amount {
            let movement = if target_amount > current_amount {
                target_amount - current_amount
            } else {
                current_amount - target_amount
            };
            total_movements += movement;
        }
    }
    
    // Simulate slippage (random between 0 and max_slippage/2)
    let actual_slippage = (total_movements % max_slippage as u64) as u16 / 2;
    
    // Simulate gas usage
    let gas_used = 150_000 + (total_movements / 1000); // Base gas + dynamic based on movements
    
    msg!(
        "Rebalance executed: {} total movements, {}% slippage, {} gas used",
        total_movements,
        actual_slippage as f64 / 100.0,
        gas_used
    );
    
    Ok((actual_slippage, gas_used))
}
