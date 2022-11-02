use crate::bank_state::Bank;
use crate::bank_state::BankFlags;
use crate::bank_state::GemDepositReceipt;
use crate::bank_state::Rarity;
use crate::bank_state::Vault;
use crate::bank_state::WhitelistProof;
use crate::bank_state::WhitelistType;
use crate::state::Farm;
use crate::state::Farmer;
use crate::state::FarmerStakedMints;
use anchor_lang::prelude::*;
//use anchor_lang::solana_program::{system_instruction, program::invoke};
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};
// use gem_bank::instructions::calc_rarity_points;
// use gem_bank::{
//     self,
//     cpi::accounts::{DepositGem, SetVaultLock},
//     program::GemBank,
//     state::{Bank, Vault},
// };
use std::str::FromStr;
use anchor_lang::Discriminator;
use arrayref::array_ref;
use gem_common::{errors::ErrorCode, *};
use gem_common::TryAdd;
use gem_common::now_ts;
use metaplex_token_metadata::state::Metadata;
// use std::str::FromStr;
// use crate::instructions::FEE_WALLET;
// const FEE_LAMPORTS: u64 = 2_000_000; // 0.002 SOL per stake/unstake
// const FD_FEE_LAMPORTS: u64 = 1_000_000; // half of that for FDs

#[derive(Accounts)]
#[instruction(bump_farmer: u8, bump_farmer_staked_mints: u8, bump_vault_auth: u8,
    bump_rarity: u8, index: u32)]
pub struct FlashDeposit<'info> {
    // farm
    #[account(mut, has_one = farm_authority)]
    pub farm: Box<Account<'info, Farm>>,
    //skipping seeds verification to save compute budget, has_one check above should be enough
    /// CHECK:
    pub farm_authority: AccountInfo<'info>,

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
    // pub farmer_staked_mints: Account<'info, FarmerStakedMints>,
    pub farmer_staked_mints: AccountLoader<'info, FarmerStakedMints>,
    
    #[account(mut)]
    pub identity: Signer<'info>,

    // cpi
    #[account(has_one = farm_authority  )]
    pub bank: Box<Account<'info, Bank>>,
    // #[account(mut)]
    #[account(mut, has_one = bank, has_one = identity, has_one = vault_authority)]
    pub vault: Box<Account<'info, Vault>>,
    /// CHECK:
    #[account(seeds = [vault.key().as_ref()], bump = bump_vault_auth)]
    pub vault_authority: AccountInfo<'info>,
    // trying to deserialize here leads to errors (doesn't exist yet)
    /// CHECK:
    #[account(init_if_needed, seeds = [
        b"gem_box".as_ref(),
        vault.key().as_ref(),
        gem_mint.key().as_ref(),
        ],
        bump,
        token::mint = gem_mint,
        token::authority = vault_authority,
        payer = identity)]
    pub gem_box: Box<Account<'info, TokenAccount>>,
    // trying to deserialize here leads to errors (doesn't exist yet)
    /// CHECK:
    #[account(init_if_needed, seeds = [
        b"gem_deposit_receipt".as_ref(),
        vault.key().as_ref(),
        gem_mint.key().as_ref(),
    ],
    bump,
    payer = identity,
    space = 8 + std::mem::size_of::<GemDepositReceipt>())]
    pub gem_deposit_receipt: Box<Account<'info, GemDepositReceipt>>,
    #[account(mut)]
    pub gem_source: Box<Account<'info, TokenAccount>>,
    pub gem_mint: Box<Account<'info, Mint>>,
    /// CHECK:
    #[account(seeds = [
        b"gem_rarity".as_ref(),
        bank.key().as_ref(),
        gem_mint.key().as_ref()
    ],
    bump = bump_rarity)]
    pub gem_rarity: AccountInfo<'info>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
    // pub gem_bank: Program<'info, GemBank>,
    //
    // remaining accounts could be passed, in this order:
    // - mint_whitelist_proof
    // - gem_metadata <- if we got to this point we can assume gem = NFT, not a fungible token
    // - creator_whitelist_proof
}

