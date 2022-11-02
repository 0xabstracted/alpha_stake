use anchor_lang::prelude::*;

#[proc_macros::assert_size(4)]
#[repr(C)]
#[derive(Debug, Copy, Clone, AnchorDeserialize, AnchorSerialize)]
pub enum FixedRateRewardTier {
    Base,
    Tier1,
    Tier2,
    Tier3,
}
