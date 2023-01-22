use anchor_lang::prelude::*;

// use super::IndexPermissions;

// Index Types enum
#[derive(AnchorDeserialize, AnchorSerialize, Clone, Copy, Debug, PartialEq)]
pub enum IndexType {
    GrowableIndex,
    PrepaidIndex,
}

#[account]
pub struct BigList {
    pub version: u8,
    pub index_type: IndexType,
    pub authority: Pubkey,
    pub len: u16,
    pub depth: u8,
    pub index: u8,
    pub total_elements: u32,
    pub elements: Vec<Pubkey>,
    pub created_at: i64,
}

impl BigList {
    pub const BASE_LEN: usize
        = 8  // discriminator
        + 1  // version            u8
        + 1  // type               u8
        + 32 // authority          Pubkey
        + 2  // len              u8
        + 1  // depth              u32
        + 1  // index              u32
        + 4  // total_elements     u32
        + 4  // elements           (empty vector)
        + 8  // created_at.        i64
        ;

    pub fn size(items: usize) -> usize {
        Self::BASE_LEN + items * 32
    }

    pub fn init(
        &mut self,
        depth: u8,
        index: u8,
        authority: Pubkey,
        created_at: i64,
        first_element: Option<Pubkey>,
    ) {
        self.version = 0;
        self.index_type = IndexType::GrowableIndex;
        self.authority = authority;
        self.depth = depth;
        self.index = index;
        self.created_at = created_at;
        match first_element {
            Some(pubkey) => {
                self.elements = vec![pubkey];
                self.len = 1;
            }
            None => {
                self.elements = vec![];
                self.len = 0;
            }
        }
    }
}
