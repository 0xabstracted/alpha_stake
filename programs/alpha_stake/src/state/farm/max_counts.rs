use anchor_lang::prelude::*;

//refers to staked counts
#[proc_macros::assert_size(12)]
#[repr(C)]
#[derive(Debug, Copy, Clone, AnchorDeserialize, AnchorSerialize)]
pub struct MaxCounts {
    pub max_farmers: u32,
    pub max_gems: u32,
    pub max_rarity_points: u32,
}
