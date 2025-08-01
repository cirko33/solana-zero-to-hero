import * as anchor from "@coral-xyz/anchor";
import { assert } from "chai";
import { Program, web3 } from "@coral-xyz/anchor";
import { Database } from "../target/types/database";
import { Buffer } from "buffer";

describe("Test", () => {
  // setup
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const program = anchor.workspace.Database as Program<Database>;

  const owner = provider.wallet;

  const createItemPda = (id: anchor.BN, user: anchor.web3.PublicKey) => {
    return anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("items"), user.toBuffer(), id.toArrayLike(Buffer, "le", 8)],
      program.programId
    );
  };

  it("set more items successfully", async () => {
    await provider.connection.requestAirdrop(
      owner.publicKey,
      5 * web3.LAMPORTS_PER_SOL
    );

    const times = 5;
    for (let i = 0; i < times; i++) {
      const id = new anchor.BN(i);
      const [pda] = createItemPda(id, owner.publicKey);
      console.log(pda.toBase58());

      const tx = await program.methods
        .setItem(id, "cirko" + i, "cirko" + i)
        .accounts({
          owner: owner.publicKey,
          item: pda,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .rpc();

      console.log("SIGNATURE", tx);
    }

    for (let i = 0; i < times; i++) {
      const id = new anchor.BN(i);
      const [pda] = createItemPda(id, owner.publicKey);

      const itemAccount = await program.account.item.fetch(pda);
      console.log("Stored item:", itemAccount);

      assert.ok(itemAccount.id.eq(id));
      assert.equal(itemAccount.username, "cirko" + i);
      assert.equal(itemAccount.secret, "cirko" + i);
    }
  });
});
