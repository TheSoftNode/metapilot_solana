use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount, Mint};
use crate::state::*;
use crate::constants::*;
use crate::errors::*;
use crate::events::*;
use crate::utils::*;

/// Parameters for stop loss orders
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct StopLossParams {
    pub token_mint: Pubkey,
    pub amount: u64,
    pub stop_price: u64,
    pub expires_at: i64,
    pub max_slippage: u16,
    pub expires_in_seconds: i64,
}

/// Create a new stop loss order
#[derive(Accounts)]
#[instruction(params: StopLossParams)]
pub struct CreateStopLoss<'info> {
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
        space = StopLoss::SIZE,
        seeds = [
            STOP_LOSS_SEED, 
            token_automation.key().as_ref(),
            owner.key().as_ref(),
            &Clock::get()?.unix_timestamp.to_le_bytes()
        ],
        bump
    )]
    pub stop_loss: Account<'info, StopLoss>,
    
    #[account(mut)]
    pub owner: Signer<'info>,
    
    #[account(
        mut,
        constraint = token_account.mint == params.token_mint @ TokenAutomationError::InvalidMint,
        constraint = token_account.owner == owner.key() @ TokenAutomationError::InvalidTokenAccountOwner,
        constraint = token_account.amount >= params.amount @ TokenAutomationError::InsufficientBalance,
    )]
    pub token_account: Account<'info, TokenAccount>,
    
    #[account(
        constraint = mint.key() == params.token_mint @ TokenAutomationError::InvalidMint
    )]
    pub mint: Account<'info, Mint>,
    
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}

pub fn create_stop_loss(
    ctx: Context<CreateStopLoss>,
    params: StopLossParams,
) -> Result<()> {
    // Validate parameters
    validate_stop_loss(&params)?;
    
    let automation = &mut ctx.accounts.token_automation;
    let stop_loss = &mut ctx.accounts.stop_loss;
    let clock = Clock::get()?;
    
    // Generate unique order ID
    let order_id = generate_order_id(
        &automation.key(),
        clock.unix_timestamp,
        OrderType::StopLoss,
    );
    
    // Initialize stop loss order
    stop_loss.id = order_id;
    stop_loss.owner = ctx.accounts.owner.key();
    stop_loss.token_automation = automation.key();
    stop_loss.token_mint = params.token_mint;
    stop_loss.amount = params.amount;
    stop_loss.stop_price = params.stop_price;
    stop_loss.created_at = clock.unix_timestamp;
    stop_loss.expires_at = clock.unix_timestamp + params.expires_in_seconds;
    stop_loss.status = OrderStatus::Active;
    stop_loss.max_slippage = params.max_slippage;
    
    // Emit order creation event
    emit!(StopLossCreated {
        order_id,
        owner: ctx.accounts.owner.key(),
        automation: automation.key(),
        token_mint: params.token_mint,
        amount: params.amount,
        stop_price: params.stop_price,
        expires_at: stop_loss.expires_at,
        timestamp: clock.unix_timestamp,
    });
    
    msg!("Stop loss order created with ID: {}", order_id);
    Ok(())
}

/// Cancel a stop loss order
#[derive(Accounts)]
pub struct CancelStopLoss<'info> {
    #[account(
        mut,
        has_one = owner,
        has_one = token_automation,
        constraint = stop_loss.status == OrderStatus::Active @ TokenAutomationError::OrderNotActive,
    )]
    pub stop_loss: Account<'info, StopLoss>,
    
    pub token_automation: Account<'info, TokenAutomation>,
    
    pub owner: Signer<'info>,
}

pub fn cancel_stop_loss(ctx: Context<CancelStopLoss>) -> Result<()> {
    let stop_loss = &mut ctx.accounts.stop_loss;
    let clock = Clock::get()?;
    
    // Update order status
    stop_loss.status = OrderStatus::Cancelled;
    
    // Emit cancellation event
    emit!(StopLossCancelled {
        order_id: stop_loss.id,
        owner: stop_loss.owner,
        automation: stop_loss.token_automation,
        timestamp: clock.unix_timestamp,
    });
    
    msg!("Stop loss order {} cancelled", stop_loss.id);
    Ok(())
}

