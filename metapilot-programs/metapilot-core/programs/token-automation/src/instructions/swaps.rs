use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount};
use crate::state::*;
use crate::constants::*;
use crate::errors::*;
use crate::events::*;
use crate::utils::*;

/// Parameters for swap operations
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct SwapParams {
    pub token_mint_a: Pubkey,
    pub token_mint_b: Pubkey,
    pub amount_in: u64,
    pub minimum_amount_out: u64,
    pub max_slippage: u16,
    pub ai_confidence: u8,
    pub price_data: MarketData,
    pub execution_type: ExecutionType,
}

/// Execute an automated swap based on AI analysis
#[derive(Accounts)]
pub struct ExecuteAiSwap<'info> {
    #[account(
        mut,
        constraint = token_automation.is_active @ TokenAutomationError::AutomationInactive,
        constraint = !token_automation.is_paused @ TokenAutomationError::AutomationPaused,
        constraint = token_automation.ai_automation_enabled @ TokenAutomationError::AiAutomationDisabled,
    )]
    pub token_automation: Account<'info, TokenAutomation>,
    
    #[account(
        mut,
        constraint = user_token_account_a.owner == owner.key() @ TokenAutomationError::InvalidTokenAccountOwner,
    )]
    pub user_token_account_a: Account<'info, TokenAccount>,
    
    #[account(
        mut,
        constraint = user_token_account_b.owner == owner.key() @ TokenAutomationError::InvalidTokenAccountOwner,
    )]
    pub user_token_account_b: Account<'info, TokenAccount>,
    
    pub owner: Signer<'info>,
    
    /// CHECK: DEX program for executing the swap
    pub dex_program: UncheckedAccount<'info>,
    
    /// CHECK: DEX market account
    pub dex_market: UncheckedAccount<'info>,
    
    pub token_program: Program<'info, Token>,
}

pub fn execute_ai_swap(
    ctx: Context<ExecuteAiSwap>,
    params: SwapParams,
) -> Result<()> {
    // Validate swap parameters
    validate_swap_params(&params)?;
    
    let automation = &mut ctx.accounts.token_automation;
    let clock = Clock::get()?;
    
    // Check if AI confidence is above minimum threshold
    require!(
        params.ai_confidence >= MIN_AI_CONFIDENCE_THRESHOLD,
        TokenAutomationError::LowAiConfidence
    );
    
    // Check if user has sufficient balance
    require!(
        ctx.accounts.user_token_account_a.amount >= params.amount_in,
        TokenAutomationError::InsufficientBalance
    );
    
    // Calculate dynamic slippage based on market conditions
    let dynamic_slippage = calculate_dynamic_slippage(
        automation.max_slippage,
        params.price_data.price_change_24h.abs() as u16, // Volatility proxy
        params.price_data.volume_24h,
        200, // Base market impact
    );
    
    // Validate execution meets slippage requirements
    let expected_amount = (params.amount_in * params.price_data.exchange_rate) 
        / (10u64.pow(params.price_data.decimals as u32));
    let slippage_adjusted_min = expected_amount
        .saturating_mul(10000 - dynamic_slippage as u64)
        .saturating_div(10000);
    
    require!(
        params.minimum_amount_out >= slippage_adjusted_min,
        TokenAutomationError::SlippageExceeded
    );
    
    // Update automation stats
    automation.total_executions += 1;
    automation.total_volume = automation.total_volume.saturating_add(params.amount_in);
    automation.last_updated = clock.unix_timestamp;
    
    // Calculate and collect execution fee based on AI complexity
    let execution_fee = calculate_execution_fee(
        automation.gas_limit,
        AI_SWAP_COMPLEXITY,
        BASE_EXECUTION_FEE,
    );
    
    automation.total_fees_collected = automation.total_fees_collected.saturating_add(execution_fee);
    
    // Emit AI swap event
    emit!(AiSwapExecuted {
        automation: automation.key(),
        owner: ctx.accounts.owner.key(),
        token_mint_a: ctx.accounts.user_token_account_a.mint,
        token_mint_b: ctx.accounts.user_token_account_b.mint,
        amount_in: params.amount_in,
        amount_out: params.minimum_amount_out,
        ai_confidence: params.ai_confidence,
        execution_price: params.price_data.exchange_rate,
        execution_fee,
        execution_type: params.execution_type,
        timestamp: clock.unix_timestamp,
    });
    
    msg!(
        "AI swap executed with {}% confidence: {} -> {}",
        params.ai_confidence,
        params.amount_in,
        params.minimum_amount_out
    );
    
    Ok(())
}

/// Execute manual swap with automation features
#[derive(Accounts)]
pub struct ExecuteManualSwap<'info> {
    #[account(
        mut,
        constraint = token_automation.is_active @ TokenAutomationError::AutomationInactive,
        constraint = !token_automation.is_paused @ TokenAutomationError::AutomationPaused,
    )]
    pub token_automation: Account<'info, TokenAutomation>,
    
    #[account(
        mut,
        constraint = user_token_account_a.owner == owner.key() @ TokenAutomationError::InvalidTokenAccountOwner,
    )]
    pub user_token_account_a: Account<'info, TokenAccount>,
    
    #[account(
        mut,
        constraint = user_token_account_b.owner == owner.key() @ TokenAutomationError::InvalidTokenAccountOwner,
    )]
    pub user_token_account_b: Account<'info, TokenAccount>,
    
    pub owner: Signer<'info>,
    
    /// CHECK: DEX program for executing the swap
    pub dex_program: UncheckedAccount<'info>,
    
    /// CHECK: DEX market account
    pub dex_market: UncheckedAccount<'info>,
    
    pub token_program: Program<'info, Token>,
}

