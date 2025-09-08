/// State and account structures for Yield Optimizer Program
use anchor_lang::prelude::*;

#[account]
pub struct YieldStrategy {
    pub authority: Pubkey,                    // 32 bytes
    pub strategy_config: YieldStrategyConfig, // 40 bytes
    pub min_yield_threshold: u16,             // 2 bytes
    pub rebalance_frequency: i64,             // 8 bytes
    pub is_active: bool,                      // 1 byte
    pub created_at: i64,                      // 8 bytes
    pub last_rebalance_at: i64,               // 8 bytes
    pub last_compound_at: i64,                // 8 bytes
    pub total_deposited: u64,                 // 8 bytes
    pub total_rewards_claimed: u64,           // 8 bytes
    pub rebalance_count: u64,                 // 8 bytes
    pub compound_count: u64,                  // 8 bytes
    pub emergency_pause: bool,                // 1 byte
    pub bump: u8,                             // 1 byte
}

impl YieldStrategy {
    pub const SIZE: usize = 8 + // discriminator
        32 + // authority
        40 + // strategy_config
        2 +  // min_yield_threshold
        8 +  // rebalance_frequency
        1 +  // is_active
        8 +  // created_at
        8 +  // last_rebalance_at
        8 +  // last_compound_at
        8 +  // total_deposited
        8 +  // total_rewards_claimed
        8 +  // rebalance_count
        8 +  // compound_count
        1 +  // emergency_pause
        1;   // bump = 151 bytes

    pub fn is_operational(&self) -> bool {
        self.is_active && !self.emergency_pause
    }

    pub fn can_rebalance(&self, current_time: i64) -> bool {
        current_time - self.last_rebalance_at >= self.rebalance_frequency
    }

    pub fn can_compound(&self, current_time: i64) -> bool {
        current_time - self.last_compound_at >= self.strategy_config.compound_frequency
    }

    pub fn calculate_performance(&self, current_time: i64) -> u16 {
        if self.total_deposited == 0 {
            return 0;
        }

        let time_elapsed = current_time - self.created_at;
        if time_elapsed <= 0 {
            return 0;
        }

        // Simple APY calculation: (rewards / principal) / time_in_years * 100
        let years_elapsed = time_elapsed as f64 / (365.25 * 24.0 * 3600.0);
        let return_rate = self.total_rewards_claimed as f64 / self.total_deposited as f64;
        let apy = (return_rate / years_elapsed * 10000.0) as u16;
        
        apy.min(50000) // Cap at 500% APY for safety
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct YieldStrategyConfig {
    pub max_risk_level: u8,        // 1-10 scale (1 byte)
    pub compound_frequency: i64,   // seconds between auto-compounds (8 bytes)
    pub target_protocols: Vec<u8>, // protocol IDs (up to 31 bytes including length)
}

impl YieldStrategyConfig {
    pub fn validate(&self) -> bool {
        self.max_risk_level >= 1 
            && self.max_risk_level <= 10 
            && self.compound_frequency >= 3600 // At least 1 hour
            && self.target_protocols.len() <= 10
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct ProtocolInfo {
    pub name: String,
    pub protocol_id: u8,
    pub apy: u16, // basis points
    pub risk_level: u8,
    pub allocation_amount: u64,
    pub total_value_locked: u64,
    pub supported_tokens: Vec<Pubkey>,
    pub is_active: bool,
}

impl ProtocolInfo {
    pub fn risk_adjusted_score(&self) -> u32 {
        // Higher APY with lower risk gets higher score
        (self.apy as u32 * 1000) / (self.risk_level as u32 + 1)
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct AllocationTarget {
    pub protocol_id: u8,
    pub percentage: u16, // basis points (10000 = 100%)
    pub target_apy: u16,
    pub risk_weight: u8,
}

impl AllocationTarget {
    pub fn validate(&self) -> bool {
        self.percentage <= 10000 && self.risk_weight <= 10
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum WithdrawType {
    Normal,
    Emergency,
    Partial { percentage: u16 }, // basis points
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum StrategyType {
    Conservative,  // Low risk, stable returns
    Balanced,      // Medium risk, balanced returns
    Aggressive,    // High risk, high returns
    Custom { risk_level: u8 }, // User-defined risk level
}

impl StrategyType {
    pub fn get_risk_level(&self) -> u8 {
        match self {
            StrategyType::Conservative => 3,
            StrategyType::Balanced => 6,
            StrategyType::Aggressive => 9,
            StrategyType::Custom { risk_level } => *risk_level,
        }
    }

    pub fn get_min_yield_threshold(&self) -> u16 {
        match self {
            StrategyType::Conservative => 300,  // 3% APY
            StrategyType::Balanced => 600,      // 6% APY
            StrategyType::Aggressive => 1200,   // 12% APY
            StrategyType::Custom { risk_level } => (*risk_level as u16) * 100,
        }
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct RebalanceParams {
    pub new_allocations: Vec<AllocationTarget>,
    pub ai_confidence: u8,
    pub max_slippage: u16, // basis points
    pub deadline: i64,     // Unix timestamp
}

impl RebalanceParams {
    pub fn validate(&self) -> bool {
        if self.new_allocations.is_empty() || self.ai_confidence < 50 {
            return false;
        }

        let total_allocation: u16 = self.new_allocations.iter()
            .map(|a| a.percentage)
            .sum();

        total_allocation == 10000 && // Must sum to 100%
        self.new_allocations.iter().all(|a| a.validate()) &&
        self.max_slippage <= 1000 // Max 10% slippage
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct CompoundParams {
    pub estimated_rewards: u64,
    pub compound_all_protocols: bool,
    pub specific_protocols: Option<Vec<u8>>,
    pub min_compound_amount: u64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct DepositParams {
    pub amount: u64,
    pub auto_compound: bool,
    pub preferred_protocols: Option<Vec<u8>>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct WithdrawParams {
    pub amount: u64,
    pub withdraw_type: WithdrawType,
    pub emergency_reason: Option<String>,
}

/// Performance tracking structure
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct PerformanceMetrics {
    pub total_return: u64,
    pub annualized_return: u16, // basis points
    pub max_drawdown: u16,      // basis points
    pub sharpe_ratio: u16,      // scaled by 100
    pub win_rate: u16,          // basis points
    pub total_transactions: u64,
    pub successful_rebalances: u64,
    pub failed_rebalances: u64,
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self {
            total_return: 0,
            annualized_return: 0,
            max_drawdown: 0,
            sharpe_ratio: 0,
            win_rate: 0,
            total_transactions: 0,
            successful_rebalances: 0,
            failed_rebalances: 0,
        }
    }
}
