use crate::{state::*, bank_state::{Bank, LATEST_BANK_VERSION}};
use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};
// use gem_bank::{self, cpi::accounts::InitBank, program::GemBank};
use gem_common::errors::ErrorCode;

#[derive(Accounts)]
#[instruction(bump_auth: u8)]
pub struct InitFixedFarm<'info> {
    // farm
    #[account(init, payer = payer, space = 8 + std::mem::size_of::<Farm>())]
    pub farm: Box<Account<'info, Farm>>,
    pub farm_manager: Signer<'info>,
    /// CHECK:
    #[account(mut, seeds = [farm.key().as_ref()], bump = bump_auth)]
    pub farm_authority: AccountInfo<'info>,

    // reward a
    #[account(init, seeds = [
            b"reward_pot".as_ref(),
            farm.key().as_ref(),
            reward_a_mint.key().as_ref(),
        ],
        bump,
        token::mint = reward_a_mint,
        token::authority = farm_authority,
        payer = payer)]
    pub reward_a_pot: Box<Account<'info, TokenAccount>>,
    pub reward_a_mint: Box<Account<'info, Mint>>,
    #[account(init, seeds = [
            b"token_treasury".as_ref(),
            farm.key().as_ref(),
        ],
        bump,
        token::mint = reward_a_mint,
        token::authority = farm_authority,
        payer = payer)]
    pub farm_treasury_token: Box<Account<'info, TokenAccount>>,
    #[account(init, payer = payer, space = 8 + std::mem::size_of::<Bank>())]
    pub bank: Box<Account<'info, Bank>>,

    // misc
    #[account(mut)]
    pub payer: Signer<'info>,
    pub rent: Sysvar<'info, Rent>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}


pub fn handler(
    ctx: Context<InitFixedFarm>,
    bump_auth: u8,
    farm_config: FarmConfig,
    max_counts: Option<MaxCounts>,
    farm_treasury_token: Pubkey,
) -> Result<()> {
    //record new farm details
    let farm = &mut ctx.accounts.farm;
    
    // manually verify treasury
    let (pk, _bump) = Pubkey::find_program_address(
        &[b"token_treasury".as_ref(),
        farm.key().as_ref()
        ],
        ctx.program_id,
    );
    if farm_treasury_token.key() != pk {
        return Err(error!(ErrorCode::InvalidParameter));
    }

    // if farm_config.unstaking_fee_percent > 100 {
    //     return Err(error!(ErrorCode::InvalidUnstakingFee));
    // }

    
    farm.version = LATEST_FARM_VERSION;
    farm.farm_manager = ctx.accounts.farm_manager.key();
    farm.farm_treasury_token = farm_treasury_token;
    farm.farm_authority = ctx.accounts.farm_authority.key();
    farm.farm_authority_seed = farm.key();
    farm.farm_authority_bump_seed = [bump_auth];
    farm.bank = ctx.accounts.bank.key();
    farm.config = farm_config;

    farm.reward_a.reward_mint = ctx.accounts.reward_a_mint.key();
    farm.reward_a.reward_pot = ctx.accounts.reward_a_pot.key();
    farm.reward_a.reward_type = RewardType::Fixed;
    farm.reward_a.fixed_rate_reward.schedule = FixedRateSchedule::default();
   
    if let Some(max_counts) = max_counts {
        farm.max_counts = max_counts;
    }
    msg!("Init farm: config {:?}", farm.config);
    msg!("Init farm: reward_a {:?}", farm.reward_a);
    
    let bank = &mut ctx.accounts.bank;

    bank.version = LATEST_BANK_VERSION;
    bank.farm_authority = ctx.accounts.farm_authority.key();
    // ctx.accounts.transfer_fee()?;
    //msg!("new farm initialized");
   
    Ok(())
}
