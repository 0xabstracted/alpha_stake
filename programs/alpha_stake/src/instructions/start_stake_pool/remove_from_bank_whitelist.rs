use anchor_lang::prelude::*;
use gem_common::{close_account, TrySub};
// use gem_bank::{
//     self,
//     cpi::accounts::RemoveFromWhitelist,
//     program::GemBank,
//     state::{Bank, WhitelistProof},
// };

use crate::{state::Farm, bank_state::{Bank, WhitelistProof, WhitelistType}};

#[derive(Accounts)]
#[instruction(bump_auth: u8, bump_wl:u8)]
pub struct RemoveFromBankWhitelist<'info> {
    #[account(has_one = farm_manager, has_one = farm_authority, has_one = bank)]
    pub farm: Box<Account<'info, Farm>>,
    #[account(mut)]
    pub farm_manager: Signer<'info>,
    /// CHECK:
    #[account(mut, seeds = [farm.key().as_ref()], bump = bump_auth)]
    pub farm_authority: AccountInfo<'info>,

    // cpi
    // #[account(mut)]
    #[account(mut, has_one = farm_authority)]
    pub bank: Box<Account<'info, Bank>>,
    /// CHECK:
    pub address_to_remove: AccountInfo<'info>,
    // #[account(mut)]
    #[account(mut, has_one = bank, seeds = [
        b"whitelist".as_ref(),
        bank.key().as_ref(),
        address_to_remove.key().as_ref(),
    ],
    bump = bump_wl)]
    pub whitelist_proof: Box<Account<'info, WhitelistProof>>,
    // pub gem_bank: Program<'info, GemBank>,
}

impl<'info> RemoveFromBankWhitelist<'info> {
    // fn remove_from_whitelist_ctx(
    //     &self,
    // ) -> CpiContext<'_, '_, '_, 'info, RemoveFromWhitelist<'info>> {
    //     CpiContext::new(
    //         self.gem_bank.to_account_info(),
    //         RemoveFromWhitelist {
    //             bank: self.bank.to_account_info(),
    //             bank_manager: self.farm_authority.clone(),
    //             address_to_remove: self.address_to_remove.clone(),
    //             whitelist_proof: self.whitelist_proof.to_account_info(),
    //             funds_receiver: self.farm_manager.to_account_info(),
    //         },
    //     )
    // }
}

pub fn handler(ctx: Context<RemoveFromBankWhitelist>) -> Result<()> {
    // gem_bank::cpi::remove_from_whitelist(
    //     ctx.accounts
    //         .remove_from_whitelist_ctx()
    //         .with_signer(&[&ctx.accounts.farm.farm_seeds()]),
    //     bump_wl,
    // )?;

    let bank = &mut ctx.accounts.bank;
    let proof = &mut ctx.accounts.whitelist_proof;

    if let Ok(()) = proof.contains_type(WhitelistType::MINT) {
        bank.whitelisted_mints.try_sub_assign(1)?;
    }
    if let Ok(()) = proof.contains_type(WhitelistType::CREATOR) {
        bank.whitelisted_creators.try_sub_assign(1)?;
    }

    // delete whitelist proof
    close_account(
        &mut proof.to_account_info(),
        &mut ctx.accounts.farm_manager.to_account_info(),
    )?;

    // msg!(
    //     "{} removed from whitelist",
    //     &ctx.accounts.address_to_remove.key()
    // );

    msg!(
        "{} removed from bank whitelist",
        &ctx.accounts.address_to_remove.key()
    );
    Ok(())
}