/// Execute a stop loss order when triggered
#[derive(Accounts)]
pub struct ExecuteStopLoss<'info> {
    #[account(
        mut,
        constraint = stop_loss.status == OrderStatus::Active @ TokenAutomationError::OrderNotActive,
        constraint = stop_loss.is_valid() @ TokenAutomationError::OrderExpired,
    )]
    pub stop_loss: Account<'info, StopLoss>,
    
    #[account(
        mut,
        constraint = token_automation.is_active @ TokenAutomationError::AutomationInactive,
        constraint = !token_automation.is_paused @ TokenAutomationError::AutomationPaused,
    )]
    pub token_automation: Account<'info, TokenAutomation>,
    
    #[account(
        mut,
        constraint = user_token_account.mint == stop_loss.token_mint @ TokenAutomationError::InvalidMint,
        constraint = user_token_account.owner == stop_loss.owner @ TokenAutomationError::InvalidTokenAccountOwner,
    )]
    pub user_token_account: Account<'info, TokenAccount>,
    
    #[account(
        mut,
        constraint = destination_token_account.owner == stop_loss.owner @ TokenAutomationError::InvalidTokenAccountOwner,
    )]
    pub destination_token_account: Account<'info, TokenAccount>,
    
    /// CHECK: DEX program for executing the swap
    pub dex_program: UncheckedAccount<'info>,
    
    /// CHECK: DEX market account
    pub dex_market: UncheckedAccount<'info>,
    
    pub token_program: Program<'info, Token>,
}

pub fn execute_stop_loss(
    ctx: Context<ExecuteStopLoss>,
    current_price: u64,
    amount_out: u64,
) -> Result<()> {
    let stop_loss = &mut ctx.accounts.stop_loss;
    let automation = &mut ctx.accounts.token_automation;
    let clock = Clock::get()?;
    
    // Check if stop loss should be triggered
    require!(
        stop_loss.should_trigger(current_price),
        TokenAutomationError::StopLossNotTriggered
    );
    
    // Calculate expected minimum amount considering slippage
    let expected_amount = (stop_loss.amount * current_price) / (10u64.pow(6)); // Assuming 6 decimals
    let slippage_adjusted_min = expected_amount
        .saturating_mul(10000 - stop_loss.max_slippage as u64)
        .saturating_div(10000);
    
    require!(
        amount_out >= slippage_adjusted_min,
        TokenAutomationError::InsufficientAmountOut
    );
    
    // Update order status
    stop_loss.status = OrderStatus::Executed;
    
    // Update automation stats
    automation.total_executions += 1;
    automation.total_volume = automation.total_volume.saturating_add(stop_loss.amount);
    automation.last_updated = clock.unix_timestamp;
    
    // Calculate and collect execution fee
    let execution_fee = calculate_execution_fee(
        automation.gas_limit,
        STOP_LOSS_COMPLEXITY,
        BASE_EXECUTION_FEE,
    );
    
    automation.total_fees_collected = automation.total_fees_collected.saturating_add(execution_fee);
    
    // Emit execution event
    emit!(StopLossExecuted {
        order_id: stop_loss.id,
        owner: stop_loss.owner,
        automation: automation.key(),
        amount_in: stop_loss.amount,
        amount_out,
        trigger_price: current_price,
        execution_fee,
        timestamp: clock.unix_timestamp,
    });
    
    msg!("Stop loss order {} executed at price {}", stop_loss.id, current_price);
    Ok(())
}

/// Update stop loss parameters
#[derive(Accounts)]
pub struct UpdateStopLoss<'info> {
    #[account(
        mut,
        has_one = owner,
        constraint = stop_loss.status == OrderStatus::Active @ TokenAutomationError::OrderNotActive,
    )]
    pub stop_loss: Account<'info, StopLoss>,
    
    pub owner: Signer<'info>,
}

pub fn update_stop_loss(
    ctx: Context<UpdateStopLoss>,
    new_stop_price: Option<u64>,
    new_expires_at: Option<i64>,
    new_max_slippage: Option<u16>,
) -> Result<()> {
    let stop_loss = &mut ctx.accounts.stop_loss;
    let clock = Clock::get()?;
    
    // Update stop price if provided
    if let Some(stop_price) = new_stop_price {
        require!(
            stop_price > 0,
            TokenAutomationError::InvalidStopPrice
        );
        stop_loss.stop_price = stop_price;
    }
    
    // Update expiration if provided
    if let Some(expires_at) = new_expires_at {
        require!(
            expires_at > clock.unix_timestamp,
            TokenAutomationError::InvalidExpirationTime
        );
        stop_loss.expires_at = expires_at;
    }
    
    // Update slippage if provided
    if let Some(max_slippage) = new_max_slippage {
        require!(
            max_slippage <= MAX_SLIPPAGE_BPS,
            TokenAutomationError::InvalidSlippage
        );
        stop_loss.max_slippage = max_slippage;
    }
    
    // Emit update event
    emit!(StopLossUpdated {
        order_id: stop_loss.id,
        owner: stop_loss.owner,
        new_stop_price,
        new_expires_at,
        new_max_slippage,
        timestamp: clock.unix_timestamp,
    });
    
    msg!("Stop loss order {} updated", stop_loss.id);
    Ok(())
}
