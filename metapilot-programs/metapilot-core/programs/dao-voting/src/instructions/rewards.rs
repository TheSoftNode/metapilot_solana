use super::*;

/// Withdraw accumulated rewards from governance participation
pub fn withdraw_governance_rewards(
    ctx: Context<WithdrawGovernanceRewards>,
    amount: u64,
) -> Result<()> {
    let dao_automation = &ctx.accounts.dao_automation;
    let clock = Clock::get()?;
    
    // Check if DAO is active and not paused
    require!(dao_automation.is_active, DaoVotingError::AutomationInactive);
    require!(!dao_automation.is_paused, DaoVotingError::AutomationPaused);
    
    // Verify treasury has sufficient balance
    require!(
        ctx.accounts.treasury.amount >= amount,
        DaoVotingError::InsufficientRewardBalance
    );
    
    // Create transfer instruction
    let transfer_instruction = Transfer {
        from: ctx.accounts.treasury.to_account_info(),
        to: ctx.accounts.user_token_account.to_account_info(),
        authority: ctx.accounts.treasury.to_account_info(),
    };
    
    let dao_key = dao_automation.key();
    let seeds = &[
        TREASURY_SEED,
        dao_key.as_ref(),
        &[ctx.bumps.treasury]
    ];
    let signer_seeds = &[&seeds[..]];
    
    let cpi_context = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        transfer_instruction,
        signer_seeds,
    );
    
    // Execute transfer
    transfer(cpi_context, amount)?;
    
    emit!(GovernanceRewardsWithdrawn {
        user: ctx.accounts.user.key(),
        dao_automation: dao_automation.key(),
        amount,
        reward_type: crate::events::RewardType::VotingParticipation, // Could be dynamic
        timestamp: clock.unix_timestamp,
    });
    
    msg!("Governance rewards withdrawn: {} tokens to {}", amount, ctx.accounts.user.key());
    Ok(())
}

#[derive(Accounts)]
#[instruction(amount: u64)]
pub struct WithdrawGovernanceRewards<'info> {
    #[account(
        seeds = [DAO_AUTOMATION_SEED, dao_automation.authority.as_ref()],
        bump
    )]
    pub dao_automation: Account<'info, DaoAutomation>,
    
    /// Treasury account that holds reward tokens
    #[account(
        mut,
        seeds = [TREASURY_SEED, dao_automation.key().as_ref()],
        bump
    )]
    pub treasury: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub user: Signer<'info>,
    
    /// User's token account to receive rewards
    #[account(
        mut,
        constraint = user_token_account.owner == user.key(),
        constraint = user_token_account.mint == governance_token_mint.key()
    )]
    pub user_token_account: Account<'info, TokenAccount>,
    
    /// Governance token mint
    pub governance_token_mint: Account<'info, Mint>,
    
    pub token_program: Program<'info, Token>,
}
