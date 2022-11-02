use anchor_lang::prelude::Result;
use gem_common::{TryMul, TrySub};

/// a tenure which we can definitively apply the reward rate to
/// needed for caluclation only, not stored anywhere in final struct

#[repr(C)]
pub struct HeldTenure {
    pub definitive_start: u64,
    pub definitive_end: u64,
    pub reward_rate: u64,
}

impl HeldTenure {
    pub fn new(
        reward_rate: u64,
        start_from: u64,
        end_at: u64,
        lower_bound: u64,
        upper_bound: u64,
    ) -> Option<Self> {
        let definitive_start = std::cmp::max(start_from, lower_bound);
        let definitive_end = std::cmp::min(end_at, upper_bound);

        match definitive_end < definitive_start {
            false => Some(Self {
                definitive_start,
                definitive_end,
                reward_rate,
            }),
            true => None,
        }
    }

    /// multplies definitive start & end by the rate
    pub fn get_reward(&self) -> Result<u64> {
        let duration = self.definitive_end.try_sub(self.definitive_start)?;
        self.reward_rate.try_mul(duration)
    }
}
