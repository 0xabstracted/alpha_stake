use anchor_lang::prelude::*;

#[derive(Accounts)]
#[instruction]
pub struct FsmInitNew<'info> {
     // farmer
     #[account(mut, has_one = farm, has_one = identity, has_one = vault,
        seeds = [
            b"farmer".as_ref(),
            farm.key().as_ref(),
            identity.key().as_ref(),
        ],
        bump = bump_farmer)]
    pub farmer: Box<Account<'info, Farmer>>,
    #[account(
        mut,
        seeds = [
            b"farmer_staked_mints".as_ref(), 
            // &index.to_le_bytes(),
            farmer.key().as_ref(),
        ],
        bump = farmer_staked_mints.load()?.bump,
        has_one = farmer,
    )]
    pub farmer_staked_mints: AccountLoader<'info, FarmerStakedMints>,
   
}