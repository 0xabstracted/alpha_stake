use anchor_lang::prelude::*;

use crate::{ bank_state::Bank};

#[derive(Accounts)]
pub struct SetBankFlags<'info> {
    // bank
    #[account(mut, has_one = farm_authority)]
    pub bank: Box<Account<'info, Bank>>,
    pub farm_authority: Signer<'info>,
}

pub fn handler(ctx: Context<SetBankFlags>, flags: u32) -> Result<()> {
    let bank = &mut ctx.accounts.bank;

    let flags = Bank::read_flags(flags)?;
    bank.reset_flags(flags);

    //msg!("flags set: {:?}", flags);
    Ok(())
}
