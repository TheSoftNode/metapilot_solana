use anchor_lang::prelude::*;

declare_id!("CSxir1KkCbcKZpmUuEouTL4uLzwL4C7MTYyZrkxcKjv1");

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

// Import just the initialize module to test
pub use instructions::initialize::{initialize_dao_automation, InitializeDaoAutomation};
pub use state::{AiAutomationSettings, InitializeDaoParams, VoteChoice, VotingRules};

#[program]
pub mod dao_voting {
    use super::*;

    /// Initialize a new DAO voting automation account
    pub fn initialize_dao_automation(
        ctx: Context<InitializeDaoAutomation>,
        params: InitializeDaoParams,
    ) -> Result<()> {
        instructions::initialize::initialize_dao_automation(ctx, params)
    }
}
