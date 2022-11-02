use anchor_lang::prelude::*;

#[proc_macros::assert_size(4)]
#[repr(C)]
#[derive(Debug, Clone, Copy, AnchorDeserialize, AnchorSerialize, PartialEq)]
pub enum FarmerState {
    Unstaked,
    Staked,
    PendingCooldown,
}
