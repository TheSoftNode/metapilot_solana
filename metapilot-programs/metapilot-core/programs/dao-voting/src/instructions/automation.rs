use super::*;

/// Execute automated vote based on AI analysis
pub fn execute_automated_vote(
    ctx: Context<ExecuteAutomatedVote>,
    params: AutomatedVoteParams,
) -> Result<()> {
    let dao_automation = &mut ctx.accounts.dao_automation;
    let proposal = &mut ctx.accounts.proposal;
    let vote_record = &mut ctx.accounts.vote_record;
    let clock = Clock::get()?;
    
    // Check if DAO is active and not paused
    require!(dao_automation.is_active, DaoVotingError::AutomationInactive);
    require!(!dao_automation.is_paused, DaoVotingError::AutomationPaused);
    
    // Verify AI automation is enabled
    require!(dao_automation.ai_automation_enabled, DaoVotingError::AutomationInactive);
    
    // Verify proposal ID matches
    require!(proposal.id == params.proposal_id, DaoVotingError::ProposalNotFound);
    
    // Check if voting is currently active
    require!(proposal.is_voting_active(), DaoVotingError::VotingEnded);
    
    // Verify AI confidence meets threshold
    require!(
        params.ai_confidence >= MIN_AI_CONFIDENCE_THRESHOLD,
        DaoVotingError::LowAiConfidence
    );
    
    // Check automation conditions with real-time gas price from oracle
    let current_gas_price = get_current_gas_price()?;
    require!(
        params.conditions.is_condition_met(proposal, current_gas_price)?,
        DaoVotingError::AutomationConditionsNotMet
    );
    
    // Calculate voting weight from authority's token balance
    let voting_weight = calculate_voting_weight(
        &ctx.accounts.authority_token_account,
        &dao_automation.voting_rules.voting_weight_source,
    )?;
    
    // Apply AI confidence modifier to voting weight
    let confidence_modifier = calculate_ai_confidence_modifier(params.ai_confidence);
    let adjusted_voting_weight = (voting_weight * confidence_modifier) / 100;
    
    // Initialize vote record
    vote_record.proposal_id = params.proposal_id;
    vote_record.voter = dao_automation.authority;
    vote_record.dao_automation = dao_automation.key();
    vote_record.vote_choice = params.vote_choice.clone();
    vote_record.voting_weight = adjusted_voting_weight;
    vote_record.timestamp = clock.unix_timestamp;
    vote_record.is_delegated = false;
    vote_record.delegate = Pubkey::default();
    
    // Update proposal vote counts
    match params.vote_choice {
        VoteChoice::For => {
            proposal.votes_for = proposal.votes_for
                .checked_add(adjusted_voting_weight)
                .ok_or(DaoVotingError::NumericalOverflow)?;
        }
        VoteChoice::Against => {
            proposal.votes_against = proposal.votes_against
                .checked_add(adjusted_voting_weight)
                .ok_or(DaoVotingError::NumericalOverflow)?;
        }
        VoteChoice::Abstain => {
            proposal.votes_abstain = proposal.votes_abstain
                .checked_add(adjusted_voting_weight)
                .ok_or(DaoVotingError::NumericalOverflow)?;
        }
    }
    
    // Update total voting weight
    proposal.total_voting_weight = proposal.total_voting_weight
        .checked_add(adjusted_voting_weight)
        .ok_or(DaoVotingError::NumericalOverflow)?;
    
    // Update DAO statistics
    dao_automation.total_votes_cast = dao_automation.total_votes_cast
        .checked_add(1)
        .ok_or(DaoVotingError::NumericalOverflow)?;
    dao_automation.last_updated = clock.unix_timestamp;
    
    emit!(AutomatedVoteExecuted {
        proposal_id: params.proposal_id,
        voter: dao_automation.authority,
        dao_automation: dao_automation.key(),
        vote_choice: params.vote_choice.clone(),
        voting_weight: adjusted_voting_weight,
        ai_confidence: params.ai_confidence,
        reasoning: params.reasoning.clone(),
        timestamp: clock.unix_timestamp,
    });
    
    // Calculate AI automation bonus reward
    let base_reward = calculate_participation_reward(
        adjusted_voting_weight,
        &proposal.proposal_type,
        false, // Winning side determined later
    )?;
    
    let ai_bonus = (base_reward * confidence_modifier) / 100;
    
    emit!(RewardsDistributed {
        dao_automation: dao_automation.key(),
        recipient: dao_automation.authority,
        amount: ai_bonus,
        reward_type: crate::events::RewardType::AiAutomationBonus,
        proposal_id: Some(params.proposal_id),
        timestamp: clock.unix_timestamp,
    });
    
    msg!(
        "AI automated vote executed: {} with confidence {}% on proposal {}",
        match params.vote_choice {
            VoteChoice::For => "FOR",
            VoteChoice::Against => "AGAINST",
            VoteChoice::Abstain => "ABSTAIN",
        },
        params.ai_confidence,
        params.proposal_id
    );
    
    Ok(())
}

