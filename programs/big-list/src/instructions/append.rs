use crate::{
    constants::MAX_LIST_VECTOR_SIZE,
    state::BigList,
    utils::{get_j, get_k},
};
use anchor_lang::prelude::*;

#[derive(Accounts)]
#[instruction(id: String, addresses: Vec<Pubkey>)]
pub struct Append<'info> {
    #[account(
        has_one = authority,
        mut,
        seeds = [id.as_ref(), authority.key().as_ref()],
        bump,
    )]
    pub big_list: Account<'info, BigList>,
    #[account(
        mut,
        seeds = [get_j(big_list.total_elements as u32).to_string().as_ref(), id.as_ref(), authority.key().as_ref()],
        bump,
    )]
    pub big_list_j: Account<'info, BigList>,
    #[account(
        mut,
        seeds = [get_k(big_list.total_elements as u32).to_string().as_ref(), get_j(big_list.total_elements as u32).to_string().as_ref(), id.as_ref(), authority.key().as_ref()],
        bump,
        realloc = BigList::size(((addresses.len() + (big_list_k.len as usize))) as usize),
        realloc::payer = authority,
        realloc::zero = false,
    )]
    pub big_list_k: Account<'info, BigList>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

pub fn assert_list_does_not_exced_max_len(big_list: &BigList) {
    if big_list.elements.len() > MAX_LIST_VECTOR_SIZE {
        panic!("Lists can not hold more than 256")
    }
}

pub fn process(ctx: Context<Append>, _id: String, addresses: Vec<Pubkey>) -> Result<()> {
    let big_list = &mut ctx.accounts.big_list;
    let big_list_j = &mut ctx.accounts.big_list_j;
    let big_list_k = &mut ctx.accounts.big_list_k;

    let addresses_len = addresses.len();

    let next_size = big_list_k.len + (addresses_len as u16);
    if next_size > 256 {
        panic!("Going over 256")
    }

    big_list_k.len = next_size;
    let new_size = (big_list_k.len) as usize;
    big_list.total_elements += addresses_len as u32;
    big_list_j.total_elements += addresses_len as u32;
    big_list_k.total_elements += addresses_len as u32;
    assert_list_does_not_exced_max_len(&big_list);
    assert_list_does_not_exced_max_len(&big_list_j);
    assert_list_does_not_exced_max_len(&big_list_k);
    big_list_k.elements.append(&mut addresses.clone());
    big_list_k.elements.resize(new_size, Pubkey::default());
    Ok(())
}
