use anchor_lang::prelude::*;
use gem_common::TryAdd;
use gem_common::TrySub;


use crate::bank_state::Bank;
use crate::bank_state::WhitelistProof;
use crate::bank_state::WhitelistType;
use crate::state::Farm;

#[derive(Accounts)]
#[instruction(bump_auth: u8)]
pub struct AddToBankWhitelist<'info> {
    // farm
    #[account(has_one = farm_manager, has_one = farm_authority, has_one = bank)]
    pub farm: Box<Account<'info, Farm>>,
    #[account(mut)]
    pub farm_manager: Signer<'info>,
    /// CHECK:
    #[account(seeds = [farm.key().as_ref()], bump = bump_auth)]
    pub farm_authority: AccountInfo<'info>,

    // cpi
    #[account(mut, has_one = farm_authority)]
    pub bank: Box<Account<'info, Bank>>,
    /// CHECK:
    pub address_to_whitelist: AccountInfo<'info>,
    // trying to deserialize here leads to errors (doesn't exist yet)
    /// CHECK:
    #[account(init_if_needed,
        seeds = [
            b"whitelist".as_ref(),
            bank.key().as_ref(),
            address_to_whitelist.key().as_ref(),
        ],
        bump,
        payer = farm_manager,
        space = 8 + std::mem::size_of::<WhitelistProof>())]
    pub whitelist_proof: Box<Account<'info, WhitelistProof>>,
    pub system_program: Program<'info, System>,
}


pub fn handler(ctx: Context<AddToBankWhitelist>, whitelist_type: u8) -> Result<()> {
    // create/update whitelist proof
    let proof = &mut ctx.accounts.whitelist_proof;
    
    // if this is an update, decrement counts from existing whitelist
    if proof.whitelist_type > 0 {
        let existing_whitelist = WhitelistProof::read_type(proof.whitelist_type)?;
        let bank = &mut ctx.accounts.bank;

        if existing_whitelist.contains(WhitelistType::CREATOR) {
            bank.whitelisted_creators.try_sub_assign(1)?;
        }
        if existing_whitelist.contains(WhitelistType::MINT) {
            bank.whitelisted_mints.try_sub_assign(1)?;
        }
    }

    // record new whitelist and increment counts
    let new_whitelist = WhitelistProof::read_type(whitelist_type)?;

    proof.reset_type(new_whitelist);
    proof.whitelisted_address = ctx.accounts.address_to_whitelist.key();
    proof.bank = ctx.accounts.bank.key();
    //msg!("proof.whitelist_type: {}, proof.whitelisted_address {}, new_whitelist {:?} ", proof.whitelist_type, proof.whitelisted_address, new_whitelist);

    let bank = &mut ctx.accounts.bank;

    if new_whitelist.contains(WhitelistType::CREATOR) {
        bank.whitelisted_creators.try_add_assign(1)?;
    }
    if new_whitelist.contains(WhitelistType::MINT) {
        bank.whitelisted_mints.try_add_assign(1)?;
    }

    msg!(
        "{} added to bank whitelist",
        &ctx.accounts.address_to_whitelist.key()
    );
    Ok(())
}
