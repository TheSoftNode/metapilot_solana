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

// Import all instruction handlers and structures
#[allow(ambiguous_glob_reexports)]
pub use instructions::*;

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

    /// Create a new proposal
    pub fn create_proposal(
        ctx: Context<CreateProposal>,
        params: CreateProposalParams,
    ) -> Result<()> {
        instructions::proposals::create_proposal(ctx, params)
    }

    /// Cast a vote on a proposal
    pub fn cast_vote(
        ctx: Context<CastVote>,
        proposal_id: u64,
        vote_choice: VoteChoice,
        voting_weight: u64,
    ) -> Result<()> {
        instructions::voting::cast_vote(ctx, proposal_id, vote_choice, voting_weight)
    }

    /// Cast a delegated vote
    pub fn cast_delegated_vote(
        ctx: Context<CastDelegatedVote>,
        proposal_id: u64,
        vote_choice: VoteChoice,
    ) -> Result<()> {
        instructions::voting::cast_delegated_vote(ctx, proposal_id, vote_choice)
    }

    /// Execute automated vote based on AI analysis
    pub fn execute_automated_vote(
        ctx: Context<ExecuteAutomatedVote>,
        params: AutomatedVoteParams,
    ) -> Result<()> {
        instructions::automation::execute_automated_vote(ctx, params)
    }

    /// Finalize a proposal after voting period ends
    pub fn finalize_proposal(ctx: Context<FinalizeProposal>, proposal_id: u64) -> Result<()> {
        instructions::proposals::finalize_proposal(ctx, proposal_id)
    }

    /// Execute a finalized proposal
    pub fn execute_proposal(ctx: Context<ExecuteProposal>, proposal_id: u64) -> Result<()> {
        instructions::proposals::execute_proposal(ctx, proposal_id)
    }

    /// Update voting rules for DAO automation
    pub fn update_voting_rules(
        ctx: Context<UpdateVotingRules>,
        new_rules: VotingRules,
    ) -> Result<()> {
        instructions::automation::update_voting_rules(ctx, new_rules)
    }

    /// Toggle automation on/off
    pub fn toggle_automation(ctx: Context<ToggleAutomation>, is_active: bool) -> Result<()> {
        instructions::automation::toggle_automation(ctx, is_active)
    }

    /// Delegate voting power to another account
    pub fn delegate_voting_power(
        ctx: Context<DelegateVotingPower>,
        delegate: Pubkey,
        amount: u64,
    ) -> Result<()> {
        instructions::delegation::delegate_voting_power(ctx, delegate, amount)
    }

    /// Revoke previously delegated voting power
    pub fn revoke_delegation(ctx: Context<RevokeDelegation>, delegate: Pubkey) -> Result<()> {
        instructions::delegation::revoke_delegation(ctx, delegate)
    }

    /// Withdraw accumulated rewards from governance participation
    pub fn withdraw_governance_rewards(
        ctx: Context<WithdrawGovernanceRewards>,
        amount: u64,
    ) -> Result<()> {
        instructions::rewards::withdraw_governance_rewards(ctx, amount)
    }

    /// Update AI automation settings
    pub fn update_ai_settings(
        ctx: Context<UpdateAiSettings>,
        settings: AiAutomationSettings,
    ) -> Result<()> {
        instructions::automation::update_ai_settings(ctx, settings)
    }

    /// Emergency pause for critical situations
    pub fn emergency_pause(ctx: Context<EmergencyPause>) -> Result<()> {
        instructions::admin::emergency_pause(ctx)
    }

    /// Resume operations after emergency pause
    pub fn resume_operations(ctx: Context<ResumeOperations>) -> Result<()> {
        instructions::admin::resume_operations(ctx)
    }
}
