// use crate::number128::Number128;
use crate::state::farmer::farmer_fixed_rate_reward::*;
// use crate::state::farmer::farmer_probable_rate_reward::*;
// use crate::state::farmer::farmer_variable_rate_reward::*;
use anchor_lang::prelude::*;
use gem_common::TryAdd;
use gem_common::TrySub;

#[proc_macros::assert_size(200)]
#[repr(C)]
#[derive(Debug, Clone, Copy, AnchorDeserialize, AnchorSerialize)]
pub struct FarmerReward {
    /// paid_out_reward is calucalted   Never goes down (ie )
    pub paid_out_reward: u64, //8
    pub accrued_reward: u64,                       //8
    pub fixed_rate: FarmerFixedRateReward,       //136
    _reserved: [u8; 32],                           //32
}

impl FarmerReward {
    pub fn outstanding_reward(&self) -> Result<u64> {
        self.accrued_reward.try_sub(self.paid_out_reward)
    }

    pub fn claim_reward(&mut self, pot_balance: u64) -> Result<u64> {
        let outstanding = self.outstanding_reward()?;
        let to_claim = std::cmp::min(outstanding, pot_balance);
        msg!("FarmerReward claim_reward \t outstanding: {}",outstanding);
        msg!("FarmerReward claim_reward \t to_claim: {}",to_claim);
        self.paid_out_reward.try_add_assign(to_claim)?;
        Ok(to_claim)
    }

    pub fn update_fixed_reward(&mut self, now_ts: u64, newly_accrued_reward: u64) -> Result<()> {
        self.accrued_reward.try_add_assign(newly_accrued_reward)?;
        msg!("FarmerReward update_fixed_reward \t self.accrued_reward: {}",self.accrued_reward);
        self.fixed_rate.last_updated_ts = self.fixed_rate.reward_upper_bound(now_ts)?;
        Ok(())
    }
}
