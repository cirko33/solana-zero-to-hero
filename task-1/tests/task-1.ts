import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { web3 } from "@coral-xyz/anchor";
import { Task1 } from "../target/types/task_1";
import { assert } from "chai";

describe("task-1", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const sp = web3.SystemProgram.programId;
  const program = anchor.workspace.Task1 as Program<Task1>;

  const proposer = web3.Keypair.generate();
  const accepter = web3.Keypair.generate();

  const swapAddr = web3.PublicKey.findProgramAddressSync([
    Buffer.from("swap"),
    proposer.publicKey.toBuffer(),
    accepter.publicKey.toBuffer()
  ], program.programId)[0]

  const createTreasuryPDA = (account: web3.PublicKey) => {
    return web3.PublicKey.findProgramAddressSync([
      Buffer.from("treasury"),
      account.toBuffer(),
    ], program.programId)[0]
  }

  const [proposerAmount, accepterAmount] = [new anchor.BN(1000), new anchor.BN(2000)]

  it("happy path", async () => {
    const tx1 = await provider.connection.requestAirdrop(proposer.publicKey, 5 * web3.LAMPORTS_PER_SOL);
    await provider.connection.confirmTransaction(tx1);

    const tx2 = await provider.connection.requestAirdrop(accepter.publicKey, 5 * web3.LAMPORTS_PER_SOL);
    await provider.connection.confirmTransaction(tx2);

    await program.methods
      .proposeSwap(
        proposerAmount,
        accepterAmount
      ).accounts({
        swap: swapAddr,
        proposer: proposer.publicKey,
        accepter: accepter.publicKey,
      }).signers([proposer]).rpc()

    await program.methods
      .acceptSwap()
      .accounts({
        swap: swapAddr,
        accepter: accepter.publicKey,
      })
      .signers([accepter])
      .rpc()

    await program.methods
      .addToTreasury(new anchor.BN(1000))
      .accounts({
        treasury: createTreasuryPDA(proposer.publicKey),
        signer: proposer.publicKey,
      })
      .signers([proposer])
      .rpc()

    await program.methods
      .addToTreasury(new anchor.BN(2000))
      .accounts({
        treasury: createTreasuryPDA(accepter.publicKey),
        signer: accepter.publicKey,
      })
      .signers([accepter])
      .rpc()

    await program.methods.executeSwap().accounts({
      swap: swapAddr,
      proposerTreasury: createTreasuryPDA(proposer.publicKey),
      accepterTreasury: createTreasuryPDA(accepter.publicKey),
    }).rpc()

    const swaps = await program.account.swap.all()
    console.log(swaps)
    assert.equal(swaps.length, 1)

    const thatSwap = swaps[0].account;
    assert.isTrue(thatSwap.accepted)
    assert.isTrue(thatSwap.executed)
  })
});
