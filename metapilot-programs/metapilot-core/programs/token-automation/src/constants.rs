/// Constants for the token automation program

/// Maximum slippage constants (in basis points)
pub const MAX_SLIPPAGE_BPS: u16 = 1000; // 10%
pub const DEFAULT_MAX_SLIPPAGE_BPS: u16 = 100; // 1%
pub const MIN_SLIPPAGE_BPS: u16 = 1; // 0.01%

/// Time intervals for strategies (in seconds)
pub const MIN_DCA_INTERVAL: i64 = 60 * 60; // 1 hour
pub const MAX_DCA_INTERVAL: i64 = 30 * 24 * 60 * 60; // 30 days
pub const DEFAULT_DCA_INTERVAL: i64 = 24 * 60 * 60; // 24 hours

/// Gas and execution limits
pub const MIN_GAS_LIMIT: u64 = 100_000;
pub const MAX_GAS_LIMIT: u64 = 10_000_000;
pub const DEFAULT_GAS_LIMIT: u64 = 1_000_000;

/// AI confidence thresholds
pub const MIN_AI_CONFIDENCE: u8 = 50; // 50%
pub const DEFAULT_AI_CONFIDENCE_THRESHOLD: u8 = 80; // 80%
pub const MIN_AI_CONFIDENCE_THRESHOLD: u8 = 70; // 70% minimum for AI actions
pub const MAX_AI_CONFIDENCE: u8 = 100; // 100%

/// Amount limits
pub const MIN_TRADE_AMOUNT: u64 = 1_000; // Minimum trade amount (adjusted for token decimals)
pub const MAX_ORDERS_PER_USER: u8 = 50; // Maximum active orders per user

/// Price precision and calculations
pub const PRICE_PRECISION: u64 = 1_000_000; // 1M for price calculations
pub const BPS_PRECISION: u64 = 10_000; // 10,000 basis points = 100%

/// Order and strategy constants
pub const MAX_ORDER_LIFETIME: i64 = 365 * 24 * 60 * 60; // 1 year
pub const DEFAULT_ORDER_LIFETIME: i64 = 30 * 24 * 60 * 60; // 30 days
pub const MIN_ORDER_LIFETIME: i64 = 60 * 60; // 1 hour

/// Fee constants
pub const AUTOMATION_FEE_BPS: u16 = 10; // 0.1% automation fee
pub const ARBITRAGE_FEE_BPS: u16 = 50; // 0.5% arbitrage fee
pub const DCA_FEE_BPS: u16 = 5; // 0.05% DCA fee

/// Account size constants
pub const TOKEN_AUTOMATION_SIZE: usize = 8 + // discriminator
    32 + // authority
    32 + // treasury
    4 + // supported_strategies (Vec<StrategyType>)
    2 + // max_slippage
    8 + // gas_limit
    1 + // is_active
    1 + // is_paused
    8 + // created_at
    8 + // last_updated
    8 + // total_executions
    8 + // total_volume
    8 + // total_fees_collected
    32 + // emergency_authority
    1; // ai_automation_enabled

pub const LIMIT_ORDER_SIZE: usize = 8 + // discriminator
    8 + // id
    32 + // owner
    32 + // token_automation
    32 + // token_mint_a
    32 + // token_mint_b
    8 + // amount_in
    8 + // min_amount_out
    8 + // price_threshold
    8 + // created_at
    8 + // expires_at
    1 + // order_side
    1 + // status
    2 + // max_slippage
    1; // fill_or_kill

pub const STOP_LOSS_SIZE: usize = 8 + // discriminator
    8 + // id
    32 + // owner
    32 + // token_automation
    32 + // token_mint
    8 + // amount
    8 + // stop_price
    8 + // created_at
    8 + // expires_at
    1 + // status
    2; // max_slippage

pub const DCA_STRATEGY_SIZE: usize = 8 + // discriminator
    8 + // id
    32 + // owner
    32 + // token_automation
    32 + // token_mint_in
    32 + // token_mint_out
    8 + // amount_per_execution
    8 + // interval_seconds
    8 + // total_executions
    8 + // executed_count
    8 + // next_execution_at
    8 + // created_at
    1 + // status
    2; // max_slippage

