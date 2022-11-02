use anchor_lang::prelude::*;

#[proc_macros::assert_size(16)]
#[repr(C)]
#[derive(Debug, Copy, Clone, AnchorDeserialize, AnchorSerialize)]
pub struct TierConfig {
    /// tokens per denominator per rairy point /sec
    pub reward_rate: u64,
    /// min amount of time that needs to pass for the above rate to come into effect
    pub required_tenure: u64,
}
