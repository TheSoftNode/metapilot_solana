/// Constants for the DAO voting program
pub const MAX_PROPOSAL_TITLE_LEN: usize = 64;
pub const MAX_PROPOSAL_DESCRIPTION_LEN: usize = 1000;
pub const MAX_VOTING_RULES_LEN: usize = 500;
pub const MAX_PROPOSAL_LINK_LEN: usize = 200;

/// Voting period constants (in seconds)
pub const MIN_VOTING_PERIOD: i64 = 24 * 60 * 60; // 1 day
pub const MAX_VOTING_PERIOD: i64 = 30 * 24 * 60 * 60; // 30 days
pub const DEFAULT_VOTING_PERIOD: i64 = 7 * 24 * 60 * 60; // 7 days

/// Quorum constants
pub const MIN_QUORUM_PERCENTAGE: u8 = 1; // 1%
pub const MAX_QUORUM_PERCENTAGE: u8 = 100; // 100%
pub const DEFAULT_QUORUM_PERCENTAGE: u8 = 10; // 10%

/// AI automation constants
pub const MIN_AI_CONFIDENCE_THRESHOLD: u8 = 50; // 50%
pub const MAX_AI_CONFIDENCE_THRESHOLD: u8 = 100; // 100%
pub const DEFAULT_AI_CONFIDENCE_THRESHOLD: u8 = 80; // 80%

/// Gas price limits
pub const MAX_GAS_PRICE_MULTIPLIER: u64 = 10; // 10x normal gas price
pub const DEFAULT_MAX_GAS_PRICE: u64 = 1_000_000; // 1M lamports

/// Delegation constants
pub const MAX_DELEGATIONS_PER_USER: u8 = 10;
pub const MIN_DELEGATION_AMOUNT: u64 = 1_000_000; // 1 token (assuming 6 decimals)

/// Reward constants
pub const REWARD_PRECISION: u64 = 1_000_000; // 1M for percentage calculations
pub const BASE_PARTICIPATION_REWARD: u64 = 100_000; // Base reward for voting
pub const PROPOSAL_CREATION_REWARD: u64 = 500_000; // Reward for creating proposals

/// Account size constants
pub const DAO_AUTOMATION_SIZE: usize = 8 + // discriminator
    32 + // authority
    32 + // treasury
    4 + MAX_VOTING_RULES_LEN + // voting_rules
    8 + // min_proposal_age
    8 + // max_gas_price
    1 + // is_active
    1 + // is_paused
    8 + // created_at
    8 + // last_updated
    8 + // total_proposals
    8 + // total_votes_cast
    8 + // total_rewards_distributed
    32 + // emergency_authority
    1; // ai_automation_enabled

pub const PROPOSAL_SIZE: usize = 8 + // discriminator
    8 + // id
    32 + // proposer
    32 + // dao_automation
    4 + MAX_PROPOSAL_TITLE_LEN + // title
    4 + MAX_PROPOSAL_DESCRIPTION_LEN + // description
    4 + MAX_PROPOSAL_LINK_LEN + // link
    1 + // proposal_type
    8 + // created_at
    8 + // voting_starts_at
    8 + // voting_ends_at
    8 + // votes_for
    8 + // votes_against
    8 + // votes_abstain
    8 + // total_voting_weight
    1 + // status
    1 + // executed
    4 + 200; // execution_data (variable size, max 200 bytes)

pub const VOTE_RECORD_SIZE: usize = 8 + // discriminator
    8 + // proposal_id
    32 + // voter
    32 + // dao_automation
    1 + // vote_choice
    8 + // voting_weight
    8 + // timestamp
    1 + // is_delegated
    32; // delegate (if applicable)

pub const DELEGATION_SIZE: usize = 8 + // discriminator
    32 + // delegator
    32 + // delegate
    32 + // dao_automation
    8 + // amount
    8 + // created_at
    8; // expires_at

/// Seed constants
pub const DAO_AUTOMATION_SEED: &[u8] = b"dao_automation";
pub const PROPOSAL_SEED: &[u8] = b"proposal";
pub const VOTE_RECORD_SEED: &[u8] = b"vote_record";
pub const DELEGATION_SEED: &[u8] = b"delegation";
pub const TREASURY_SEED: &[u8] = b"treasury";
