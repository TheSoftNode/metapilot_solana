/// Constants for Yield Optimizer Program

/// Seed constants for PDA derivation
pub const YIELD_STRATEGY_SEED: &[u8] = b"yield_strategy";

/// Minimum confidence thresholds
pub const MIN_YIELD_FARMING_CONFIDENCE: u8 = 85;
pub const MIN_REBALANCE_CONFIDENCE: u8 = 80;
pub const MIN_AI_CONFIDENCE: u8 = 75;

/// Amount thresholds
pub const MIN_DEPOSIT_AMOUNT: u64 = 10_000_000; // 0.01 SOL
pub const MIN_WITHDRAW_AMOUNT: u64 = 1_000_000; // 0.001 SOL
pub const MIN_COMPOUND_AMOUNT: u64 = 1_000_000; // 0.001 SOL
pub const MIN_PROTOCOL_ALLOCATION: u64 = 1_000_000; // 0.001 SOL
pub const MIN_APY_THRESHOLD: u16 = 100; // 1% APY
pub const MIN_ALLOCATION_BASIS_POINTS: u16 = 100; // 1%

/// Risk thresholds
pub const RISK_HIGH_APY_THRESHOLD: u16 = 2000; // 20% APY
pub const MAX_SINGLE_PROTOCOL_ALLOCATION: u16 = 5000; // 50%

/// Rebalance thresholds
pub const REBALANCE_THRESHOLD_BASIS_POINTS: u16 = 500; // 5%

/// Gas cost estimates
pub const BASE_COMPOUND_GAS_COST: u64 = 20000; // Base gas cost for compound operations

/// Fee constants
pub const WITHDRAW_FEE_BASIS_POINTS: u16 = 25; // 0.25%
pub const EMERGENCY_WITHDRAW_FEE_BASIS_POINTS: u16 = 100; // 1%

/// Time constants (in seconds)
pub const MIN_COMPOUND_INTERVAL: i64 = 3600; // 1 hour
pub const MIN_REBALANCE_INTERVAL: i64 = 86400; // 24 hours
pub const EARLY_WITHDRAWAL_PERIOD: i64 = 7 * 24 * 3600; // 7 days

/// Percentage constants (basis points)
pub const FULL_ALLOCATION: u16 = 10000; // 100%
pub const EMERGENCY_WITHDRAWAL_PENALTY: u16 = 500; // 5%
pub const EARLY_WITHDRAWAL_PENALTY: u16 = 200; // 2%

/// Risk levels
pub const MIN_RISK_LEVEL: u8 = 1;
pub const MAX_RISK_LEVEL: u8 = 10;

/// Strategy limits
pub const MAX_PROTOCOLS_PER_STRATEGY: usize = 10;
pub const MAX_ALLOCATION_TARGETS: usize = 20;

/// Account space calculations
pub const YIELD_STRATEGY_SIZE: usize = 8 + // discriminator
    32 + // authority
    40 + // strategy_config (YieldStrategyConfig)
    2 +  // min_yield_threshold
    8 +  // rebalance_frequency
    1 +  // is_active
    8 +  // created_at
    8 +  // last_rebalance_at
    8 +  // last_compound_at
    8 +  // total_deposited
    8 +  // total_rewards_claimed
    8; // rebalance_count = 139 bytes

/// Fee constants
pub const PROTOCOL_FEE_BASIS_POINTS: u16 = 50; // 0.5%
pub const PERFORMANCE_FEE_BASIS_POINTS: u16 = 1000; // 10%

/// Default configurations
pub const DEFAULT_COMPOUND_FREQUENCY: i64 = 24 * 3600; // Daily
pub const DEFAULT_REBALANCE_FREQUENCY: i64 = 7 * 24 * 3600; // Weekly
pub const DEFAULT_MIN_YIELD_THRESHOLD: u16 = 500; // 5% APY

/// Protocol identifiers
pub const PROTOCOL_MARINADE: u8 = 1;
pub const PROTOCOL_SOLEND: u8 = 2;
pub const PROTOCOL_TULIP: u8 = 3;
pub const PROTOCOL_FRANCIUM: u8 = 4;
pub const PROTOCOL_MANGO: u8 = 5;
