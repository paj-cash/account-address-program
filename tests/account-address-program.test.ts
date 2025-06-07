import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { AccountAddressProgram } from "../target/types/account_address_program";
import { assert } from "chai";
import {
  Keypair,
  SystemProgram,
  LAMPORTS_PER_SOL,
  PublicKey,
  Transaction,
} from "@solana/web3.js";
import {
  createMint,
  getOrCreateAssociatedTokenAccount,
  mintTo,
  transfer,
  getAccount,
} from "@solana/spl-token";
import { join } from "path";
import { readFileSync } from "fs";

// typescript

describe("account-address-program", () => {
  anchor.setProvider(anchor.AnchorProvider.env());
  const program = anchor.workspace
    .accountAddressProgram as Program<AccountAddressProgram>;
  const provider = anchor.getProvider();
  const connection = provider.connection;
  const payer = provider.wallet.payer;

  let user1: Keypair;
  let mint: PublicKey;

  const accountName = "Matthew Chukwuemeka";
  const accountNumber = "0025635480";
  const bankCode = "044";
  const region = "NG";

  //   const bankCode = "044";
  const tochiABAA = getAccountAddressPda();

  before(async () => {
    //loader user 1
    user1 = loadKeypair("user1");
    registerKeypair(user1, connection);

    //load mint
    const mintKeypair = loadKeypair("mint");
    mint = await createMyMint(mintKeypair, payer);
  });

  it.skip("Create Account Address", async () => {
    const accountInfo = getAccountInfoPda();
    console.log({
      accountAddress: tochiABAA[0].toString(),
      accountInfo: accountInfo[0].toString(),
    });

    const tx = await program.methods
      .createAccountAddress(accountName, accountNumber, bankCode, region)
      .accounts({
        payer: payer.publicKey,
      })
      .signers([payer])
      .rpc();

    // Fetch the created account (replace with your account fetch logic)
    // const tochiAccount = await getAccount(connection, tochiABAA[0]);
    // const account = await program.account.accountAddress.fetch(tochiABAA[0]);
    // console.log("Account Address Lamport:", tochiAccount.amount);
    // assert.equal(account.owner.toBase58(), payer.publicKey.toBase58());
    assert.isString(tx, "Transaction signature should be a string");
  });

  it.skip("Transfer SOL to Account Address", async () => {
    const initialBalance = await connection.getBalance(tochiABAA[0]);

    const transaction = new Transaction().add(
      SystemProgram.transfer({
        fromPubkey: payer.publicKey,
        toPubkey: tochiABAA[0],
        lamports: LAMPORTS_PER_SOL * 10,
      })
    );
    const signature = await provider.sendAndConfirm(transaction, [payer]);
    console.log("Transfer SOL Transaction Signature:", signature);

    const finalBalance = await connection.getBalance(tochiABAA[0]);
    assert.isAbove(
      finalBalance,
      initialBalance,
      "Recipient should have more SOL after transfer"
    );
  });

  it.skip("Transfer SPL Token to Pool", async () => {
    const initialBalance = await connection.getBalance(tochiABAA[0]);

    const tx = await program.methods
      .transferSolToPool(accountNumber, bankCode, region)
      .accounts({ payer: payer.publicKey, txPool: user1.publicKey })
      .signers([payer])
      .rpc();
    console.log("Transfer SOL to Tx Pool Transaction Signature:", tx);

    const finalBalance = await connection.getBalance(tochiABAA[0]);
    console.log("Final Balance:", finalBalance);
    assert.isAbove(
      initialBalance,
      finalBalance,
      "Recipient should have more SOL after transfer to pool"
    );
  });

  it.skip("Transfer SPL Token to Account Address", async () => {
    const senderAta = await getATA(payer.publicKey);
    console.log("Sender Token Account:", senderAta.toBase58());

    const tochiATA = await getPDA_ATA(tochiABAA[0]);
    console.log("Recipient Token Account:", tochiATA.toBase58());

    // Mint tokens to sender
    await mintTo(
      connection,
      payer,
      mint,
      senderAta,
      payer.publicKey,
      1000_000_000 // 1000 tokens (with 9 decimals)
    );

    const recipientAccountInfoBefore = await getAccount(connection, tochiATA);
    const balanceBefore = Number(recipientAccountInfoBefore.amount);

    // Transfer tokens to recipient
    await transfer(
      connection,
      payer,
      senderAta,
      tochiATA,
      payer.publicKey,
      100_000_000 // 100 tokens
    );

    // Check recipient balance
    const recipientAccountInfo = await getAccount(connection, tochiATA);
    assert.equal(
      Number(recipientAccountInfo.amount),
      100_000_000,
      "Recipient should have received SPL tokens"
    );

    assert.isTrue(
      Number(recipientAccountInfo.amount) >= balanceBefore + 100_000_00,
      "Recipient should have tokens"
    );
  });

  it.only("Transfer SPL Token to Pool", async () => {
    const tochiABAA_ATA = await getPDA_ATA(tochiABAA[0]);
    console.log("Tochi Account Token Account:", tochiABAA_ATA.toBase58());
    const poolATA = await getATA(user1.publicKey);
    console.log("Pool Token Account:", poolATA.toBase58());

    const initialBalance = await getAccount(connection, tochiABAA_ATA);

    const txSig = await program.methods
      .transferTokenToPool(accountNumber, bankCode, region)
      .accounts({
        payer: payer.publicKey,
        mint: mint,
        accountTokenAddress: tochiABAA_ATA,
        txPool: user1.publicKey,
        txPoolTokenAccount: await getATA(poolATA),
      })
      .signers([payer])
      .rpc();

    assert.isString(txSig, "Transaction signature should be a string");
  });

  function getAccountInfoPda(): [PublicKey, number] {
    return PublicKey.findProgramAddressSync(
      [
        Buffer.from("info"),
        Buffer.from(accountNumber),
        Buffer.from(bankCode),
        Buffer.from(region),
      ],
      program.programId
    );
  }

  function getAccountAddressPda(): [PublicKey, number] {
    return PublicKey.findProgramAddressSync(
      [Buffer.from(accountNumber), Buffer.from(bankCode), Buffer.from(region)],
      program.programId
    );
  }

  async function getATA(address: PublicKey) {
    const result = await getOrCreateAssociatedTokenAccount(
      connection,
      payer,
      mint,
      address
    );
    return result.address;
  }

  async function getPDA_ATA(address: PublicKey): Promise<PublicKey> {
    const result = await getOrCreateAssociatedTokenAccount(
      connection,
      payer,
      mint,
      address,
      true
    );
    return result.address;
  }

  async function createMyMint(
    mint: anchor.web3.Keypair,
    payer: anchor.web3.Keypair
  ) {
    const account = await connection.getAccountInfo(mint.publicKey);

    if (!account) {
      await createMint(
        connection,
        payer,
        payer.publicKey,
        payer.publicKey,
        6,
        mint
      );
    }
    return mint.publicKey;
  }
});

function loadKeypair(name) {
  const keyPath = join(__dirname, "keys", `${name}.json`);
  const secret = Uint8Array.from(JSON.parse(readFileSync(keyPath, "utf-8")));
  return Keypair.fromSecretKey(secret);
}

async function registerKeypair(
  keypair: anchor.web3.Keypair,
  connection: anchor.web3.Connection
) {
  const account = await connection.getAccountInfo(keypair.publicKey);

  if (!account) {
    const sig = await connection.requestAirdrop(
      keypair.publicKey,
      LAMPORTS_PER_SOL
    );
    const latestBlockhash = await connection.getLatestBlockhash();
    await connection.confirmTransaction(
      {
        blockhash: latestBlockhash.blockhash,
        lastValidBlockHeight: latestBlockhash.lastValidBlockHeight,
        signature: sig,
      },
      "confirmed"
    );
    console.log(`Airdropped 1 SOL to ${keypair.publicKey.toBase58()}`);
  }
}
