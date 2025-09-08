use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount, Mint};
use crate::state::*;
use crate::constants::*;
use crate::errors::*;
use crate::events::*;
use crate::utils::*;

/// Parameters for DCA (Dollar Cost Averaging) strategies
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct DcaParams {
    pub token_mint_in: Pubkey,
    pub token_mint_out: Pubkey,
    pub amount_per_execution: u64,
    pub interval_seconds: i64,
    pub total_executions: u64,
    pub max_slippage: u16,
}

/// Create a new DCA strategy
#[derive(Accounts)]
#[instruction(params: DcaParams)]
pub struct CreateDcaStrategy<'info> {
    #[account(
        mut,
        seeds = [AUTOMATION_SEED, owner.key().as_ref()],
        bump,
        constraint = token_automation.is_active @ TokenAutomationError::AutomationInactive,
        constraint = !token_automation.is_paused @ TokenAutomationError::AutomationPaused,
    )]
    pub token_automation: Account<'info, TokenAutomation>,
    
    #[account(
        init,
        payer = owner,
        space = DcaStrategy::SIZE,
        seeds = [
            DCA_STRATEGY_SEED, 
            token_automation.key().as_ref(),
            owner.key().as_ref(),
            &Clock::get()?.unix_timestamp.to_le_bytes()
        ],
        bump
    )]
    pub dca_strategy: Account<'info, DcaStrategy>,
    
    #[account(mut)]
    pub owner: Signer<'info>,
    
    #[account(
        mut,
        constraint = token_account_in.mint == params.token_mint_in @ TokenAutomationError::InvalidMint,
        constraint = token_account_in.owner == owner.key() @ TokenAutomationError::InvalidTokenAccountOwner,
    )]
    pub token_account_in: Account<'info, TokenAccount>,
    
    #[account(
        constraint = token_account_out.mint == params.token_mint_out @ TokenAutomationError::InvalidMint,
        constraint = token_account_out.owner == owner.key() @ TokenAutomationError::InvalidTokenAccountOwner,
    )]
    pub token_account_out: Account<'info, TokenAccount>,
    
    #[account(
        constraint = mint_in.key() == params.token_mint_in @ TokenAutomationError::InvalidMint
    )]
    pub mint_in: Account<'info, Mint>,
    
    #[account(
        constraint = mint_out.key() == params.token_mint_out @ TokenAutomationError::InvalidMint
    )]
    pub mint_out: Account<'info, Mint>,
    
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}

pub fn create_dca_strategy(
    ctx: Context<CreateDcaStrategy>,
    params: DcaParams,
) -> Result<()> {
    // Validate parameters
    validate_dca_strategy(&params)?;
    
    let automation = &mut ctx.accounts.token_automation;
    let dca_strategy = &mut ctx.accounts.dca_strategy;
    let clock = Clock::get()?;
    
    // Calculate total required amount
    let total_required = params.amount_per_execution
        .checked_mul(params.total_executions)
        .ok_or(TokenAutomationError::MathOverflow)?;
    
    // Check if user has sufficient balance
    require!(
        ctx.accounts.token_account_in.amount >= total_required,
        TokenAutomationError::InsufficientBalance
    );
    
    // Generate unique strategy ID
    let strategy_id = generate_order_id(
        &automation.key(),
        clock.unix_timestamp,
        OrderType::DcaStrategy,
    );
    
    // Initialize DCA strategy
    dca_strategy.id = strategy_id;
    dca_strategy.owner = ctx.accounts.owner.key();
    dca_strategy.token_automation = automation.key();
    dca_strategy.token_mint_in = params.token_mint_in;
    dca_strategy.token_mint_out = params.token_mint_out;
    dca_strategy.amount_per_execution = params.amount_per_execution;
    dca_strategy.interval_seconds = params.interval_seconds;
    dca_strategy.total_executions = params.total_executions;
    dca_strategy.executed_count = 0;
    dca_strategy.next_execution_at = clock.unix_timestamp + params.interval_seconds;
    dca_strategy.created_at = clock.unix_timestamp;
    dca_strategy.status = StrategyStatus::Active;
    dca_strategy.max_slippage = params.max_slippage;
    
    // Emit strategy creation event
    emit!(DcaStrategyCreated {
        strategy_id,
        owner: ctx.accounts.owner.key(),
        automation: automation.key(),
        token_mint_in: params.token_mint_in,
        token_mint_out: params.token_mint_out,
        amount_per_execution: params.amount_per_execution,
        interval_seconds: params.interval_seconds,
        total_executions: params.total_executions,
        next_execution_at: dca_strategy.next_execution_at,
        timestamp: clock.unix_timestamp,
    });
    
    msg!("DCA strategy created with ID: {}", strategy_id);
    Ok(())
}

