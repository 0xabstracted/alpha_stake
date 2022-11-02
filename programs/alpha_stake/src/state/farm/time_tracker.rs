use anchor_lang::prelude::*;
use gem_common::TrySub;

#[proc_macros::assert_size(24)]
#[repr(C)]
#[derive(Debug, Clone, Copy, AnchorDeserialize, AnchorSerialize)]
pub struct TimeTracker {
    /// total duration for which the reward has been funded
    /// updated with each new funding round
    pub duration_sec: u64,
    pub reward_end_ts: u64,
    /// this will be set to reward_end_ts if farm manager decides to lock the rewards
    /// gives stakers certainity that it won't be withdrawn.
    pub lock_end_ts: u64,
}

impl TimeTracker {
    pub fn reward_begin_ts(&self) -> Result<u64> {
        self.reward_end_ts.try_sub(self.duration_sec)
    }

    pub fn remaining_duration(&self, now_ts: u64) -> Result<u64> {
        if now_ts >= self.reward_end_ts {
            return Ok(0);
        }
        self.reward_end_ts.try_sub(now_ts)
    }

    pub fn passed_duration(&self, now_ts: u64) -> Result<u64> {
        self.reward_end_ts.try_sub(self.remaining_duration(now_ts)?)
    }

    pub fn end_reward(&mut self, now_ts: u64) -> Result<()> {
        self.duration_sec
            .try_sub_assign(self.remaining_duration(now_ts)?)?;
        self.reward_end_ts = std::cmp::min(now_ts, self.reward_end_ts);
        Ok(())
    }

    // returns whihever comes first - now or the end of the reward
    pub fn reward_upper_bound(&self, now_ts: u64) -> u64 {
        std::cmp::min(self.reward_end_ts, now_ts)
    }

    // returns whichevr comes last - begining of the reward or begining of farmer's staking
    pub fn reward_lower_bound(&self, farmer_begin_staking_ts: u64) -> Result<u64> {
        Ok(std::cmp::max(
            self.reward_begin_ts()?,
            farmer_begin_staking_ts,
        ))
    }
}
