use super::*;

/// Emergency pause for critical situations
pub fn emergency_pause(
    ctx: Context<EmergencyPause>,
) -> Result<()> {
    let dao_automation = &mut ctx.accounts.dao_automation;
    let clock = Clock::get()?;
    
    // Only emergency authority can pause
    require!(
        ctx.accounts.emergency_authority.key() == dao_automation.emergency_authority,
        DaoVotingError::EmergencyAuthorityRequired
    );
    
    dao_automation.is_paused = true;
    dao_automation.last_updated = clock.unix_timestamp;
    
    emit!(EmergencyPauseActivated {
        dao_automation: dao_automation.key(),
        emergency_authority: ctx.accounts.emergency_authority.key(),
        reason: "Emergency pause activated".to_string(),
        timestamp: clock.unix_timestamp,
    });
    
    msg!("Emergency pause activated by {}", ctx.accounts.emergency_authority.key());
    Ok(())
}

/// Resume operations after emergency pause
pub fn resume_operations(
    ctx: Context<ResumeOperations>,
) -> Result<()> {
    let dao_automation = &mut ctx.accounts.dao_automation;
    let clock = Clock::get()?;
    
    // Only authority can resume operations
    require!(
        ctx.accounts.authority.key() == dao_automation.authority,
        DaoVotingError::Unauthorized
    );
    
    dao_automation.is_paused = false;
    dao_automation.last_updated = clock.unix_timestamp;
    
    emit!(OperationsResumed {
        dao_automation: dao_automation.key(),
        authority: ctx.accounts.authority.key(),
        timestamp: clock.unix_timestamp,
    });
    
    msg!("Operations resumed by {}", ctx.accounts.authority.key());
    Ok(())
}

#[derive(Accounts)]
pub struct EmergencyPause<'info> {
    #[account(
        mut,
        seeds = [DAO_AUTOMATION_SEED, dao_automation.authority.as_ref()],
        bump
    )]
    pub dao_automation: Account<'info, DaoAutomation>,
    
    /// Emergency authority that can pause operations
    pub emergency_authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct ResumeOperations<'info> {
    #[account(
        mut,
        seeds = [DAO_AUTOMATION_SEED, authority.key().as_ref()],
        bump,
        has_one = authority
    )]
    pub dao_automation: Account<'info, DaoAutomation>,
    
    /// Main authority that can resume operations
    pub authority: Signer<'info>,
}
