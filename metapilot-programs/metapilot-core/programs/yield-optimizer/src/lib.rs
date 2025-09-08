use anchor_lang::prelude::*;

// Import all modules
pub mod constants;
pub mod errors;
pub mod events;
pub mod instructions;
pub mod state;
pub mod utils;

// Re-export commonly used items
pub use constants::*;
pub use errors::*;
pub use events::*;
pub use instructions::*;
pub use state::*;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod yield_optimizer {
    use super::*;

    // Strategy Management
    pub fn create_strategy(
        ctx: Context<CreateStrategy>,
        config: YieldStrategyConfig,
        strategy_type: StrategyType,
    ) -> Result<()> {
        instructions::create_strategy::handler(ctx, config, strategy_type)
    }

    // Fund Management
    pub fn deposit(ctx: Context<Deposit>, params: DepositParams) -> Result<()> {
        instructions::deposit::handler(ctx, params)
    }

    pub fn withdraw(ctx: Context<Withdraw>, params: WithdrawParams) -> Result<()> {
        instructions::withdraw::handler(ctx, params)
    }

    // AI Optimization
    pub fn rebalance(ctx: Context<Rebalance>, params: RebalanceParams) -> Result<()> {
        instructions::rebalance::handler(ctx, params)
    }

    pub fn compound(ctx: Context<Compound>, params: CompoundParams) -> Result<()> {
        instructions::compound::handler(ctx, params)
    }

    // Emergency Functions
    pub fn emergency_withdraw(
        ctx: Context<EmergencyWithdraw>,
        params: EmergencyWithdrawParams,
    ) -> Result<()> {
        instructions::emergency_withdraw::handler(ctx, params)
    }

    pub fn reactivate_strategy(ctx: Context<ReactivateStrategy>) -> Result<()> {
        instructions::emergency_withdraw::reactivate_strategy_handler(ctx)
    }
}
