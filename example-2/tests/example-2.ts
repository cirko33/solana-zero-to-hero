import * as anchor from "@coral-xyz/anchor";
import { web3 } from "@coral-xyz/anchor";
import { assert } from "chai";
import { MultisigWallet } from "../target/types/multisig_wallet";

describe("TEST", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const program = anchor.workspace
    .MultisigWallet as anchor.Program<MultisigWallet>;
  const sp = web3.SystemProgram.programId;

  const owner = provider.wallet;

  const signersKP = [
    web3.Keypair.generate(),
    web3.Keypair.generate(),
    web3.Keypair.generate(),
  ];

  let signers = [
    signersKP[0].publicKey,
    signersKP[1].publicKey,
    signersKP[2].publicKey,
  ];

  const to = web3.Keypair.generate().publicKey;


  signers.forEach(async (signer) => {
    await provider.connection.requestAirdrop(signer, web3.LAMPORTS_PER_SOL * 5);
    const bal = await provider.connection.getBalance(signer);
    assert.equal(bal, web3.LAMPORTS_PER_SOL * 5);
  });

  const quorum = 2;

  const createWalletPda = (payer: web3.PublicKey) => {
    return web3.PublicKey.findProgramAddressSync(
      [Buffer.from("wallet"), payer.toBytes()],
      program.programId
    )[0];
  };

  const createTransactionPda = (
    wallet: web3.PublicKey,
    proposer: web3.PublicKey
  ) => {
    return web3.PublicKey.findProgramAddressSync(
      [Buffer.from("transaction"), wallet.toBytes(), proposer.toBytes()],
      program.programId
    )[0];
  };

  it("happy path", async () => {
    // needs to have rent 
    await provider.connection.requestAirdrop(to, web3.LAMPORTS_PER_SOL * 5);

    await provider.connection.requestAirdrop(
      owner.publicKey,
      web3.LAMPORTS_PER_SOL * 10
    );
    await program.provider.sendAndConfirm(
      (() => {
        const tx = new web3.Transaction();
        tx.add(
          web3.SystemProgram.transfer({
            fromPubkey: owner.publicKey,
            toPubkey: createWalletPda(owner.publicKey),
            lamports: web3.LAMPORTS_PER_SOL * 5,
          })
        );
        return tx;
      })(),
      []
    );
    const bal = await provider.connection.getBalance(
      createWalletPda(owner.publicKey)
    );
    assert.equal(bal, web3.LAMPORTS_PER_SOL * 5);

    const walletAddr = createWalletPda(owner.publicKey);
    const txAddr = createTransactionPda(walletAddr, owner.publicKey);

    await program.methods
      .initializeWallet(signers, quorum)
      .accounts({
        wallet: createWalletPda(owner.publicKey),
        payer: owner.publicKey,
        systemProgram: sp,
      })
      .rpc();

    await program.methods
      .proposeTransaction(to, new anchor.BN(1000))
      .accounts({
        wallet: walletAddr,
        transaction: txAddr,
        proposer: owner.publicKey,
        systemProgram: sp,
      })
      .rpc();

    await program.methods
      .approveTransaction()
      .accounts({
        wallet: walletAddr,
        transaction: txAddr,
        signer: signers[0],
      })
      .signers([signersKP[0]])
      .rpc();

    await program.methods
      .approveTransaction()
      .accounts({
        wallet: walletAddr,
        transaction: txAddr,
        signer: signers[1],
      })
      .signers([signersKP[1]])
      .rpc();

    const resp = await program.methods
      .executeTransaction()
      .accounts({
        wallet: walletAddr,
        transaction: txAddr,
        signer: signers[0],
        to: to,
      })
      .signers([signersKP[0]])
      .rpc();

    const balance = await provider.connection.getBalance(to);
    assert.equal(balance, 5*web3.LAMPORTS_PER_SOL+1000);
  });
});
