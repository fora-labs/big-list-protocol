use crate::{
    state::BigList, utils::{get_j, get_k},
};
use anchor_lang::prelude::*;

#[derive(Accounts)]
#[instruction(id: String, depth: u8)]
pub struct InitializeBigList<'info> {
    #[account(
        init,
        seeds = [id.as_ref(), authority.key().as_ref()],
        bump,
        space = BigList::size(1),
        payer = signer
    )]
    pub big_list: Account<'info, BigList>,
    #[account(
        init,
        seeds = [get_j(0 as u32).to_string().as_ref(), id.as_ref(), authority.key().as_ref()],
        bump,
        space = BigList::size(1),
        payer = signer
    )]
    pub big_list_j: Account<'info, BigList>,
    #[account(
        init,
        seeds = [get_k(0 as u32).to_string().as_ref(), get_j(0 as u32).to_string().as_ref(), id.as_ref(), authority.key().as_ref()],
        bump,
        space = BigList::size(0),
        payer = signer
    )]
    pub big_list_k: Account<'info, BigList>,
    /// CHECK: Can be any account.
    pub authority: AccountInfo<'info>,
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

pub fn process<'a, 'b, 'c, 'info>(
    ctx: Context<'a, 'b, 'c, 'info, InitializeBigList<'info>>,
    _id: String,
    _depth: u8,
) -> Result<()> {
    let big_list = &mut ctx.accounts.big_list;
    let big_list_j = &mut ctx.accounts.big_list_j;
    let big_list_k = &mut ctx.accounts.big_list_k;

    let authority = &ctx.accounts.authority;
    let now = Clock::get().unwrap().unix_timestamp;

    big_list.init(0, 0, authority.key(), now, Some(big_list_j.key()));
    big_list_j.init(1, 0, authority.key(), now, Some(big_list_k.key()));
    big_list_k.init(2, 0, authority.key(), now, None);

    // for i in 1..=depth  {
    //     let nested_list = Account::<BigList>::try_from(&ctx.remaining_accounts[i as usize]);
    // }

    // Ok(big_list.clone().into_inner())
    Ok(())
}