pub const ARBITRAGE_OPPORTUNITY_SIZE: usize = 8 + // discriminator
    8 + // id
    32 + // discoverer
    32 + // token_mint_a
    32 + // token_mint_b
    32 + // dex_a
    32 + // dex_b
    8 + // price_diff_bps
    8 + // max_amount
    8 + // discovered_at
    8 + // expires_at
    1; // status

/// Seed constants
pub const TOKEN_AUTOMATION_SEED: &[u8] = b"token_automation";
pub const AUTOMATION_SEED: &[u8] = b"automation"; // Alias for backward compatibility
pub const LIMIT_ORDER_SEED: &[u8] = b"limit_order";
pub const STOP_LOSS_SEED: &[u8] = b"stop_loss";
pub const DCA_STRATEGY_SEED: &[u8] = b"dca_strategy";
pub const ARBITRAGE_SEED: &[u8] = b"arbitrage";
pub const TREASURY_SEED: &[u8] = b"treasury";

/// DEX integration constants
pub const JUPITER_PROGRAM_ID: &str = "JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4";
pub const ORCA_PROGRAM_ID: &str = "9W959DqEETiGZocYWCQPaJ6sBmUzgfxXfqGeTEdp3aQP";
pub const RAYDIUM_PROGRAM_ID: &str = "675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8";

/// Market data constants
pub const MAX_PRICE_AGE_SECONDS: i64 = 60; // Maximum age for price data
pub const MIN_LIQUIDITY_THRESHOLD: u64 = 10_000_000; // Minimum liquidity for execution

/// Volume and volatility thresholds
pub const LOW_VOLUME_THRESHOLD: u64 = 10_000_000; // $10M daily volume in cents
pub const MEDIUM_VOLUME_THRESHOLD: u64 = 50_000_000; // $50M daily volume in cents
pub const HIGH_VOLUME_THRESHOLD: u64 = 200_000_000; // $200M daily volume in cents

pub const LOW_VOLATILITY_THRESHOLD: u16 = 500; // 5% volatility in basis points
pub const MEDIUM_VOLATILITY_THRESHOLD: u16 = 1000; // 10% volatility in basis points
pub const HIGH_VOLATILITY_THRESHOLD: u16 = 2000; // 20% volatility in basis points

/// Additional constants for validation
pub const MAX_STRATEGIES_PER_AUTOMATION: usize = 10; // Maximum strategies per automation account
pub const MIN_ORDER_DURATION: i64 = MIN_ORDER_LIFETIME; // Alias for consistency
pub const MAX_ORDER_DURATION: i64 = MAX_ORDER_LIFETIME; // Alias for consistency
pub const MAX_DCA_EXECUTIONS: u64 = 10000; // Maximum DCA executions
pub const MIN_PRICE_CONFIDENCE: u8 = MIN_AI_CONFIDENCE; // Alias for consistency
pub const MIN_ARBITRAGE_PROFIT_BPS: u16 = 50; // Minimum arbitrage profit 0.5%
pub const DEFAULT_GAS_PRICE: u64 = 1000; // Default gas price in lamports
pub const MAX_SCAN_PAIRS: usize = 100; // Maximum token pairs to scan for arbitrage

/// Execution complexity and fee constants
pub const LIMIT_ORDER_COMPLEXITY: u64 = 1; // Complexity factor for limit orders
pub const STOP_LOSS_COMPLEXITY: u64 = 1; // Complexity factor for stop loss orders
pub const DCA_COMPLEXITY: u64 = 2; // Complexity factor for DCA strategies
pub const AI_SWAP_COMPLEXITY: u64 = 3; // Complexity factor for AI-driven swaps
pub const MANUAL_SWAP_COMPLEXITY: u64 = 1; // Complexity factor for manual swaps
pub const AUTOMATED_SWAP_COMPLEXITY: u64 = 2; // Complexity factor for automated swaps
pub const ARBITRAGE_COMPLEXITY: u64 = 4; // Complexity factor for arbitrage execution
pub const BASE_EXECUTION_FEE: u64 = 1000; // Base execution fee in lamports

/// Batch processing constants
pub const MAX_BATCH_SIZE: usize = 10; // Maximum number of operations in a batch

/// Arbitrage constants
pub const ARBITRAGE_OPPORTUNITY_SEED: &[u8] = b"arbitrage_opportunity";
pub const DEFAULT_ARBITRAGE_EXPIRY: i64 = 60 * 5; // 5 minutes
pub const DISCOVERER_REWARD_BPS: u64 = 100; // 1% reward for discoverer
