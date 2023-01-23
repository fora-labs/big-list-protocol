import {
  AccountsCoder,
  BN,
  IdlTypes,
  Program,
  ProgramAccount,
  web3,
} from "@project-serum/anchor";
import { PublicKey } from "@solana/web3.js";
import { BigList } from "../target/types/big_list";
import _ from "lodash";

export const BIG_LIST_PROGRAM_ID = new web3.PublicKey(
  "Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS"
);

export const getBigList = (
  authority: web3.PublicKey,
  id: string,
  ...args: number[] | string[]
) => {
  const argBuffers = [];
  args.reverse().forEach((arg, i) => {
    argBuffers[i] = Buffer.from(arg.toString());
  });
  return web3.PublicKey.findProgramAddressSync(
    [...argBuffers, Buffer.from(id), authority.toBuffer()],
    BIG_LIST_PROGRAM_ID
  )[0];
};

const MAX_LIST_VECTOR_SIZE = 256;

export const getCurrentIndices = (
  totalElements: number
): [number, number, number] => {
  if (totalElements === 0) {
    return [0, 0, 0];
  }

  if (totalElements > 256 * 256 * 256) {
    throw "Total is too large";
  }
  let input = totalElements - 1;

  let j = Math.floor(input / (256 * 256));
  let k = Math.floor((input % (256 * 256)) / 256);
  let l = input % 256;

  return [j, k, l];
};

export const deriveAccountsForCurrentAndNextSize = async (
  listId,
  listLen,
  addressToAppendLength,
  program
) => {
  const [j, k] = getCurrentIndices(listLen);
  const [jNext, kNext] = getCurrentIndices(listLen + addressToAppendLength);

  const bigList = getBigList(program.provider.publicKey, listId);
  const bigListJ = getBigList(program.provider.publicKey, listId, j);
  const bigListK = getBigList(program.provider.publicKey, listId, j, k);
  const bigListJNext = getBigList(program.provider.publicKey, listId, jNext);
  const bigListKNext = getBigList(
    program.provider.publicKey,
    listId,
    jNext,
    kNext
  );

  return {
    bigList,
    bigListJ,
    bigListJNext:
      bigListJ.toBase58() === bigListJNext.toBase58()
        ? undefined
        : bigListJNext,
    bigListK,
    bigListKNext:
      bigListK.toBase58() === bigListKNext.toBase58()
        ? undefined
        : bigListKNext,
    bigListProgram: BIG_LIST_PROGRAM_ID,
  };
};

export const appendATonOfAddresses = async (
  listId: string,
  addresses: PublicKey[],
  program: Program<BigList>
) => {
  const batches: PublicKey[][] = _.chunk(addresses, 25);

  const txs = batches.map((addressBatch) => async () => {
    try {
      const bigListAccount = await program.account.bigList.fetch(
        getBigList(program.provider.publicKey, listId)
      );
      // console.info(
      //   "Processing batch: ",
      //   bigListAccount.totalElements + addressBatch.length
      // );
      const accounts = await deriveAccountsForCurrentAndNextSize(
        listId,
        bigListAccount.totalElements,
        addressBatch.length,
        program
      );

      if (accounts.bigListKNext) {
        const signature = await program.methods
          .appendRolloverK("my_big_list", addressBatch)
          .accounts({
            ...accounts,
            // bigListKNext,
            authority: program.provider.publicKey,
          })
          .rpc();
        console.info(
          "Success   : total ",
          bigListAccount.totalElements + addressBatch.length,
          "\nSignature : ",
          signature
        );
        return signature;
      } else {
        const signature = await program.methods
          .append("my_big_list", addressBatch)
          .accounts({
            ...accounts,
            // bigListKNext,
            authority: program.provider.publicKey,
          })
          .rpc();

        console.info(
          "Success   : total ",
          bigListAccount.totalElements + addressBatch.length,
          "\nSignature : ",
          signature
        );
        return signature;
      }
    } catch (error) {
      // TODO: Retry logic
      console.error("error!", error);
    }
  });

  const processBatch = async (txs: (() => Promise<string>)[], results = []) => {
    if (txs.length === 0) {
      return results;
    }
    try {
      const [nextTx, ...remainingTxs] = txs;
      const signature = await nextTx();
      return await processBatch(remainingTxs, results.concat([signature]));
    } catch (error) {
      console.error("error", error);
      throw error;
    }
  };
  return await processBatch(txs);
};

// pub fn get_current_indices(total_elements: u32) -> (u8, u8, u8) {
//   if total_elements == 0 {
//       return (0, 0, 0);
//   }
//   if total_elements > 256 * 256 * 256 {
//       panic!("total is too large")
//   }
//   let input = total_elements - 1;
//   let j = (input / (256 * 256)) as u8;
//   let k = input % ((256 * 256) as u32) / 256;
//   let l = input % 256;

//   println!("j,k,l {},{},{} ", j, k, l);
//   return (j as u8, k as u8, l as u8);
// }

// pub fn get_current_depth(big_list: &BigList) -> u8 {
//   let depth = big_list.total_elements.checked_div(256).unwrap();
//   return depth as u8;
// }

// pub fn get_current_breadth(big_list: &BigList) -> u8 {
//   let breadth = big_list.total_elements % 256;
//   return breadth as u8;
// }

// pub fn get_current_indices(big_list: BigList) -> (u8, u8) {
//   let depth = get_current_depth(&big_list);
//   let breadth = get_current_breadth(&big_list);
//   return (depth as u8, breadth as u8);
// }
