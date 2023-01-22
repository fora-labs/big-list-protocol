import * as anchor from "@project-serum/anchor";
import { Program, ProgramAccount } from "@project-serum/anchor";
import { BigList } from "../target/types/big_list";
import {
  deriveAccountsForCurrentSize,
  getBigList,
  getCurrentIndices,
} from "../js";
import { Keypair, PublicKey } from "@solana/web3.js";
import { assert } from "chai";
import _ from "lodash";

describe("big-list", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.BigList as Program<BigList>;

  it("Calcs current indices correctly from 0", () => {
    let [j, k, l] = getCurrentIndices(0);
    assert(j === 0);
    assert(k === 0);
    assert(l === 0);
    [j, k, l] = getCurrentIndices(256);
    assert(j === 0);
    assert(k === 0);
    assert(l === 255);
    [j, k, l] = getCurrentIndices(257);
    assert(j === 0);
    assert(k === 1);
    assert(l === 0);
    [j, k, l] = getCurrentIndices(255);
    assert(j === 0);
    assert(k === 0);
    assert(l === 254);
    [j, k, l] = getCurrentIndices(16777216);
    assert(j === 255);
    assert(k === 255);
    assert(l === 255);
    [j, k, l] = getCurrentIndices(10000);
    assert(j === 0);
    assert(k === 39);
    assert(l === 15);
  });

  it("Initializes a big list.", async () => {
    const bigList = getBigList(program.provider.publicKey, "my_big_list");
    const [j, k] = getCurrentIndices(0);

    const bigListJ = getBigList(program.provider.publicKey, "my_big_list", j);
    const bigListK = getBigList(
      program.provider.publicKey,
      "my_big_list",
      j,
      k
    );
    // console.log(bigList.toBase58(),  getBigListAny(program.provider.publicKey, "my_big_list", 0).toBase58(), bigListJ.toBase58(), bigListK.toBase58());

    // Add your test here.
    const tx = await program.methods
      .initialize("my_big_list", 3)
      .accounts({
        bigList,
        bigListJ,
        bigListK,
        authority: program.provider.publicKey,
      })
      .rpc();
    console.log("Your transaction signature", tx);
    const bigListAccount = await program.account.bigList.fetch(bigList);
    const bigListJAccount = await program.account.bigList.fetch(bigListJ);
    const bigListKAccount = await program.account.bigList.fetch(bigListK);

    assert(bigListAccount.len === 1);
    assert(bigListJAccount.len === 1);
    assert(bigListKAccount.len === 0);

    assert(bigListAccount.totalElements === 0);
    assert(bigListJAccount.totalElements === 0);
    assert(bigListKAccount.totalElements === 0);

    console.log(bigListAccount, bigListJAccount, bigListKAccount);
  });

  const appendATonOfAddresses = async (
    listId: string,
    addresses: PublicKey[],
    program: Program<BigList>
  ) => {
    const bigListAccount = await program.account.bigList.fetch(
      getBigList(program.provider.publicKey, listId)
    );
    console.log('bigListAccount totalElements', bigListAccount);
    const accounts = await deriveAccountsForCurrentSize(
      listId,
      bigListAccount.totalElements,
      program
    );

    // const [j, k] = getCurrentIndices(1000);

    const batches: PublicKey[][] = _.chunk(addresses, 25);

    console.log("batches", batches);

    const txs = batches.map((addressBatch) => async () => {
      try {
        const signature = await program.methods
          .append("my_big_list", addressBatch)
          .accounts({
            ...accounts,
            // bigListKNext,
            authority: program.provider.publicKey,
          })
          .rpc();
        console.log("transaction: ", signature);
        return signature;
      } catch (error) {
        // TODO: Retry logic
        console.log("error!", error);
      }
    });

    const processBatch = async (
      txs: (() => Promise<string>)[],
      results = []
    ) => {
      if (txs.length === 0) {
        return results;
      }
      try {
        const [nextTx, ...remainingTxs] = txs;
        const signature = await nextTx();
        return await processBatch(remainingTxs, results.concat([signature]));
      } catch (error) {
        console.log("error", error);
        throw error;
      }
    };
    return await processBatch(txs);
  };

  it("Appends a bunch of addresses", async () => {
    const addresses: PublicKey[] = new Array(28)
      .fill(0)
      .map(() => new Keypair().publicKey);
      let bigListAccount = await program.account.bigList.fetch(
        getBigList(program.provider.publicKey, "my_big_list")
      );

      const listAccounts = await deriveAccountsForCurrentSize(
        "my_big_list",
        bigListAccount.totalElements,
        program
      );
    // console.log(bigList.toBase58(),  getBigListAny(program.provider.publicKey, "my_big_list", 0).toBase58(), bigListJ.toBase58(), bigListK.toBase58());

    try {
      const tx = await program.methods
        .append("my_big_list", addresses)
        .accounts({
          ...listAccounts,
          authority: program.provider.publicKey,
        })
        .rpc();
    } catch (error) {
      console.log(error);
    }

    bigListAccount = await program.account.bigList.fetch(listAccounts.bigList);
    const bigListJAccount = await program.account.bigList.fetch(listAccounts.bigList);
    const bigListKAccount = await program.account.bigList.fetch(listAccounts.bigList);

    console.log('bigListKAccount', bigListKAccount)
    assert(bigListAccount.totalElements === 28);
    assert(bigListJAccount.totalElements === 28);
    assert(bigListKAccount.totalElements === 28);
    // assert(bigListKAccount.elements.length === 28);
  });

  it("Appends a large amount of tx", async () => {

    const addresses: PublicKey[] = new Array(100)
      .fill(0)
      .map(() => new Keypair().publicKey);

    const signatures = await appendATonOfAddresses(
      "my_big_list",
      addresses,
      program
    );
    const bigListK = getBigList(
      program.provider.publicKey,
      "my_big_list",
      0,
      0
    );
    const bigListKAccount = await program.account.bigList.fetch(bigListK);
    console.log('bigListKaccount', bigListKAccount)
    assert(bigListKAccount.elements.length === 128);
  });

  it("Batch appends up to 256", async () => {
    const [j, k] = getCurrentIndices(128);
    const bigListK = getBigList(
      program.provider.publicKey,
      "my_big_list",
      j,
      k
    );

    const addresses: PublicKey[] = new Array(128)
      .fill(0)
      .map(() => new Keypair().publicKey);

    const signatures = await appendATonOfAddresses(
      "my_big_list",
      addresses,
      program
    );
    console.log("signatures", signatures);


    const bigListKAccount = await program.account.bigList.fetch(bigListK);
    console.log('bigListKaccount', bigListKAccount)
    assert(bigListKAccount.elements.length === 256);
  });

  it("Rolls over to next leaf", async () => {
    const [j, k] = getCurrentIndices(256);
    const addresses: PublicKey[] = new Array(1)
      .fill(0)
      .map(() => new Keypair().publicKey);

    await appendATonOfAddresses(
      "my_big_list",
      addresses,
      program
    );

  });

  // it("Append one more", async () => {
  //   const [j, k] = getCurrentIndices(257);
  //   const bigListK = getBigList(
  //     program.provider.publicKey,
  //     "my_big_list",
  //     j,
  //     k
  //   );

  //   const addresses: PublicKey[] = new Array(1)
  //     .fill(0)
  //     .map(() => new Keypair().publicKey);

  //   const signatures = await appendATonOfAddresses(
  //     "my_big_list",
  //     addresses,
  //     program
  //   );
  //   console.log("signatures", signatures);
  //   const bigListKAccount = await program.account.bigList.fetch(bigListK);
  //   assert(bigListKAccount.elements.length === 256);
  // });
});
