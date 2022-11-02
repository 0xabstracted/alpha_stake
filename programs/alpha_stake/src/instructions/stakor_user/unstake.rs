use anchor_lang::prelude::*;
use anchor_spl::{token::{ Token, Transfer, TokenAccount, Mint}, associated_token::AssociatedToken};

// use gem_bank::{
//     self,
//     cpi::accounts::SetVaultLock,
//     cpi::accounts::WithdrawGem,
//     program::GemBank,
//     state::{Bank, Vault},
// };
use gem_common::{now_ts};
use crate::{state::{Farm, Farmer, FarmerStakedMints}, bank_state::{Bank, Vault, BankFlags, GemDepositReceipt}};
use anchor_spl::{
    token::{self, CloseAccount},
};
use gem_common::{errors::ErrorCode, *};

use super::calc_rarity_points;



#[derive(Accounts)]
#[instruction(bump_auth: u8, bump_token_treasury: u8, bump_farmer: u8, bump_gem_box: u8, bump_gdr:u8, bump_rarity: u8,  index: u32)]
pub struct Unstake<'info> {
    // farm
    #[account(mut, has_one = farm_authority, has_one = farm_treasury_token, has_one = bank)]
    pub farm: Box<Account<'info, Farm>>,
    /// CHECK:
    #[account(seeds = [farm.key().as_ref()], bump = bump_auth)]
    pub farm_authority: AccountInfo<'info>,
    #[account( seeds = [
        b"token_treasury".as_ref(),
        farm.key().as_ref(),
    ],
    bump = bump_token_treasury,
    )]
    pub farm_treasury_token: Box<Account<'info, TokenAccount>>,

    // farmer
    #[account(mut, has_one = farm, has_one = identity, has_one = vault,
        seeds = [
            b"farmer".as_ref(),
            farm.key().as_ref(),
            identity.key().as_ref(),
        ],
        bump = bump_farmer)]
    pub farmer: Box<Account<'info, Farmer>>,
    #[account(
        mut,
        seeds = [
            b"farmer_staked_mints".as_ref(), 
            // &index.to_le_bytes(),
            farmer.key().as_ref(),
        ],
        bump = farmer_staked_mints.load()?.bump,
        has_one = farmer,
    )]
    // #[account(mut)]
    // pub farmer_staked_mints: Account<'info, FarmerStakedMints>,
    pub farmer_staked_mints: AccountLoader<'info, FarmerStakedMints>,
    #[account(mut)]
    pub identity: Signer<'info>,

    // cpi
    // #[account(constraint = bank.bank_manager == farm_authority.key())]
    #[account(has_one = farm_authority)]
    pub bank: Box<Account<'info, Bank>>,
    #[account(mut, has_one = bank, has_one = identity, has_one = vault_authority)]
    // #[account(mut, has_one = bank)]
    pub vault: Box<Account<'info, Vault>>,
    /// CHECK:
    #[account(seeds = [vault.key().as_ref()], bump = bump_auth)]
    pub vault_authority: AccountInfo<'info>,
    // trying to deserialize here leads to errors (doesn't exist yet)
    /// CHECK:
    // #[account(mut)]
    // gem
    #[account(mut, seeds = [
        b"gem_box".as_ref(),
        vault.key().as_ref(),
        gem_mint.key().as_ref(),
    ],
    bump = bump_gem_box)]
    pub gem_box: Box<Account<'info, TokenAccount>>,
    // pub gem_box: AccountInfo<'info>,
    // trying to deserialize here leads to errors (doesn't exist yet)
    /// CHECK:
    // #[account(mut)]
    
    #[account(mut, has_one = vault, has_one = gem_mint, seeds = [
        b"gem_deposit_receipt".as_ref(),
        vault.key().as_ref(),
        gem_mint.key().as_ref(),
    ],
    bump = bump_gdr)]
    pub gem_deposit_receipt: Box<Account<'info, GemDepositReceipt>>,
    // pub gem_deposit_receipt: AccountInfo<'info>,
    // #[account(mut)]
    #[account(init_if_needed,
        associated_token::mint = gem_mint,
        associated_token::authority = identity,
        payer = identity)]
    pub gem_destination: Box<Account<'info, TokenAccount>>,
    pub gem_mint: Box<Account<'info, Mint>>,
    /// CHECK:
    #[account(seeds = [
        b"gem_rarity".as_ref(),
        bank.key().as_ref(),
        gem_mint.key().as_ref()
    ], bump = bump_rarity)]
    pub gem_rarity: AccountInfo<'info>,
    // pub gem_bank: Program<'info, GemBank>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
}

