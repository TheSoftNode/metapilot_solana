use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount};
use crate::state::*;
use crate::constants::*;
use crate::errors::*;
use crate::events::*;
use crate::utils::*;

/// Parameters for arbitrage opportunities
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct ArbitrageParams {
    pub token_mint_a: Pubkey,
    pub token_mint_b: Pubkey,
    pub dex_a: Pubkey,
    pub dex_b: Pubkey,
    pub amount: u64,
    pub expected_profit_bps: u64,
}

/// Create a new arbitrage opportunity
#[derive(Accounts)]
#[instruction(params: ArbitrageParams)]
pub struct CreateArbitrageOpportunity<'info> {
    #[account(
        mut,
        constraint = token_automation.is_active @ TokenAutomationError::AutomationInactive,
        constraint = !token_automation.is_paused @ TokenAutomationError::AutomationPaused,
    )]
    pub token_automation: Account<'info, TokenAutomation>,
    
    #[account(
        init,
        payer = discoverer,
        space = ArbitrageOpportunity::SIZE,
        seeds = [
            ARBITRAGE_OPPORTUNITY_SEED, 
            token_automation.key().as_ref(),
            discoverer.key().as_ref(),
            &Clock::get()?.unix_timestamp.to_le_bytes()
        ],
        bump
    )]
    pub arbitrage_opportunity: Account<'info, ArbitrageOpportunity>,
    
    #[account(mut)]
    pub discoverer: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

pub fn create_arbitrage_opportunity(
    ctx: Context<CreateArbitrageOpportunity>,
    params: ArbitrageParams,
) -> Result<()> {
    // Validate arbitrage parameters
    validate_arbitrage_params(&params)?;
    
    let automation = &ctx.accounts.token_automation;
    let opportunity = &mut ctx.accounts.arbitrage_opportunity;
    let clock = Clock::get()?;
    
    // Generate unique opportunity ID
    let opportunity_id = generate_order_id(
        &automation.key(),
        clock.unix_timestamp,
        OrderType::LimitOrder, // Reuse for arbitrage
    );
    
    // Initialize arbitrage opportunity
    opportunity.id = opportunity_id;
    opportunity.discoverer = ctx.accounts.discoverer.key();
    opportunity.token_mint_a = params.token_mint_a;
    opportunity.token_mint_b = params.token_mint_b;
    opportunity.dex_a = params.dex_a;
    opportunity.dex_b = params.dex_b;
    opportunity.price_diff_bps = params.expected_profit_bps;
    opportunity.max_amount = params.amount;
    opportunity.discovered_at = clock.unix_timestamp;
    opportunity.expires_at = clock.unix_timestamp + DEFAULT_ARBITRAGE_EXPIRY;
    opportunity.status = OpportunityStatus::Active;
    
    // Emit opportunity creation event
    emit!(ArbitrageOpportunityCreated {
        opportunity_id,
        discoverer: ctx.accounts.discoverer.key(),
        automation: automation.key(),
        token_mint_a: params.token_mint_a,
        token_mint_b: params.token_mint_b,
        dex_a: params.dex_a,
        dex_b: params.dex_b,
        expected_profit_bps: params.expected_profit_bps,
        max_amount: params.amount,
        expires_at: opportunity.expires_at,
        timestamp: clock.unix_timestamp,
    });
    
    msg!("Arbitrage opportunity created with ID: {}", opportunity_id);
    Ok(())
}

/// Execute arbitrage opportunity
#[derive(Accounts)]
pub struct ExecuteArbitrage<'info> {
    #[account(
        mut,
        constraint = arbitrage_opportunity.status == OpportunityStatus::Active @ TokenAutomationError::OpportunityNotActive,
        constraint = arbitrage_opportunity.is_valid() @ TokenAutomationError::OpportunityExpired,
    )]
    pub arbitrage_opportunity: Account<'info, ArbitrageOpportunity>,
    
    #[account(
        mut,
        constraint = token_automation.is_active @ TokenAutomationError::AutomationInactive,
        constraint = !token_automation.is_paused @ TokenAutomationError::AutomationPaused,
    )]
    pub token_automation: Account<'info, TokenAutomation>,
    
    #[account(
        mut,
        constraint = executor_token_account_a.mint == arbitrage_opportunity.token_mint_a @ TokenAutomationError::InvalidMint,
        constraint = executor_token_account_a.owner == executor.key() @ TokenAutomationError::InvalidTokenAccountOwner,
    )]
    pub executor_token_account_a: Account<'info, TokenAccount>,
    
    #[account(
        mut,
        constraint = executor_token_account_b.mint == arbitrage_opportunity.token_mint_b @ TokenAutomationError::InvalidMint,
        constraint = executor_token_account_b.owner == executor.key() @ TokenAutomationError::InvalidTokenAccountOwner,
    )]
    pub executor_token_account_b: Account<'info, TokenAccount>,
    
    pub executor: Signer<'info>,
    
    /// CHECK: DEX A program
    pub dex_a_program: UncheckedAccount<'info>,
    
    /// CHECK: DEX A market
    pub dex_a_market: UncheckedAccount<'info>,
    
    /// CHECK: DEX B program
    pub dex_b_program: UncheckedAccount<'info>,
    
    /// CHECK: DEX B market
    pub dex_b_market: UncheckedAccount<'info>,
    
    pub token_program: Program<'info, Token>,
}

