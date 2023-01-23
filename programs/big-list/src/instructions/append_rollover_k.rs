use crate::{
    constants::MAX_LIST_VECTOR_SIZE,
    state::BigList,
    utils::{get_j, get_k},
};
use anchor_lang::prelude::*;

#[derive(Accounts)]
#[instruction(id: String, addresses: Vec<Pubkey>)]
pub struct AppendRolloverK<'info> {
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
        realloc = BigList::size((big_list_j.len + 1) as usize),
        realloc::payer = authority,
        realloc::zero = false,
    )]
    pub big_list_j: Account<'info, BigList>,
    #[account(
        mut,
        seeds = [get_k(big_list.total_elements as u32).to_string().as_ref(), get_j(big_list.total_elements as u32).to_string().as_ref(), id.as_ref(), authority.key().as_ref()],
        bump,
        realloc = BigList::size(256),
        realloc::payer = authority,
        realloc::zero = false,
    )]
    pub big_list_k: Account<'info, BigList>,

    #[account(
        init,
        seeds = [get_k(((big_list.total_elements as u32) + (addresses.len() as u32)) as u32).to_string().as_ref(), get_j(((big_list.total_elements as u32) + (addresses.len() as u32)) as u32).to_string().as_ref(), id.as_ref(), authority.key().as_ref()],
        bump,
        space = {
            let remaining_space = ((256 as usize) - (big_list_k.len as usize)) as usize;
            let rollover_space = addresses.len() - remaining_space;
            BigList::size(rollover_space)
        },
        payer = authority,
    )]

    pub big_list_k_next: Account<'info, BigList>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,

}

pub fn assert_list_does_not_exced_max_len(big_list: &BigList) {
    if big_list.elements.len() > MAX_LIST_VECTOR_SIZE {
        panic!("Lists can not hold more than 256")
    }
}

pub fn process(ctx: Context<AppendRolloverK>, _id: String, addresses: Vec<Pubkey>) -> Result<()> {
    let big_list = &mut ctx.accounts.big_list;
    let big_list_j = &mut ctx.accounts.big_list_j;
    let big_list_k = &mut ctx.accounts.big_list_k;
    let big_list_k_next = &mut ctx.accounts.big_list_k_next;

    let remaining_k_space = (((256 as u16) - &big_list_k.len) as u16) as usize;

    let mut addresses_to_append = addresses.clone();

    let (k_address, k_next_addresses) = addresses_to_append.split_at_mut(remaining_k_space);

    big_list.total_elements += addresses.len() as u32;

    big_list_k.total_elements += k_address.len() as u32;
    big_list_k.len += k_address.len() as u16;

    big_list_k_next.total_elements += k_next_addresses.len() as u32;
    big_list_k_next.len += k_next_addresses.len() as u16;

    for address in k_address {
        big_list_k.elements.push(*address);
    }

    for address in k_next_addresses {
        big_list_k_next.elements.push(*address);
    }

    big_list_j.elements.push(big_list_k_next.key());
    big_list_j.len += 1;
    big_list_j.total_elements += addresses.len() as u32;

    Ok(())
}
