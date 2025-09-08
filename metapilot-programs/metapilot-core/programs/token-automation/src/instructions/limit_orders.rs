use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount, Mint};
use crate::state::*;
use crate::constants::*;
use crate::errors::*;
use crate::events::*;
use crate::utils::*;

/// Parameters for limit orders
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct LimitOrderParams {
    pub token_mint_a: Pubkey,
    pub token_mint_b: Pubkey,
    pub amount_in: u64,
    pub min_amount_out: u64,
    pub price_threshold: u64,
    pub order_side: OrderSide,
    pub execution_type: ExecutionType,
    pub max_slippage: u16,
    pub expires_in_seconds: i64,
    pub fill_or_kill: bool,
}

/// Create a new limit order
#[derive(Accounts)]
#[instruction(params: LimitOrderParams)]
pub struct CreateLimitOrder<'info> {
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
        space = LimitOrder::SIZE,
        seeds = [
            LIMIT_ORDER_SEED, 
            token_automation.key().as_ref(),
            owner.key().as_ref(),
            &Clock::get()?.unix_timestamp.to_le_bytes()
        ],
        bump
    )]
    pub limit_order: Account<'info, LimitOrder>,
    
    #[account(mut)]
    pub owner: Signer<'info>,
    
    #[account(
        mut,
        constraint = token_account_a.mint == params.token_mint_a @ TokenAutomationError::InvalidMint,
        constraint = token_account_a.owner == owner.key() @ TokenAutomationError::InvalidTokenAccountOwner,
        constraint = token_account_a.amount >= params.amount_in @ TokenAutomationError::InsufficientBalance,
    )]
    pub token_account_a: Account<'info, TokenAccount>,
    
    #[account(
        constraint = token_account_b.mint == params.token_mint_b @ TokenAutomationError::InvalidMint,
        constraint = token_account_b.owner == owner.key() @ TokenAutomationError::InvalidTokenAccountOwner,
    )]
    pub token_account_b: Account<'info, TokenAccount>,
    
    #[account(
        constraint = mint_a.key() == params.token_mint_a @ TokenAutomationError::InvalidMint
    )]
    pub mint_a: Account<'info, Mint>,
    
    #[account(
        constraint = mint_b.key() == params.token_mint_b @ TokenAutomationError::InvalidMint
    )]
    pub mint_b: Account<'info, Mint>,
    
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}

pub fn create_limit_order(
    ctx: Context<CreateLimitOrder>,
    params: LimitOrderParams,
) -> Result<()> {
    // Validate parameters
    validate_limit_order(&params)?;
    
    let automation = &mut ctx.accounts.token_automation;
    let limit_order = &mut ctx.accounts.limit_order;
    let clock = Clock::get()?;
    
    // Generate unique order ID
    let order_id = generate_order_id(
        &automation.key(),
        clock.unix_timestamp,
        OrderType::LimitOrder,
    );
    
    // Initialize limit order
    limit_order.id = order_id;
    limit_order.owner = ctx.accounts.owner.key();
    limit_order.token_automation = automation.key();
    limit_order.token_mint_a = params.token_mint_a;
    limit_order.token_mint_b = params.token_mint_b;
    limit_order.amount_in = params.amount_in;
    limit_order.min_amount_out = params.min_amount_out;
    limit_order.price_threshold = params.price_threshold;
    limit_order.created_at = clock.unix_timestamp;
    limit_order.expires_at = clock.unix_timestamp + params.expires_in_seconds;
    limit_order.order_side = params.order_side;
    limit_order.status = OrderStatus::Active;
    limit_order.max_slippage = params.max_slippage;
    limit_order.fill_or_kill = params.fill_or_kill;
    
    // Emit order creation event
    emit!(LimitOrderCreated {
        order_id,
        owner: ctx.accounts.owner.key(),
        automation: automation.key(),
        token_mint_a: params.token_mint_a,
        token_mint_b: params.token_mint_b,
        amount_in: params.amount_in,
        price_threshold: params.price_threshold,
        order_side: params.order_side,
        expires_at: limit_order.expires_at,
        timestamp: clock.unix_timestamp,
    });
    
    msg!("Limit order created with ID: {}", order_id);
    Ok(())
}

/// Cancel a limit order
#[derive(Accounts)]
pub struct CancelLimitOrder<'info> {
    #[account(
        mut,
        has_one = owner,
        has_one = token_automation,
        constraint = limit_order.status == OrderStatus::Active @ TokenAutomationError::OrderNotActive,
    )]
    pub limit_order: Account<'info, LimitOrder>,
    
    pub token_automation: Account<'info, TokenAutomation>,
    
    pub owner: Signer<'info>,
}

pub fn cancel_limit_order(ctx: Context<CancelLimitOrder>) -> Result<()> {
    let limit_order = &mut ctx.accounts.limit_order;
    let clock = Clock::get()?;
    
    // Update order status
    limit_order.status = OrderStatus::Cancelled;
    
    // Emit cancellation event
    emit!(LimitOrderCancelled {
        order_id: limit_order.id,
        owner: limit_order.owner,
        automation: limit_order.token_automation,
        timestamp: clock.unix_timestamp,
    });
    
    msg!("Limit order {} cancelled", limit_order.id);
    Ok(())
}

