use anchor_lang::prelude::*;

declare_id!("C7QmRHfSMg9YtcFyjGdfqA1PxBjZzefxxsBNggFTgcqR");

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
pub use state::{
    ArbitrageOpportunity, ArbitrageStatus, DcaStatus, DcaStrategy, ExecutionType, LimitOrder,
    MarketData, OrderSide, OrderStatus, StopLoss, StrategyType, TokenAutomation, Treasury,
};
pub use utils::*;

// Import all instruction handlers and structures
#[allow(ambiguous_glob_reexports)]
pub use instructions::*;

#[program]
pub mod token_automation {
    use super::*;

    // Initialization and management
    pub fn initialize_token_automation(
        ctx: Context<InitializeTokenAutomation>,
        params: InitializeTokenParams,
    ) -> Result<()> {
        instructions::initialize::initialize_token_automation(ctx, params)
    }

    pub fn update_automation_settings(
        ctx: Context<UpdateAutomationSettings>,
        params: UpdateParams,
    ) -> Result<()> {
        instructions::initialize::update_automation_settings(ctx, params)
    }

    pub fn toggle_automation_state(
        ctx: Context<ToggleAutomationState>,
        active: bool,
    ) -> Result<()> {
        instructions::initialize::toggle_automation_state(ctx, active)
    }

    pub fn emergency_pause(ctx: Context<EmergencyPause>) -> Result<()> {
        instructions::initialize::emergency_pause(ctx)
    }

    pub fn resume_from_pause(ctx: Context<ResumeFromPause>) -> Result<()> {
        instructions::initialize::resume_from_pause(ctx)
    }

    pub fn transfer_authority(ctx: Context<TransferAuthority>) -> Result<()> {
        instructions::initialize::transfer_authority(ctx)
    }

    pub fn close_automation(ctx: Context<CloseAutomation>) -> Result<()> {
        instructions::initialize::close_automation(ctx)
    }

    // Limit Orders
    pub fn create_limit_order(
        ctx: Context<CreateLimitOrder>,
        params: LimitOrderParams,
    ) -> Result<()> {
        instructions::limit_orders::create_limit_order(ctx, params)
    }

    pub fn cancel_limit_order(ctx: Context<CancelLimitOrder>) -> Result<()> {
        instructions::limit_orders::cancel_limit_order(ctx)
    }

    pub fn execute_limit_order(
        ctx: Context<ExecuteLimitOrder>,
        current_price: u64,
        amount_out: u64,
    ) -> Result<()> {
        instructions::limit_orders::execute_limit_order(ctx, current_price, amount_out)
    }

    pub fn update_limit_order(
        ctx: Context<UpdateLimitOrder>,
        new_price_threshold: Option<u64>,
        new_expires_at: Option<i64>,
        new_max_slippage: Option<u16>,
    ) -> Result<()> {
        instructions::limit_orders::update_limit_order(
            ctx,
            new_price_threshold,
            new_expires_at,
            new_max_slippage,
        )
    }

    // Stop Loss Orders
    pub fn create_stop_loss(ctx: Context<CreateStopLoss>, params: StopLossParams) -> Result<()> {
        instructions::stop_loss::create_stop_loss(ctx, params)
    }

    pub fn cancel_stop_loss(ctx: Context<CancelStopLoss>) -> Result<()> {
        instructions::stop_loss::cancel_stop_loss(ctx)
    }

    pub fn execute_stop_loss(
        ctx: Context<ExecuteStopLoss>,
        current_price: u64,
        amount_out: u64,
    ) -> Result<()> {
        instructions::stop_loss::execute_stop_loss(ctx, current_price, amount_out)
    }

    pub fn update_stop_loss(
        ctx: Context<UpdateStopLoss>,
        new_stop_price: Option<u64>,
        new_expires_at: Option<i64>,
        new_max_slippage: Option<u16>,
    ) -> Result<()> {
        instructions::stop_loss::update_stop_loss(
            ctx,
            new_stop_price,
            new_expires_at,
            new_max_slippage,
        )
    }

    // DCA Strategies
    pub fn create_dca_strategy(ctx: Context<CreateDcaStrategy>, params: DcaParams) -> Result<()> {
        instructions::dca::create_dca_strategy(ctx, params)
    }

    pub fn execute_dca_order(
        ctx: Context<ExecuteDcaOrder>,
        current_price: u64,
        amount_out: u64,
    ) -> Result<()> {
        instructions::dca::execute_dca_order(ctx, current_price, amount_out)
    }

    pub fn pause_dca_strategy(ctx: Context<PauseDcaStrategy>) -> Result<()> {
        instructions::dca::pause_dca_strategy(ctx)
    }

    pub fn resume_dca_strategy(ctx: Context<ResumeDcaStrategy>) -> Result<()> {
        instructions::dca::resume_dca_strategy(ctx)
    }

    pub fn cancel_dca_strategy(ctx: Context<CancelDcaStrategy>) -> Result<()> {
        instructions::dca::cancel_dca_strategy(ctx)
    }

    pub fn update_dca_strategy(
        ctx: Context<UpdateDcaStrategy>,
        new_interval_seconds: Option<i64>,
        new_max_slippage: Option<u16>,
    ) -> Result<()> {
        instructions::dca::update_dca_strategy(ctx, new_interval_seconds, new_max_slippage)
    }

    // Swaps
    pub fn execute_ai_swap(ctx: Context<ExecuteAiSwap>, params: SwapParams) -> Result<()> {
        instructions::swaps::execute_ai_swap(ctx, params)
    }

    pub fn execute_manual_swap(
        ctx: Context<ExecuteManualSwap>,
        amount_in: u64,
        minimum_amount_out: u64,
        max_slippage: u16,
    ) -> Result<()> {
        instructions::swaps::execute_manual_swap(ctx, amount_in, minimum_amount_out, max_slippage)
    }

    pub fn batch_swap(ctx: Context<BatchSwap>, swap_instructions: Vec<SwapParams>) -> Result<()> {
        instructions::swaps::batch_swap(ctx, swap_instructions)
    }

    pub fn emergency_stop_swaps(ctx: Context<EmergencyStopSwaps>) -> Result<()> {
        instructions::swaps::emergency_stop_swaps(ctx)
    }

    // Arbitrage
    pub fn create_arbitrage_opportunity(
        ctx: Context<CreateArbitrageOpportunity>,
        params: ArbitrageParams,
    ) -> Result<()> {
        instructions::arbitrage::create_arbitrage_opportunity(ctx, params)
    }

    pub fn execute_arbitrage(
        ctx: Context<ExecuteArbitrage>,
        amount: u64,
        price_a: u64,
        price_b: u64,
    ) -> Result<()> {
        instructions::arbitrage::execute_arbitrage(ctx, amount, price_a, price_b)
    }

    pub fn cancel_arbitrage_opportunity(ctx: Context<CancelArbitrageOpportunity>) -> Result<()> {
        instructions::arbitrage::cancel_arbitrage_opportunity(ctx)
    }

    pub fn scan_arbitrage_opportunities(
        ctx: Context<ScanArbitrageOpportunities>,
        token_pairs: Vec<(Pubkey, Pubkey)>,
        dex_pairs: Vec<(Pubkey, Pubkey)>,
    ) -> Result<()> {
        instructions::arbitrage::scan_arbitrage_opportunities(ctx, token_pairs, dex_pairs)
    }
}
