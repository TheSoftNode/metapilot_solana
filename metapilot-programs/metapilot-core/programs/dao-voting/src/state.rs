use crate::constants::*;
use anchor_lang::prelude::*;

/// Main DAO automation account that stores governance settings and state
#[account]
pub struct DaoAutomation {
    /// Authority that can modify settings
    pub authority: Pubkey,

    /// Treasury account for storing governance tokens and rewards
    pub treasury: Pubkey,

    /// Voting rules and parameters
    pub voting_rules: VotingRules,

    /// Minimum age for proposals before they can be voted on (in seconds)
    pub min_proposal_age: i64,

    /// Maximum gas price for automated operations
    pub max_gas_price: u64,

    /// Whether automation is active
    pub is_active: bool,

    /// Whether operations are paused (emergency)
    pub is_paused: bool,

    /// When this automation was created
    pub created_at: i64,

    /// Last time settings were updated
    pub last_updated: i64,

    /// Total number of proposals created
    pub total_proposals: u64,

    /// Total votes cast through this automation
    pub total_votes_cast: u64,

    /// Total rewards distributed
    pub total_rewards_distributed: u64,

    /// Emergency authority for critical situations
    pub emergency_authority: Pubkey,

    /// Whether AI automation is enabled
    pub ai_automation_enabled: bool,
}

impl DaoAutomation {
    pub const SIZE: usize = DAO_AUTOMATION_SIZE;
}

/// Governance proposal
#[account]
pub struct Proposal {
    /// Unique proposal ID
    pub id: u64,

    /// Who created this proposal
    pub proposer: Pubkey,

    /// Associated DAO automation account
    pub dao_automation: Pubkey,

    /// Proposal title
    pub title: String,

    /// Detailed description
    pub description: String,

    /// Optional link to additional information
    pub link: String,

    /// Type of proposal
    pub proposal_type: ProposalType,

    /// When proposal was created
    pub created_at: i64,

    /// When voting starts
    pub voting_starts_at: i64,

    /// When voting ends
    pub voting_ends_at: i64,

    /// Votes in favor
    pub votes_for: u64,

    /// Votes against
    pub votes_against: u64,

    /// Abstain votes
    pub votes_abstain: u64,

    /// Total voting weight participated
    pub total_voting_weight: u64,

    /// Current status
    pub status: ProposalStatus,

    /// Whether proposal has been executed
    pub executed: bool,

    /// Data for proposal execution
    pub execution_data: Vec<u8>,
}

impl Proposal {
    pub const SIZE: usize = PROPOSAL_SIZE;

    /// Check if voting is currently active
    pub fn is_voting_active(&self) -> bool {
        let now = Clock::get().unwrap().unix_timestamp;
        now >= self.voting_starts_at && now <= self.voting_ends_at
    }

    /// Check if proposal has enough quorum
    pub fn has_quorum(&self, quorum_threshold: u64, total_supply: u64) -> bool {
        let quorum_required = (total_supply * quorum_threshold) / 100;
        self.total_voting_weight >= quorum_required
    }

    /// Calculate if proposal passes
    pub fn is_approved(&self) -> bool {
        self.votes_for > self.votes_against
    }
}

/// Individual vote record
#[account]
pub struct VoteRecord {
    /// Associated proposal ID
    pub proposal_id: u64,

    /// Who cast the vote
    pub voter: Pubkey,

    /// Associated DAO automation
    pub dao_automation: Pubkey,

    /// Vote choice
    pub vote_choice: VoteChoice,

    /// Weight of the vote
    pub voting_weight: u64,

    /// When vote was cast
    pub timestamp: i64,

    /// Whether this vote was cast through delegation
    pub is_delegated: bool,

    /// If delegated, who the delegate is
    pub delegate: Pubkey,
}

impl VoteRecord {
    pub const SIZE: usize = VOTE_RECORD_SIZE;
}

/// Delegation of voting power
#[account]
pub struct Delegation {
    /// Who is delegating their voting power
    pub delegator: Pubkey,

    /// Who receives the delegated voting power
    pub delegate: Pubkey,

    /// Associated DAO automation
    pub dao_automation: Pubkey,

    /// Amount of voting power delegated
    pub amount: u64,

    /// When delegation was created
    pub created_at: i64,

    /// When delegation expires (0 means no expiration)
    pub expires_at: i64,
}

impl Delegation {
    pub const SIZE: usize = DELEGATION_SIZE;

    /// Check if delegation is still valid
    pub fn is_valid(&self) -> bool {
        if self.expires_at == 0 {
            return true; // No expiration
        }
        let now = Clock::get().unwrap().unix_timestamp;
        now <= self.expires_at
    }
}

