use anchor_lang::prelude::*;

#[error_code]
pub enum TokenAutomationError {
    #[msg("Unauthorized access")]
    Unauthorized,

    #[msg("Automation is not active")]
    AutomationInactive,

    #[msg("Automation is currently paused")]
    AutomationPaused,

    #[msg("Invalid slippage percentage")]
    InvalidSlippage,

    #[msg("Invalid gas limit")]
    InvalidGasLimit,

    #[msg("No strategies provided")]
    NoStrategiesProvided,

    #[msg("Too many strategies for single automation")]
    TooManyStrategies,

    #[msg("Invalid amount")]
    InvalidAmount,

    #[msg("Invalid price threshold")]
    InvalidPriceThreshold,

    #[msg("Invalid expiration time")]
    InvalidExpirationTime,

    #[msg("Same token mints provided")]
    SameTokenMints,

    #[msg("Invalid stop price")]
    InvalidStopPrice,

    #[msg("Invalid DCA interval")]
    InvalidDcaInterval,

    #[msg("Invalid execution count")]
    InvalidExecutionCount,

    #[msg("Invalid confidence level")]
    InvalidConfidenceLevel,

    #[msg("Price data is stale")]
    StalePrice,

    #[msg("Price confidence too low")]
    LowPriceConfidence,

    #[msg("Insufficient profit for arbitrage")]
    InsufficientProfit,

    #[msg("Same DEXs provided for arbitrage")]
    SameDexes,

    #[msg("Math overflow")]
    MathOverflow,

    #[msg("Insufficient price data")]
    InsufficientPriceData,

    #[msg("Invalid token account owner")]
    InvalidTokenAccountOwner,

    #[msg("Execution too early")]
    ExecutionTooEarly,

    #[msg("Order is not active")]
    OrderNotActive,

    #[msg("Order has expired")]
    OrderExpired,

    #[msg("Price condition not met")]
    PriceConditionNotMet,

    #[msg("Insufficient amount out")]
    InsufficientAmountOut,

    #[msg("Slippage exceeded")]
    SlippageExceeded,

    #[msg("Stop loss not triggered")]
    StopLossNotTriggered,

    #[msg("Strategy is not active")]
    StrategyNotActive,

    #[msg("Execution not ready")]
    ExecutionNotReady,

    #[msg("Strategy not paused")]
    StrategyNotPaused,

    #[msg("Strategy already cancelled")]
    StrategyAlreadyCancelled,

    #[msg("Strategy completed")]
    StrategyCompleted,

    #[msg("Cannot update strategy")]
    CannotUpdateStrategy,

    #[msg("AI automation is disabled")]
    AiAutomationDisabled,

    #[msg("AI confidence too low")]
    LowAiConfidence,

    #[msg("Batch too large")]
    BatchTooLarge,

    #[msg("Empty batch")]
    EmptyBatch,

    #[msg("Unauthorized emergency action")]
    UnauthorizedEmergencyAction,

    #[msg("Arbitrage opportunity not active")]
    OpportunityNotActive,

    #[msg("Arbitrage opportunity expired")]
    OpportunityExpired,

    #[msg("Invalid mint")]
    InvalidMint,

    #[msg("Invalid arbitrage amount")]
    InvalidArbitrageAmount,

    #[msg("Too many pairs to scan")]
    TooManyPairsToScan,

    #[msg("Mismatched pair count")]
    MismatchedPairCount,

    #[msg("Insufficient balance")]
    InsufficientBalance,

    #[msg("Insufficient liquidity")]
    InsufficientLiquidity,

    #[msg("Invalid time interval")]
    InvalidTimeInterval,

    #[msg("Slippage tolerance exceeded")]
    SlippageTooHigh,

    #[msg("Invalid strategy type for this operation")]
    InvalidStrategy,

    #[msg("Too early to execute next trade")]
    TooEarly,

    #[msg("Order not found")]
    OrderNotFound,

    #[msg("Order already executed")]
    OrderAlreadyExecuted,

    #[msg("Price threshold not met")]
    PriceThresholdNotMet,

    #[msg("Invalid price data")]
    InvalidPriceData,

    #[msg("Price data too old")]
    PriceDataTooOld,

    #[msg("Invalid order parameters")]
    InvalidOrderParams,

    #[msg("Minimum trade amount not met")]
    MinTradeAmountNotMet,

    #[msg("Maximum orders per user exceeded")]
    MaxOrdersExceeded,

    #[msg("DCA strategy not found")]
    DcaStrategyNotFound,

    #[msg("DCA strategy already completed")]
    DcaStrategyCompleted,

    #[msg("Stop loss already triggered")]
    StopLossTriggered,

    #[msg("Arbitrage opportunity expired")]
    ArbitrageExpired,

    #[msg("Insufficient arbitrage profit")]
    InsufficientArbitrageProfit,

    #[msg("Invalid token pair")]
    InvalidTokenPair,

    #[msg("DEX integration error")]
    DexIntegrationError,

    #[msg("Swap failed")]
    SwapFailed,

    #[msg("Gas limit exceeded")]
    GasLimitExceeded,

    #[msg("Emergency authority required")]
    EmergencyAuthorityRequired,

    #[msg("Fee calculation overflow")]
    FeeCalculationOverflow,

    #[msg("Numerical overflow")]
    NumericalOverflow,

    #[msg("Division by zero")]
    DivisionByZero,

    #[msg("Invalid decimal precision")]
    InvalidDecimalPrecision,

    #[msg("Account not properly initialized")]
    AccountNotInitialized,

    #[msg("Account already initialized")]
    AccountAlreadyInitialized,

    #[msg("Oracle price feed error")]
    OraclePriceFeedError,

    #[msg("Market conditions not suitable")]
    UnfavorableMarketConditions,
}
