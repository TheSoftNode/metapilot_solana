use super::*;

/// Initialize a new DAO voting automation account
pub fn initialize_dao_automation(
    ctx: Context<InitializeDaoAutomation>,
    params: InitializeDaoParams,
) -> Result<()> {
    // Validate input parameters
    validate_voting_rules(&params.voting_rules)?;
    
    if let Some(ref ai_settings) = params.ai_automation_settings {
        validate_ai_settings(ai_settings)?;
    }
    
    let dao_automation = &mut ctx.accounts.dao_automation;
    let clock = Clock::get()?;
    
    // Initialize the DAO automation account
    dao_automation.authority = ctx.accounts.authority.key();
    dao_automation.treasury = ctx.accounts.treasury.key();
    dao_automation.voting_rules = params.voting_rules.clone();
    dao_automation.min_proposal_age = params.min_proposal_age;
    dao_automation.max_gas_price = params.max_gas_price;
    dao_automation.is_active = true;
    dao_automation.is_paused = false;
    dao_automation.created_at = clock.unix_timestamp;
    dao_automation.last_updated = clock.unix_timestamp;
    dao_automation.total_proposals = 0;
    dao_automation.total_votes_cast = 0;
    dao_automation.total_rewards_distributed = 0;
    dao_automation.emergency_authority = params.emergency_authority;
    dao_automation.ai_automation_enabled = params.ai_automation_settings.is_some();
    
    emit!(DaoAutomationInitialized {
        authority: dao_automation.authority,
        treasury: dao_automation.treasury,
        voting_rules: params.voting_rules,
        timestamp: clock.unix_timestamp,
    });
    
    msg!(
        "DAO automation initialized for authority: {} with treasury: {}",
        ctx.accounts.authority.key(),
        ctx.accounts.treasury.key()
    );
    
    Ok(())
}

#[derive(Accounts)]
pub struct InitializeDaoAutomation<'info> {
    #[account(
        init,
        payer = authority,
        space = DaoAutomation::SIZE,
        seeds = [DAO_AUTOMATION_SEED, authority.key().as_ref()],
        bump
    )]
    pub dao_automation: Account<'info, DaoAutomation>,
    
    #[account(mut)]
    pub authority: Signer<'info>,
    
    /// Treasury account for storing governance tokens and rewards
    #[account(
        init,
        payer = authority,
        space = 8 + 32 + 32 + 8, // Basic token account space
        seeds = [TREASURY_SEED, dao_automation.key().as_ref()],
        bump
    )]
    pub treasury: Account<'info, TokenAccount>,
    
    /// Governance token mint
    pub governance_token_mint: Account<'info, Mint>,
    
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}
