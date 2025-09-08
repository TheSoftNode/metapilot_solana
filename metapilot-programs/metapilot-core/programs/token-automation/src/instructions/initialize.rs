use crate::{constants::*, errors::*, events::*, state::*};
use anchor_lang::prelude::*;

// Initialize Token Automation
#[derive(Accounts)]
pub struct InitializeTokenAutomation<'info> {
    #[account(
        init,
        payer = authority,
        space = TokenAutomation::LEN,
        seeds = [AUTOMATION_SEED, authority.key().as_ref()],
        bump
    )]
    pub automation: Account<'info, TokenAutomation>,

    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        init,
        payer = authority,
        space = 8 + 32,
        seeds = [TREASURY_SEED, authority.key().as_ref()],
        bump
    )]
    pub treasury: Account<'info, Treasury>,

    pub system_program: Program<'info, System>,
}

#[derive(Clone, AnchorSerialize, AnchorDeserialize)]
pub struct InitializeTokenParams {
    pub max_slippage: u16,
    pub gas_limit: u64,
    pub ai_automation_enabled: bool,
    pub supported_strategies: Vec<StrategyType>,
    pub emergency_authority: Option<Pubkey>,
}

pub fn initialize_token_automation(
    ctx: Context<InitializeTokenAutomation>,
    params: InitializeTokenParams,
) -> Result<()> {
    let automation = &mut ctx.accounts.automation;
    let treasury = &mut ctx.accounts.treasury;
    let authority = &ctx.accounts.authority;

    // Validate parameters
    require!(
        params.max_slippage <= MAX_SLIPPAGE_BPS,
        TokenAutomationError::InvalidSlippage
    );
    require!(
        params.gas_limit >= MIN_GAS_LIMIT && params.gas_limit <= MAX_GAS_LIMIT,
        TokenAutomationError::InvalidGasLimit
    );
    require!(
        !params.supported_strategies.is_empty(),
        TokenAutomationError::InvalidStrategy
    );

    // Initialize automation account
    automation.authority = authority.key();
    automation.treasury = treasury.key();
    automation.supported_strategies = params.supported_strategies.clone();
    automation.max_slippage = params.max_slippage;
    automation.gas_limit = params.gas_limit;
    automation.is_active = true;
    automation.is_paused = false;
    automation.created_at = Clock::get()?.unix_timestamp;
    automation.last_updated = Clock::get()?.unix_timestamp;
    automation.total_executions = 0;
    automation.total_volume = 0;
    automation.total_fees_collected = 0;
    automation.emergency_authority = params.emergency_authority.unwrap_or(authority.key());
    automation.ai_automation_enabled = params.ai_automation_enabled;

    // Initialize treasury account
    treasury.authority = authority.key();
    treasury.balance = 0;

    // Emit initialization event
    emit!(TokenAutomationInitialized {
        automation: automation.key(),
        authority: authority.key(),
        treasury: treasury.key(),
        supported_strategies: params.supported_strategies,
        max_slippage: params.max_slippage,
        timestamp: Clock::get()?.unix_timestamp,
    });

    Ok(())
}

// Update Automation Settings
#[derive(Accounts)]
pub struct UpdateAutomationSettings<'info> {
    #[account(
        mut,
        seeds = [AUTOMATION_SEED, authority.key().as_ref()],
        bump
    )]
    pub automation: Account<'info, TokenAutomation>,

    #[account(mut)]
    pub authority: Signer<'info>,
}

#[derive(Clone, AnchorSerialize, AnchorDeserialize)]
pub struct UpdateParams {
    pub max_slippage: Option<u16>,
    pub gas_limit: Option<u64>,
    pub ai_automation_enabled: Option<bool>,
    pub emergency_authority: Option<Pubkey>,
}

pub fn update_automation_settings(
    ctx: Context<UpdateAutomationSettings>,
    params: UpdateParams,
) -> Result<()> {
    let automation = &mut ctx.accounts.automation;

    // Verify authority
    require!(
        automation.authority == ctx.accounts.authority.key(),
        TokenAutomationError::Unauthorized
    );

    // Update parameters if provided
    if let Some(max_slippage) = params.max_slippage {
        require!(
            max_slippage <= MAX_SLIPPAGE_BPS,
            TokenAutomationError::InvalidSlippage
        );
        automation.max_slippage = max_slippage;
    }

    if let Some(gas_limit) = params.gas_limit {
        require!(
            gas_limit >= MIN_GAS_LIMIT && gas_limit <= MAX_GAS_LIMIT,
            TokenAutomationError::InvalidGasLimit
        );
        automation.gas_limit = gas_limit;
    }

    if let Some(ai_automation_enabled) = params.ai_automation_enabled {
        automation.ai_automation_enabled = ai_automation_enabled;
    }

    if let Some(emergency_authority) = params.emergency_authority {
        automation.emergency_authority = emergency_authority;
    }

    automation.last_updated = Clock::get()?.unix_timestamp;

    // Emit update event
    emit!(AutomationSettingsUpdated {
        automation: automation.key(),
        authority: ctx.accounts.authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
    });

    Ok(())
}

