use anchor_lang::prelude::*;

/// if this PDA exists, this means the authorized_funder is eligible to fund the farm recorded below.

#[proc_macros::assert_size(96)]
#[repr(C)]
#[account]
pub struct AuthorizationProof {
    pub authorized_funder: Pubkey,
    pub farm: Pubkey,

    /// reserved for future updates, has to be /8
    _reserved: [u8; 32],
}
