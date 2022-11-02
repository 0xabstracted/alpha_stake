use anchor_lang::prelude::*;
use gem_common::{errors::ErrorCode, TryAdd, TrySub};

pub const MAX_NFTS_ALLOWED: usize = 100;

const DEFAULT_STAKED_MINT: Pubkey = Pubkey::new_from_array([0; 32]);

// #[account(zero_copy)]
#[account(zero_copy)]
#[derive(AnchorDeserialize, Debug)]
pub struct FarmerStakedMints {
    pub bump : u8,
    pub farmer: Pubkey,
    pub index: u32,
    pub no_of_nfts_staked: u64,
    pub farmer_staked_mints: [Pubkey; 64],
}

impl FarmerStakedMints {
    pub fn append_nft(&mut self, farmer_staked_mint: Pubkey) -> Result<()> {
        if self.no_of_nfts_staked >= MAX_NFTS_ALLOWED as u64 {
            return Err(error!(ErrorCode::NotEnoughSpaceForStakedMint));
        }
        msg!("farmer_staked_mints: {}",farmer_staked_mint);
        for i in 0..MAX_NFTS_ALLOWED{
            if self.farmer_staked_mints[i] == DEFAULT_STAKED_MINT 
            //    && self.farmer_staked_mints[i] != farmer_staked_mint
            //    && farmer_staked_mint!= DEFAULT_STAKED_MINT
            {
                self.farmer_staked_mints[i] = farmer_staked_mint;
                self.no_of_nfts_staked.try_add(1)?;
                // msg!("i: {}, self.farmer_staked_mints[i]: {}",i, self.farmer_staked_mints[i]);
                // msg!("self.no_of_nfts_staked{}", self.no_of_nfts_staked);
                break;
            }
        }
        Ok(())
    }
    pub fn remove_nft(&mut self, farmer_staked_mint: Pubkey) -> Result<()> {
        if self.no_of_nfts_staked >= MAX_NFTS_ALLOWED as u64 {
            return Err(error!(ErrorCode::NotEnoughSpaceForStakedMint));
        }
        msg!("farmer_staked_mints: {}",farmer_staked_mint);

        for i in 0..MAX_NFTS_ALLOWED{
            if self.farmer_staked_mints[i] == farmer_staked_mint 
                // farmer_staked_mint!= DEFAULT_STAKED_MINT
            {
                self.farmer_staked_mints[i] = DEFAULT_STAKED_MINT;
                // msg!("i: {}, self.farmer_staked_mints[i]: {}",i, self.farmer_staked_mints[i]);
                // msg!("self.no_of_nfts_staked{}", self.no_of_nfts_staked);
                self.no_of_nfts_staked.try_sub(1)?;
                break;
            }
        }
        Ok(())
    }
}
