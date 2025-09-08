// Program-specific constants
pub const TRANSACTION_SCHEDULER_SEED: &[u8] = b"transaction_scheduler";
pub const SCHEDULED_TX_SEED: &[u8] = b"scheduled_tx";

// Operational limits
pub const MAX_PENDING_TRANSACTIONS: usize = 100;
pub const MAX_BATCH_SIZE: usize = 10;
pub const MAX_RETRY_ATTEMPTS: u8 = 3;
pub const MAX_INSTRUCTION_DATA_SIZE: usize = 1000;
pub const MAX_ACCOUNTS_PER_TX: usize = 20;

// Time constants (in seconds)
pub const MIN_EXECUTION_DELAY: i64 = 60; // 1 minute
pub const MAX_EXECUTION_DELAY: i64 = 7 * 24 * 3600; // 7 days
pub const DEFAULT_EXPIRY_TIME: i64 = 24 * 3600; // 24 hours
pub const BATCH_EXECUTION_WINDOW: i64 = 300; // 5 minutes

// Gas optimization constants
pub const MIN_GAS_PRICE: u64 = 1; // Minimum gas price in lamports
pub const MAX_GAS_PRICE: u64 = 1_000_000; // Maximum gas price in lamports
pub const GAS_OPTIMIZATION_THRESHOLD: u64 = 100; // Minimum gas savings to justify delay
pub const PRIORITY_GAS_MULTIPLIER: u16 = 150; // 1.5x for priority execution

// Fee constants (in basis points)
pub const SCHEDULER_FEE_BASIS_POINTS: u16 = 50; // 0.5% fee
pub const BATCH_DISCOUNT_BASIS_POINTS: u16 = 25; // 0.25% discount for batch
pub const PRIORITY_FEE_BASIS_POINTS: u16 = 100; // 1% priority fee

// Performance thresholds
pub const MIN_SUCCESS_RATE: u8 = 95; // 95% minimum success rate
pub const MAX_AVERAGE_DELAY: i64 = 3600; // 1 hour max average delay

// Account sizes
pub const TRANSACTION_SCHEDULER_SIZE: usize = 8 + // discriminator
    32 + // authority
    8 +  // max_gas_price
    1 +  // optimization_mode
    1 +  // is_active
    8 +  // created_at
    8 +  // total_transactions_scheduled
    8 +  // total_transactions_executed
    8 +  // total_gas_saved
    8 +  // total_fees_collected
    4 + (MAX_PENDING_TRANSACTIONS * SCHEDULED_TRANSACTION_SIZE) + // pending_transactions
    1; // bump = ~12,500 bytes

pub const SCHEDULED_TRANSACTION_SIZE: usize = 8 + // id
    32 + // authority
    1 +  // transaction_type
    32 + // target_program
    4 + MAX_INSTRUCTION_DATA_SIZE + // instruction_data
    4 + (MAX_ACCOUNTS_PER_TX * 32) + // accounts
    8 +  // earliest_execution_time
    8 +  // max_gas_price
    9 +  // expiry_time (Option<i64>)
    1 +  // retry_attempts
    8 +  // created_at
    1 +  // status
    1; // attempts = ~2,000 bytes

// Transaction types as constants for indexing
pub const TX_TYPE_TOKEN_TRANSFER: u8 = 0;
pub const TX_TYPE_DAO_VOTE: u8 = 1;
pub const TX_TYPE_YIELD_CLAIM: u8 = 2;
pub const TX_TYPE_TOKEN_SWAP: u8 = 3;
pub const TX_TYPE_CONTRACT_CALL: u8 = 4;
pub const TX_TYPE_NFT_MINT: u8 = 5;

// Status codes as constants
pub const STATUS_PENDING: u8 = 0;
pub const STATUS_EXECUTED: u8 = 1;
pub const STATUS_FAILED: u8 = 2;
pub const STATUS_EXPIRED: u8 = 3;
pub const STATUS_CANCELLED: u8 = 4;

// Optimization modes
pub const OPT_MODE_GAS_OPTIMIZED: u8 = 0;
pub const OPT_MODE_TIME_OPTIMIZED: u8 = 1;
pub const OPT_MODE_BALANCED: u8 = 2;
