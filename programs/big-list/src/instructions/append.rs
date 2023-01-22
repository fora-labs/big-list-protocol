use crate::{
    constants::MAX_LIST_VECTOR_SIZE,
    state::BigList,
    utils::{get_j, get_k, get_l},
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
    // #[account(
    //     init_if_needed,
    //     seeds = [get_j((big_list.len + 1) as u32).to_string().as_ref(), id.as_ref(), authority.key().as_ref()],
    //     space = BigList::size(((addresses.len() + big_list_j_next.total_elements as usize)) as usize),
    //     payer = authority,
    //     bump,
    // )]
    /// CHECK: Checked in process
    pub big_list_j_next: AccountInfo<'info>,
    // #[account(
    //     init_if_needed,
    //     seeds = [get_k((big_list.len + 1) as u32).to_string().as_ref(), get_j((big_list.len + 1 )as u32).to_string().as_ref(), authority.key().as_ref()],
    //     bump,
    //     space = BigList::size(((addresses.len() + big_list_k_next.total_elements as usize)) as usize),
    //     payer = authority,
    // )]
    /// CHECK: Checked in process
    pub big_list_k_next: AccountInfo<'info>,
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
    let addresses = &mut addresses.clone();

    let addresses_len = addresses.len();

    let next_size = big_list_k.len + addresses_len as u16;

    if next_size > 256 {
        panic!("we need to initialize the next_k and the next_j account");
    }

    big_list_k.len += addresses_len as u16;
    let new_size = (big_list_k.len) as usize;

    big_list.total_elements += addresses_len as u32;
    big_list_j.total_elements += addresses_len as u32;
    big_list_k.total_elements += addresses_len as u32;

    msg!("new_len_k, new_len_j, new_len_big_list!: {}", addresses_len);

    assert_list_does_not_exced_max_len(&big_list);
    assert_list_does_not_exced_max_len(&big_list_j);
    assert_list_does_not_exced_max_len(&big_list_k);
    msg!("Size!: {}", big_list_k.elements.len());

    msg!("Expected Size!: {}", new_size);

    big_list_k.elements.append(addresses);
    big_list_k.elements.resize(new_size, Pubkey::default());
    msg!("New Size!: {}", big_list_k.elements.len());

    Ok(())
}
