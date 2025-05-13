import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { InvoiceFinancing } from "../target/types/invoice_financing";
import { assert } from "chai";
import { TOKEN_2022_PROGRAM_ID, TOKEN_PROGRAM_ID } from "@solana/spl-token"
import { randomBytes } from "crypto"
describe("invoice-financing2", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env()
  anchor.setProvider(provider);
  const ourWallet = provider.wallet.payer // Admin wallet
  const program = anchor.workspace.invoiceFinancing as Program<InvoiceFinancing>;
  const invoiceId = new anchor.BN(randomBytes(8))
  const [invoice] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("invoice"), invoiceId.toArrayLike(Buffer, "le", 8)],
    program.programId
  )
  const bidder1 = anchor.web3.Keypair.generate() // Investor 1
  const bidder2 = anchor.web3.Keypair.generate() // Investor 2 (this is the auction winner)

  const METADATA_SEED = "metadata"
  const TOKEN_METADATA_PROGRAM_ID = new anchor.web3.PublicKey("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s")
  it("Is initialized!", async () => {
    const tx1 = await provider.connection.requestAirdrop(bidder1.publicKey, anchor.web3.LAMPORTS_PER_SOL * 5)
    const tx2 = await provider.connection.requestAirdrop(bidder2.publicKey, anchor.web3.LAMPORTS_PER_SOL * 5)
    await provider.connection.confirmTransaction(tx1)
    await provider.connection.confirmTransaction(tx2)
  });
  it("can create a auction", async () => {
    const faceValue = new anchor.BN(anchor.web3.LAMPORTS_PER_SOL)
    const tx = await program.methods.createInvoiceListing(invoiceId, faceValue)
      .accountsPartial({
        signer: ourWallet.publicKey,
        invoice
      })
      .signers([ourWallet])
      .rpc()
    console.log("Creates an invoice listing: ", tx)
  })
  it("can bid on the auction (this will be called by the investor)", async () => {
    const [invoice] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("invoice"), invoiceId.toArrayLike(Buffer, "le", 8)],
      program.programId
    )
    const amount = new anchor.BN(anchor.web3.LAMPORTS_PER_SOL * 0.5)
    const data = await program.account.invoiceListing.fetch(invoice)
    const tx = await program.methods.placeBid(amount)
      .accountsPartial({
        bidder: bidder1.publicKey,
        invoice,
        previousBidder: data.currentBidder
      })
      .signers([bidder1])
      .rpc()
    console.log("Bid 1 on the auction: ", tx)
  })
  it("can another bid on the auction and previous bidder gets refund (this will be called by the investor)", async () => {
    const amount = new anchor.BN(anchor.web3.LAMPORTS_PER_SOL * 0.6)
    const data = await program.account.invoiceListing.fetch(invoice)
    const tx = await program.methods.placeBid(amount)
      .accountsPartial({
        bidder: bidder2.publicKey,
        invoice,
        previousBidder: data.currentBidder
      })
      .signers([bidder2])
      .rpc()
    console.log("Bid 2 on the auction: ", tx)
  })
  it("can resolve on the auction (admin calls this)", async () => {

    const data = await program.account.invoiceListing.fetch(invoice)
    const [mint] = anchor.web3.PublicKey.findProgramAddressSync(
      [
        Buffer.from("mint"),
        ourWallet.publicKey.toBuffer(),
        invoiceId.toArrayLike(Buffer, "le", 8)
      ],
      program.programId
    )
    const [metadataAddress] = anchor.web3.PublicKey.findProgramAddressSync(
      [
        Buffer.from(METADATA_SEED),
        TOKEN_METADATA_PROGRAM_ID.toBuffer(),
        mint.toBuffer(),
      ],
      TOKEN_METADATA_PROGRAM_ID
    );
    console.log(mint.toBase58(), metadataAddress.toBase58(), invoice.toBase58())
    const tx = await program.methods.resolveAuction()

      .accountsPartial({
        signer: ourWallet.publicKey,
        invoice,
        mint,
        investor: data.currentBidder,
        tokenProgram: TOKEN_PROGRAM_ID,
        nftMetadata: metadataAddress,
        tokenMetadataProgram: TOKEN_METADATA_PROGRAM_ID
      })
      .signers([ourWallet])
      .rpc()
    console.log("Resolved the auction and got funds: ", tx)
  })
  it("can repay for the invoice", async () => {
    const bidderVault = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("bidderVault"), invoice.toBuffer()],
      program.programId
    )[0]
    const data = await program.account.invoiceListing.fetch(invoice)
    const tx = await provider.connection.requestAirdrop(bidderVault, data.faceValue.toNumber())
    await provider.connection.confirmTransaction(tx)
    console.log("Bidder vault repaid")
  })
  it("can settle all payments and close the invoice listing (this is called by winner of auction)", async () => {

    let tx = await program.methods.settleInvoicePayment()
      .accountsPartial({
        mainWallet: ourWallet.publicKey, // This is the admin wallet/our wallet
        investor: bidder2.publicKey, // The winner has to sign the transaction
        invoice,
        tokenProgram: TOKEN_PROGRAM_ID
      })
      .signers([bidder2])
      .rpc()
    console.log("Settled payment and winner got money", tx)
  })
  it("can fetch all the invoices available", async () => {
    let data = await program.account.invoiceListing.all()
    console.log(data)
  })
  it("can fetch only specific invoice listing", async () => {
    let data = await program.account.invoiceListing.fetch(invoice)
    console.log(data)
  })
});
