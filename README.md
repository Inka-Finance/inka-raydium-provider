<p align="center">
  <a href="http://inka.finance/" target="blank"><img src="./2.svg" width="200" alt="Inka Logo" /></a>
</p>
<p align="center">The most friendly DeFi wallet and aggregator.</p>

## Table of contents

- [Description](#description)
- [How it works](#how-it-works)
- [Built With](#built-with)
- [How to run](#how-to-run)
- [Future Updates](#future-updates)
- [Author](#author)
- [Support](#support)

## Description

Raydium is an automated market maker (AMM) built on the Solana blockchain which leverages the central order book of the Serum decentralized exchange (DEX) to enable lightning-fast trades, shared liquidity and new features for earning yield. Inka wallet integrates with the Raydium service through smart contracts in the Rust language, which allows you to have easy access to swap and add liquidity to the service.

<p>A smart contract for using the Raydium service takes a commission that is charged to the Inka wallet.</p>

## How it works

<p>Inka raydium provider program is a special layer for integration with the Raydium service. This layer provides easier access to perform operations on the service.</p>

## Built With

- Rust
- Cargo
- Solana CLI

## How to run

```
$ cargo build-bpf

$ solana deploy <path> // path to program binary
```

## Future Updates

- [x] Swap tokens
- [x] Add liquidity

## Author

- [Profile](https://github.com/Inka-Finance "Inka Finance Development Team")
- [Email](mailto:a.zhaxybayev@inka.finance?subject=Hi "Hi!")
- [WebSite](https://inka.finance/ "Welcome")

## 🤝 Support

Contributions, issues, and feature requests are welcome!

Give a ⭐️ if you like this project!
