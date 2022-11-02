use crate::state::farm::funds_tracker::*;
use crate::state::farm::reward_type::*;
use crate::state::farm::time_tracker::*;
use crate::state::farmer::farmer_reward::*;
use crate::state::fixed_rewards::fixed_rate_reward::*;
// use crate::state::probable_rewards::probable_rate_reward::*;
use crate::state::FixedRateConfig;
// use crate::state::ProbableRateConfig;
use anchor_lang::prelude::*;
use gem_common::errors::ErrorCode;

#[proc_macros::assert_size(296)]
#[repr(C)]
#[derive(Debug, Clone, Copy, AnchorDeserialize, AnchorSerialize)]
pub struct FarmReward {
    /// Once reward mint, pot and pot type are set only once. Start a new farm if new config is needed
    pub reward_mint: Pubkey, //32
    /// where the reward is stored for this farm and funded by farm funders authorized by farm manager
    pub reward_pot: Pubkey, //32
    /// Type of reward configured for the pot
    pub reward_type: RewardType, //4
    /// Only one of these three reward types is configured per reward
    pub fixed_rate_reward: FixedRateReward, //128
    pub funds: FundsTracker,               //24
    pub times: TimeTracker,                //24
    _reserved: [u8; 32],                   //32
}

impl FarmReward {
    /// This operation is IREVERSIBLE
    /// locking ensures the commiited reward cannot be withdrawn/changed by any malicious farm operator
    /// once locked any funding/cancellation ixs are nonexecutable until reward_end_ts is reached.
    pub fn lock_reward(&mut self) -> Result<()> {
        self.times.lock_end_ts = self.times.reward_end_ts;
        // msg!("locked reward up to {}", self.times.reward_end_ts);
        Ok(())
    }

    pub fn is_locked(&self, now_ts: u64) -> bool {
        now_ts < self.times.lock_end_ts
    }

    pub fn fund_reward_by_type(
        &mut self,
        now_ts: u64,
        fixed_rate_config: Option<FixedRateConfig>,
    ) -> Result<()> {
        if self.is_locked(now_ts) {
            return Err(error!(ErrorCode::RewardLocked));
        }

        match self.reward_type {
            RewardType::Fixed => self.fixed_rate_reward.fund_reward(
                now_ts,
                &mut self.times,
                &mut self.funds,
                fixed_rate_config.unwrap(),
            ),
        }
    }
    
    pub fn cancel_reward_by_type(&mut self, now_ts: u64) -> Result<u64> {
        if self.is_locked(now_ts) {
            return Err(error!(ErrorCode::RewardLocked));
        }
        match self.reward_type {
            RewardType::Fixed => {
                self.fixed_rate_reward
                    .cancel_reward(now_ts, &mut self.times, &mut self.funds)
            }
        }
    }

    pub fn update_accrued_reward_by_type(
        &mut self,
        now_ts: u64,
        _farm_rarity_points_staked: u64,
        farmer_rarity_points_staked: Option<u64>,
        farmer_reward: Option<&mut FarmerReward>,
        reenroll: bool,
    ) -> Result<()> {

        match self.reward_type {
            RewardType::Fixed => {
                // for fixed reward we only update if farmer reward is passed
                if farmer_reward.is_none() {
                    msg!("farmer_reward not present, no farmer");
                    return Ok(());
                }
                self.fixed_rate_reward.update_accrued_reward(
                    now_ts,
                    &mut self.times,
                    &mut self.funds,
                    farmer_rarity_points_staked.unwrap(),
                    farmer_reward.unwrap(),
                    reenroll,
                )
            }
        }
    }

}
