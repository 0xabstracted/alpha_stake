use anchor_lang::prelude::*;
pub mod instructions;
pub mod number128;
pub mod state;
pub mod bank_state;

use instructions::*;
use state::*;

declare_id!("2gMWzDAsEn9hepG1yZ2A9zR5KRzFWzUGdFndhfxfXsXD");

#[program]
pub mod alpha_stake {
    use super::*;
    pub fn init_fixed_farm(
        ctx: Context<InitFixedFarm>,
        bump_auth: u8,
        farm_config: FarmConfig,
        max_counts: Option<MaxCounts>,
        farm_treasury_token: Pubkey,
    ) -> Result<()> {
        msg!("init fixed farm");
        instructions::init_fixed_farm::handler(
            ctx,
            bump_auth,
            farm_config,
            max_counts,
            farm_treasury_token
        )
    }
    
    pub fn update_farm(
        ctx: Context<UpdateFarm>,
        config: Option<FarmConfig>,
        manager: Option<Pubkey>,
        max_counts: Option<MaxCounts>,
    ) -> Result<()> {
        instructions::update_farm::handler(ctx, config, manager, max_counts)
    }

    pub fn payout_from_treasury(
        ctx: Context<TreasuryPayout>,
        _bump_auth: u8,
        bump_treasury_token: u8,
        lamports: u64,
    ) -> Result<()> {
        msg!("payout");
        instructions::treasury_payout::handler(ctx, bump_treasury_token, lamports)
    }

    pub fn add_to_bank_whitelist(
        ctx: Context<AddToBankWhitelist>,
        _bump_auth: u8,
        whitelist_type: u8,
    ) -> Result<()> {
        msg!("add to bank whitelist");
        instructions::add_to_bank_whitelist::handler(ctx, whitelist_type)
    }

    pub fn remove_from_bank_whitelist(
        ctx: Context<RemoveFromBankWhitelist>,
        _bump_auth: u8,
        _bump_wl: u8,
    ) -> Result<()> {
        msg!("remove from bank whitelist");
        instructions::remove_from_bank_whitelist::handler(ctx)
    }

    // --------------------------------------- farmer ops

    pub fn init_fixed_farmer(ctx: Context<InitFixedFarmer>, index: u32) -> Result<()> {
        msg!("init farmer fixed");
        instructions::init_fixed_farmer::handler(ctx, index)
    }
    
    pub fn stake(ctx: Context<Stake>, _bump_auth: u8, _bump_farmer: u8) -> Result<()> {
        msg!("stake");
        instructions::stake::handler(ctx)
    }
    
    pub fn unstake(
        ctx: Context<Unstake>,
        _bump_auth: u8,
        _bump_treasury_token: u8,
        _bump_farmer: u8,
        _bump_gem_box: u8,
        _bump_gdr:u8,
        _bump_rarity: u8,
        amount: u64,
        index: u32,
        skip_rewards: bool,
    ) -> Result<()> {
        msg!("unstake");
        instructions::unstake::handler(ctx, skip_rewards, amount, index)
    }

    pub fn claim(
        ctx: Context<Claim>,
        _bump_auth: u8,
        _bump_farmer: u8,
        _bump_pot_a: u8,
        _bump_pot_b: u8,
    ) -> Result<()> {
        msg!("claim");
        instructions::claim::handler(ctx)
    }

    pub fn flash_deposit<'a, 'b, 'c, 'info>(
        ctx: Context<'a, 'b, 'c, 'info, FlashDeposit<'info>>,
        _bump_farmer: u8,
        _bump_farmer_staked_mints: u8,
        _bump_vault_auth: u8,
        _bump_rarity: u8,
        index: u32,
        amount: u64,
    ) -> Result<()> {
        instructions::flash_deposit::handler(ctx, index, amount)
    }

    pub fn refresh_farmer(ctx: Context<RefreshFarmer>, _bump: u8) -> Result<()> {
        msg!("refresh farmer");
        instructions::refresh_farmer::handler(ctx)
    }

    /// this one needs to be called by the farmer themselves
    /// it's useful if for some reason they can't re-enroll in another fixed reward cycle (eg reward exhausted)
    /// but they want to be able to refresh themselves and claim their earned rewards up to this point
    pub fn refresh_farmer_signed(
        ctx: Context<RefreshFarmerSigned>,
        _bump: u8,
        reenroll: bool,
    ) -> Result<()> {
        msg!("refresh farmer signed");
        instructions::refresh_farmer_signed::handler(ctx, reenroll)
    }

    // --------------------------------------- funder ops

    pub fn authorize_funder(ctx: Context<AuthorizeFunder>) -> Result<()> {
        msg!("authorize funder");
        instructions::authorize_funder::handler(ctx)
    }

    pub fn deauthorize_funder(ctx: Context<DeauthorizeFunder>, _bump: u8) -> Result<()> {
        msg!("feauthorize funder");
        instructions::deauthorize_funder::handler(ctx)
    }

    // --------------------------------------- reward ops

    pub fn fund_reward(
        ctx: Context<FundReward>,
        _bump_proof: u8,
        _bump_pot: u8,
        fixed_rate_config: Option<FixedRateConfig>,
    ) -> Result<()> {
        msg!("fund reward");
        instructions::fund_reward::handler(
            ctx,
            fixed_rate_config,
        )
    }

    pub fn cancel_reward(ctx: Context<CancelReward>, _bump_auth: u8, _bump_pot: u8) -> Result<()> {
        msg!("cancel reward");
        instructions::cancel_reward::handler(ctx)
    }

    pub fn lock_reward(ctx: Context<LockReward>) -> Result<()> {
        msg!("lock reward");
        instructions::lock_reward::handler(ctx)
    }

    // --------------------------------------- rarities

    pub fn add_rarities_to_bank<'a, 'b, 'c, 'info>(
        ctx: Context<'a, 'b, 'c, 'info, AddRaritiesToBank<'info>>,
        _bump_auth: u8,
        rarity_configs: Vec<RarityConfig>,
    ) -> Result<()> {
        msg!("add rarities to bank");
        instructions::add_rarities_to_bank::handler(ctx, rarity_configs)
    }

}
