use super::*;

/// Cast a vote on a proposal
pub fn cast_vote(
    ctx: Context<CastVote>,
    proposal_id: u64,
    vote_choice: VoteChoice,
    voting_weight: u64,
) -> Result<()> {
    let dao_automation = &ctx.accounts.dao_automation;
    let proposal = &mut ctx.accounts.proposal;
    let vote_record = &mut ctx.accounts.vote_record;
    let clock = Clock::get()?;
    
    // Check if DAO is active and not paused
    require!(dao_automation.is_active, DaoVotingError::AutomationInactive);
    require!(!dao_automation.is_paused, DaoVotingError::AutomationPaused);
    
    // Verify proposal ID matches
    require!(proposal.id == proposal_id, DaoVotingError::ProposalNotFound);
    
    // Check if voting is currently active
    require!(proposal.is_voting_active(), DaoVotingError::VotingEnded);
    
    // Calculate actual voting weight based on token balance
    let actual_voting_weight = calculate_voting_weight(
        &ctx.accounts.voter_token_account,
        &dao_automation.voting_rules.voting_weight_source,
    )?;
    
    // Verify requested voting weight doesn't exceed actual balance
    require!(
        voting_weight <= actual_voting_weight,
        DaoVotingError::InsufficientVotingWeight
    );
    
    // Initialize vote record
    vote_record.proposal_id = proposal_id;
    vote_record.voter = ctx.accounts.voter.key();
    vote_record.dao_automation = dao_automation.key();
    vote_record.vote_choice = vote_choice.clone();
    vote_record.voting_weight = voting_weight;
    vote_record.timestamp = clock.unix_timestamp;
    vote_record.is_delegated = false;
    vote_record.delegate = Pubkey::default();
    
    // Update proposal vote counts
    match vote_choice {
        VoteChoice::For => {
            proposal.votes_for = proposal.votes_for
                .checked_add(voting_weight)
                .ok_or(DaoVotingError::NumericalOverflow)?;
        }
        VoteChoice::Against => {
            proposal.votes_against = proposal.votes_against
                .checked_add(voting_weight)
                .ok_or(DaoVotingError::NumericalOverflow)?;
        }
        VoteChoice::Abstain => {
            proposal.votes_abstain = proposal.votes_abstain
                .checked_add(voting_weight)
                .ok_or(DaoVotingError::NumericalOverflow)?;
        }
    }
    
    // Update total voting weight
    proposal.total_voting_weight = proposal.total_voting_weight
        .checked_add(voting_weight)
        .ok_or(DaoVotingError::NumericalOverflow)?;
    
    emit!(VoteCast {
        proposal_id,
        voter: ctx.accounts.voter.key(),
        dao_automation: dao_automation.key(),
        vote_choice: vote_choice.clone(),
        voting_weight,
        is_delegated: false,
        delegate: None,
        timestamp: clock.unix_timestamp,
    });
    
    // Calculate and emit participation reward
    let is_winning_side = match vote_choice {
        VoteChoice::For => proposal.votes_for > proposal.votes_against,
        VoteChoice::Against => proposal.votes_against > proposal.votes_for,
        VoteChoice::Abstain => false, // Abstain doesn't have a "winning" side
    };
    
    let reward_amount = calculate_participation_reward(
        voting_weight,
        &proposal.proposal_type,
        is_winning_side,
    )?;
    
    emit!(RewardsDistributed {
        dao_automation: dao_automation.key(),
        recipient: ctx.accounts.voter.key(),
        amount: reward_amount,
        reward_type: crate::events::RewardType::VotingParticipation,
        proposal_id: Some(proposal_id),
        timestamp: clock.unix_timestamp,
    });
    
    msg!("Vote cast: {} with weight {} on proposal {}", 
         match vote_choice {
             VoteChoice::For => "FOR",
             VoteChoice::Against => "AGAINST", 
             VoteChoice::Abstain => "ABSTAIN",
         },
         voting_weight, 
         proposal_id
    );
    
    Ok(())
}

