use anchor_lang::prelude::*;

#[error_code]
pub enum DaoVotingError {
    #[msg("DAO automation is not active")]
    AutomationInactive,
    
    #[msg("DAO automation is paused")]
    AutomationPaused,
    
    #[msg("AI confidence too low for automated voting")]
    LowAiConfidence,
    
    #[msg("Proposal too recent for voting")]
    ProposalTooRecent,
    
    #[msg("Gas price too high")]
    GasPriceTooHigh,
    
    #[msg("Automation conditions not met")]
    AutomationConditionsNotMet,
    
    #[msg("Invalid voting period")]
    InvalidVotingPeriod,
    
    #[msg("Invalid quorum percentage")]
    InvalidQuorumPercentage,
    
    #[msg("Proposal not found")]
    ProposalNotFound,
    
    #[msg("Voting period has not started")]
    VotingNotStarted,
    
    #[msg("Voting period has ended")]
    VotingEnded,
    
    #[msg("Already voted on this proposal")]
    AlreadyVoted,
    
    #[msg("Insufficient voting weight")]
    InsufficientVotingWeight,
    
    #[msg("Proposal already finalized")]
    ProposalAlreadyFinalized,
    
    #[msg("Proposal not finalized")]
    ProposalNotFinalized,
    
    #[msg("Proposal already executed")]
    ProposalAlreadyExecuted,
    
    #[msg("Proposal execution failed")]
    ProposalExecutionFailed,
    
    #[msg("Unauthorized access")]
    Unauthorized,
    
    #[msg("Invalid proposal type")]
    InvalidProposalType,
    
    #[msg("Quorum not reached")]
    QuorumNotReached,
    
    #[msg("Proposal rejected by voters")]
    ProposalRejected,
    
    #[msg("Invalid delegation amount")]
    InvalidDelegationAmount,
    
    #[msg("Delegation not found")]
    DelegationNotFound,
    
    #[msg("Delegation expired")]
    DelegationExpired,
    
    #[msg("Maximum delegations reached")]
    MaxDelegationsReached,
    
    #[msg("Cannot delegate to self")]
    CannotDelegateToSelf,
    
    #[msg("Insufficient balance for rewards")]
    InsufficientRewardBalance,
    
    #[msg("Reward calculation overflow")]
    RewardCalculationOverflow,
    
    #[msg("Invalid AI automation settings")]
    InvalidAiSettings,
    
    #[msg("Emergency authority required")]
    EmergencyAuthorityRequired,
    
    #[msg("String too long")]
    StringTooLong,
    
    #[msg("Numerical overflow")]
    NumericalOverflow,
    
    #[msg("Division by zero")]
    DivisionByZero,
    
    #[msg("Invalid timestamp")]
    InvalidTimestamp,
    
    #[msg("Account not properly initialized")]
    AccountNotInitialized,
    
    #[msg("Account already initialized")]
    AccountAlreadyInitialized,
    
    #[msg("Insufficient treasury funds")]
    InsufficientTreasuryFunds,
}
