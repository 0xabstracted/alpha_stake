use anchor_lang::prelude::*;
use gem_common::errors::ErrorCode;

use crate::bank_state::{Bank, Vault, BankFlags};

#[derive(Accounts)]
pub struct UpdateVaultOwner<'info> {
    // bank
    pub bank: Box<Account<'info, Bank>>,

    // vault
    // same rationale for not verifying the PDA as in deposit
    #[account(mut, has_one = bank, has_one = identity)]
    pub vault: Box<Account<'info, Vault>>,
    pub identity: Signer<'info>,
}

pub fn handler(ctx: Context<UpdateVaultOwner>, new_owner: Pubkey) -> Result<()> {
    let bank = &ctx.accounts.bank;
    let vault = &mut ctx.accounts.vault;

    if Bank::read_flags(bank.flags)?.contains(BankFlags::FREEZE_VAULTS) {
        return Err(error!(ErrorCode::VaultAccessSuspended));
    }

    vault.identity = new_owner;

    //msg!("owner updated to: {}", new_owner);
    Ok(())
}