/// Voting rules and parameters
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct VotingRules {
    /// Minimum voting period in seconds
    pub min_voting_period: i64,

    /// Maximum voting period in seconds
    pub max_voting_period: i64,

    /// Default voting period in seconds
    pub default_voting_period: i64,

    /// Quorum percentage required (0-100)
    pub quorum_percentage: u8,

    /// Whether proposals can be executed automatically
    pub auto_execution_enabled: bool,

    /// Minimum tokens required to create a proposal
    pub min_proposal_threshold: u64,

    /// Voting weight calculation method
    pub voting_weight_source: VotingWeightSource,
}

impl Default for VotingRules {
    fn default() -> Self {
        Self {
            min_voting_period: MIN_VOTING_PERIOD,
            max_voting_period: MAX_VOTING_PERIOD,
            default_voting_period: DEFAULT_VOTING_PERIOD,
            quorum_percentage: DEFAULT_QUORUM_PERCENTAGE,
            auto_execution_enabled: false,
            min_proposal_threshold: MIN_DELEGATION_AMOUNT,
            voting_weight_source: VotingWeightSource::TokenBalance,
        }
    }
}

/// AI automation settings
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct AiAutomationSettings {
    /// Minimum confidence level for AI to auto-vote
    pub confidence_threshold: u8,

    /// Whether AI can vote on all proposal types
    pub enabled_for_all_types: bool,

    /// Specific proposal types AI can vote on
    pub enabled_proposal_types: Vec<ProposalType>,

    /// Maximum number of AI votes per day
    pub max_votes_per_day: u16,

    /// Whether to require human approval for AI votes
    pub require_human_approval: bool,
}

impl Default for AiAutomationSettings {
    fn default() -> Self {
        Self {
            confidence_threshold: DEFAULT_AI_CONFIDENCE_THRESHOLD,
            enabled_for_all_types: false,
            enabled_proposal_types: vec![],
            max_votes_per_day: 10,
            require_human_approval: true,
        }
    }
}

/// Types of proposals
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq, Debug)]
pub enum ProposalType {
    /// General governance proposal
    General,

    /// Treasury spending proposal
    Treasury,

    /// Protocol parameter change
    ParameterChange,

    /// Smart contract upgrade
    Upgrade,

    /// Emergency action
    Emergency,

    /// Community initiative
    Community,

    /// Partnership proposal
    Partnership,

    /// Token economics change
    Tokenomics,
}

/// Vote choices
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum VoteChoice {
    For,
    Against,
    Abstain,
}

/// Proposal status
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq, Debug)]
pub enum ProposalStatus {
    Draft,
    Active,
    Succeeded,
    Defeated,
    Expired,
    Executed,
    Cancelled,
}

/// Sources for voting weight calculation
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum VotingWeightSource {
    /// Use token balance as voting weight
    TokenBalance,

    /// Use staked token amount
    StakedTokens,

    /// Use NFT holdings
    NftHoldings,

    /// Custom calculation
    Custom,
}

/// Parameters for initializing DAO automation
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct InitializeDaoParams {
    pub voting_rules: VotingRules,
    pub min_proposal_age: i64,
    pub max_gas_price: u64,
    pub emergency_authority: Pubkey,
    pub ai_automation_settings: Option<AiAutomationSettings>,
}

/// Parameters for creating a proposal
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct CreateProposalParams {
    pub title: String,
    pub description: String,
    pub link: String,
    pub proposal_type: ProposalType,
    pub voting_period: Option<i64>, // If None, use default
    pub execution_data: Vec<u8>,
}

/// Parameters for automated voting
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct AutomatedVoteParams {
    pub proposal_id: u64,
    pub vote_choice: VoteChoice,
    pub ai_confidence: u8,
    pub reasoning: String,
    pub conditions: AutomationConditions,
}

/// Conditions for automation execution
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct AutomationConditions {
    pub min_proposal_age: i64,
    pub max_gas_price: u64,
    pub quorum_threshold: u64,
    pub ai_confidence_threshold: u8,
    pub check_sentiment: bool,
    pub require_stake: bool,
}

impl AutomationConditions {
    pub fn is_condition_met(&self, proposal: &Proposal, current_gas_price: u64) -> Result<bool> {
        let current_time = Clock::get()?.unix_timestamp;

        // Check proposal age
        if current_time - proposal.created_at < self.min_proposal_age {
            return Ok(false);
        }

        // Check gas price
        if current_gas_price > self.max_gas_price {
            return Ok(false);
        }

        // Additional condition checks can be added here
        Ok(true)
    }
}