/// Execute next DCA order
#[derive(Accounts)]
pub struct ExecuteDcaOrder<'info> {
    #[account(
        mut,
        constraint = dca_strategy.status == StrategyStatus::Active @ TokenAutomationError::StrategyNotActive,
        constraint = dca_strategy.ready_for_execution() @ TokenAutomationError::ExecutionNotReady,
    )]
    pub dca_strategy: Account<'info, DcaStrategy>,
    
    #[account(
        mut,
        constraint = token_automation.is_active @ TokenAutomationError::AutomationInactive,
        constraint = !token_automation.is_paused @ TokenAutomationError::AutomationPaused,
    )]
    pub token_automation: Account<'info, TokenAutomation>,
    
    #[account(
        mut,
        constraint = user_token_account_in.mint == dca_strategy.token_mint_in @ TokenAutomationError::InvalidMint,
        constraint = user_token_account_in.owner == dca_strategy.owner @ TokenAutomationError::InvalidTokenAccountOwner,
    )]
    pub user_token_account_in: Account<'info, TokenAccount>,
    
    #[account(
        mut,
        constraint = user_token_account_out.mint == dca_strategy.token_mint_out @ TokenAutomationError::InvalidMint,
        constraint = user_token_account_out.owner == dca_strategy.owner @ TokenAutomationError::InvalidTokenAccountOwner,
    )]
    pub user_token_account_out: Account<'info, TokenAccount>,
    
    /// CHECK: DEX program for executing the swap
    pub dex_program: UncheckedAccount<'info>,
    
    /// CHECK: DEX market account
    pub dex_market: UncheckedAccount<'info>,
    
    pub token_program: Program<'info, Token>,
}

pub fn execute_dca_order(
    ctx: Context<ExecuteDcaOrder>,
    current_price: u64,
    amount_out: u64,
) -> Result<()> {
    let dca_strategy = &mut ctx.accounts.dca_strategy;
    let automation = &mut ctx.accounts.token_automation;
    let clock = Clock::get()?;
    
    // Check if user has sufficient balance
    require!(
        ctx.accounts.user_token_account_in.amount >= dca_strategy.amount_per_execution,
        TokenAutomationError::InsufficientBalance
    );
    
    // Calculate expected minimum amount considering slippage
    let expected_amount = (dca_strategy.amount_per_execution * current_price) / (10u64.pow(6)); // Assuming 6 decimals
    let slippage_adjusted_min = expected_amount
        .saturating_mul(10000 - dca_strategy.max_slippage as u64)
        .saturating_div(10000);
    
    require!(
        amount_out >= slippage_adjusted_min,
        TokenAutomationError::InsufficientAmountOut
    );
    
    // Update strategy state
    dca_strategy.executed_count += 1;
    dca_strategy.next_execution_at = clock.unix_timestamp + dca_strategy.interval_seconds;
    
    // Check if strategy is completed
    if dca_strategy.is_completed() {
        dca_strategy.status = StrategyStatus::Completed;
    }
    
    // Update automation stats
    automation.total_executions += 1;
    automation.total_volume = automation.total_volume.saturating_add(dca_strategy.amount_per_execution);
    automation.last_updated = clock.unix_timestamp;
    
    // Calculate and collect execution fee
    let execution_fee = calculate_execution_fee(
        automation.gas_limit,
        DCA_COMPLEXITY,
        BASE_EXECUTION_FEE,
    );
    
    automation.total_fees_collected = automation.total_fees_collected.saturating_add(execution_fee);
    
    // Emit execution event
    emit!(DcaOrderExecuted {
        strategy_id: dca_strategy.id,
        owner: dca_strategy.owner,
        automation: automation.key(),
        execution_number: dca_strategy.executed_count,
        amount_in: dca_strategy.amount_per_execution,
        amount_out,
        execution_price: current_price,
        execution_fee,
        next_execution_at: dca_strategy.next_execution_at,
        is_completed: dca_strategy.is_completed(),
        timestamp: clock.unix_timestamp,
    });
    
    msg!(
        "DCA order executed: {}/{} for strategy {}",
        dca_strategy.executed_count,
        dca_strategy.total_executions,
        dca_strategy.id
    );
    
    Ok(())
}

/// Pause DCA strategy
#[derive(Accounts)]
pub struct PauseDcaStrategy<'info> {
    #[account(
        mut,
        has_one = owner,
        constraint = dca_strategy.status == StrategyStatus::Active @ TokenAutomationError::StrategyNotActive,
    )]
    pub dca_strategy: Account<'info, DcaStrategy>,
    
    pub owner: Signer<'info>,
}

