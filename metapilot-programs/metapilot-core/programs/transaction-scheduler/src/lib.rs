use anchor_lang::prelude::*;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnT");

// Module declarations
pub mod constants;
pub mod errors;
pub mod events;
pub mod instructions;
pub mod state;
pub mod utils;

// Re-exports for external access
pub use constants::*;
pub use errors::*;
pub use events::*;
pub use state::*;
pub use utils::*;

// Import all instruction handlers and structures
#[allow(ambiguous_glob_reexports)]
pub use instructions::*;

#[program]
pub mod transaction_scheduler {
    use super::*;

    /// Initialize transaction scheduler
    pub fn initialize_scheduler(
        ctx: Context<InitializeScheduler>,
        params: InitializeSchedulerParams,
    ) -> Result<()> {
        instructions::initialize_scheduler::handler(ctx, params)
    }

    /// Schedule a transaction for optimal execution
    pub fn schedule_transaction(
        ctx: Context<ScheduleTransaction>,
        params: ScheduleTransactionParams,
    ) -> Result<()> {
        instructions::schedule_transaction::handler(ctx, params)
    }

    /// Execute a scheduled transaction
    pub fn execute_transaction(
        ctx: Context<ExecuteTransaction>,
        params: ExecuteTransactionParams,
    ) -> Result<()> {
        instructions::execute_transaction::handler(ctx, params)
    }

    /// Batch execute multiple transactions
    pub fn batch_execute(ctx: Context<BatchExecute>, params: BatchExecuteParams) -> Result<()> {
        instructions::batch_execute::handler(ctx, params)
    }

    /// Cancel a scheduled transaction
    pub fn cancel_transaction(
        ctx: Context<CancelTransaction>,
        params: CancelTransactionParams,
    ) -> Result<()> {
        instructions::cancel_transaction::handler(ctx, params)
    }

    /// Update gas price settings
    pub fn update_gas_settings(
        ctx: Context<UpdateGasSettings>,
        params: UpdateGasSettingsParams,
    ) -> Result<()> {
        instructions::update_gas_settings::handler(ctx, params)
    }
}
