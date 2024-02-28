import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { WeeklyRaffleAtoz } from "../target/types/weekly_raffle_atoz";
import { Connection, PublicKey } from "@solana/web3.js";
import fs from "fs";
import NodeWallet from "@coral-xyz/anchor/dist/cjs/nodewallet";
import { BN } from "bn.js";
import dayjs from "dayjs";
import {
  TOKEN_PROGRAM_ID,
  getOrCreateAssociatedTokenAccount,
} from "@solana/spl-token";
describe("weekly_raffle_atoz", () => {
  const keypairPath = "/home/safouene/.config/solana/id.json";
  const keypairBuffer = fs.readFileSync(keypairPath);
  const payerPath = "./tests/payer-wallet.json";
  const payerBuffer = fs.readFileSync(payerPath);
  const buyerWallet = anchor.web3.Keypair.fromSecretKey(
    Uint8Array.from(JSON.parse(keypairBuffer.toString()))
  );
  const payerWallet = anchor.web3.Keypair.fromSecretKey(
    Uint8Array.from(JSON.parse(keypairBuffer.toString()))
  );
  // Configure the client to use the local cluster.
  const connection = new Connection(
    "https://little-practical-wildflower.solana-devnet.discover.quiknode.pro/b33c1731ef24950ad5a92445bc1133e16e4271ed/"
  );
  const provider = new anchor.AnchorProvider(
    connection,
    new NodeWallet(payerWallet),
    { commitment: "confirmed" }
  );
  anchor.setProvider(provider);
  const prizeMint = new PublicKey(
    "Gh9ZwEmdLJ8DscKNTkTqPbNwLNNBjuSzaG9Vp2KGtKJr"
  );

  const program = anchor.workspace
    .WeeklyRaffleAtoz as Program<WeeklyRaffleAtoz>;

  // it("Is initialized!", async () => {
  //   try {
  //     console.log("Payer", payerWallet.publicKey.toBase58());
  //     const startDate = dayjs(Date.now()).unix().toString();
  //     const endDate = dayjs(Date.now()).add(7, "days").unix();
  //     let [raffleAdr] = anchor.web3.PublicKey.findProgramAddressSync(
  //       [
  //         Buffer.from(anchor.utils.bytes.utf8.encode("atoz")),
  //         payerWallet.publicKey.toBuffer(),
  //         Buffer.from(
  //           anchor.utils.bytes.utf8.encode(`Weekly_Mint_${startDate}`)
  //         ),
  //       ],
  //       program.programId
  //     );
  //     const payerAta = await getOrCreateAssociatedTokenAccount(
  //       connection,
  //       payerWallet,
  //       prizeMint,
  //       payerWallet.publicKey
  //     );
  //     const [prizeTokenAdr, nonce] = PublicKey.findProgramAddressSync(
  //       [
  //         raffleAdr.toBuffer(),
  //         Buffer.from(anchor.utils.bytes.utf8.encode("proceeds")),
  //       ],
  //       program.programId
  //     );

  //     const tx = await program.methods
  //       .initialize(
  //         new BN(dayjs(Date.now()).unix()),
  //         new BN(dayjs(Date.now()).add(7, "days").unix()),
  //         new BN(100 * Math.pow(10, 6)),
  //         new BN(1),

  //         `Weekly_Mint_${startDate}`,
  //         prizeMint,
  //         [
  //           {
  //             winnerPlace: new BN(1),
  //             winnerPrizeAmount: new BN(1500),
  //           },
  //           {
  //             winnerPlace: new BN(2),
  //             winnerPrizeAmount: new BN(1000),
  //           },
  //           {
  //             winnerPlace: new BN(2),
  //             winnerPrizeAmount: new BN(500),
  //           },
  //         ]
  //       )
  //       .accounts({
  //         raffleAccount: raffleAdr,
  //         signer: payerWallet.publicKey,
  //         prizeTokenAccount: prizeTokenAdr,
  //         prizeMint: prizeMint,
  //         signerTokenAccount: payerAta.address,
  //         tokenProgram: TOKEN_PROGRAM_ID,
  //       })
  //       .signers([payerWallet])
  //       .rpc();
  //     console.log("Your transaction signature", tx);
  //   } catch (error) {
  //     console.log("Error", error);
  //   }
  // });
  // it("can buy ticket", async () => {
  //   try {
  //     const raffleList = await program.account.raffle.all();
  //     console.log("Raffle ", raffleList);
  //     const [prizeTokenAdr, nonce] = PublicKey.findProgramAddressSync(
  //       [
  //         raffleList[0].publicKey.toBuffer(),
  //         Buffer.from(anchor.utils.bytes.utf8.encode("proceeds")),
  //       ],
  //       program.programId
  //     );
  //     const ticketNumber = 3;
  //     const [ticket] = PublicKey.findProgramAddressSync(
  //       [
  //         Buffer.from(anchor.utils.bytes.utf8.encode("ticket_atoz")),
  //         raffleList[0].publicKey.toBuffer(),
  //         Buffer.from(anchor.utils.bytes.utf8.encode(ticketNumber.toString())),
  //       ],
  //       program.programId
  //     );
  //     const payerAta = await getOrCreateAssociatedTokenAccount(
  //       connection,
  //       payerWallet,
  //       prizeMint,
  //       payerWallet.publicKey
  //     );
  //     const tx = await program.methods
  //       .buyTicket(new BN(3), new BN(1 * Math.pow(10, 6)))
  //       .accounts({
  //         ticket: ticket,
  //         prizeTokenAccount: prizeTokenAdr,
  //         raffleAccount: raffleList[0].publicKey,
  //         signerTokenAccount: payerAta.address,
  //         signer: payerWallet.publicKey,
  //         tokenProgram: TOKEN_PROGRAM_ID,
  //       })
  //       .signers([payerWallet])
  //       .rpc();
  //     console.log("Your transaction signature", tx);
  //   } catch (error) {
  //     console.log("error", error);
  //   }
  // });
  it("Can Assign winners", async () => {
    try {
      const raffleList = await program.account.raffle.all();
      for (const raffle of raffleList) {
        console.log("Raffle", raffle.account.winners);
        const tx = await program.methods
          .pickWinner(raffle.account.name)
          .accounts({
            raffleAccount: raffle.publicKey,
            prizeMint: prizeMint,
            creator: new PublicKey(
              "5rt8BP7oPV1fm5ty7QbaKg6cL9FgDRsYXJCroXWGex4h"
            ),
            signer: raffle.publicKey,
          })
          .rpc();
        console.log("Your transaction signature", tx);
      }
    } catch (error) {
      console.log("Error", error);
    }
  });
  // it("Can Close All Accounts", async () => {
  //   const raffleList = await program.account.raffle.all();
  //   raffleList.forEach(async (account) => {
  //     const sig = await program.methods
  //       .closeRaffle()
  //       .accounts({
  //         account: account.publicKey,
  //         destination: payerWallet.publicKey,
  //       })
  //       .signers([payerWallet])
  //       .rpc();
  //     console.log("Sig", sig);
  //   });
  // });
});