fn assert_valid_metadata(
    gem_metadata: &AccountInfo,
    gem_mint: &Pubkey,
) -> core::result::Result<Metadata, ProgramError> {
    let metadata_program = Pubkey::from_str("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s").unwrap();

    // 1 verify the owner of the account is metaplex's metadata program
    assert_eq!(gem_metadata.owner, &metadata_program);

    // 2 verify the PDA seeds match
    let seed = &[
        b"metadata".as_ref(),
        metadata_program.as_ref(),
        gem_mint.as_ref(),
    ];

    let (metadata_addr, _bump) = Pubkey::find_program_address(seed, &metadata_program);
    assert_eq!(metadata_addr, gem_metadata.key());

    Metadata::from_account_info(gem_metadata)
}

fn assert_valid_whitelist_proof<'info>(
    whitelist_proof: &AccountInfo<'info>,
    bank: &Pubkey,
    address_to_whitelist: &Pubkey,
    program_id: &Pubkey,
    expected_whitelist_type: WhitelistType,
) -> Result<()> {
    // 1 verify the PDA seeds match
    let seed = &[
        b"whitelist".as_ref(),
        bank.as_ref(),
        address_to_whitelist.as_ref(),
    ];
    let (whitelist_addr, _bump) = Pubkey::find_program_address(seed, program_id);
    //msg!("mint whitelisted: {}, whitelist_addr {} , whitelist_proof.key(){} ", address_to_whitelist, whitelist_addr, whitelist_proof.key());

    // we can't use an assert_eq statement, we want to catch this error and continue along to creator testing
    if whitelist_addr != whitelist_proof.key() {
        return Err(error!(ErrorCode::NotWhitelisted));
    }

    // 2 no need to verify ownership, deserialization does that for us
    // https://github.com/project-serum/anchor/blob/fcb07eb8c3c9355f3cabc00afa4faa6247ccc960/lang/src/account.rs#L36
    let proof = Account::<'info, WhitelistProof>::try_from(whitelist_proof)?;

    // 3 verify whitelist type matches
    proof.contains_type(expected_whitelist_type)
}

fn assert_whitelisted(ctx: &Context<FlashDeposit>) -> Result<()> {
    let bank = &*ctx.accounts.bank;
    let mint = &*ctx.accounts.gem_mint;
    let remaining_accs = &mut ctx.remaining_accounts.iter();

    // whitelisted mint is always the 1st optional account
    // this is because it's applicable to both NFTs and standard fungible tokens
    let mint_whitelist_proof_info = next_account_info(remaining_accs)?;
   // msg!("mint whitelisted: {}, going ahead, bank.whitelisted_mints {}, bank.whitelisted_creators {} mint_whitelist_proof_info.key(){}", &mint.key(), bank.whitelisted_mints, bank.whitelisted_creators, mint_whitelist_proof_info.key());

    // attempt to verify based on mint
    if bank.whitelisted_mints > 0 {
        if let Ok(()) = assert_valid_whitelist_proof(
            mint_whitelist_proof_info,
            &bank.key(),
            &mint.key(),
            ctx.program_id,
            WhitelistType::MINT,
        ) {
           // msg!("mint whitelisted: {}, going ahead", &mint.key());
            return Ok(());
        }
    }

    // if mint verification above failed, attempt to verify based on creator
    if bank.whitelisted_creators > 0 {
        // 2 additional accounts are expected - metadata and creator whitelist proof
        let metadata_info = next_account_info(remaining_accs)?;
        let creator_whitelist_proof_info = next_account_info(remaining_accs)?;
       // msg!("metadata_info.key() {}, creator_whitelist_proof_info.key() {}", metadata_info.key(), creator_whitelist_proof_info.key());
        // verify metadata is legit
        let metadata = assert_valid_metadata(metadata_info, &mint.key())?;

        // metaplex constraints this to max 5, so won't go crazy on compute
        // (empirical testing showed there's practically 0 diff between stopping at 0th and 5th creator)
        for creator in &metadata.data.creators.unwrap() {
            // verify creator actually signed off on this nft
            if !creator.verified {
                continue;
            }

            // check if creator is whitelisted, returns an error if not
            let attempted_proof = assert_valid_whitelist_proof(
                creator_whitelist_proof_info,
                &bank.key(),
                &creator.address,
                ctx.program_id,
                WhitelistType::CREATOR,
            );

            match attempted_proof {
                //proof succeeded, return out of the function, no need to continue looping
                Ok(()) => return Ok(()),
                //proof failed, continue to check next creator
                Err(_e) => continue,
            }
        }
    }

    // if both conditions above failed tok return Ok(()), then verification failed
    Err(error!(ErrorCode::NotWhitelisted))
}