pub fn execute_manual_swap(
    ctx: Context<ExecuteManualSwap>,
    amount_in: u64,
    minimum_amount_out: u64,
    max_slippage: u16,
) -> Result<()> {
    let automation = &mut ctx.accounts.token_automation;
    let clock = Clock::get()?;
    
    // Validate parameters
    require!(amount_in > 0, TokenAutomationError::InvalidAmount);
    require!(minimum_amount_out > 0, TokenAutomationError::InvalidAmount);
    require!(max_slippage <= MAX_SLIPPAGE_BPS, TokenAutomationError::InvalidSlippage);
    
    // Check if user has sufficient balance
    require!(
        ctx.accounts.user_token_account_a.amount >= amount_in,
        TokenAutomationError::InsufficientBalance
    );
    
    // Update automation stats
    automation.total_executions += 1;
    automation.total_volume = automation.total_volume.saturating_add(amount_in);
    automation.last_updated = clock.unix_timestamp;
    
    // Calculate execution fee
    let execution_fee = calculate_execution_fee(
        automation.gas_limit,
        MANUAL_SWAP_COMPLEXITY,
        BASE_EXECUTION_FEE,
    );
    
    automation.total_fees_collected = automation.total_fees_collected.saturating_add(execution_fee);
    
    // Emit manual swap event
    emit!(ManualSwapExecuted {
        automation: automation.key(),
        owner: ctx.accounts.owner.key(),
        token_mint_a: ctx.accounts.user_token_account_a.mint,
        token_mint_b: ctx.accounts.user_token_account_b.mint,
        amount_in,
        minimum_amount_out,
        max_slippage,
        execution_fee,
        timestamp: clock.unix_timestamp,
    });
    
    msg!("Manual swap executed: {} -> {}", amount_in, minimum_amount_out);
    Ok(())
}

/// Batch execute multiple swaps
#[derive(Accounts)]
pub struct BatchSwap<'info> {
    #[account(
        mut,
        constraint = token_automation.is_active @ TokenAutomationError::AutomationInactive,
        constraint = !token_automation.is_paused @ TokenAutomationError::AutomationPaused,
    )]
    pub token_automation: Account<'info, TokenAutomation>,
    
    pub owner: Signer<'info>,
    
    /// CHECK: DEX program for executing the swaps
    pub dex_program: UncheckedAccount<'info>,
    
    pub token_program: Program<'info, Token>,
}

pub fn batch_swap(
    ctx: Context<BatchSwap>,
    swap_instructions: Vec<SwapParams>,
) -> Result<()> {
    let automation = &mut ctx.accounts.token_automation;
    let clock = Clock::get()?;
    
    // Validate batch size
    require!(
        swap_instructions.len() <= MAX_BATCH_SIZE,
        TokenAutomationError::BatchTooLarge
    );
    
    require!(
        !swap_instructions.is_empty(),
        TokenAutomationError::EmptyBatch
    );
    
    // Validate each swap instruction
    for swap_params in &swap_instructions {
        validate_swap_params(swap_params)?;
    }
    
    let mut total_volume = 0u64;
    let mut total_fees = 0u64;
    
    // Calculate totals
    for swap_params in &swap_instructions {
        total_volume = total_volume.saturating_add(swap_params.amount_in);
        
        let complexity = match swap_params.execution_type {
            ExecutionType::AiDriven => AI_SWAP_COMPLEXITY,
            ExecutionType::Automated => AUTOMATED_SWAP_COMPLEXITY,
            _ => MANUAL_SWAP_COMPLEXITY,
        };
        
        let fee = calculate_execution_fee(
            automation.gas_limit / (swap_instructions.len() as u64), // Distribute gas
            complexity,
            BASE_EXECUTION_FEE,
        );
        
        total_fees = total_fees.saturating_add(fee);
    }
    
    // Update automation stats
    automation.total_executions = automation.total_executions.saturating_add(swap_instructions.len() as u64);
    automation.total_volume = automation.total_volume.saturating_add(total_volume);
    automation.total_fees_collected = automation.total_fees_collected.saturating_add(total_fees);
    automation.last_updated = clock.unix_timestamp;
    
    // Emit batch swap event
    emit!(BatchSwapExecuted {
        automation: automation.key(),
        owner: ctx.accounts.owner.key(),
        swap_count: swap_instructions.len() as u32,
        total_volume,
        total_fees,
        timestamp: clock.unix_timestamp,
    });
    
    msg!("Batch swap executed: {} swaps, total volume: {}", swap_instructions.len(), total_volume);
    Ok(())
}

/// Emergency stop all swap operations
#[derive(Accounts)]
pub struct EmergencyStopSwaps<'info> {
    #[account(
        mut,
        constraint = token_automation.emergency_authority == emergency_authority.key() 
            @ TokenAutomationError::UnauthorizedEmergencyAction
    )]
    pub token_automation: Account<'info, TokenAutomation>,
    
    pub emergency_authority: Signer<'info>,
}

pub fn emergency_stop_swaps(ctx: Context<EmergencyStopSwaps>) -> Result<()> {
    let automation = &mut ctx.accounts.token_automation;
    let clock = Clock::get()?;
    
    automation.is_paused = true;
    automation.last_updated = clock.unix_timestamp;
    
    // Emit emergency stop event
    emit!(SwapsEmergencyStopped {
        automation: automation.key(),
        emergency_authority: automation.emergency_authority,
        timestamp: clock.unix_timestamp,
    });
    
    msg!("All swap operations emergency stopped");
    Ok(())
}
