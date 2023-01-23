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
  "2dcZKYRfijTg3TMU2xocaCKVv6LJTzzdwtLBbMUyKzKi"
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
        // console.info(
        //   "Success   : total ",
        //   bigListAccount.totalElements + addressBatch.length,
        //   "\nSignature : ",
        //   signature
        // );
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

        // console.info(
        //   "Success   : total ",
        //   bigListAccount.totalElements + addressBatch.length,
        //   "\nSignature : ",
        //   signature
        // );
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


export const CLOCKWORK_THREAD_PROGRAM_ID = new PublicKey(
  "3XXuUFfweXBwFgFfYaejLvZE4cGZiHgKiGfMtdxNzYmv"
);

export const SEED_THREAD = "thread";

export const getClockworkThreadPDA = async (
  authority: PublicKey,
  id: string
): Promise<PublicKey> => {
  const [pubkey] = await PublicKey.findProgramAddress(
    [Buffer.from(SEED_THREAD), authority.toBuffer(), Buffer.from(id)],
    CLOCKWORK_THREAD_PROGRAM_ID
  );
  return pubkey;
};

export const getBatchProccessPDA = async (
  authority: PublicKey,
  id: string
): Promise<PublicKey> => {
  const [pubkey] = await PublicKey.findProgramAddress(
    [Buffer.from("batch_process"), Buffer.from(id), authority.toBuffer()],
    BIG_LIST_PROGRAM_ID
  );
  return pubkey;
};