pub fn pause_dca_strategy(ctx: Context<PauseDcaStrategy>) -> Result<()> {
    let dca_strategy = &mut ctx.accounts.dca_strategy;
    let clock = Clock::get()?;
    
    dca_strategy.status = StrategyStatus::Paused;
    
    // Emit pause event
    emit!(DcaStrategyPaused {
        strategy_id: dca_strategy.id,
        owner: dca_strategy.owner,
        timestamp: clock.unix_timestamp,
    });
    
    msg!("DCA strategy {} paused", dca_strategy.id);
    Ok(())
}

/// Resume DCA strategy
#[derive(Accounts)]
pub struct ResumeDcaStrategy<'info> {
    #[account(
        mut,
        has_one = owner,
        constraint = dca_strategy.status == StrategyStatus::Paused @ TokenAutomationError::StrategyNotPaused,
    )]
    pub dca_strategy: Account<'info, DcaStrategy>,
    
    pub owner: Signer<'info>,
}

pub fn resume_dca_strategy(ctx: Context<ResumeDcaStrategy>) -> Result<()> {
    let dca_strategy = &mut ctx.accounts.dca_strategy;
    let clock = Clock::get()?;
    
    dca_strategy.status = StrategyStatus::Active;
    
    // Reset next execution time
    dca_strategy.next_execution_at = clock.unix_timestamp + dca_strategy.interval_seconds;
    
    // Emit resume event
    emit!(DcaStrategyResumed {
        strategy_id: dca_strategy.id,
        owner: dca_strategy.owner,
        next_execution_at: dca_strategy.next_execution_at,
        timestamp: clock.unix_timestamp,
    });
    
    msg!("DCA strategy {} resumed", dca_strategy.id);
    Ok(())
}

/// Cancel DCA strategy
#[derive(Accounts)]
pub struct CancelDcaStrategy<'info> {
    #[account(
        mut,
        has_one = owner,
        constraint = dca_strategy.status != StrategyStatus::Cancelled @ TokenAutomationError::StrategyAlreadyCancelled,
        constraint = dca_strategy.status != StrategyStatus::Completed @ TokenAutomationError::StrategyCompleted,
    )]
    pub dca_strategy: Account<'info, DcaStrategy>,
    
    pub owner: Signer<'info>,
}

pub fn cancel_dca_strategy(ctx: Context<CancelDcaStrategy>) -> Result<()> {
    let dca_strategy = &mut ctx.accounts.dca_strategy;
    let clock = Clock::get()?;
    
    dca_strategy.status = StrategyStatus::Cancelled;
    
    // Emit cancellation event
    emit!(DcaStrategyCancelled {
        strategy_id: dca_strategy.id,
        owner: dca_strategy.owner,
        executions_completed: dca_strategy.executed_count,
        timestamp: clock.unix_timestamp,
    });
    
    msg!("DCA strategy {} cancelled", dca_strategy.id);
    Ok(())
}

/// Update DCA strategy parameters
#[derive(Accounts)]
pub struct UpdateDcaStrategy<'info> {
    #[account(
        mut,
        has_one = owner,
        constraint = dca_strategy.status == StrategyStatus::Active || dca_strategy.status == StrategyStatus::Paused 
            @ TokenAutomationError::CannotUpdateStrategy,
    )]
    pub dca_strategy: Account<'info, DcaStrategy>,
    
    pub owner: Signer<'info>,
}

pub fn update_dca_strategy(
    ctx: Context<UpdateDcaStrategy>,
    new_interval_seconds: Option<i64>,
    new_max_slippage: Option<u16>,
) -> Result<()> {
    let dca_strategy = &mut ctx.accounts.dca_strategy;
    let clock = Clock::get()?;
    
    // Update interval if provided
    if let Some(interval_seconds) = new_interval_seconds {
        require!(
            interval_seconds >= MIN_DCA_INTERVAL && interval_seconds <= MAX_DCA_INTERVAL,
            TokenAutomationError::InvalidDcaInterval
        );
        dca_strategy.interval_seconds = interval_seconds;
        
        // Update next execution time if strategy is active
        if dca_strategy.status == StrategyStatus::Active {
            dca_strategy.next_execution_at = clock.unix_timestamp + interval_seconds;
        }
    }
    
    // Update slippage if provided
    if let Some(max_slippage) = new_max_slippage {
        require!(
            max_slippage <= MAX_SLIPPAGE_BPS,
            TokenAutomationError::InvalidSlippage
        );
        dca_strategy.max_slippage = max_slippage;
    }
    
    // Emit update event
    emit!(DcaStrategyUpdated {
        strategy_id: dca_strategy.id,
        owner: dca_strategy.owner,
        new_interval_seconds,
        new_max_slippage,
        timestamp: clock.unix_timestamp,
    });
    
    msg!("DCA strategy {} updated", dca_strategy.id);
    Ok(())
}
