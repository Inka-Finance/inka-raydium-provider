import {
    Connection,
    LAMPORTS_PER_SOL,
    PublicKey,
    Signer,
    Keypair,
    Account,
    Transaction,
    SystemProgram,
    clusterApiUrl,
    TransactionInstruction,
    TransactionSignature
} from "@solana/web3.js";
const lo = require('buffer-layout');

import { initializeAccount } from '@project-serum/serum/lib/token-instructions';
import { ASSOCIATED_TOKEN_PROGRAM_ID, TOKEN_PROGRAM_ID } from "./ids";
import { ACCOUNT_LAYOUT } from "./layouts";
import { Token } from "@solana/spl-token";
import { TOKENS } from "./tokens";
export async function createTokenAccountIfNotExist(
    connection: Connection,
    account: string | undefined | null,
    owner: PublicKey,
    mintAddress: string,
    lamports: number | null,
  
    transaction: Transaction,
    signer: Array<Account>
  ) {
    let publicKey
  
    if (account) {
      publicKey = new PublicKey(account)
    } else {
      publicKey = await createProgramAccountIfNotExist(
        connection,
        account,
        owner,
        TOKEN_PROGRAM_ID,
        lamports,
        ACCOUNT_LAYOUT,
        transaction,
        signer
      )
  
      transaction.add(
        initializeAccount({
          account: publicKey,
          mint: new PublicKey(mintAddress),
          owner
        })
      )
    }
  
    return publicKey
  }

  export async function createProgramAccountIfNotExist(
    connection: Connection,
    account: string | undefined | null,
    owner: PublicKey,
    programId: PublicKey,
    lamports: number | null,
    layout: any,
  
    transaction: Transaction,
    signer: Array<Account>
  ) {
    let publicKey
  
    if (account) {
      publicKey = new PublicKey(account)
    } else {
      const newAccount = new Account()
      publicKey = newAccount.publicKey
  
      transaction.add(
        SystemProgram.createAccount({
          fromPubkey: owner,
          newAccountPubkey: publicKey,
          lamports: lamports ?? (await connection.getMinimumBalanceForRentExemption(layout.span)),
          space: layout.span,
          programId
        })
      )
  
      signer.push(newAccount)
    }
  
    return publicKey
  }


  export async function createAssociatedTokenAccountIfNotExist(
    account: string | undefined | null,
    owner: PublicKey,
    mintAddress: string,
  
    transaction: Transaction,
    atas: string[] = []
  ) {
    let publicKey
    if (account) {
      publicKey = new PublicKey(account)
    }
  
    const mint = new PublicKey(mintAddress)
    // @ts-ignore without ts ignore, yarn build will failed
    const ata = await Token.getAssociatedTokenAddress(ASSOCIATED_TOKEN_PROGRAM_ID, TOKEN_PROGRAM_ID, mint, owner, true)
  
    if (
      (!publicKey || !ata.equals(publicKey)) &&
      mintAddress !== TOKENS.WSOL.mintAddress &&
      !atas.includes(ata.toBase58())
    ) {
      transaction.add(
        Token.createAssociatedTokenAccountInstruction(
          ASSOCIATED_TOKEN_PROGRAM_ID,
          TOKEN_PROGRAM_ID,
          mint,
          ata,
          owner,
          owner
        )
      )
      atas.push(ata.toBase58())
    }
  
    return ata
  }
  export async function findProgramAddress(seeds: Array<Buffer | Uint8Array>, programId: PublicKey) {
    const [publicKey, nonce] = await PublicKey.findProgramAddress(seeds, programId)
    return { publicKey, nonce }
  }
  export async function findAssociatedTokenAddress(walletAddress: PublicKey, tokenMintAddress: PublicKey) {
    const { publicKey } = await findProgramAddress(
      [walletAddress.toBuffer(), TOKEN_PROGRAM_ID.toBuffer(), tokenMintAddress.toBuffer()],
      ASSOCIATED_TOKEN_PROGRAM_ID
    )
    return publicKey
  }
  

  export function swapInstruction(
    programId: PublicKey,
    ammProgramId: PublicKey,
    // tokenProgramId: PublicKey,
    // amm
    ammId: PublicKey,
    ammAuthority: PublicKey,
    ammOpenOrders: PublicKey,
    ammTargetOrders: PublicKey,
    poolCoinTokenAccount: PublicKey,
    poolPcTokenAccount: PublicKey,
    // serum
    serumProgramId: PublicKey,
    serumMarket: PublicKey,
    serumBids: PublicKey,
    serumAsks: PublicKey,
    serumEventQueue: PublicKey,
    serumCoinVaultAccount: PublicKey,
    serumPcVaultAccount: PublicKey,
    serumVaultSigner: PublicKey,
    // user
    userSourceTokenAccount: PublicKey,
    userDestTokenAccount: PublicKey,
    userOwner: PublicKey,
  
    amountIn: number,
    minAmountOut: number
  ): TransactionInstruction {
    const dataLayout = lo.struct([lo.u8('instruction'), lo.nu64('amountIn'), lo.nu64('minAmountOut')])
  
    const keys = [
      // spl token
      { pubkey: TOKEN_PROGRAM_ID, isSigner: false, isWritable: false },
      { pubkey: ammProgramId, isSigner: false, isWritable: false },
      // amm
      { pubkey: ammId, isSigner: false, isWritable: true },
      { pubkey: ammAuthority, isSigner: false, isWritable: false },
      { pubkey: ammOpenOrders, isSigner: false, isWritable: true },
      { pubkey: ammTargetOrders, isSigner: false, isWritable: true },
      { pubkey: poolCoinTokenAccount, isSigner: false, isWritable: true },
      { pubkey: poolPcTokenAccount, isSigner: false, isWritable: true },
      // serum
      { pubkey: serumProgramId, isSigner: false, isWritable: false },
      { pubkey: serumMarket, isSigner: false, isWritable: true },
      { pubkey: serumBids, isSigner: false, isWritable: true },
      { pubkey: serumAsks, isSigner: false, isWritable: true },
      { pubkey: serumEventQueue, isSigner: false, isWritable: true },
      { pubkey: serumCoinVaultAccount, isSigner: false, isWritable: true },
      { pubkey: serumPcVaultAccount, isSigner: false, isWritable: true },
      { pubkey: serumVaultSigner, isSigner: false, isWritable: false },
      { pubkey: userSourceTokenAccount, isSigner: false, isWritable: true },
      { pubkey: userDestTokenAccount, isSigner: false, isWritable: true },
      { pubkey: userOwner, isSigner: true, isWritable: false }
    ]
  
    const data = Buffer.alloc(dataLayout.span)
    dataLayout.encode(
      {
        instruction: 9,
        amountIn,
        minAmountOut
      },
      data
    )
  
    return new TransactionInstruction({
      keys,
      programId,
      data
    })
  }

  export async function sendTransaction(
    connection: Connection,
    wallet: any,
    transaction: Transaction,
    signers: Array<Account> = []
  ) {
    const txid: TransactionSignature = await wallet.sendTransaction(transaction, connection, {
      signers,
      skipPreflight: true,
      preflightCommitment: 'confirmed'
    })
  
    return txid
  }