impl<'info> Unstake<'info> {
    fn transfer_ctx(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        CpiContext::new(
            self.token_program.to_account_info(),
            Transfer {
                from: self.gem_box.to_account_info(),
                to: self.gem_destination.to_account_info(),
                authority: self.vault_authority.to_account_info(),
            },
        )
    }

    fn close_context(&self) -> CpiContext<'_, '_, '_, 'info, CloseAccount<'info>> {
        CpiContext::new(
            self.token_program.to_account_info(),
            CloseAccount {
                account: self.gem_box.to_account_info(),
                destination: self.identity.to_account_info(),
                authority: self.vault_authority.clone(),
            },
        )
    }
    // fn set_lock_vault_ctx(&self) -> CpiContext<'_, '_, '_, 'info, SetVaultLock<'info>> {
    //     CpiContext::new(
    //         self.gem_bank.to_account_info(),
    //         SetVaultLock {
    //             bank: self.bank.to_account_info(),
    //             vault: self.vault.to_account_info(),
    //             bank_manager: self.farm_authority.clone(),
    //         },
    //     )
    // }
    // fn withdraw_gem_ctx(&self) -> CpiContext<'_, '_, '_, 'info, WithdrawGem<'info>> {
    //     CpiContext::new(
    //         self.gem_bank.to_account_info(),
    //         WithdrawGem {
    //             bank: self.bank.to_account_info(),
    //             vault: self.vault.to_account_info(),
    //             owner: self.identity.to_account_info(),
    //             authority: self.vault_authority.clone(),
    //             gem_box: self.gem_box.clone(),
    //             gem_deposit_receipt: self.gem_deposit_receipt.clone(),
    //             gem_destination: self.gem_destination.to_account_info(),
    //             gem_mint: self.gem_mint.to_account_info(),
    //             gem_rarity: self.gem_rarity.clone(),
    //             receiver: self.identity.to_account_info(),
    //             token_program: self.token_program.to_account_info(),
    //             associated_token_program: self.associated_token_program.to_account_info(),
    //             system_program: self.system_program.to_account_info(),
    //             rent: self.rent.to_account_info(),
    //         },
    //     )
    // }
    fn _pay_tokens_treasury_ctx(&self) -> CpiContext<'_,'_,'_,'info, Transfer<'info>> {
        CpiContext::new(
            self.token_program.to_account_info(), 
            Transfer { 
                from: self.identity.to_account_info(), 
                to: self.farm_treasury_token.to_account_info(), 
                authority: self.identity.to_account_info(), 
            },
        )
    }
}

