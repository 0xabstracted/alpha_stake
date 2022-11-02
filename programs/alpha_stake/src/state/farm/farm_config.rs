use anchor_lang::prelude::*;

#[proc_macros::assert_size(24)]
#[repr(C)]
#[derive(Debug, Copy, Clone, AnchorSerialize, AnchorDeserialize)]
pub struct FarmConfig {
    // min time the NFT has to be staked
    pub min_staking_period_sec: u64,
    // time after user decides to unstake before they can actually withdraw
    pub cooldown_period_sec: u64,
    pub unstaking_fee_percent: u64,
}