/// Execute a limit order when conditions are met
#[derive(Accounts)]
pub struct ExecuteLimitOrder<'info> {
    #[account(
        mut,
        constraint = limit_order.status == OrderStatus::Active @ TokenAutomationError::OrderNotActive,
        constraint = limit_order.is_valid() @ TokenAutomationError::OrderExpired,
    )]
    pub limit_order: Account<'info, LimitOrder>,
    
    #[account(
        mut,
        constraint = token_automation.is_active @ TokenAutomationError::AutomationInactive,
        constraint = !token_automation.is_paused @ TokenAutomationError::AutomationPaused,
    )]
    pub token_automation: Account<'info, TokenAutomation>,
    
    #[account(
        mut,
        constraint = user_token_account_a.mint == limit_order.token_mint_a @ TokenAutomationError::InvalidMint,
        constraint = user_token_account_a.owner == limit_order.owner @ TokenAutomationError::InvalidTokenAccountOwner,
    )]
    pub user_token_account_a: Account<'info, TokenAccount>,
    
    #[account(
        mut,
        constraint = user_token_account_b.mint == limit_order.token_mint_b @ TokenAutomationError::InvalidMint,
        constraint = user_token_account_b.owner == limit_order.owner @ TokenAutomationError::InvalidTokenAccountOwner,
    )]
    pub user_token_account_b: Account<'info, TokenAccount>,
    
    /// CHECK: DEX program for executing the swap
    pub dex_program: UncheckedAccount<'info>,
    
    /// CHECK: DEX market account
    pub dex_market: UncheckedAccount<'info>,
    
    pub token_program: Program<'info, Token>,
}

pub fn execute_limit_order(
    ctx: Context<ExecuteLimitOrder>,
    current_price: u64,
    amount_out: u64,
) -> Result<()> {
    let limit_order = &mut ctx.accounts.limit_order;
    let automation = &mut ctx.accounts.token_automation;
    let clock = Clock::get()?;
    
    // Check if price condition is met
    require!(
        limit_order.price_condition_met(current_price),
        TokenAutomationError::PriceConditionNotMet
    );
    
    // Check minimum amount out
    require!(
        amount_out >= limit_order.min_amount_out,
        TokenAutomationError::InsufficientAmountOut
    );
    
    // Check slippage
    let expected_amount = (limit_order.amount_in * current_price) / (10u64.pow(6)); // Assuming 6 decimals
    let slippage_bps = if amount_out < expected_amount {
        ((expected_amount - amount_out) * 10000) / expected_amount
    } else {
        0
    };
    
    require!(
        slippage_bps <= limit_order.max_slippage as u64,
        TokenAutomationError::SlippageExceeded
    );
    
    // Update order status
    limit_order.status = OrderStatus::Executed;
    
    // Update automation stats
    automation.total_executions += 1;
    automation.total_volume = automation.total_volume.saturating_add(limit_order.amount_in);
    automation.last_updated = clock.unix_timestamp;
    
    // Calculate and collect execution fee
    let execution_fee = calculate_execution_fee(
        automation.gas_limit,
        LIMIT_ORDER_COMPLEXITY,
        BASE_EXECUTION_FEE,
    );
    
    automation.total_fees_collected = automation.total_fees_collected.saturating_add(execution_fee);
    
    // Emit execution event
    emit!(LimitOrderExecuted {
        order_id: limit_order.id,
        owner: limit_order.owner,
        automation: automation.key(),
        amount_in: limit_order.amount_in,
        amount_out,
        execution_price: current_price,
        execution_fee,
        timestamp: clock.unix_timestamp,
    });
    
    msg!("Limit order {} executed at price {}", limit_order.id, current_price);
    Ok(())
}

/// Update limit order parameters
#[derive(Accounts)]
pub struct UpdateLimitOrder<'info> {
    #[account(
        mut,
        has_one = owner,
        constraint = limit_order.status == OrderStatus::Active @ TokenAutomationError::OrderNotActive,
    )]
    pub limit_order: Account<'info, LimitOrder>,
    
    pub owner: Signer<'info>,
}

pub fn update_limit_order(
    ctx: Context<UpdateLimitOrder>,
    new_price_threshold: Option<u64>,
    new_expires_at: Option<i64>,
    new_max_slippage: Option<u16>,
) -> Result<()> {
    let limit_order = &mut ctx.accounts.limit_order;
    let clock = Clock::get()?;
    
    // Update price threshold if provided
    if let Some(price_threshold) = new_price_threshold {
        require!(
            price_threshold > 0,
            TokenAutomationError::InvalidPriceThreshold
        );
        limit_order.price_threshold = price_threshold;
    }
    
    // Update expiration if provided
    if let Some(expires_at) = new_expires_at {
        require!(
            expires_at > clock.unix_timestamp,
            TokenAutomationError::InvalidExpirationTime
        );
        limit_order.expires_at = expires_at;
    }
    
    // Update slippage if provided
    if let Some(max_slippage) = new_max_slippage {
        require!(
            max_slippage <= MAX_SLIPPAGE_BPS,
            TokenAutomationError::InvalidSlippage
        );
        limit_order.max_slippage = max_slippage;
    }
    
    // Emit update event
    emit!(LimitOrderUpdated {
        order_id: limit_order.id,
        owner: limit_order.owner,
        new_price_threshold,
        new_expires_at,
        new_max_slippage,
        timestamp: clock.unix_timestamp,
    });
    
    msg!("Limit order {} updated", limit_order.id);
    Ok(())
}
