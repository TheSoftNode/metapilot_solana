use anchor_lang::prelude::*;
use crate::state::*;

#[event]
pub struct DaoAutomationInitialized {
    pub authority: Pubkey,
    pub treasury: Pubkey,
    pub voting_rules: VotingRules,
    pub timestamp: i64,
}

#[event]
pub struct ProposalCreated {
    pub proposal_id: u64,
    pub proposer: Pubkey,
    pub dao_automation: Pubkey,
    pub title: String,
    pub proposal_type: ProposalType,
    pub voting_starts_at: i64,
    pub voting_ends_at: i64,
    pub timestamp: i64,
}

#[event]
pub struct VoteCast {
    pub proposal_id: u64,
    pub voter: Pubkey,
    pub dao_automation: Pubkey,
    pub vote_choice: VoteChoice,
    pub voting_weight: u64,
    pub is_delegated: bool,
    pub delegate: Option<Pubkey>,
    pub timestamp: i64,
}

#[event]
pub struct AutomatedVoteExecuted {
    pub proposal_id: u64,
    pub voter: Pubkey,
    pub dao_automation: Pubkey,
    pub vote_choice: VoteChoice,
    pub voting_weight: u64,
    pub ai_confidence: u8,
    pub reasoning: String,
    pub timestamp: i64,
}

#[event]
pub struct ProposalFinalized {
    pub proposal_id: u64,
    pub dao_automation: Pubkey,
    pub status: ProposalStatus,
    pub votes_for: u64,
    pub votes_against: u64,
    pub votes_abstain: u64,
    pub total_voting_weight: u64,
    pub quorum_reached: bool,
    pub timestamp: i64,
}

#[event]
pub struct ProposalExecuted {
    pub proposal_id: u64,
    pub dao_automation: Pubkey,
    pub executor: Pubkey,
    pub success: bool,
    pub timestamp: i64,
}

#[event]
pub struct VotingRulesUpdated {
    pub dao_automation: Pubkey,
    pub authority: Pubkey,
    pub old_rules: VotingRules,
    pub new_rules: VotingRules,
    pub timestamp: i64,
}

#[event]
pub struct AutomationToggled {
    pub dao_automation: Pubkey,
    pub authority: Pubkey,
    pub is_active: bool,
    pub timestamp: i64,
}

#[event]
pub struct VotingPowerDelegated {
    pub delegator: Pubkey,
    pub delegate: Pubkey,
    pub dao_automation: Pubkey,
    pub amount: u64,
    pub expires_at: i64,
    pub timestamp: i64,
}

#[event]
pub struct DelegationRevoked {
    pub delegator: Pubkey,
    pub delegate: Pubkey,
    pub dao_automation: Pubkey,
    pub amount: u64,
    pub timestamp: i64,
}

#[event]
pub struct GovernanceRewardsWithdrawn {
    pub user: Pubkey,
    pub dao_automation: Pubkey,
    pub amount: u64,
    pub reward_type: RewardType,
    pub timestamp: i64,
}

#[event]
pub struct AiSettingsUpdated {
    pub dao_automation: Pubkey,
    pub authority: Pubkey,
    pub old_settings: AiAutomationSettings,
    pub new_settings: AiAutomationSettings,
    pub timestamp: i64,
}

#[event]
pub struct EmergencyPauseActivated {
    pub dao_automation: Pubkey,
    pub emergency_authority: Pubkey,
    pub reason: String,
    pub timestamp: i64,
}

#[event]
pub struct OperationsResumed {
    pub dao_automation: Pubkey,
    pub authority: Pubkey,
    pub timestamp: i64,
}

#[event]
pub struct RewardsDistributed {
    pub dao_automation: Pubkey,
    pub recipient: Pubkey,
    pub amount: u64,
    pub reward_type: RewardType,
    pub proposal_id: Option<u64>,
    pub timestamp: i64,
}

/// Different types of rewards in the governance system
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum RewardType {
    VotingParticipation,
    ProposalCreation,
    ProposalExecution,
    DelegationReward,
    AiAutomationBonus,
}
