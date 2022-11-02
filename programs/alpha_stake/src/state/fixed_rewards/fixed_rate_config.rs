use crate::state::fixed_rewards::fixed_rate_schedule::*;
use anchor_lang::prelude::*;

#[proc_macros::assert_size(120)]
#[repr(C)]
#[derive(Debug, Clone, Copy, AnchorDeserialize, AnchorSerialize)]
pub struct FixedRateConfig {
    pub schedule: FixedRateSchedule,
    /// total amount of reward tokens that are being sent with the ix. Will be added on TOP of existing funding
    pub amount: u64,
    /// duration the fund is being commited for
    /// If funding is done for 100 sec and a farmer stakes 3 secs after the commitment, the farmer is gaurenteed
    /// 97 sec of reward tokens
    /// Every farmer enrolled to the farm will be reserved an amount of reward tokens for the schedule
    /// of the duration
    pub duration_sec: u64,

}
