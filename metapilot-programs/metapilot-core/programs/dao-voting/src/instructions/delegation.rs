use super::*;

/// Delegate voting power to another account
pub fn delegate_voting_power(
    ctx: Context<DelegateVotingPower>,
    delegate: Pubkey,
    amount: u64,
) -> Result<()> {
    let dao_automation = &ctx.accounts.dao_automation;
    let delegation = &mut ctx.accounts.delegation;
    let clock = Clock::get()?;
    
    // Check if DAO is active and not paused
    require!(dao_automation.is_active, DaoVotingError::AutomationInactive);
    require!(!dao_automation.is_paused, DaoVotingError::AutomationPaused);
    
    // Validate delegation amount
    require!(amount >= MIN_DELEGATION_AMOUNT, DaoVotingError::InvalidDelegationAmount);
    
    // Check that user is not delegating to themselves
    require!(
        ctx.accounts.delegator.key() != delegate,
        DaoVotingError::CannotDelegateToSelf
    );
    
    // Verify user has sufficient balance
    check_sufficient_balance(&ctx.accounts.delegator_token_account, amount)?;
    
    // Calculate expiration (default 30 days from now)
    let expires_at = clock.unix_timestamp + (30 * 24 * 60 * 60);
    
    // Initialize delegation
    delegation.delegator = ctx.accounts.delegator.key();
    delegation.delegate = delegate;
    delegation.dao_automation = dao_automation.key();
    delegation.amount = amount;
    delegation.created_at = clock.unix_timestamp;
    delegation.expires_at = expires_at;
    
    emit!(VotingPowerDelegated {
        delegator: ctx.accounts.delegator.key(),
        delegate,
        dao_automation: dao_automation.key(),
        amount,
        expires_at,
        timestamp: clock.unix_timestamp,
    });
    
    msg!(
        "Voting power delegated: {} tokens from {} to {} (expires: {})",
        amount,
        ctx.accounts.delegator.key(),
        delegate,
        expires_at
    );
    
    Ok(())
}

/// Revoke previously delegated voting power
pub fn revoke_delegation(
    ctx: Context<RevokeDelegation>,
    delegate: Pubkey,
) -> Result<()> {
    let dao_automation = &ctx.accounts.dao_automation;
    let delegation = &ctx.accounts.delegation;
    let clock = Clock::get()?;
    
    // Verify delegation exists and is valid
    require!(
        delegation.delegator == ctx.accounts.delegator.key(),
        DaoVotingError::Unauthorized
    );
    require!(delegation.delegate == delegate, DaoVotingError::DelegationNotFound);
    
    let amount = delegation.amount;
    
    emit!(DelegationRevoked {
        delegator: ctx.accounts.delegator.key(),
        delegate,
        dao_automation: dao_automation.key(),
        amount,
        timestamp: clock.unix_timestamp,
    });
    
    msg!(
        "Delegation revoked: {} tokens from {} to {}",
        amount,
        ctx.accounts.delegator.key(),
        delegate
    );
    
    Ok(())
}

#[derive(Accounts)]
#[instruction(delegate: Pubkey, amount: u64)]
pub struct DelegateVotingPower<'info> {
    #[account(
        init,
        payer = delegator,
        space = Delegation::SIZE,
        seeds = [
            DELEGATION_SEED,
            delegator.key().as_ref(),
            delegate.as_ref(),
            dao_automation.key().as_ref()
        ],
        bump
    )]
    pub delegation: Account<'info, Delegation>,
    
    #[account(
        seeds = [DAO_AUTOMATION_SEED, dao_automation.authority.as_ref()],
        bump
    )]
    pub dao_automation: Account<'info, DaoAutomation>,
    
    #[account(mut)]
    pub delegator: Signer<'info>,
    
    /// Delegator's token account to verify sufficient balance
    #[account(
        constraint = delegator_token_account.owner == delegator.key(),
        constraint = delegator_token_account.mint == governance_token_mint.key()
    )]
    pub delegator_token_account: Account<'info, TokenAccount>,
    
    /// Governance token mint
    pub governance_token_mint: Account<'info, Mint>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(delegate: Pubkey)]
pub struct RevokeDelegation<'info> {
    #[account(
        mut,
        close = delegator,
        seeds = [
            DELEGATION_SEED,
            delegator.key().as_ref(),
            delegate.as_ref(),
            dao_automation.key().as_ref()
        ],
        bump
    )]
    pub delegation: Account<'info, Delegation>,
    
    #[account(
        seeds = [DAO_AUTOMATION_SEED, dao_automation.authority.as_ref()],
        bump
    )]
    pub dao_automation: Account<'info, DaoAutomation>,
    
    #[account(mut)]
    pub delegator: Signer<'info>,
}