pub fn execute_arbitrage(
    ctx: Context<ExecuteArbitrage>,
    amount: u64,
    price_a: u64,
    price_b: u64,
) -> Result<()> {
    let opportunity = &mut ctx.accounts.arbitrage_opportunity;
    let automation = &mut ctx.accounts.token_automation;
    let clock = Clock::get()?;
    
    // Validate execution amount
    require!(
        amount > 0 && amount <= opportunity.max_amount,
        TokenAutomationError::InvalidArbitrageAmount
    );
    
    // Calculate actual profit in basis points
    let actual_profit_bps = if price_b > price_a {
        ((price_b - price_a) * 10000) / price_a
    } else {
        return Err(TokenAutomationError::InsufficientProfit.into());
    };
    
    // Ensure minimum profit threshold is met
    require!(
        actual_profit_bps >= MIN_ARBITRAGE_PROFIT_BPS.into(),
        TokenAutomationError::InsufficientProfit
    );
    
    // Check executor has sufficient balance
    require!(
        ctx.accounts.executor_token_account_a.amount >= amount,
        TokenAutomationError::InsufficientBalance
    );
    
    // Update opportunity status
    opportunity.status = OpportunityStatus::Executed;
    
    // Update automation stats
    automation.total_executions += 1;
    automation.total_volume = automation.total_volume.saturating_add(amount);
    automation.last_updated = clock.unix_timestamp;
    
    // Calculate execution fee and profit sharing
    let execution_fee = calculate_execution_fee(
        automation.gas_limit,
        ARBITRAGE_COMPLEXITY,
        BASE_EXECUTION_FEE,
    );
    
    let profit_amount = (amount * actual_profit_bps) / 10000;
    let discoverer_reward = (profit_amount * DISCOVERER_REWARD_BPS) / 10000;
    
    automation.total_fees_collected = automation.total_fees_collected
        .saturating_add(execution_fee)
        .saturating_add(discoverer_reward);
    
    // Emit arbitrage execution event
    emit!(ArbitrageExecuted {
        token_automation: automation.key(),
        token_mint_a: opportunity.token_mint_a,
        token_mint_b: opportunity.token_mint_b,
        amount_in: amount,
        profit: actual_profit_bps,
        executor: ctx.accounts.executor.key(),
        dex_a: opportunity.dex_a,
        dex_b: opportunity.dex_b,
        timestamp: Clock::get()?.unix_timestamp,
    });
    
    msg!(
        "Arbitrage executed: {} profit with {}% gain",
        amount,
        actual_profit_bps as f64 / 100.0
    );
    
    Ok(())
}

/// Cancel arbitrage opportunity
#[derive(Accounts)]
pub struct CancelArbitrageOpportunity<'info> {
    #[account(
        mut,
        has_one = discoverer,
        constraint = arbitrage_opportunity.status == OpportunityStatus::Active @ TokenAutomationError::OpportunityNotActive,
    )]
    pub arbitrage_opportunity: Account<'info, ArbitrageOpportunity>,
    
    pub discoverer: Signer<'info>,
}

pub fn cancel_arbitrage_opportunity(ctx: Context<CancelArbitrageOpportunity>) -> Result<()> {
    let opportunity = &mut ctx.accounts.arbitrage_opportunity;
    let clock = Clock::get()?;
    
    opportunity.status = OpportunityStatus::Invalidated;
    
    // Emit cancellation event
    emit!(ArbitrageOpportunityCancelled {
        opportunity_id: opportunity.id,
        discoverer: opportunity.discoverer,
        timestamp: clock.unix_timestamp,
    });
    
    msg!("Arbitrage opportunity {} cancelled", opportunity.id);
    Ok(())
}

/// Scan for arbitrage opportunities
#[derive(Accounts)]
pub struct ScanArbitrageOpportunities<'info> {
    #[account(
        constraint = token_automation.is_active @ TokenAutomationError::AutomationInactive,
        constraint = !token_automation.is_paused @ TokenAutomationError::AutomationPaused,
    )]
    pub token_automation: Account<'info, TokenAutomation>,
    
    pub scanner: Signer<'info>,
    
    /// CHECK: Price oracle account A
    pub price_oracle_a: UncheckedAccount<'info>,
    
    /// CHECK: Price oracle account B
    pub price_oracle_b: UncheckedAccount<'info>,
}

pub fn scan_arbitrage_opportunities(
    ctx: Context<ScanArbitrageOpportunities>,
    token_pairs: Vec<(Pubkey, Pubkey)>,
    dex_pairs: Vec<(Pubkey, Pubkey)>,
) -> Result<()> {
    let automation = &ctx.accounts.token_automation;
    let clock = Clock::get()?;
    
    // Validate scan parameters
    require!(
        token_pairs.len() <= MAX_SCAN_PAIRS,
        TokenAutomationError::TooManyPairsToScan
    );
    
    require!(
        token_pairs.len() == dex_pairs.len(),
        TokenAutomationError::MismatchedPairCount
    );
    
    let mut opportunities_found = 0u32;
    
    // Scan each token pair across DEX pairs
    for (i, (token_a, token_b)) in token_pairs.iter().enumerate() {
        let (dex_a, dex_b) = dex_pairs[i];
        
        // Simulate price checking (in real implementation, this would query price oracles)
        // For now, we'll emit a scan event
        msg!(
            "Scanning arbitrage for tokens {:?} - {:?} across DEXs {:?} - {:?}",
            token_a, token_b, dex_a, dex_b
        );
        
        opportunities_found += 1;
    }
    
    // Emit scan completion event
    emit!(ArbitrageScanCompleted {
        automation: automation.key(),
        scanner: ctx.accounts.scanner.key(),
        pairs_scanned: token_pairs.len() as u32,
        opportunities_found,
        timestamp: clock.unix_timestamp,
    });
    
    msg!("Arbitrage scan completed: {} pairs scanned, {} opportunities found", 
         token_pairs.len(), opportunities_found);
    
    Ok(())
}
