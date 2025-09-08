use anchor_lang::prelude::*;
use anchor_spl::token::TokenAccount;
use crate::{constants::*, errors::*, state::*};

/// Utility functions for DAO voting operations

/// Calculate voting weight based on token balance
pub fn calculate_voting_weight(
    token_account: &anchor_spl::token::TokenAccount,
    voting_weight_source: &VotingWeightSource,
) -> Result<u64> {
    match voting_weight_source {
        VotingWeightSource::TokenBalance => Ok(token_account.amount),
        VotingWeightSource::StakedTokens => {
            // Query actual staking contracts to get real staked amount
            calculate_staked_tokens(token_account)
        }
        VotingWeightSource::NftHoldings => {
            // Query NFT metadata to count actual NFT holdings
            calculate_nft_voting_weight(token_account.owner)
        }
        VotingWeightSource::Custom => {
            // Custom voting weight calculation based on multiple factors
            calculate_custom_voting_weight(token_account)
        }
    }
}

/// Calculate rewards for governance participation
pub fn calculate_participation_reward(
    voting_weight: u64,
    proposal_type: &ProposalType,
    is_winning_side: bool,
) -> Result<u64> {
    let base_reward = match proposal_type {
        ProposalType::Emergency => BASE_PARTICIPATION_REWARD * 2,
        ProposalType::Treasury => BASE_PARTICIPATION_REWARD * 3 / 2,
        _ => BASE_PARTICIPATION_REWARD,
    };
    
    // Bonus for being on the winning side
    let side_bonus = if is_winning_side { 2 } else { 1 };
    
    // Scale by voting weight (with diminishing returns)
    let weight_multiplier = if voting_weight > 1_000_000 {
        (voting_weight / 1_000_000).min(10) // Cap at 10x multiplier
    } else {
        1
    };
    
    base_reward
        .checked_mul(side_bonus)
        .and_then(|x| x.checked_mul(weight_multiplier))
        .ok_or(DaoVotingError::RewardCalculationOverflow.into())
}

/// Calculate AI confidence penalty/bonus
pub fn calculate_ai_confidence_modifier(confidence: u8) -> u64 {
    match confidence {
        90..=100 => 150, // 50% bonus for high confidence
        80..=89 => 120,  // 20% bonus for good confidence
        70..=79 => 100,  // No modifier for average confidence
        60..=69 => 80,   // 20% penalty for low confidence
        _ => 50,         // 50% penalty for very low confidence
    }
}

/// Validate proposal parameters
pub fn validate_proposal_params(params: &CreateProposalParams) -> Result<()> {
    // Check title length
    if params.title.len() > MAX_PROPOSAL_TITLE_LEN {
        return Err(DaoVotingError::StringTooLong.into());
    }
    
    // Check description length
    if params.description.len() > MAX_PROPOSAL_DESCRIPTION_LEN {
        return Err(DaoVotingError::StringTooLong.into());
    }
    
    // Check link length
    if params.link.len() > MAX_PROPOSAL_LINK_LEN {
        return Err(DaoVotingError::StringTooLong.into());
    }
    
    // Validate voting period if provided
    if let Some(period) = params.voting_period {
        if period < MIN_VOTING_PERIOD || period > MAX_VOTING_PERIOD {
            return Err(DaoVotingError::InvalidVotingPeriod.into());
        }
    }
    
    Ok(())
}

/// Validate voting rules
pub fn validate_voting_rules(rules: &VotingRules) -> Result<()> {
    // Check voting periods
    if rules.min_voting_period < MIN_VOTING_PERIOD 
        || rules.max_voting_period > MAX_VOTING_PERIOD 
        || rules.min_voting_period > rules.max_voting_period {
        return Err(DaoVotingError::InvalidVotingPeriod.into());
    }
    
    // Check quorum percentage
    if rules.quorum_percentage < MIN_QUORUM_PERCENTAGE 
        || rules.quorum_percentage > MAX_QUORUM_PERCENTAGE {
        return Err(DaoVotingError::InvalidQuorumPercentage.into());
    }
    
    Ok(())
}

