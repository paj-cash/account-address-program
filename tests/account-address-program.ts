import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { AccountAddressProgram } from "../target/types/account_address_program";

describe.skip("account-address-program", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace
    .accountAddressProgram as Program<AccountAddressProgram>;

  it("Create Account Address", async () => {
    // Add your test here.
    const redactedAccountName = "Matthew Chukwuemeka";
    const accountNumber = "0025635480";
    const bankCode = "";
    const bankRegion = "NG";
    const tx = await program.methods.createAccountAddress("", "", "", "").rpc();
    console.log("Your transaction signature", tx);
  });
});