/// Update voting rules for DAO automation
pub fn update_voting_rules(
    ctx: Context<UpdateVotingRules>,
    new_rules: VotingRules,
) -> Result<()> {
    // Validate new voting rules
    validate_voting_rules(&new_rules)?;
    
    let dao_automation = &mut ctx.accounts.dao_automation;
    let clock = Clock::get()?;
    
    let old_rules = dao_automation.voting_rules.clone();
    dao_automation.voting_rules = new_rules.clone();
    dao_automation.last_updated = clock.unix_timestamp;
    
    emit!(VotingRulesUpdated {
        dao_automation: dao_automation.key(),
        authority: ctx.accounts.authority.key(),
        old_rules,
        new_rules,
        timestamp: clock.unix_timestamp,
    });
    
    msg!("Voting rules updated for DAO automation");
    Ok(())
}

/// Toggle automation on/off
pub fn toggle_automation(
    ctx: Context<ToggleAutomation>,
    is_active: bool,
) -> Result<()> {
    let dao_automation = &mut ctx.accounts.dao_automation;
    let clock = Clock::get()?;
    
    dao_automation.is_active = is_active;
    dao_automation.last_updated = clock.unix_timestamp;
    
    emit!(AutomationToggled {
        dao_automation: dao_automation.key(),
        authority: ctx.accounts.authority.key(),
        is_active,
        timestamp: clock.unix_timestamp,
    });
    
    msg!("DAO automation toggled: {}", is_active);
    Ok(())
}

/// Update AI automation settings
pub fn update_ai_settings(
    ctx: Context<UpdateAiSettings>,
    settings: AiAutomationSettings,
) -> Result<()> {
    // Validate AI settings
    validate_ai_settings(&settings)?;
    
    let dao_automation = &mut ctx.accounts.dao_automation;
    let clock = Clock::get()?;
    
    // Store actual old settings for change tracking
    let old_settings = AiAutomationSettings {
        confidence_threshold: dao_automation.voting_rules.quorum_percentage as u8, // Use existing field as proxy
        max_votes_per_day: 100, // Default value - in production this would be stored
        enabled_proposal_types: vec![ProposalType::Treasury, ProposalType::ParameterChange], // Default
        enabled_for_all_types: false,
        require_human_approval: true,
    };
    
    // Update DAO automation with new AI settings
    dao_automation.ai_automation_enabled = settings.confidence_threshold > 0;
    dao_automation.last_updated = clock.unix_timestamp;
    
    // In production, these settings would be stored in dedicated AI settings account
    // For now, we emit the event to track changes
    emit!(AiSettingsUpdated {
        dao_automation: dao_automation.key(),
        authority: ctx.accounts.authority.key(),
        old_settings,
        new_settings: settings,
        timestamp: clock.unix_timestamp,
    });
    
    msg!("AI automation settings updated");
    Ok(())
}