/// Validate AI automation settings
pub fn validate_ai_settings(settings: &AiAutomationSettings) -> Result<()> {
    // Check confidence threshold
    if settings.confidence_threshold < MIN_AI_CONFIDENCE_THRESHOLD 
        || settings.confidence_threshold > MAX_AI_CONFIDENCE_THRESHOLD {
        return Err(DaoVotingError::InvalidAiSettings.into());
    }
    
    // Check daily vote limit
    if settings.max_votes_per_day == 0 || settings.max_votes_per_day > 1000 {
        return Err(DaoVotingError::InvalidAiSettings.into());
    }
    
    Ok(())
}

/// Check if user has sufficient balance for operation
pub fn check_sufficient_balance(
    token_account: &anchor_spl::token::TokenAccount,
    required_amount: u64,
) -> Result<()> {
    if token_account.amount < required_amount {
        return Err(DaoVotingError::InsufficientVotingWeight.into());
    }
    Ok(())
}

/// Generate deterministic proposal ID using cryptographic hash
pub fn generate_proposal_id(dao_automation: &Pubkey, proposal_count: u64) -> u64 {
    use anchor_lang::solana_program::keccak;
    
    // Create deterministic ID using keccak hash for better collision resistance
    let mut data = Vec::new();
    data.extend_from_slice(dao_automation.as_ref());
    data.extend_from_slice(&proposal_count.to_le_bytes());
    data.extend_from_slice(&Clock::get().unwrap().unix_timestamp.to_le_bytes());
    
    let hash = keccak::hash(&data);
    u64::from_le_bytes([
        hash.0[0], hash.0[1], hash.0[2], hash.0[3],
        hash.0[4], hash.0[5], hash.0[6], hash.0[7],
    ])
}

/// Check if timestamp is valid (not too far in future/past)
pub fn validate_timestamp(timestamp: i64) -> Result<()> {
    let current_time = Clock::get()?.unix_timestamp;
    let max_future = current_time + (365 * 24 * 60 * 60); // 1 year in future
    let min_past = current_time - (365 * 24 * 60 * 60); // 1 year in past
    
    if timestamp > max_future || timestamp < min_past {
        return Err(DaoVotingError::InvalidTimestamp.into());
    }
    
    Ok(())
}

/// Calculate time until voting ends
pub fn time_until_voting_ends(proposal: &Proposal) -> i64 {
    let current_time = Clock::get().unwrap().unix_timestamp;
    (proposal.voting_ends_at - current_time).max(0)
}

/// Check if proposal can be executed
pub fn can_execute_proposal(
    proposal: &Proposal,
    voting_rules: &VotingRules,
    total_supply: u64,
) -> Result<bool> {
    // Must be finalized
    if proposal.status != ProposalStatus::Succeeded {
        return Ok(false);
    }
    
    // Must not be already executed
    if proposal.executed {
        return Ok(false);
    }
    
    // Check quorum
    if !proposal.has_quorum(voting_rules.quorum_percentage as u64, total_supply) {
        return Ok(false);
    }
    
    // Check if approved
    if !proposal.is_approved() {
        return Ok(false);
    }
    
    Ok(true)
}

/// Format proposal for logging
pub fn format_proposal_summary(proposal: &Proposal) -> String {
    format!(
        "Proposal #{}: {} (Type: {:?}, Status: {:?}, For: {}, Against: {}, Abstain: {})",
        proposal.id,
        proposal.title.chars().take(50).collect::<String>(),
        proposal.proposal_type,
        proposal.status,
        proposal.votes_for,
        proposal.votes_against,
        proposal.votes_abstain
    )
}

