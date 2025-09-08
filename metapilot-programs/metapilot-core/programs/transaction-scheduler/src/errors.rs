/// Error types for Transaction Scheduler Program
use anchor_lang::prelude::*;

#[error_code]
pub enum TransactionSchedulerError {
    #[msg("Transaction scheduler is not active")]
    SchedulerInactive,
    
    #[msg("Too many pending transactions")]
    TooManyPendingTransactions,
    
    #[msg("Transaction not found")]
    TransactionNotFound,
    
    #[msg("Execution conditions not met")]
    ConditionsNotMet,
    
    #[msg("Batch size too large")]
    BatchTooLarge,
    
    #[msg("Transaction has expired")]
    TransactionExpired,
    
    #[msg("Invalid gas price")]
    InvalidGasPrice,
    
    #[msg("Invalid execution time - must be in the future")]
    InvalidExecutionTime,
    
    #[msg("Invalid transaction data")]
    InvalidTransactionData,
    
    #[msg("Unauthorized access to transaction")]
    UnauthorizedAccess,
    
    #[msg("Transaction already executed")]
    TransactionAlreadyExecuted,
    
    #[msg("Transaction retry limit exceeded")]
    RetryLimitExceeded,
    
    #[msg("Invalid optimization mode")]
    InvalidOptimizationMode,
    
    #[msg("Instruction data too large")]
    InstructionDataTooLarge,
    
    #[msg("Too many accounts in transaction")]
    TooManyAccounts,
    
    #[msg("Transaction execution failed")]
    ExecutionFailed,
    
    #[msg("Insufficient funds for gas payment")]
    InsufficientFunds,
    
    #[msg("Gas price too high")]
    GasPriceTooHigh,
    
    #[msg("Gas price too low")]
    GasPriceTooLow,
    
    #[msg("Batch execution window expired")]
    BatchWindowExpired,
    
    #[msg("Invalid target program")]
    InvalidTargetProgram,
    
    #[msg("Transaction scheduler is paused")]
    SchedulerPaused,
    
    #[msg("Math overflow in calculations")]
    MathOverflow,
    
    #[msg("Invalid fee calculation")]
    InvalidFeeCalculation,
    
    #[msg("Duplicate transaction ID")]
    DuplicateTransactionId,
    
    #[msg("Transaction deadline exceeded")]
    DeadlineExceeded,
    
    #[msg("Minimum delay not met")]
    MinimumDelayNotMet,
    
    #[msg("Maximum delay exceeded")]
    MaximumDelayExceeded,
    
    #[msg("Invalid transaction type")]
    InvalidTransactionType,
    
    #[msg("Network congestion - retry later")]
    NetworkCongestion,
    
    #[msg("Priority execution not available")]
    PriorityNotAvailable,
    
    #[msg("Batch optimization threshold not met")]
    BatchThresholdNotMet,
    
    #[msg("Transaction requires higher gas price")]
    GasPriceInsufficientForPriority,
    
    #[msg("Account validation failed")]
    AccountValidationFailed,
    
    #[msg("Cross-program invocation failed")]
    CPIFailed,
    
    #[msg("Scheduler at capacity")]
    SchedulerAtCapacity,
    
    #[msg("Invalid time window for execution")]
    InvalidTimeWindow,
    
    #[msg("Transaction dependencies not met")]
    DependenciesNotMet,
    
    #[msg("Slippage tolerance exceeded")]
    SlippageExceeded,
}
