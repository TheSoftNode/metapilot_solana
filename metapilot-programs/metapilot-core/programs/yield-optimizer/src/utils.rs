/// Utility functions for Yield Optimizer Program
use anchor_lang::prelude::*;
use crate::state::{ProtocolInfo, AllocationTarget, YieldStrategyConfig, PerformanceMetrics};
use crate::constants::*;
use crate::errors::YieldOptimizerError;

pub struct YieldCalculator;

impl YieldCalculator {
    /// Calculate the optimal allocation based on APY and risk levels
    pub fn calculate_optimal_allocation(
        protocols: &[ProtocolInfo],
        max_risk: u8,
        target_yield: u16,
    ) -> Result<Vec<AllocationTarget>> {
        if protocols.is_empty() {
            return Err(YieldOptimizerError::NoProtocolsAvailable.into());
        }

        // Filter protocols by risk level and active status
        let eligible_protocols: Vec<&ProtocolInfo> = protocols
            .iter()
            .filter(|p| p.risk_level <= max_risk && p.is_active && p.apy >= MIN_APY_THRESHOLD)
            .collect();

        if eligible_protocols.is_empty() {
            return Err(YieldOptimizerError::NoEligibleProtocols.into());
        }

        // Calculate risk-adjusted scores
        let mut scored_protocols: Vec<(u8, u32, u16)> = eligible_protocols
            .iter()
            .map(|p| (p.protocol_id, p.risk_adjusted_score(), p.apy))
            .collect();

        // Sort by risk-adjusted score (descending)
        scored_protocols.sort_by(|a, b| b.1.cmp(&a.1));

        // Calculate allocations using modern portfolio theory principles
        let mut allocations = Vec::new();
        let total_score: u32 = scored_protocols.iter().map(|(_, score, _)| score).sum();

        if total_score == 0 {
            return Err(YieldOptimizerError::InvalidAllocation.into());
        }

        let mut remaining_allocation = 10000u16; // 100% in basis points

        for (i, (protocol_id, score, apy)) in scored_protocols.iter().enumerate() {
            let base_allocation = if i == scored_protocols.len() - 1 {
                // Last protocol gets remaining allocation
                remaining_allocation
            } else {
                // Calculate proportional allocation based on score
                let proportion = (*score as u64 * 10000u64) / total_score as u64;
                let allocation = (proportion as u16).min(remaining_allocation);
                
                // Apply minimum allocation threshold
                if allocation < MIN_ALLOCATION_BASIS_POINTS {
                    0
                } else {
                    allocation
                }
            };

            if base_allocation > 0 {
                remaining_allocation = remaining_allocation.saturating_sub(base_allocation);
                
                allocations.push(AllocationTarget {
                    protocol_id: *protocol_id,
                    percentage: base_allocation,
                    target_apy: *apy,
                    risk_weight: eligible_protocols
                        .iter()
                        .find(|p| p.protocol_id == *protocol_id)
                        .map(|p| p.risk_level)
                        .unwrap_or(5),
                });
            }
        }

        // Ensure we have valid allocations
        if allocations.is_empty() {
            return Err(YieldOptimizerError::InvalidAllocation.into());
        }

        // Normalize allocations to exactly 100%
        Self::normalize_allocations(&mut allocations)?;

        Ok(allocations)
    }

    /// Normalize allocations to sum to exactly 100%
    fn normalize_allocations(allocations: &mut Vec<AllocationTarget>) -> Result<()> {
        let total: u16 = allocations.iter().map(|a| a.percentage).sum();
        
        if total == 0 {
            return Err(YieldOptimizerError::InvalidAllocation.into());
        }

        if total != 10000 {
            // Adjust the largest allocation to make total exactly 10000
            if let Some(largest) = allocations.iter_mut().max_by_key(|a| a.percentage) {
                let adjustment = 10000i32 - total as i32;
                let new_percentage = (largest.percentage as i32 + adjustment).max(0) as u16;
                largest.percentage = new_percentage;
            }
        }

        Ok(())
    }

    /// Calculate expected portfolio yield based on allocations
    pub fn calculate_expected_yield(allocations: &[AllocationTarget]) -> u16 {
        let weighted_yield: u32 = allocations
            .iter()
            .map(|a| (a.target_apy as u32 * a.percentage as u32) / 10000)
            .sum();

        weighted_yield as u16
    }

    /// Calculate portfolio risk score
    pub fn calculate_portfolio_risk(allocations: &[AllocationTarget]) -> u8 {
        let weighted_risk: u32 = allocations
            .iter()
            .map(|a| (a.risk_weight as u32 * a.percentage as u32) / 10000)
            .sum();

        (weighted_risk as u8).min(10)
    }

    /// Check if rebalancing is needed based on current vs target allocations
    pub fn should_rebalance(
        current_allocations: &[AllocationTarget],
        target_allocations: &[AllocationTarget],
        threshold_basis_points: u16,
    ) -> bool {
        for target in target_allocations {
            let current_percentage = current_allocations
                .iter()
                .find(|c| c.protocol_id == target.protocol_id)
                .map(|c| c.percentage)
                .unwrap_or(0);

            let deviation = if current_percentage > target.percentage {
                current_percentage - target.percentage
            } else {
                target.percentage - current_percentage
            };

            if deviation > threshold_basis_points {
                return true;
            }
        }

        false
    }
}

pub struct RiskManager;

impl RiskManager {
    /// Assess the risk of a protocol based on various factors
    pub fn assess_protocol_risk(protocol: &ProtocolInfo, tvl_threshold: u64) -> u8 {
        let mut risk_score = protocol.risk_level;

        // Adjust risk based on TVL (lower TVL = higher risk)
        if protocol.total_value_locked < tvl_threshold {
            risk_score = (risk_score + 2).min(10);
        }

        // Adjust risk based on APY (unusually high APY = higher risk)
        if protocol.apy > RISK_HIGH_APY_THRESHOLD {
            risk_score = (risk_score + 1).min(10);
        }

        risk_score
    }

