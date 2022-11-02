use crate::state::fixed_rewards::fixed_rate_schedule::*;
use anchor_lang::prelude::*;
use gem_common::{TryAdd, TrySub};

#[proc_macros::assert_size(152)]
#[repr(C)]
#[derive(Debug, Clone, Copy, Default, AnchorDeserialize, AnchorSerialize)]
pub struct FarmerFixedRateReward {
    /// this is the time the farmer staked
    /// can be WAY BACK in the past, if we have rolled multiple times
    pub begin_staking_ts: u64, //8
    /// this is the time the latest reward_schedule they subscribed to begins
    /// this + promised_duration = end_schedule_ts
    pub begin_schedule_ts: u64, //8
    /// always set to upper bound, not just now_ts (except when funding)
    pub last_updated_ts: u64, //8
    pub promised_schedule: FixedRateSchedule, //88
    pub promised_duration: u64,               //8
    _reserved: [u8; 16],                      //16
}

impl FarmerFixedRateReward {
    // accrued to rolled stakers whose begin_staking_ts < begin_schedule_ts
    pub fn loyal_staker_bonus_time(&self) -> Result<u64> {
        self.begin_schedule_ts.try_sub(self.begin_staking_ts)
    }

    pub fn end_schedule_ts(&self) -> Result<u64> {
        self.begin_schedule_ts.try_add(self.promised_duration)
    }

    pub fn is_staked(&self) -> bool {
        // these are zeroed out when farmer graduates
        self.begin_schedule_ts > 0 && self.begin_staking_ts > 0
    }

    pub fn is_time_to_graduate(&self, now_ts: u64) -> Result<bool> {
        Ok(now_ts >= self.end_schedule_ts()?)
    }

    pub fn reward_upper_bound(&self, now_ts: u64) -> Result<u64> {
        Ok(std::cmp::min(now_ts, self.end_schedule_ts()?))
    }

    pub fn time_from_staking_to_update(&self) -> Result<u64> {
        self.last_updated_ts.try_sub(self.begin_staking_ts)
    }

    pub fn voided_reward(&self, rarity_points: u64) -> Result<u64> {
        let start_from = self.time_from_staking_to_update()?;
        let end_at = self
            .end_schedule_ts()?
            .try_sub(self.begin_staking_ts)?;
        msg!("voided_reward rarity_points:{}",rarity_points);
        self.promised_schedule
            .reward_amount(start_from, end_at, rarity_points)
    }

    pub fn newly_accrued_reward(&self, now_ts: u64, rarity_points: u64) -> Result<u64> {
        let start_from = self.time_from_staking_to_update()?;
        let end_at = self
            .reward_upper_bound(now_ts)?
            .try_sub(self.begin_staking_ts)?;
        msg!("newly_accrued_reward rarity_points:{}",rarity_points);
        self.promised_schedule
            .reward_amount(start_from, end_at, rarity_points)
    }
}