pub fn handler(ctx: Context<Unstake>, 
        skip_rewards: bool , 
        amount: u64,
        index: u32,
    ) -> Result<()> {
    // collect any unstaking fee
     
    let farm = &mut ctx.accounts.farm;
    let farmer = &mut ctx.accounts.farmer;
    let now_ts = now_ts()?;

    // skipping rewards is an EMERGENCY measure in case farmer's rewards are overflowing
    // at least this lets them get their assets out
    if !skip_rewards {
        farm.update_rewards(now_ts, Some(farmer), false)?;
        // farm.update_lp_points(now_ts, Some(farmer), false)?;
    }
    // end staking (will cycle through state on repeated calls)
    farm.end_staking(now_ts, farmer)?;
    // let farm = &ctx.accounts.farm;
    // let farmer = &ctx.accounts.farmer;

    // if farmer.state == FarmerState::Unstaked && farm.config.unstaking_fee_percent > 0 && farm.config.unstaking_fee_percent < 100 {
    //    let unstake_fee_tokens = farmer.reward_a.accrued_reward.try_mul(farm.config.unstaking_fee_percent.try_div(100)?)?;
       
    //    token::transfer(
    //         ctx.accounts
    //         .pay_tokens_treasury_ctx()
    //         .with_signer(&[&ctx.accounts.farm.farm_seeds()]),
    //         unstake_fee_tokens,
    //     )?;
    //     let farmer = &mut ctx.accounts.farmer;
    //     farmer.reward_a.accrued_reward.try_sub_assign(unstake_fee_tokens)?; 
    
    // }
    // let farmer = &ctx.accounts.farmer;
      
    // if farmer.state == FarmerState::Unstaked {
        // unlock the vault so the user can withdraw their gems
        // gem_bank::cpi::set_vault_lock(
        //     ctx.accounts
        //         .set_lock_vault_ctx()
        //         .with_signer(&[&ctx.accounts.farm.farm_seeds()]),
        //     false,
        // )?;

        let bank = &ctx.accounts.bank;
        let vault = &mut ctx.accounts.vault;
    
        if Bank::read_flags(bank.flags)?.contains(BankFlags::FREEZE_VAULTS) {
            return Err(error!(ErrorCode::VaultAccessSuspended));
        }
    
        vault.locked = false;
        // gem_bank::cpi::withdraw_gem(
        //     ctx.accounts.withdraw_gem_ctx(),
        //     bump_auth, 
        //     bump_gem_box, 
        //     bump_gdr, 
        //     bump_rarity, 
        //     amount,
        // )?;
        // verify vault not suspended
    let bank = &*ctx.accounts.bank;
    let vault = &ctx.accounts.vault;

    if vault.access_suspended(bank.flags)? {
        return Err(error!(ErrorCode::VaultAccessSuspended));
    }

    // do the transfer
    token::transfer(
        ctx.accounts
            .transfer_ctx()
            .with_signer(&[&vault.vault_seeds()]),
        amount,
    )?;

    // update the gdr
    let gdr = &mut *ctx.accounts.gem_deposit_receipt;
    let gem_box = &ctx.accounts.gem_box;

    gdr.gem_count.try_sub_assign(amount)?;

    // this check is semi-useless but won't hurt
    if gdr.gem_count != gem_box.amount.try_sub(amount)? {
        return Err(error!(ErrorCode::AmountMismatch));
    }

    // if gembox empty, close both the box and the GDR, and return funds to user
    if gdr.gem_count == 0 {
        // close gem box
        token::close_account(
            ctx.accounts
                .close_context()
                .with_signer(&[&vault.vault_seeds()]),
        )?;

        // close GDR
        let gdr = &mut (*ctx.accounts.gem_deposit_receipt).to_account_info();

        close_account(gdr, &mut ctx.accounts.identity.to_account_info())?;

        // decrement gem box count stored in vault's state
        let vault = &mut ctx.accounts.vault;
        vault.gem_box_count.try_sub_assign(1)?;
    }

    // decrement gem count as well
    let vault = &mut ctx.accounts.vault;
    vault.gem_count.try_sub_assign(amount)?;
    vault
        .rarity_points
        .try_sub_assign(calc_rarity_points(&ctx.accounts.gem_rarity, amount)?)?;

    //msg!("{} gems withdrawn from ${} gem box", amount, gem_box.key());
    
    // }
    let mut farmer_staked_mints = ctx.accounts.farmer_staked_mints.load_mut()?;
    if farmer_staked_mints.index == index {
        for _ in 0..amount{
            farmer_staked_mints.remove_nft(ctx.accounts.gem_mint.key())?;
        }
    }
    // if farmer_staked_mints.no_of_nfts_staked == 0 {
    //     close_account(farmer_staked_mints, &mut ctx.accounts.identity.to_account_info())?;
    // }
   
    Ok(())
}