/// if rarity account is present, extract rarities from there - else use 1 * amount
pub fn calc_rarity_points(gem_rarity: &AccountInfo, amount: u64) -> Result<u64> {
    if !gem_rarity.data_is_empty() {
        let rarity_account = Account::<Rarity>::try_from(gem_rarity)?;
        amount.try_mul(rarity_account.points as u64)
    } else {
        Ok(amount)
    }
}


impl<'info> FlashDeposit<'info> {
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

    // fn deposit_gem_ctx(&self) -> CpiContext<'_, '_, '_, 'info, DepositGem<'info>> {
    //     CpiContext::new(
    //         self.gem_bank.to_account_info(),
    //         DepositGem {
    //             bank: self.bank.to_account_info(),
    //             vault: self.vault.to_account_info(),
    //             owner: self.identity.to_account_info(),
    //             authority: self.vault_authority.clone(),
    //             gem_box: self.gem_box.clone(),
    //             gem_deposit_receipt: self.gem_deposit_receipt.clone(),
    //             gem_source: self.gem_source.to_account_info(),
    //             gem_mint: self.gem_mint.to_account_info(),
    //             gem_rarity: self.gem_rarity.clone(),
    //             token_program: self.token_program.to_account_info(),
    //             system_program: self.system_program.to_account_info(),
    //             rent: self.rent.to_account_info(),
    //         },
    //     )
    // }

    fn transfer_ctx(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        CpiContext::new(
            self.token_program.to_account_info(),
            Transfer {
                from: self.gem_source.to_account_info(),
                to: self.gem_box.to_account_info(),
                authority: self.identity.to_account_info(),
            },
        )
    }
}