/// Cast a delegated vote
pub fn cast_delegated_vote(
    ctx: Context<CastDelegatedVote>,
    proposal_id: u64,
    vote_choice: VoteChoice,
) -> Result<()> {
    let dao_automation = &ctx.accounts.dao_automation;
    let proposal = &mut ctx.accounts.proposal;
    let vote_record = &mut ctx.accounts.vote_record;
    let delegation = &ctx.accounts.delegation;
    let clock = Clock::get()?;
    
    // Check if DAO is active and not paused
    require!(dao_automation.is_active, DaoVotingError::AutomationInactive);
    require!(!dao_automation.is_paused, DaoVotingError::AutomationPaused);
    
    // Verify proposal ID matches
    require!(proposal.id == proposal_id, DaoVotingError::ProposalNotFound);
    
    // Check if voting is currently active
    require!(proposal.is_voting_active(), DaoVotingError::VotingEnded);
    
    // Verify delegation is valid
    require!(delegation.is_valid(), DaoVotingError::DelegationExpired);
    require!(
        delegation.delegate == ctx.accounts.delegate.key(),
        DaoVotingError::Unauthorized
    );
    
    let voting_weight = delegation.amount;
    
    // Initialize vote record
    vote_record.proposal_id = proposal_id;
    vote_record.voter = delegation.delegator;
    vote_record.dao_automation = dao_automation.key();
    vote_record.vote_choice = vote_choice.clone();
    vote_record.voting_weight = voting_weight;
    vote_record.timestamp = clock.unix_timestamp;
    vote_record.is_delegated = true;
    vote_record.delegate = ctx.accounts.delegate.key();
    
    // Update proposal vote counts
    match vote_choice {
        VoteChoice::For => {
            proposal.votes_for = proposal.votes_for
                .checked_add(voting_weight)
                .ok_or(DaoVotingError::NumericalOverflow)?;
        }
        VoteChoice::Against => {
            proposal.votes_against = proposal.votes_against
                .checked_add(voting_weight)
                .ok_or(DaoVotingError::NumericalOverflow)?;
        }
        VoteChoice::Abstain => {
            proposal.votes_abstain = proposal.votes_abstain
                .checked_add(voting_weight)
                .ok_or(DaoVotingError::NumericalOverflow)?;
        }
    }
    
    // Update total voting weight
    proposal.total_voting_weight = proposal.total_voting_weight
        .checked_add(voting_weight)
        .ok_or(DaoVotingError::NumericalOverflow)?;
    
    emit!(VoteCast {
        proposal_id,
        voter: delegation.delegator,
        dao_automation: dao_automation.key(),
        vote_choice: vote_choice.clone(),
        voting_weight,
        is_delegated: true,
        delegate: Some(ctx.accounts.delegate.key()),
        timestamp: clock.unix_timestamp,
    });
    
    // Reward both delegator and delegate
    let base_reward = calculate_participation_reward(
        voting_weight,
        &proposal.proposal_type,
        false, // Calculate winning side later
    )?;
    
    // Delegator gets 80% of reward
    emit!(RewardsDistributed {
        dao_automation: dao_automation.key(),
        recipient: delegation.delegator,
        amount: (base_reward * 80) / 100,
        reward_type: crate::events::RewardType::VotingParticipation,
        proposal_id: Some(proposal_id),
        timestamp: clock.unix_timestamp,
    });
    
    // Delegate gets 20% of reward
    emit!(RewardsDistributed {
        dao_automation: dao_automation.key(),
        recipient: ctx.accounts.delegate.key(),
        amount: (base_reward * 20) / 100,
        reward_type: crate::events::RewardType::DelegationReward,
        proposal_id: Some(proposal_id),
        timestamp: clock.unix_timestamp,
    });
    
    msg!("Delegated vote cast: {} with weight {} on proposal {} by delegate {}", 
         match vote_choice {
             VoteChoice::For => "FOR",
             VoteChoice::Against => "AGAINST", 
             VoteChoice::Abstain => "ABSTAIN",
         },
         voting_weight, 
         proposal_id,
         ctx.accounts.delegate.key()
    );
    
    Ok(())
}

#[derive(Accounts)]
#[instruction(proposal_id: u64, vote_choice: VoteChoice, voting_weight: u64)]
pub struct CastVote<'info> {
    #[account(
        init,
        payer = voter,
        space = VoteRecord::SIZE,
        seeds = [
            VOTE_RECORD_SEED,
            proposal.key().as_ref(),
            voter.key().as_ref()
        ],
        bump
    )]
    pub vote_record: Account<'info, VoteRecord>,
    
    #[account(
        mut,
        seeds = [
            PROPOSAL_SEED,
            dao_automation.key().as_ref(),
            &proposal_id.to_le_bytes()
        ],
        bump
    )]
    pub proposal: Account<'info, Proposal>,
    
    #[account(
        mut,
        seeds = [DAO_AUTOMATION_SEED, dao_automation.authority.as_ref()],
        bump
    )]
    pub dao_automation: Account<'info, DaoAutomation>,
    
    #[account(mut)]
    pub voter: Signer<'info>,
    
    /// Voter's token account to verify voting weight
    #[account(
        constraint = voter_token_account.owner == voter.key(),
        constraint = voter_token_account.mint == governance_token_mint.key()
    )]
    pub voter_token_account: Account<'info, TokenAccount>,
    
    /// Governance token mint
    pub governance_token_mint: Account<'info, Mint>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(proposal_id: u64, vote_choice: VoteChoice)]
pub struct CastDelegatedVote<'info> {
    #[account(
        init,
        payer = delegate,
        space = VoteRecord::SIZE,
        seeds = [
            VOTE_RECORD_SEED,
            proposal.key().as_ref(),
            delegation.delegator.as_ref()
        ],
        bump
    )]
    pub vote_record: Account<'info, VoteRecord>,
    
    #[account(
        mut,
        seeds = [
            PROPOSAL_SEED,
            dao_automation.key().as_ref(),
            &proposal_id.to_le_bytes()
        ],
        bump
    )]
    pub proposal: Account<'info, Proposal>,
    
    #[account(
        mut,
        seeds = [DAO_AUTOMATION_SEED, dao_automation.authority.as_ref()],
        bump
    )]
    pub dao_automation: Account<'info, DaoAutomation>,
    
    #[account(
        seeds = [
            DELEGATION_SEED,
            delegation.delegator.as_ref(),
            delegate.key().as_ref(),
            dao_automation.key().as_ref()
        ],
        bump
    )]
    pub delegation: Account<'info, Delegation>,
    
    #[account(mut)]
    pub delegate: Signer<'info>,
    
    /// Governance token mint
    pub governance_token_mint: Account<'info, Mint>,
    
    pub system_program: Program<'info, System>,
}