// Toggle Automation State
#[derive(Accounts)]
pub struct ToggleAutomationState<'info> {
    #[account(
        mut,
        seeds = [AUTOMATION_SEED, authority.key().as_ref()],
        bump
    )]
    pub automation: Account<'info, TokenAutomation>,

    #[account(mut)]
    pub authority: Signer<'info>,
}

pub fn toggle_automation_state(ctx: Context<ToggleAutomationState>, active: bool) -> Result<()> {
    let automation = &mut ctx.accounts.automation;

    // Verify authority
    require!(
        automation.authority == ctx.accounts.authority.key(),
        TokenAutomationError::Unauthorized
    );

    automation.is_active = active;
    automation.last_updated = Clock::get()?.unix_timestamp;

    // Emit state change event
    emit!(AutomationStateChanged {
        automation: automation.key(),
        is_active: active,
        timestamp: Clock::get()?.unix_timestamp,
    });

    Ok(())
}

// Emergency Pause
#[derive(Accounts)]
pub struct EmergencyPause<'info> {
    #[account(
        mut,
        seeds = [AUTOMATION_SEED, emergency_authority.key().as_ref()],
        bump
    )]
    pub automation: Account<'info, TokenAutomation>,

    #[account(mut)]
    pub emergency_authority: Signer<'info>,
}

pub fn emergency_pause(ctx: Context<EmergencyPause>) -> Result<()> {
    let automation = &mut ctx.accounts.automation;

    // Verify emergency authority
    require!(
        automation.emergency_authority == ctx.accounts.emergency_authority.key(),
        TokenAutomationError::Unauthorized
    );

    automation.is_paused = true;
    automation.last_updated = Clock::get()?.unix_timestamp;

    // Emit emergency pause event
    emit!(EmergencyPauseActivated {
        automation: automation.key(),
        emergency_authority: ctx.accounts.emergency_authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
    });

    Ok(())
}

// Resume From Pause
#[derive(Accounts)]
pub struct ResumeFromPause<'info> {
    #[account(
        mut,
        seeds = [AUTOMATION_SEED, authority.key().as_ref()],
        bump
    )]
    pub automation: Account<'info, TokenAutomation>,

    #[account(mut)]
    pub authority: Signer<'info>,
}

pub fn resume_from_pause(ctx: Context<ResumeFromPause>) -> Result<()> {
    let automation = &mut ctx.accounts.automation;

    // Verify authority
    require!(
        automation.authority == ctx.accounts.authority.key(),
        TokenAutomationError::Unauthorized
    );

    automation.is_paused = false;
    automation.last_updated = Clock::get()?.unix_timestamp;

    // Emit resume event
    emit!(EmergencyPauseDeactivated {
        automation: automation.key(),
        resumed_by: ctx.accounts.authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
    });

    Ok(())
}

// Transfer Authority
#[derive(Accounts)]
pub struct TransferAuthority<'info> {
    #[account(
        mut,
        seeds = [AUTOMATION_SEED, current_authority.key().as_ref()],
        bump
    )]
    pub automation: Account<'info, TokenAutomation>,

    #[account(mut)]
    pub current_authority: Signer<'info>,

    /// CHECK: New authority account
    pub new_authority: AccountInfo<'info>,
}

pub fn transfer_authority(ctx: Context<TransferAuthority>) -> Result<()> {
    let automation = &mut ctx.accounts.automation;

    // Verify current authority
    require!(
        automation.authority == ctx.accounts.current_authority.key(),
        TokenAutomationError::Unauthorized
    );

    let old_authority = automation.authority;
    automation.authority = ctx.accounts.new_authority.key();
    automation.last_updated = Clock::get()?.unix_timestamp;

    // Emit authority transfer event
    emit!(AuthorityTransferred {
        automation: automation.key(),
        old_authority,
        new_authority: ctx.accounts.new_authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
    });

    Ok(())
}

// Close Automation
#[derive(Accounts)]
pub struct CloseAutomation<'info> {
    #[account(
        mut,
        close = authority,
        seeds = [AUTOMATION_SEED, authority.key().as_ref()],
        bump
    )]
    pub automation: Account<'info, TokenAutomation>,

    #[account(mut)]
    pub authority: Signer<'info>,
}

pub fn close_automation(ctx: Context<CloseAutomation>) -> Result<()> {
    let automation = &ctx.accounts.automation;

    // Verify authority
    require!(
        automation.authority == ctx.accounts.authority.key(),
        TokenAutomationError::Unauthorized
    );

    // Emit closure event
    emit!(AutomationClosed {
        automation: automation.key(),
        authority: ctx.accounts.authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
    });

    // Account will be closed automatically by the close constraint
    Ok(())
}