    /// Validate that portfolio doesn't exceed maximum risk tolerance
    pub fn validate_portfolio_risk(
        allocations: &[AllocationTarget],
        max_risk: u8,
    ) -> Result<()> {
        let portfolio_risk = YieldCalculator::calculate_portfolio_risk(allocations);
        
        if portfolio_risk > max_risk {
            return Err(YieldOptimizerError::RiskToleranceExceeded.into());
        }

        Ok(())
    }

    /// Check for concentration risk (too much allocation to single protocol)
    pub fn check_concentration_risk(allocations: &[AllocationTarget]) -> Result<()> {
        for allocation in allocations {
            if allocation.percentage > MAX_SINGLE_PROTOCOL_ALLOCATION {
                return Err(YieldOptimizerError::ConcentrationRiskTooHigh.into());
            }
        }

        Ok(())
    }
}

pub struct PerformanceTracker;

impl PerformanceTracker {
    /// Update performance metrics after a rebalance or compound operation
    pub fn update_metrics(
        metrics: &mut PerformanceMetrics,
        operation_success: bool,
        new_return: u64,
        time_elapsed: i64,
    ) {
        metrics.total_transactions += 1;
        
        if operation_success {
            metrics.successful_rebalances += 1;
            metrics.total_return = metrics.total_return.saturating_add(new_return);
            
            // Update win rate
            metrics.win_rate = (metrics.successful_rebalances * 10000 / metrics.total_transactions) as u16;
            
            // Simple annualized return calculation
            if time_elapsed > 0 {
                let years_elapsed = time_elapsed as f64 / (365.25 * 24.0 * 3600.0);
                if years_elapsed > 0.0 {
                    metrics.annualized_return = ((metrics.total_return as f64 / years_elapsed) * 10000.0 / 1_000_000.0) as u16;
                }
            }
        } else {
            metrics.failed_rebalances += 1;
        }
    }

    /// Calculate Sharpe ratio (simplified version)
    pub fn calculate_sharpe_ratio(
        returns: &[u64],
        risk_free_rate: u16, // basis points
    ) -> u16 {
        if returns.len() < 2 {
            return 0;
        }

        let mean_return = returns.iter().sum::<u64>() / returns.len() as u64;
        let variance = returns
            .iter()
            .map(|&r| {
                let diff = if r > mean_return { r - mean_return } else { mean_return - r };
                diff * diff
            })
            .sum::<u64>() / returns.len() as u64;

        let std_dev = (variance as f64).sqrt() as u64;
        
        if std_dev == 0 {
            return 0;
        }

        let excess_return = mean_return.saturating_sub(risk_free_rate as u64);
        ((excess_return * 100) / std_dev) as u16
    }
}

pub struct ValidationUtils;

impl ValidationUtils {
    /// Validate strategy configuration
    pub fn validate_strategy_config(config: &YieldStrategyConfig) -> Result<()> {
        if !config.validate() {
            return Err(YieldOptimizerError::InvalidStrategyConfig.into());
        }

        if config.target_protocols.is_empty() {
            return Err(YieldOptimizerError::NoProtocolsSelected.into());
        }

        // Check for duplicate protocols
        let mut seen_protocols = std::collections::HashSet::new();
        for &protocol_id in &config.target_protocols {
            if !seen_protocols.insert(protocol_id) {
                return Err(YieldOptimizerError::DuplicateProtocol.into());
            }
        }

        Ok(())
    }

    /// Validate allocation amounts
    pub fn validate_allocation_amounts(
        allocations: &[AllocationTarget],
        total_amount: u64,
    ) -> Result<()> {
        if total_amount < MIN_DEPOSIT_AMOUNT {
            return Err(YieldOptimizerError::DepositTooSmall.into());
        }

        let total_percentage: u16 = allocations.iter().map(|a| a.percentage).sum();
        if total_percentage != 10000 {
            return Err(YieldOptimizerError::InvalidAllocation.into());
        }

        // Check minimum allocation amounts
        for allocation in allocations {
            let allocation_amount = (total_amount * allocation.percentage as u64) / 10000;
            if allocation_amount < MIN_PROTOCOL_ALLOCATION {
                return Err(YieldOptimizerError::AllocationTooSmall.into());
            }
        }

        Ok(())
    }

    /// Validate timing constraints
    pub fn validate_timing(
        last_operation: i64,
        min_interval: i64,
        current_time: i64,
    ) -> Result<()> {
        if current_time < last_operation + min_interval {
            return Err(YieldOptimizerError::OperationTooFrequent.into());
        }

        Ok(())
    }
}

/// Helper function to calculate slippage tolerance
pub fn calculate_slippage_tolerance(
    expected_amount: u64,
    actual_amount: u64,
) -> u16 {
    if expected_amount == 0 {
        return 0;
    }

    if actual_amount >= expected_amount {
        return 0; // No slippage, actually got more
    }

    let slippage = expected_amount - actual_amount;
    ((slippage * 10000) / expected_amount) as u16
}

/// Helper function to format APY for display
pub fn format_apy(apy_basis_points: u16) -> String {
    let percentage = apy_basis_points as f64 / 100.0;
    format!("{:.2}%", percentage)
}

/// Helper function to calculate compound frequency in human-readable format
pub fn calculate_compound_frequency_days(frequency_seconds: i64) -> f64 {
    frequency_seconds as f64 / 86400.0 // Convert seconds to days
}
