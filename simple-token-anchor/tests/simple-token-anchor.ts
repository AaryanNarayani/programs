import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { SimpleTokenAnchor } from "../target/types/simple_token_anchor";
import { BN } from "bn.js";
import { expect } from "chai";

describe("simple-token-anchor", () => {
  anchor.setProvider(anchor.AnchorProvider.env());
  const provider = anchor.getProvider() as anchor.AnchorProvider;
  const program = anchor.workspace
    .SimpleTokenAnchor as Program<SimpleTokenAnchor>;

  const mintKeypair = anchor.web3.Keypair.generate();
  const user1 = anchor.web3.Keypair.generate();
  const user2 = anchor.web3.Keypair.generate();
  const delegate = anchor.web3.Keypair.generate();

  let ata1Pda: anchor.web3.PublicKey;
  let ata2Pda: anchor.web3.PublicKey;

  // Fund all accounts before running tests
  before(async () => {
    // Request airdrops for all users
    await provider.connection.requestAirdrop(user1.publicKey, 2 * anchor.web3.LAMPORTS_PER_SOL);
    await provider.connection.requestAirdrop(user2.publicKey, 2 * anchor.web3.LAMPORTS_PER_SOL);
    await provider.connection.requestAirdrop(delegate.publicKey, 2 * anchor.web3.LAMPORTS_PER_SOL);
    
    // Wait for confirmations
    await new Promise(resolve => setTimeout(resolve, 3000));
  });

  it("Initialize Mint", async () => {
    const tx = await program.methods
      .initialize(8, new BN(1_000_000))
      .accounts({
        mintAccount: mintKeypair.publicKey,
        payer: user1.publicKey,
        owner: user1.publicKey,
      })
      .signers([mintKeypair, user1])
      .rpc();

    console.log("Mint initialized tx:", tx);

    const mintAccount = await program.account.mint.fetch(
      mintKeypair.publicKey
    );
    expect(mintAccount.decimals).to.equal(8);
    expect(Number(mintAccount.supply)).to.equal(1_000_000);
    expect(mintAccount.mintAuthority.toBase58()).to.equal(user1.publicKey.toBase58());
  });

  it("Create ATA for user1", async () => {
    const tx = await program.methods
      .createAta()
      .accounts({
        payer: user1.publicKey,
        mint: mintKeypair.publicKey,
      })
      .signers([user1])
      .rpc();

    // Derive the PDA after creation to get the address for future tests
    [ata1Pda] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from("ata"), user1.publicKey.toBuffer(), mintKeypair.publicKey.toBuffer()],
      program.programId
    );

    console.log("ATA1 created tx:", tx);

    const ata1 = await program.account.ata.fetch(ata1Pda);
    expect(ata1.owner.toBase58()).to.equal(user1.publicKey.toBase58());
    expect(ata1.mint.toBase58()).to.equal(mintKeypair.publicKey.toBase58());
    expect(Number(ata1.amount)).to.equal(0);
  });

  it("Create ATA for user2", async () => {
    const tx = await program.methods
      .createAta()
      .accounts({
        payer: user2.publicKey,
        mint: mintKeypair.publicKey,
      })
      .signers([user2])
      .rpc();

    // Derive the PDA after creation to get the address for future tests
    [ata2Pda] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from("ata"), user2.publicKey.toBuffer(), mintKeypair.publicKey.toBuffer()],
      program.programId
    );

    console.log("ATA2 created tx:", tx);

    const ata2 = await program.account.ata.fetch(ata2Pda);
    expect(ata2.owner.toBase58()).to.equal(user2.publicKey.toBase58());
    expect(ata2.mint.toBase58()).to.equal(mintKeypair.publicKey.toBase58());
    expect(Number(ata2.amount)).to.equal(0);
  });

  it("Mint tokens to user1 ATA", async () => {
    const tx = await program.methods
      .mint(new BN(500))
      .accounts({
        mintAccount: mintKeypair.publicKey,
        ataAccount: ata1Pda,
        owner: user1.publicKey,
      })
      .signers([user1])
      .rpc();

    console.log("Mint to ATA1 tx:", tx);

    const ata1 = await program.account.ata.fetch(ata1Pda);
    expect(Number(ata1.amount)).to.equal(500);
  });

  it("Normal transfer from user1 ATA to user2 ATA", async () => {
    const tx = await program.methods
      .transfer(new BN(200))
      .accounts({
        from: ata1Pda,
        to: ata2Pda,
        signer: user1.publicKey,
      })
      .signers([user1])
      .rpc();

    console.log("Normal transfer tx:", tx);

    const ata1 = await program.account.ata.fetch(ata1Pda);
    const ata2 = await program.account.ata.fetch(ata2Pda);
    expect(Number(ata1.amount)).to.equal(300);
    expect(Number(ata2.amount)).to.equal(200);
  });

  it("Set delegate for user1 ATA", async () => {
    const tx = await program.methods
      .delegation(new BN(100)) // allow delegate to spend 100
      .accountsPartial({
        delegate: delegate.publicKey,
        ataAccount: ata1Pda,
        owner: user1.publicKey,
      })
      .signers([user1])
      .rpc();

    console.log("Set delegate tx:", tx);

    const ata1 = await program.account.ata.fetch(ata1Pda);
    expect(ata1.delegate.toBase58()).to.equal(delegate.publicKey.toBase58());
    expect(Number(ata1.delegateAmount)).to.equal(100);
  });

  it("Delegate transfers from user1 ATA to user2 ATA", async () => {
    const tx = await program.methods
      .transfer(new BN(50))
      .accounts({
        from: ata1Pda,
        to: ata2Pda,
        signer: delegate.publicKey,
      })
      .signers([delegate])
      .rpc();

    console.log("Delegate transfer tx:", tx);

    const ata1 = await program.account.ata.fetch(ata1Pda);
    const ata2 = await program.account.ata.fetch(ata2Pda);
    expect(Number(ata1.amount)).to.equal(250);
    expect(Number(ata1.delegateAmount)).to.equal(50); // reduced by transfer
    expect(Number(ata2.amount)).to.equal(250);
  });

  it("Freeze mint", async () => {
    const tx = await program.methods
      .freeze()
      .accounts({
        mintAccount: mintKeypair.publicKey,
        owner: user1.publicKey,
      })
      .signers([user1])
      .rpc();

    console.log("Mint frozen tx:", tx);

    const mintAccount = await program.account.mint.fetch(mintKeypair.publicKey);
    expect(mintAccount.isFrozen).to.equal(true);
  });

  it("Thaw mint", async () => {
    const tx = await program.methods
      .thaw()
      .accounts({
        mintAccount: mintKeypair.publicKey,
        owner: user1.publicKey,
      })
      .signers([user1])
      .rpc();

    console.log("Mint thawed tx:", tx);

    const mintAccount = await program.account.mint.fetch(mintKeypair.publicKey);
    expect(mintAccount.isFrozen).to.equal(false);
  });
});