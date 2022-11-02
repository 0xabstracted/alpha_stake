use anchor_lang::prelude::*;

use crate::{bank_state::Bank};

#[derive(Accounts)]
pub struct UpdateBankManager<'info> {
    // bank
    #[account(mut, has_one = farm_authority)]
    pub bank: Box<Account<'info, Bank>>,
    pub farm_authority: Signer<'info>,
}

pub fn handler(ctx: Context<UpdateBankManager>, new_manager: Pubkey) -> Result<()> {
    let bank = &mut ctx.accounts.bank;

    bank.farm_authority = new_manager;

    //msg!("bank manager updated to: {}", new_manager);
    Ok(())
}
