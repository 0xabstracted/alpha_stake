use anchor_lang::prelude::*;
use gem_common::TrySub;

#[proc_macros::assert_size(24)]
#[repr(C)]
#[derive(Debug, Clone, Copy, AnchorDeserialize, AnchorSerialize)]
pub struct FundsTracker {
    pub total_funded: u64,
    pub total_refunded: u64,
    pub total_accured_to_stakers: u64,
}

impl FundsTracker {
    pub fn pending_amount(&self) -> Result<u64> {
        self.total_funded
            .try_sub(self.total_refunded)?
            .try_sub(self.total_accured_to_stakers)
    }
}
