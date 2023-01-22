use crate::{constants::MAX_LIST_VECTOR_SIZE, state::BigList};

pub fn get_j(total: u32) -> u8 {
    if total == 0 {
        return 0;
    }
    if total > 256 * 256 * 256 {
        panic!("total is too large")
    }
    let input = total - 1;
    let j = (input / (256 * 256)) as u8;
    return j;
}

pub fn get_k(total: u32) -> u8 {
    if total == 0 {
        return 0;
    }
    if total > 256 * 256 * 256 {
        panic!("total is too large")
    }
    let input = total - 1;
    let j = (input / (256 * 256)) as u8;
    return j;
}

pub fn get_l(total: u32) -> u8 {
    if total == 0 {
        return 0;
    }
    if total > 256 * 256 * 256 {
        panic!("total is too large")
    }
    let input = total - 1;
    let l = (input % 256) as u8;
    return l;
}

pub fn get_current_indices(total_elements: u32) -> (u8, u8, u8) {
    if total_elements == 0 {
        return (0, 0, 0);
    }
    if total_elements > 256 * 256 * 256 {
        panic!("total is too large")
    }
    let input = total_elements - 1;
    let j = (input / (256 * 256)) as u8;
    let k = input % ((256 * 256) as u32) / 256;
    let l = input % 256;

    println!("j,k,l {},{},{} ", j, k, l);
    return (j as u8, k as u8, l as u8);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::IndexType;
    use anchor_lang::prelude::Pubkey;

    pub fn gen_big_list(total_elements: u32) -> BigList {
        BigList {
            version: 0,
            index_type: IndexType::GrowableIndex,
            authority: Pubkey::default(),
            len: 0,
            depth: 0,
            index: 0,
            created_at: 86400 as i64,
            total_elements: total_elements,
            elements: vec![
                Pubkey::default(),
                Pubkey::default(),
                Pubkey::default(),
                Pubkey::default(),
            ],
        }
    }

    #[test]
    pub fn it_returns_the_expected_indices() {
        let total_elements = gen_big_list(256).total_elements;
        let (j, k, l) = get_current_indices(total_elements);
        assert_eq!(j, 0);
        assert_eq!(k, 0);
        assert_eq!(l, 255);

        let total_elements = gen_big_list(257).total_elements;
        let (j, k, l) = get_current_indices(total_elements);
        assert_eq!(j, 0);
        assert_eq!(k, 1);
        assert_eq!(l, 0);

        let total_elements = gen_big_list(255).total_elements;
        let (j, k, l) = get_current_indices(total_elements);
        assert_eq!(j, 0);
        assert_eq!(k, 0);
        assert_eq!(l, 254);

        let total_elements = gen_big_list(10000).total_elements;
        let (j, k, l) = get_current_indices(total_elements);
        assert_eq!(j, 0);
        assert_eq!(k, 39);
        assert_eq!(l, 15);

        // Max
        let total_elements = gen_big_list(16777216).total_elements;
        let (j, k, l) = get_current_indices(total_elements);
        assert_eq!(j, 255);
        assert_eq!(k, 255);
        assert_eq!(l, 255);
    }
}