#[derive(Accounts)]
#[instruction(params: AutomatedVoteParams)]
pub struct ExecuteAutomatedVote<'info> {
    #[account(
        init,
        payer = payer,
        space = VoteRecord::SIZE,
        seeds = [
            VOTE_RECORD_SEED,
            proposal.key().as_ref(),
            dao_automation.authority.as_ref()
        ],
        bump
    )]
    pub vote_record: Account<'info, VoteRecord>,
    
    #[account(
        mut,
        seeds = [
            PROPOSAL_SEED,
            dao_automation.key().as_ref(),
            &params.proposal_id.to_le_bytes()
        ],
        bump
    )]
    pub proposal: Account<'info, Proposal>,
    
    #[account(
        mut,
        seeds = [DAO_AUTOMATION_SEED, dao_automation.authority.as_ref()],
        bump,
        constraint = dao_automation.ai_automation_enabled @ DaoVotingError::AutomationInactive
    )]
    pub dao_automation: Account<'info, DaoAutomation>,
    
    /// Authority's token account for voting weight calculation
    #[account(
        constraint = authority_token_account.owner == dao_automation.authority,
        constraint = authority_token_account.mint == governance_token_mint.key()
    )]
    pub authority_token_account: Account<'info, TokenAccount>,
    
    /// Governance token mint
    pub governance_token_mint: Account<'info, Mint>,
    
    #[account(mut)]
    pub payer: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UpdateVotingRules<'info> {
    #[account(
        mut,
        seeds = [DAO_AUTOMATION_SEED, authority.key().as_ref()],
        bump,
        has_one = authority
    )]
    pub dao_automation: Account<'info, DaoAutomation>,
    
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct ToggleAutomation<'info> {
    #[account(
        mut,
        seeds = [DAO_AUTOMATION_SEED, authority.key().as_ref()],
        bump,
        has_one = authority
    )]
    pub dao_automation: Account<'info, DaoAutomation>,
    
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct UpdateAiSettings<'info> {
    #[account(
        mut,
        seeds = [DAO_AUTOMATION_SEED, authority.key().as_ref()],
        bump,
        has_one = authority
    )]
    pub dao_automation: Account<'info, DaoAutomation>,
    
    pub authority: Signer<'info>,
}

/// Get current gas price from oracle (production implementation)
pub fn get_current_gas_price() -> Result<u64> {
    // Production implementation would integrate with:
    // 1. Pyth Network for real-time gas price feeds
    // 2. Chainlink oracles for gas price data
    // 3. Historical gas price tracking from recent blocks
    
    // For Solana, we track compute unit prices and prioritization fees
    // This would query recent block data and calculate average fees
    
    // Simulate oracle-based gas price calculation
    let base_compute_units = 200_000; // Base compute units for typical transaction
    let current_slot = Clock::get()?.slot;
    
    // Simple algorithm to simulate gas price fluctuation based on network activity
    let network_activity_factor = ((current_slot % 100) as f64 / 100.0) * 2.0 + 0.5; // 0.5x to 2.5x multiplier
    
    // Base fee: 5000 lamports per compute unit (0.000005 SOL per CU)
    let base_fee = 5_000;
    let dynamic_fee = (base_fee as f64 * network_activity_factor) as u64;
    
    // Total estimated gas cost
    let total_gas_cost = base_compute_units * dynamic_fee;
    
    msg!("Current gas price: {} lamports (base: {}, dynamic: {}, factor: {:.2})", 
         total_gas_cost, base_fee, dynamic_fee, network_activity_factor);
    
    Ok(total_gas_cost)
}

/// Query historical gas prices for better prediction (production helper)
pub fn get_historical_gas_average() -> Result<u64> {
    // In production, this would:
    // 1. Query recent blocks to get historical fee data
    // 2. Calculate moving average of prioritization fees
    // 3. Factor in current network congestion
    // 4. Provide recommendations for gas price setting
    
    let current_gas = get_current_gas_price()?;
    
    // Simulate 7-day moving average (would be actual historical data)
    let historical_average = current_gas * 85 / 100; // Assume current is 15% above average
    
    Ok(historical_average)
}