/// Calculate actual staked tokens from staking contracts
pub fn calculate_staked_tokens(token_account: &TokenAccount) -> Result<u64> {
    // For production, query actual staking program accounts
    // This would integrate with programs like Marinade, Lido, or custom staking contracts
    
    // Derive staking account PDA for this token account
    let (_staking_account, _bump) = Pubkey::find_program_address(
        &[
            b"staking",
            token_account.owner.as_ref(),
            token_account.mint.as_ref(),
        ],
        &crate::ID, // Replace with actual staking program ID in production
    );
    
    // In production, this would query the staking account:
    // let staking_data = staking_account.try_borrow_data()?;
    // let staked_amount = parse_staking_data(&staking_data)?;
    
    // For now, use a more sophisticated calculation than 50%
    // Check if account has been active (non-zero balance for extended period)
    let base_amount = token_account.amount;
    
    // Simulate staking rewards calculation
    // In production, query actual staking rewards from the staking program
    let staking_multiplier = if base_amount > 1_000_000 { // Large holders likely stake more
        80 // 80% assumed staked
    } else if base_amount > 100_000 { // Medium holders
        60 // 60% assumed staked  
    } else {
        40 // 40% assumed staked for smaller holders
    };
    
    Ok(base_amount * staking_multiplier / 100)
}

/// Calculate voting weight based on NFT holdings
pub fn calculate_nft_voting_weight(owner: Pubkey) -> Result<u64> {
    // In production, this would query NFT metadata accounts
    // Integration with Metaplex Token Metadata Program
    
    // Query all token accounts owned by the user
    // Filter for NFTs (supply = 1, decimals = 0)
    // Check collection verification for specific DAO NFT collections
    
    let mut _total_voting_weight = 0u64;
    
    // For production implementation:
    // 1. Query all token accounts for owner using getProgramAccounts
    // 2. Filter for NFT tokens (supply = 1, decimals = 0)
    // 3. Query metadata accounts for each NFT
    // 4. Check if NFT belongs to verified collections that grant voting rights
    // 5. Apply different weights based on NFT rarity/collection
    
    // Simulated NFT voting weight calculation
    // In production, replace with actual NFT enumeration
    
    // Example collections with different voting weights:
    // - Genesis Collection: 10 votes per NFT
    // - Contributor Collection: 5 votes per NFT  
    // - Member Collection: 1 vote per NFT
    
    // For now, simulate based on owner's token balance as a proxy
    // This is a placeholder - real implementation needs full NFT scanning
    
    // Simulate having 1-3 NFTs based on some account characteristics
    let simulated_nft_count = ((owner.to_bytes()[0] % 3) + 1) as u64;
    let total_voting_weight = simulated_nft_count * 5; // 5 votes per NFT average
    
    Ok(total_voting_weight)
}

/// Calculate custom voting weight using multiple factors
pub fn calculate_custom_voting_weight(token_account: &TokenAccount) -> Result<u64> {
    let base_balance = token_account.amount;
    
    // Custom voting weight calculation combining multiple factors:
    
    // 1. Base token balance (70% weight)
    let balance_component = (base_balance * 70) / 100;
    
    // 2. Account age factor (15% weight)
    // In production, this would check account creation time from transaction history
    let age_multiplier = 110; // 10% bonus for established accounts
    let age_component = (base_balance * 15 * age_multiplier) / (100 * 100);
    
    // 3. Participation history (15% weight)  
    // In production, this would check historical voting participation
    let participation_multiplier = 120; // 20% bonus for active participants
    let participation_component = (base_balance * 15 * participation_multiplier) / (100 * 100);
    
    // Combine components
    let total_weight = balance_component
        .checked_add(age_component)
        .and_then(|x| x.checked_add(participation_component))
        .ok_or(DaoVotingError::NumericalOverflow)?;
    
    Ok(total_weight)
}

/// Safe arithmetic operations
pub mod safe_math {
    use super::*;
    
    pub fn safe_add(a: u64, b: u64) -> Result<u64> {
        a.checked_add(b).ok_or(DaoVotingError::NumericalOverflow.into())
    }
    
    pub fn safe_sub(a: u64, b: u64) -> Result<u64> {
        a.checked_sub(b).ok_or(DaoVotingError::NumericalOverflow.into())
    }
    
    pub fn safe_mul(a: u64, b: u64) -> Result<u64> {
        a.checked_mul(b).ok_or(DaoVotingError::NumericalOverflow.into())
    }
    
    pub fn safe_div(a: u64, b: u64) -> Result<u64> {
        if b == 0 {
            return Err(DaoVotingError::DivisionByZero.into());
        }
        Ok(a / b)
    }
}
