// use crate::state::farmer::farmer_lp_points::*;
use crate::state::farmer::farmer_reward::*;
use crate::state::farmer::farmer_state::*;
use anchor_lang::prelude::*;
use gem_common::TrySub;
use gem_common::errors::ErrorCode;
use gem_common::TryAdd;

const MAX_FSM_ACCOUNTS: usize = 21;
const DEFAULT_STAKED_MINT: Pubkey = Pubkey::new_from_array([0; 32]);

#[proc_macros::assert_size(1048)]
#[repr(C)]
#[account]
#[derive(Debug)]
pub struct Farmer {
    pub farm: Pubkey, //32
    /// the identiy of the farmer - Publickey
    pub identity: Pubkey, //32
    /// vault storing all of farmer's gems
    pub vault: Pubkey, //32
    pub state: FarmerState, //4
    /// total number of gems when vault is locked
    pub gems_staked: u64, //8
    ///  sum of rairty points of all the NFTs staked. If not configured raritypoints are defualted to 1.
    pub rarity_points_staked: u64, //8
    /// this will be updated when they decide to unstake according to FarmConfig settings of Farm
    pub min_staking_end_ts: u64, //8
    /// this will be updated when they decide to unstake according to FarmConfig settings of Farm
    pub cooldown_end_ts: u64, //8
    pub reward_a: FarmerReward, //384 352
    pub no_fsm_accounts : u64,
    pub fsm_account_keys: [Pubkey; MAX_FSM_ACCOUNTS],
    _reserved: [u8; 32],           //32
}

impl Farmer {
    pub fn append_fsm(&mut self, fsm_account_address: Pubkey) -> Result<()> {
        if self.no_fsm_accounts >= MAX_FSM_ACCOUNTS as u64 {
            return Err(error!(ErrorCode::MaxFSMAccountCountReached));
        }
        msg!("fsm_account_address: {}",fsm_account_address);
        for i in 0..MAX_FSM_ACCOUNTS{
            if self.fsm_account_keys[i] == DEFAULT_STAKED_MINT 
                // self.fsm_account_keys[i] != fsm_account_address &&
                // fsm_account_address!= DEFAULT_STAKED_MINT
            {
                self.fsm_account_keys[i] = fsm_account_address;
                self.no_fsm_accounts.try_add(1)?;
                // msg!("i: {}, self.fsm_account_keys[i]: {}",i, self.fsm_account_keys[i]);
                // msg!("self.no_fsm_accounts{}", self.no_fsm_accounts);
                break;
            }
        }
        Ok(())
    }
    pub fn remove_fsm(&mut self, fsm_account_address: Pubkey) -> Result<()> {
        if self.no_fsm_accounts >= MAX_FSM_ACCOUNTS as u64 {
            return Err(error!(ErrorCode::MaxFSMAccountCountReached));
        }
        msg!("fsm_account_address: {}",fsm_account_address);
        for i in 0..MAX_FSM_ACCOUNTS{
            if self.fsm_account_keys[i] == fsm_account_address  
                // fsm_account_address!= DEFAULT_STAKED_MINT
            {
                self.fsm_account_keys[i] = DEFAULT_STAKED_MINT;
                // msg!("i: {}, self.fsm_account_keys[i]: {}",i, self.fsm_account_keys[i]);
                // msg!("self.no_fsm_accounts{}", self.no_fsm_accounts);
                self.no_fsm_accounts.try_sub(1)?;
                break;
            }
        }
        Ok(())
    }

    pub fn begin_staking(
        &mut self,
        min_staking_period_sec: u64,
        now_ts: u64,
        gems_in_vault: u64, 
        rarity_points_in_vault: u64,
    ) -> Result<(u64, u64)> {
        self.state = FarmerState::Staked;
        let previous_gems_staked = self.gems_staked;
        let previous_rarity_points_staked = self.rarity_points_staked;
        // msg!("Farmer begin_staking \t previous_gems_staked:{}",previous_gems_staked);
        // msg!("Farmer begin_staking \t previous_rarity_points_staked:{}",previous_rarity_points_staked);
        // msg!("Farmer begin_staking \t gems_in_vault:{}",gems_in_vault);
        // msg!("Farmer begin_staking \t rarity_points_in_vault:{}",rarity_points_in_vault);
        // msg!("Farmer begin_staking \t min_staking_period_sec:{}",min_staking_period_sec);
        self.gems_staked = gems_in_vault;
        self.rarity_points_staked = rarity_points_in_vault;
        self.min_staking_end_ts = now_ts.try_add(min_staking_period_sec)?;
        self.cooldown_end_ts = 0; // zero it out in case it was set before

        Ok((previous_gems_staked, previous_rarity_points_staked))
    }

    pub fn end_staking_begin_cooldown(
        &mut self,
        now_ts: u64,
        cooldown_period_sec: u64,
    ) -> Result<(u64, u64)> {
        if !self.can_end_cooldown(now_ts) {
            return Err(error!(ErrorCode::MinStakingNotPassed));
        }

        self.state = FarmerState::PendingCooldown;
        let gems_unstaked = self.gems_staked;
        let rarity_points_unstaked = self.rarity_points_staked;
        self.gems_staked = 0; //no rewards will accrue during the cooldown period
        self.rarity_points_staked = 0;
        self.cooldown_end_ts = now_ts.try_add(cooldown_period_sec)?;
        msg!("end_staking_begin_cooldown \t self.cooldown_end_ts:{}",self.cooldown_end_ts);
        msg!("end_staking_begin_cooldown \t rarity_points_unstaked:{}",rarity_points_unstaked);
        msg!("end_staking_begin_cooldown \t {} gems are cooling down", gems_unstaked);
        Ok((gems_unstaked, rarity_points_unstaked))
    }

    pub fn end_cooldown(&mut self, now_ts: u64) -> Result<()> {
        if self.can_end_staking(now_ts) {
            return Err(error!(ErrorCode::CooldownNotPassed));
        }
        self.state = FarmerState::Unstaked;
        self.gems_staked = 0;
        self.rarity_points_staked = 0;
        self.cooldown_end_ts = 0;
        self.min_staking_end_ts = 0;
        msg!("end_cooldown gems now unstaked, cooldown is done and available for withdrawal");
        Ok(())
    }

    fn can_end_staking(&self, now_ts: u64) -> bool {
        now_ts >= self.min_staking_end_ts
    }

    fn can_end_cooldown(&self, now_ts: u64) -> bool {
        now_ts >= self.cooldown_end_ts
    }
}
