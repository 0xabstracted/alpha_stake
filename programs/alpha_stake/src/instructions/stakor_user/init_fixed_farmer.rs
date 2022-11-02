use std::io::Write;

use crate::bank_state::Bank;
use crate::bank_state::Vault;
use crate::state::Farm;
use crate::state::Farmer;
use crate::state::FarmerStakedMints;
use crate::state::FixedRateSchedule;
// use crate::state::LPRateSchedule;
use anchor_lang::prelude::*;
// use gem_bank::{self, cpi::accounts::InitVault, program::GemBank, state::Bank};
use gem_common::TryAdd;

#[derive(Accounts)]
#[instruction(index: u32)]
pub struct InitFixedFarmer<'info> {
    // farm
    #[account(mut, has_one = bank)]
    pub farm: Box<Account<'info, Farm>>,

    // farmer
    #[account(init, seeds = [
            b"farmer".as_ref(),
            farm.key().as_ref(),
            identity.key().as_ref(),
        ],
        bump,
        payer = payer,
        space = 8 + std::mem::size_of::<Farmer>())]
    pub farmer: Box<Account<'info, Farmer>>,
    #[account(
        init,
        seeds = [
            b"farmer_staked_mints".as_ref(), 
            // &index.to_le_bytes(),
            farmer.key().as_ref(),
        ],
        bump,
        payer = payer,
        space = 8 + std::mem::size_of::<FarmerStakedMints>(),
        // owner = *program_id,
    )]
    // #[account(zero)]
    // pub farmer_staked_mints: Account<'info, FarmerStakedMints>,
    pub farmer_staked_mints: AccountLoader<'info, FarmerStakedMints>,
    pub identity: Signer<'info>,

    // cpi
    #[account(mut)]
    pub bank: Box<Account<'info, Bank>>,
    // trying to deserialize here leads to errors (doesn't exist yet)
    /// CHECK:
    #[account(init, seeds = [
        b"vault".as_ref(),
        bank.key().as_ref(),
        identity.key().as_ref(),
    ],
    bump,
    payer = payer,
    space = 8 + std::mem::size_of::<Vault>())]
    pub vault: Box<Account<'info, Vault>>,
    // pub vault: AccountInfo<'info>,
    // pub gem_bank: Program<'info, GemBank>,

    // misc
    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

impl<'info> InitFixedFarmer<'info> {
    // fn init_vault_ctx(&self) -> CpiContext<'_, '_, '_, 'info, InitVault<'info>> {
    //     CpiContext::new(
    //         self.gem_bank.to_account_info(),
    //         InitVault {
    //             bank: self.bank.to_account_info(),
    //             vault: self.vault.clone(),
    //             // creator = the identity of the farmer
    //             creator: self.identity.to_account_info(),
    //             payer: self.payer.to_account_info(),
    //             system_program: self.system_program.to_account_info(),
    //         },
    //     )
    // }
}

pub fn handler(ctx: Context<InitFixedFarmer>, _index: u32) -> Result<()> {
    // record new farmer details
    let farmer = &mut ctx.accounts.farmer;
    farmer.farm = ctx.accounts.farm.key();
    farmer.identity = ctx.accounts.identity.key();
    farmer.vault = ctx.accounts.vault.key();
    farmer.reward_a.fixed_rate.promised_schedule = FixedRateSchedule::default();
    farmer.append_fsm(ctx.accounts.farmer_staked_mints.key())?;
    // farmer.lp_points.lp_rate.lp_promised_schedule = LPRateSchedule::default();

    // update farm
    let farm = &mut ctx.accounts.farm;
    farm.farmer_count.try_add_assign(1)?;
    // msg!("farmer.reward_a {:?}", farmer.reward_a);
    // msg!("farmer.lp_points {:?}", farmer.lp_points);
    // do a cpi call to start a new vault
    let vault_name = String::from("farm_vault");
    
    // record total number of vaults in bank's state
    let bank = &mut ctx.accounts.bank;
    let vault = &mut ctx.accounts.vault;

    bank.vault_count.try_add_assign(1)?;

    // derive the authority responsible for all token transfers within the new vault
    let vault_address = vault.key();
    let authority_seed = &[vault_address.as_ref()];
    let (authority, bump) = Pubkey::find_program_address(authority_seed, ctx.program_id);

    // record vault's state
    vault.bank = bank.key();
    vault.identity = ctx.accounts.identity.key();
    vault.creator = ctx.accounts.identity.key();
    vault.vault_authority = authority;
    vault.vault_authority_seed = vault_address;
    vault.vault_authority_bump_seed = [bump];
    vault.locked = false;
    (&mut vault.name[..]).write_all(vault_name.as_bytes())?;

  //  msg!("new vault founded by {}", &ctx.accounts.creator.key());
    
    // gem_bank::cpi::init_vault(ctx.accounts.init_vault_ctx(), vault_owner, vault_name)?;
    let mut farmer_staked_mints = ctx.accounts.farmer_staked_mints.load_init()?;
    // let mut farmer_staked_mints = ctx.accounts.farmer_staked_mints;
    farmer_staked_mints.no_of_nfts_staked = 0;
    farmer_staked_mints.bump = ctx.bumps.get("farmer_staked_mints").unwrap().clone();
    farmer_staked_mints.farmer = ctx.accounts.farmer.key();
    farmer_staked_mints.index = 0;
    // let farmer = &mut ctx.accounts.farmer;
    // farmer.reload()?;
    // msg!("farmer.fsm_account_keys[i]: {:?}", farmer.fsm_account_keys);
    // msg!("self.no_of_nfts_staked{}", farmer.no_fsm_accounts);
    // msg!("farmer_staked_mints.farmer_staked_mints[i]: {:?}", farmer_staked_mints.farmer_staked_mints);
    // msg!("self.no_of_nfts_staked{}", farmer_staked_mints.no_of_nfts_staked);
    // farmer_staked_mints.index = index;
    // let default_staked_mint: Pubkey = Pubkey::default();
    // for _ in 0..MAX_NFTS_ALLOWED{
    //     farmer_staked_mints.append_nft(default_staked_mint)?;
    // }
    // msg!("new fixed farmer initialized");
    Ok(())
}
