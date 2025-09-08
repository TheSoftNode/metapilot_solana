/// Error types for Yield Optimizer Program
use anchor_lang::prelude::*;

#[error_code]
pub enum YieldOptimizerError {
    #[msg("Yield strategy is not active")]
    StrategyInactive,

    #[msg("AI confidence too low for automated yield farming")]
    LowConfidence,

    #[msg("Yield below minimum threshold")]
    YieldBelowThreshold,

    #[msg("No rewards available to compound")]
    NoRewardsToCompound,

    #[msg("Too early to compound rewards")]
    TooEarlyToCompound,

    #[msg("Too early to rebalance portfolio")]
    TooEarlyToRebalance,

    #[msg("Invalid allocation percentages")]
    InvalidAllocation,

    #[msg("Invalid amount specified")]
    InvalidAmount,

    #[msg("Insufficient balance for withdrawal")]
    InsufficientBalance,

    #[msg("No protocols available for yield farming")]
    NoProtocolsAvailable,

    #[msg("Invalid risk level specified")]
    InvalidRiskLevel,

    #[msg("Strategy configuration invalid")]
    InvalidStrategyConfig,

    #[msg("Maximum number of protocols exceeded")]
    TooManyProtocols,

    #[msg("Protocol not supported")]
    UnsupportedProtocol,

    #[msg("Compound frequency too low")]
    CompoundFrequencyTooLow,

    #[msg("Rebalance frequency too low")]
    RebalanceFrequencyTooLow,

    #[msg("Minimum yield threshold too high")]
    MinYieldThresholdTooHigh,

    #[msg("Unauthorized operation")]
    Unauthorized,

    #[msg("Strategy not found")]
    StrategyNotFound,

    #[msg("Invalid protocol allocation")]
    InvalidProtocolAllocation,

    #[msg("Strategy already initialized")]
    StrategyAlreadyInitialized,

    #[msg("Withdrawal amount exceeds available balance")]
    WithdrawalExceedsBalance,

    #[msg("Protocol rewards calculation failed")]
    ProtocolRewardsCalculationFailed,

    #[msg("Rebalancing operation failed")]
    RebalancingFailed,

    #[msg("Auto-compound operation failed")]
    AutoCompoundFailed,

    #[msg("Protocol integration error")]
    ProtocolIntegrationError,

    #[msg("Invalid strategy state for operation")]
    InvalidStrategyState,

    #[msg("Slippage tolerance exceeded")]
    SlippageToleranceExceeded,

    #[msg("Market conditions unfavorable")]
    UnfavorableMarketConditions,

    // Additional error variants needed by the code
    #[msg("Risk tolerance exceeded")]
    RiskToleranceExceeded,

    #[msg("Strategy is not currently active")]
    StrategyNotActive,

    #[msg("Deposit amount too small")]
    DepositTooSmall,

    #[msg("Math overflow error")]
    MathOverflow,

    #[msg("No protocols selected for strategy")]
    NoProtocolsSelected,

    #[msg("Invalid withdraw percentage")]
    InvalidWithdrawPercentage,

    #[msg("Inconsistent withdraw amount")]
    InconsistentWithdrawAmount,

    #[msg("Invalid withdraw amount")]
    InvalidWithdrawAmount,

    #[msg("Insufficient deposits for withdrawal")]
    InsufficientDeposits,

    #[msg("Insufficient strategy balance")]
    InsufficientStrategyBalance,

    #[msg("Withdraw amount too small")]
    WithdrawTooSmall,

    #[msg("Invalid rebalance parameters")]
    InvalidRebalanceParams,

    #[msg("Rebalance deadline exceeded")]
    RebalanceDeadlineExceeded,

    #[msg("Insufficient AI confidence")]
    InsufficientAIConfidence,

    #[msg("Rebalance not needed")]
    RebalanceNotNeeded,

    #[msg("Rebalance operation failed")]
    RebalanceFailed,

    #[msg("Compound too frequent")]
    CompoundTooFrequent,

    #[msg("Compound amount too small")]
    CompoundAmountTooSmall,

    #[msg("Protocol not in strategy")]
    ProtocolNotInStrategy,

    #[msg("Invalid emergency reason")]
    InvalidEmergencyReason,

    #[msg("Strategy not paused")]
    StrategyNotPaused,

    #[msg("Emergency cooldown active")]
    EmergencyCooldownActive,

    #[msg("No eligible protocols")]
    NoEligibleProtocols,

    #[msg("Concentration risk too high")]
    ConcentrationRiskTooHigh,

    #[msg("Duplicate protocol in strategy")]
    DuplicateProtocol,

    #[msg("Allocation too small")]
    AllocationTooSmall,

    #[msg("Operation too frequent")]
    OperationTooFrequent,
}
