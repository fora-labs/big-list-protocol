import * as anchor from "@project-serum/anchor";
import { Program, ProgramAccount } from "@project-serum/anchor";
import { BigList } from "../target/types/big_list";
import {
  appendATonOfAddresses,
  deriveAccountsForCurrentAndNextSize,
  getBigList,
  getCurrentIndices,
} from "../js";
import { Keypair, LAMPORTS_PER_SOL, PublicKey } from "@solana/web3.js";
import { assert } from "chai";

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

    const tx = await program.methods
      .initialize("my_big_list", 3)
      .accounts({
        bigList,
        bigListJ,
        bigListK,
        authority: program.provider.publicKey,
      })
      .rpc();
    const bigListAccount = await program.account.bigList.fetch(bigList);
    const bigListJAccount = await program.account.bigList.fetch(bigListJ);
    const bigListKAccount = await program.account.bigList.fetch(bigListK);

    assert(bigListAccount.len === 1);
    assert(bigListJAccount.len === 1);
    assert(bigListKAccount.len === 0);

    assert(bigListAccount.totalElements === 0);
    assert(bigListJAccount.totalElements === 0);
    assert(bigListKAccount.totalElements === 0);
  });

  it("Appends a bunch of addresses", async () => {
    const addresses: PublicKey[] = new Array(28)
      .fill(0)
      .map(() => new Keypair().publicKey);
    let bigListAccount = await program.account.bigList.fetch(
      getBigList(program.provider.publicKey, "my_big_list")
    );

    const listAccounts = await deriveAccountsForCurrentAndNextSize(
      "my_big_list",
      bigListAccount.totalElements,
      28,
      program
    );

    try {
      const tx = await program.methods
        .append("my_big_list", addresses)
        .accounts({
          ...listAccounts,
          authority: program.provider.publicKey,
        })
        .rpc();
    } catch (error) {
      console.error(error);
    }

    bigListAccount = await program.account.bigList.fetch(listAccounts.bigList);
    const bigListJAccount = await program.account.bigList.fetch(
      listAccounts.bigListJ
    );
    const bigListKAccount = await program.account.bigList.fetch(
      listAccounts.bigListK
    );

    assert(bigListAccount.totalElements === 28);
    assert(bigListAccount.len === 1);
    assert(bigListAccount.elements.length === 1);
    assert(bigListJAccount.totalElements === 28);
    assert(bigListJAccount.len === 1);
    assert(bigListJAccount.elements.length === 1);
    assert(bigListKAccount.totalElements === 28);
    assert(bigListKAccount.elements.length === 28);
    assert(bigListKAccount.len === 28);
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
    const bigListKAccount = await program.account.bigList.fetch(bigListK);
    assert(bigListKAccount.elements.length === 256);
  });

  it("Rolls over to next leaf", async () => {
    const addresses: PublicKey[] = new Array(2)
      .fill(0)
      .map(() => new Keypair().publicKey);

    await appendATonOfAddresses("my_big_list", addresses, program);

    let bigListAccount = await program.account.bigList.fetch(
      getBigList(program.provider.publicKey, "my_big_list")
    );

    const listAccounts = await deriveAccountsForCurrentAndNextSize(
      "my_big_list",
      bigListAccount.totalElements,
      0,
      program
    );

    bigListAccount = await program.account.bigList.fetch(listAccounts.bigList);
    const bigListJAccount = await program.account.bigList.fetch(
      listAccounts.bigListJ
    );
    const bigListKAccount = await program.account.bigList.fetch(
      listAccounts.bigListK
    );

    assert(bigListAccount.totalElements === 258);
    assert(bigListAccount.len === 1);
    assert(bigListAccount.elements.length === 1);

    assert(bigListJAccount.totalElements === 258);
    assert(bigListJAccount.len === 2);
    assert(bigListJAccount.elements.length === 2);

    assert(bigListKAccount.totalElements === 2);
    assert(bigListKAccount.elements.length === 2);
    assert(bigListKAccount.len === 2);
  });

  it("Adds 1 whitelist (10K pubkeys)", async () => {
    const addresses: PublicKey[] = new Array(10000)
      .fill(0)
      .map(() => new Keypair().publicKey);

    const balanceBefore = await program.provider.connection.getBalance(
      program.provider.publicKey
    );

    let start = Date.now();
    await appendATonOfAddresses("my_big_list", addresses, program);
    let end = Date.now();

    let bigListAccount = await program.account.bigList.fetch(
      getBigList(program.provider.publicKey, "my_big_list")
    );

    const listAccounts = await deriveAccountsForCurrentAndNextSize(
      "my_big_list",
      bigListAccount.totalElements,
      0,
      program
    );

    const balanceAfter = await program.provider.connection.getBalance(
      program.provider.publicKey
    );

    console.info("10K appended : ", (end - start) / 60000, " minutes.");

    console.info(
      "SOL Cost   : ",
      (balanceBefore - balanceAfter) / LAMPORTS_PER_SOL
    );

    bigListAccount = await program.account.bigList.fetch(listAccounts.bigList);
    // const bigListJAccount = await program.account.bigList.fetch(
    //   listAccounts.bigListJ
    // );
    // const bigListKAccount = await program.account.bigList.fetch(
    //   listAccounts.bigListK
    // );

    assert(bigListAccount.totalElements === 10258);
    assert(bigListAccount.len === 1);
    assert(bigListAccount.elements.length === 1);
  });
});
