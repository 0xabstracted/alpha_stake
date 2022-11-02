pub mod farm_manager;
pub mod stakor_user;
pub mod bank_manager;
pub mod start_stake_pool;

pub use farm_manager::*;
pub use stakor_user::*;
pub use bank_manager::*;
pub use start_stake_pool::*;


// // have to duplicate or this won't show up in IDL
// use anchor_lang::prelude::*;

// #[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, Default, PartialEq)]
// pub struct RarityConfig {
//     pub mint: Pubkey,
//     pub rarity_points: u16,
// }
