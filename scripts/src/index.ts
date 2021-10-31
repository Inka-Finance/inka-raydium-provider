import {
    Connection,
    LAMPORTS_PER_SOL,
    PublicKey,
    Signer,
    Keypair,
    Account,
    clusterApiUrl,
    Transaction,
    AccountInfo, ParsedAccountData, sendAndConfirmTransaction
} from "@solana/web3.js";
import { Token, TOKEN_PROGRAM_ID } from "@solana/spl-token";
import * as bip39 from 'bip39'
import * as bip32 from 'bip32'
import * as nacl from "tweetnacl";
import { Wallet } from "solray"
import * as bs58 from "bs58";
import { createAssociatedTokenAccountIfNotExist, createTokenAccountIfNotExist, findAssociatedTokenAddress, sendTransaction, swapInstruction } from "./utils";
import { getBigNumber } from "./layouts";
import { TokenAmount } from "./safe-math";
import { LP_TOKENS, NATIVE_SOL, TOKENS } from "./tokens";
import { closeAccount } from "@project-serum/serum/lib/token-instructions";
import { LIQUIDITY_POOL_PROGRAM_ID_V4, SERUM_PROGRAM_ID_V3 } from "./ids";

// const mnemonic = "country move puzzle control coin thing poem fog hole seminar below harsh";
// const mnemonic = "venue lyrics van core modify until edit clump gate coil organ drift off quality economy require buffalo festival pair enroll give science stage neck"
const mnemonic = "load ability torch enable omit subject mass uniform thrive denial transfer famous"
// 3U6KdnUGASp4bHezHv8br7gfDf8F69hTp3c6FhfMsGfg3QXvdKcRzByuXoUkmeymj39tgTT8UC4KreEdL9W51eyS
const poolInfo = {
  name: 'SOL-USDC',
  coin: { ...NATIVE_SOL },
  pc: { ...TOKENS.USDC },
  lp: { ...LP_TOKENS['SOL-USDC-V4'] },

  version: 4,
  programId: LIQUIDITY_POOL_PROGRAM_ID_V4,

  ammId: '58oQChx4yWmvKdwLLZzBi4ChoCc2fqCUWBkwMihLYQo2',
  ammAuthority: '5Q544fKrFoe6tsEbD7S8EmxGTJYAKtTVhAW5Q5pge4j1',
  ammOpenOrders: 'HRk9CMrpq7Jn9sh7mzxE8CChHG8dneX9p475QKz4Fsfc',
  ammTargetOrders: 'CZza3Ej4Mc58MnxWA385itCC9jCo3L1D7zc3LKy1bZMR',
  // no need
  ammQuantities: NATIVE_SOL.mintAddress,
  poolCoinTokenAccount: 'DQyrAcCrDXQ7NeoqGgDCZwBvWDcYmFCjSb9JtteuvPpz',
  poolPcTokenAccount: 'HLmqeL62xR1QoZ1HKKbXRrdN1p3phKpxRMb2VVopvBBz',
  poolWithdrawQueue: 'G7xeGGLevkRwB5f44QNgQtrPKBdMfkT6ZZwpS9xcC97n',
  poolTempLpTokenAccount: 'Awpt6N7ZYPBa4vG4BQNFhFxDj4sxExAA9rpBAoBw2uok',
  serumProgramId: SERUM_PROGRAM_ID_V3,
  serumMarket: '9wFFyRfZBsuAha4YcuxcXLKwMxJR43S7fPfQLusDBzvT',
  serumBids: '14ivtgssEBoBjuZJtSAPKYgpUK7DmnSwuPMqJoVTSgKJ',
  serumAsks: 'CEQdAFKdycHugujQg9k2wbmxjcpdYZyVLfV9WerTnafJ',
  serumEventQueue: '5KKsLVU6TcbVDK4BS6K1DGDxnh4Q9xjYJ8XaDCG5t8ht',
  serumCoinVaultAccount: '36c6YqAwyGKQG66XEp2dJc5JqjaBNv7sVghEtJv4c7u6',
  serumPcVaultAccount: '8CFo8bL8mZQK8abbFyypFMwEDd8tVJjHTTojMLgQTUSZ',
  serumVaultSigner: 'F8Vyqk3unwxkXukZFQeYyGmFfTG3CAX4v24iyrjEYBJV',
  official: true
}
async function run() {
  const connection = new Connection(clusterApiUrl('mainnet-beta'))
  const transaction = new Transaction()
  const signers: Account[] = []
  let wrappedSolAccount: PublicKey | null = null
  const seed = bip39.mnemonicToSeedSync(mnemonic); // prefer async mnemonicToSeed
  const keyPair = nacl.sign.keyPair.fromSeed(seed.slice(0, 32));
  const account = new Account(keyPair.secretKey);
  const owner = account.publicKey
  const amountIn = new TokenAmount(0.0001, 9, false)
  const amountOut = new TokenAmount(0.016185, 6, false)
  wrappedSolAccount = await createTokenAccountIfNotExist(
    connection,
    wrappedSolAccount,
    owner,
    TOKENS.WSOL.mintAddress,
    getBigNumber(1000000) + 1e7,
    transaction,
    signers
  )

  const parsedTokenAccounts = await connection.getParsedTokenAccountsByOwner(owner, {programId: TOKEN_PROGRAM_ID}, 'confirmed')
  const tokenAccounts: any = {}
  const auxiliaryTokenAccounts: Array<{ pubkey: PublicKey; account: AccountInfo<ParsedAccountData> }> = []

  for (const tokenAccountInfo of parsedTokenAccounts.value) {
    const tokenAccountPubkey = tokenAccountInfo.pubkey
    const tokenAccountAddress = tokenAccountPubkey.toBase58()
    const parsedInfo = tokenAccountInfo.account.data.parsed.info
    const mintAddress = parsedInfo.mint
    const balance = new TokenAmount(parsedInfo.tokenAmount.amount, parsedInfo.tokenAmount.decimals)

    const ata = await findAssociatedTokenAddress(account.publicKey, new PublicKey(mintAddress))

    if (ata.equals(tokenAccountPubkey)) {
      tokenAccounts[mintAddress] = {
        tokenAccountAddress,
        balance
      }
    } else if (parsedInfo.tokenAmount.uiAmount > 0) {
      auxiliaryTokenAccounts.push(tokenAccountInfo)
    }
  }

  const solBalance = await connection.getBalance(account.publicKey, 'confirmed')
  tokenAccounts[NATIVE_SOL.mintAddress] = {
    tokenAccountAddress: account.publicKey.toBase58(),
    balance: new TokenAmount(solBalance, NATIVE_SOL.decimals)
  }
  const newFromTokenAccount = await createAssociatedTokenAccountIfNotExist(
    tokenAccounts[NATIVE_SOL.mintAddress].tokenAccountAddress,
    owner,
    NATIVE_SOL.mintAddress,
    transaction
  )
  const newToTokenAccount = await createAssociatedTokenAccountIfNotExist(
    tokenAccounts[TOKENS.USDC.mintAddress] ? tokenAccounts[TOKENS.USDC.mintAddress].tokenAccountAddress : null, 
    owner, 
    TOKENS.USDC.mintAddress, 
    transaction
  )
  
  transaction.add(
    swapInstruction(
      new PublicKey("22G6174cvQTxmgVxgZ88AAAFiDMTtHm9DgM6WX6dZd2k"),
      new PublicKey(poolInfo.programId),
      new PublicKey(poolInfo.ammId),
      new PublicKey(poolInfo.ammAuthority),
      new PublicKey(poolInfo.ammOpenOrders),
      new PublicKey(poolInfo.ammTargetOrders),
      new PublicKey(poolInfo.poolCoinTokenAccount),
      new PublicKey(poolInfo.poolPcTokenAccount),
      new PublicKey(poolInfo.serumProgramId),
      new PublicKey(poolInfo.serumMarket),
      new PublicKey(poolInfo.serumBids),
      new PublicKey(poolInfo.serumAsks),
      new PublicKey(poolInfo.serumEventQueue),
      new PublicKey(poolInfo.serumCoinVaultAccount),
      new PublicKey(poolInfo.serumPcVaultAccount),
      new PublicKey(poolInfo.serumVaultSigner),
      wrappedSolAccount ?? newFromTokenAccount,
      newToTokenAccount,
      owner,
      Math.floor(getBigNumber(amountIn.toWei())),
      Math.floor(getBigNumber(amountOut.toWei()))
    )
  )

  transaction.add(
    closeAccount({
      source: wrappedSolAccount,
      destination: owner,
      owner
    })
  )
  console.log(transaction)
  const tx = await sendAndConfirmTransaction(connection, transaction, signers, {
    skipPreflight: true,
    preflightCommitment: 'confirmed'
  })
  // await connection.sendTransaction(transaction, signers, {
  //   skipPreflight: true,
  //   preflightCommitment: 'confirmed'
  // })
  console.log(tx)
}
run()