pub fn handler<'a, 'b, 'c, 'info>(
    ctx: Context<'a, 'b, 'c, 'info, FlashDeposit<'info>>,
    
    index: u32,
    amount: u64,
) -> Result<()> {
    // flash deposit a gem into a locked vault
    // gem_bank::cpi::set_vault_lock(
    //     ctx.accounts
    //         .set_lock_vault_ctx()
    //         .with_signer(&[&ctx.accounts.farm.farm_seeds()]),
    //     false,
    // )?;
    // gem_bank::cpi::deposit_gem(
    //     ctx.accounts
    //         .deposit_gem_ctx()
    //         .with_remaining_accounts(ctx.remaining_accounts.to_vec()),
    //     bump_vault_auth,
    //     bump_rarity,
    //     amount,
    // )?;

    let bank = &ctx.accounts.bank;
    let vault = &mut ctx.accounts.vault;

    if Bank::read_flags(bank.flags)?.contains(BankFlags::FREEZE_VAULTS) {
        return Err(error!(ErrorCode::VaultAccessSuspended));
    }

    vault.locked = false;

    // fix missing discriminator check
    {
        let acct = ctx.accounts.gem_deposit_receipt.to_account_info();
        let data: &[u8] = &acct.try_borrow_data()?;
        let disc_bytes = array_ref![data, 0, 8];
        if disc_bytes != &GemDepositReceipt::discriminator() && disc_bytes.iter().any(|a| a != &0) {
            return Err(error!(ErrorCode::AccountDiscriminatorMismatch));
        }
    }

    // if even a single whitelist exists, verify the token against it
    let bank = &*ctx.accounts.bank;

    if bank.whitelisted_mints > 0 || bank.whitelisted_creators > 0 {
        assert_whitelisted(&ctx)?;
    }

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

    // record total number of gem boxes in vault's state
    let vault = &mut ctx.accounts.vault;
    vault.gem_box_count.try_add_assign(1)?;
    vault.gem_count.try_add_assign(amount)?;
    vault
        .rarity_points
        .try_add_assign(calc_rarity_points(&ctx.accounts.gem_rarity, amount)?)?;

    // record a gdr
    let gdr = &mut *ctx.accounts.gem_deposit_receipt;
    let gem_box = &*ctx.accounts.gem_box;

    gdr.vault = vault.key();
    gdr.gem_box_address = gem_box.key();
    gdr.gem_mint = gem_box.mint;
    gdr.gem_count.try_add_assign(amount)?;

    // this check is semi-useless but won't hurt
    if gdr.gem_count != gem_box.amount.try_add(amount)? {
        // msg!("{} {}", gdr.gem_count, gem_box.amount);
        return Err(error!(ErrorCode::AmountMismatch));
    }

    // msg!("{} gems deposited into {} gem box", amount, gem_box.key());
    // gem_bank::cpi::set_vault_lock(
    //     ctx.accounts
    //         .set_lock_vault_ctx()
    //         .with_signer(&[&ctx.accounts.farm.farm_seeds()]),
    //     true,
    // )?;

    let bank = &ctx.accounts.bank;
    let vault = &mut ctx.accounts.vault;

    if Bank::read_flags(bank.flags)?.contains(BankFlags::FREEZE_VAULTS) {
        return Err(error!(ErrorCode::VaultAccessSuspended));
    }

    vault.locked = true;

    let farmer = &mut ctx.accounts.farmer;
    let farm = &mut ctx.accounts.farm;
    let now_ts = now_ts()?;
        farm.update_rewards(now_ts, Some(farmer), true)?;

        // farm.update_lp_points(now_ts, Some(farmer), true)?;
    msg!("handler \t farmer.gems_staked:{}", farmer.gems_staked);
    msg!("handler \t farmer.reward_a.fixed_rate:{:?}", farmer.reward_a.fixed_rate);
    // msg!("handler \t farmer.lp_points.lp_rate:{:?}", farmer.lp_points.lp_rate);
    
    ctx.accounts.vault.reload()?;
    if farmer.gems_staked == 0 {
        farm.begin_staking(
            now_ts,
            ctx.accounts.vault.gem_count,
            ctx.accounts.vault.rarity_points,
            farmer,
        )?;
    } else {
        let extra_rarity = calc_rarity_points(&ctx.accounts.gem_rarity, amount)?;
        farm.stake_extra_gems(
            now_ts,
            ctx.accounts.vault.gem_count,
            ctx.accounts.vault.rarity_points,
            amount,
            extra_rarity,
            farmer,
        )?;
    } 
    // farmer.reload()?;
    
    // farm.reload()?;
    let mut farmer_staked_mints = ctx.accounts.farmer_staked_mints.load_mut()?;
    farmer_staked_mints.no_of_nfts_staked.try_add(amount)?;
    farmer_staked_mints.index = index;
    
    for _ in 0..amount{
        farmer_staked_mints.append_nft(ctx.accounts.gem_mint.key())?;
    }

    // msg!("farmer.fsm_account_keys[i]: {:?}", farmer.fsm_account_keys);
    // msg!("self.no_of_nfts_staked{}", farmer.no_fsm_accounts);
    // msg!("farmer_staked_mints.farmer_staked_mints[i]: {:?}", farmer_staked_mints.farmer_staked_mints);
    // msg!("self.no_of_nfts_staked{}", farmer_staked_mints.no_of_nfts_staked);
    
    // msg!("{} extra gems staked for {}", amount, farmer.key());
    Ok(())
}
