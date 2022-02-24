![Anchor_dark](https://user-images.githubusercontent.com/94420051/154133630-d7d07c32-7cb6-4bf3-a4df-3e597565bbf9.png)

# Anchor contest details
- $162,350 UST (TerraUSD) main award pot
- $7,650 UST (TerraUSD) gas optimization award pot
- Join [C4 Discord](https://discord.gg/code4rena) to register
- Submit findings [using the C4 form](https://code4rena.com/contests/2022-02-anchor-contest/submit)
- [Read our guidelines for more details](https://docs.code4rena.com/roles/wardens)
- Starts February 24, 2022 00:00 UTC
- Ends March 9, 2022 23:59 UTC

## What is Anchor?
Anchor is a decentralized savings protocol offering low-volatile yields on Terra stablecoin deposits. The Anchor rate is powered by a diversified stream of staking rewards from major proof-of-stake blockchains, and therefore can be expected to be much more stable than money market interest rates.  The Anchor community believes that a stable, reliable source of yield in Anchor has the opportunity to become the reference interest rate in crypto. 
The Anchor protocol defines a money market between a lender, looking to earn stable yields on their stablecoins, and a borrower, looking to borrow stablecoins on stakeable assets. To borrow stablecoins, the borrower locks up Bonded Assets (bAssets) as collateral, and borrows stablecoins below the protocol-defined borrowing ratio. The diversified stream of staking rewards accruing to the global pool of collateral then gets converted to stablecoin, and then conferred to the lender in the form of a stable yield. 

Deposited stablecoins are represented by Anchor Terra (aTerra). aTerra tokens are redeemable for the initial deposit along with accrued interest, allowing interest collection to be done just by holding on to them. Anchor is structured to provide depositors with:
*High, stable deposit yields powered by rewards of bAsset collaterals
*Instant withdrawals through pooled lending of stablecoin deposits
*Principal protection via liquidation of loans in risk of undercollateralization

Anchor is an open, permissionless savings protocol, meaning that any third-party application is free to connect and earn interest without restrictions. Through Anchor Earn, Anchor.js or EthAnchor, developers can interact with Anchor using just a few lines of code.
Further documentation of the Anchor Protocol is provided in the following pages.

For more info see https://docs.anchorprotocol.com/


[![codecov](https://codecov.io/gh/Anchor-Protocol/anchor-bAsset-contracts/branch/master/graph/badge.svg?token=GSAL9XEWNH)](https://codecov.io/gh/Anchor-Protocol/anchor-bAsset-contracts)

# Anchor bAsset Contracts

This monorepository contains the source code for the smart contracts implementing bAsset Protocol on the [Terra](https://terra.money) blockchain.

You can find information about the architecture, usage, and function of the smart contracts on the official Anchor documentation [site](https://anchorprotocol.com/).


## Contracts
| Contract                                            | Reference                                              | Description                                                                                                                        |
| --------------------------------------------------- | ------------------------------------------------------ | ---------------------------------------------------------------------------------------------------------------------------------- |
| [`anchor_basset_hub`](https://github.com/Anchor-Protocol/anchor-bAsset-contracts/tree/master/contracts/anchor_basset_hub)|[doc](https://docs.anchorprotocol.com/smart-contracts/bluna/hub-1)| Manages minted bLunas and bonded Lunas
| [`anchor_basset_reward`](https://github.com/Anchor-Protocol/anchor-bAsset-contracts/tree/master/contracts/anchor_basset_reward)|[doc](https://docs.anchorprotocol.com/smart-contracts/bluna/reward)|Manages the distribution of delegation rewards
| [`anchor_basset_token`](https://github.com/Anchor-Protocol/anchor-bAsset-contracts/tree/master/contracts/anchor_basset_token)| [doc](https://github.com/Anchor-Protocol/anchor-bAsset-contracts/tree/master/contracts/anchor_basset_token)|CW20 compliance 
| [`anchor_airdrop_registery`](https://github.com/Anchor-Protocol/anchor-bAsset-contracts/tree/master/contracts/anchor_airdrop_registry)| [doc](https://docs.anchorprotocol.com/smart-contracts/bluna/airdrop-registry)|Manages message fabricators for MIR and ANC airdrops
## Development

### Environment Setup

- Rust v1.44.1+
- `wasm32-unknown-unknown` target
- Docker

1. Install `rustup` via https://rustup.rs/

2. Run the following:

```sh
rustup default stable
rustup target add wasm32-unknown-unknown
```

3. Make sure [Docker](https://www.docker.com/) is installed

### Unit / Integration Tests

Each contract contains Rust unit tests embedded within the contract source directories. You can run:

```sh
cargo test unit-test
cargo test integration-test
```

### Compiling

After making sure tests pass, you can compile each contract with the following:

```sh
RUSTFLAGS='-C link-arg=-s' cargo wasm
cp ../../target/wasm32-unknown-unknown/release/cw1_subkeys.wasm .
ls -l cw1_subkeys.wasm
sha256sum cw1_subkeys.wasm
```

#### Production

For production builds, run the following:

```sh
docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/workspace-optimizer:0.11.5
```

This performs several optimizations which can significantly reduce the final size of the contract binaries, which will be available inside the `artifacts/` directory.


# Anchor bEth Contracts

This monorepository contains the source code for the smart contracts implementing bEth on the [Terra](https://terra.money) blockchain.

You can find information about the architecture, usage, and function of the smart contracts on the official Anchor documentation [site](https://anchorprotocol.com/).
## Development

### Environment Setup

- Rust v1.44.1+
- `wasm32-unknown-unknown` target
- Docker

1. Install `rustup` via https://rustup.rs/

2. Run the following:

```sh
rustup default stable
rustup target add wasm32-unknown-unknown
```

3. Make sure [Docker](https://www.docker.com/) is installed

### Unit / Integration Tests

Each contract contains Rust unit tests embedded within the contract source directories. You can run:

```sh
cargo test unit-test
cargo test integration-test
```

### Compiling

After making sure tests pass, you can compile each contract with the following:

```sh
RUSTFLAGS='-C link-arg=-s' cargo wasm
cp ../../target/wasm32-unknown-unknown/release/cw1_subkeys.wasm .
ls -l cw1_subkeys.wasm
sha256sum cw1_subkeys.wasm
```

#### Production

For production builds, run the following:

```sh
docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/workspace-optimizer:0.11.5
```

This performs several optimizations which can significantly reduce the final size of the contract binaries, which will be available inside the `artifacts/` directory.

## License

Copyright 2021 Anchor Protocol

Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except in compliance with the License. You may obtain a copy of the License at http://www.apache.org/licenses/LICENSE-2.0. Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.

See the License for the specific language governing permissions and limitations under the License.

[![codecov](https://codecov.io/gh/Anchor-Protocol/anchor-token-contracts/branch/main/graph/badge.svg?token=NK4H00P3KH)](https://codecov.io/gh/Anchor-Protocol/anchor-token-contracts)

# Anchor Token (ANC) Contracts
This monorepository contains the source code for the Money Market smart contracts implementing Anchor Protocol on the [Terra](https://terra.money) blockchain.

You can find information about the architecture, usage, and function of the smart contracts on the official Anchor documentation [site](https://docs.anchorprotocol.com/smart-contracts/anchor-token).

### Dependencies

Anchor Token depends on [Terraswap](https://terraswap.io) and uses its [implementation](https://github.com/terraswap/terraswap) of the CW20 token specification.


## Contracts

| Contract                                 | Reference                                                                                         | Description                                                                    |
| ---------------------------------------- | ------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------ |
| [`airdrop`](./contracts/airdrop)         | [doc](https://docs.anchorprotocol.com/smart-contracts/anchor-token/airdrop)   | Holds ANC tokens which are to be used Luna staker incentives                   |
| [`collector`](./contracts/collector)     | [doc](https://docs.anchorprotocol.com/smart-contracts/anchor-token/collector) | Accumulates protocol fees, converts them to ANC and distributes to ANC stakers |
| [`community`](../contracts/community)    | [doc](https://docs.anchorprotocol.com/smart-contracts/anchor-token/community) | Manages ANC community grants                                                   |
| [`distributor`](./contracts/distributor) | [doc](https://docs.anchorprotocol.com/smart-contracts/anchor-token/dripper)   | Holds ANC tokens which are to be used as borrower incentives                   |
| [`gov`](./contracts/gov)                 | [doc](https://docs.anchorprotocol.com/smart-contracts/anchor-token/gov)       | Handles Anchor Governance and reward distribution to ANC stakers               |
| [`staking`](./contracts/staking)         | [doc](https://docs.anchorprotocol.com/smart-contracts/anchor-token/staking)   | Handles ANC-UST pair LP token staking                                          |
| [`vesting`](./contracts/vesting)         | [doc](https://docs.anchorprotocol.com/smart-contracts/anchor-token/vesting)   | Holds ANC tokens which are to be used ANC token allocation vesting             |

## Development

### Environment Setup

- Rust v1.44.1+
- `wasm32-unknown-unknown` target
- Docker

1. Install `rustup` via https://rustup.rs/

2. Run the following:

```sh
rustup default stable
rustup target add wasm32-unknown-unknown
```

3. Make sure [Docker](https://www.docker.com/) is installed

### Unit / Integration Tests

Each contract contains Rust unit and integration tests embedded within the contract source directories. You can run:

```sh
cargo unit-test
cargo integration-test
```

### Compiling

After making sure tests pass, you can compile each contract with the following:

```sh
RUSTFLAGS='-C link-arg=-s' cargo wasm
cp ../../target/wasm32-unknown-unknown/release/cw1_subkeys.wasm .
ls -l cw1_subkeys.wasm
sha256sum cw1_subkeys.wasm
```

#### Production

For production builds, run the following:

```sh
docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/workspace-optimizer:0.11.5
```

This performs several optimizations which can significantly reduce the final size of the contract binaries, which will be available inside the `artifacts/` directory.


[![codecov](https://codecov.io/gh/Anchor-Protocol/money-market-contracts/branch/main/graph/badge.svg?token=B4B2YUSXEU)](https://codecov.io/gh/Anchor-Protocol/money-market-contracts)

# Anchor Money Market Contracts
A Rust and [CosmWasm](https://cosmwasm.com/) implementation of the Anchor Protocol money market on the [Terra blockchain](https://terra.money).

You can find information about the architecture, usage, and function of the smart contracts in the [documentation](https://docs.anchorprotocol.com/).

### Dependencies

Money Market has dependencies on [Anchor Token Contracts](https://github.com/anchor-protocol/anchor-token-contracts) and [bAsset Contracts](https://github.com/Anchor-Protocol/anchor-bAsset-contracts).

## Contracts

| Contract                                               | Reference                                                                                  | Description                                                                   |
| ------------------------------------------------------ | ------------------------------------------------------------------------------------------ | ----------------------------------------------------------------------------- |
| [`overseer`](./contracts/overseer)                     | [doc](https://docs.anchorprotocol.com/smart-contracts/money-market/overseer)               | Manages money market overalls, stores borrower information                    |
| [`market`](../contracts/market)                        | [doc](https://docs.anchorprotocol.com/smart-contracts/money-market/market)                 | Handles Terra stablecoin deposits and borrows, ANC distribution to borrowers  |
| [`custody_bluna`](./contracts/custody_bluna)           | [doc](https://docs.anchorprotocol.com/smart-contracts/money-market/custody-bluna-specific) | Handles bLuna collateral deposits and withdrawals                             |
| [`custody_beth`](./contracts/custody_beth)             | [doc](https://docs.anchorprotocol.com/smart-contracts/money-market/custody-beth)           | Handles bEth collateral deposits and withdrawals                              |
| [`interest_model`](./contracts/interest_model)         | [doc](https://docs.anchorprotocol.com/smart-contracts/money-market/interest-model)         | Calculates the current borrow interest rate based on the market situation     |
| [`distribution_model`](./contracts/distribution_model) | [doc](https://docs.anchorprotocol.com/smart-contracts/money-market/distribution-model)     | Calculates the borrower ANC emission rate based on the previous emission rate |
| [`oracle`](./contracts/oracle)                         | [doc](https://docs.anchorprotocol.com/smart-contracts/money-market/oracle)                 | Provides a price feed for bAsset collaterals                                  |
| [`liquidation`](./contracts/liquidation)               | [doc](https://docs.anchorprotocol.com/smart-contracts/liquidations)                        | OTC exchange contract for bAsset collateral liquidations                      |

## Development

### Environment Setup

- Rust v1.44.1+
- `wasm32-unknown-unknown` target
- Docker

1. Install `rustup` via https://rustup.rs/

2. Run the following:

```sh
rustup default stable
rustup target add wasm32-unknown-unknown
```

3. Make sure [Docker](https://www.docker.com/) is installed.

### Unit / Integration Tests

Each contract contains Rust unit and integration tests embedded within the contract source directories. You can run:

```sh
cargo unit-test
cargo integration-test
```

### Compiling

After making sure tests pass, you can compile each contract with the following:

```sh
RUSTFLAGS='-C link-arg=-s' cargo wasm
cp ../../target/wasm32-unknown-unknown/release/cw1_subkeys.wasm .
ls -l cw1_subkeys.wasm
sha256sum cw1_subkeys.wasm
```

#### Production

For production builds, run the following:

```sh
docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/workspace-optimizer:0.11.5
```

This performs several optimizations which can significantly reduce the final size of the contract binaries, which will be available inside the `artifacts/` directory.

## License

This repository is licensed under the Apache 2.0 license. See [LICENSE](./LICENSE) for full disclosure.

© 2021 Anchor Protocol.

# Cross-chain Contract (Work in progress, won't change during the contest though):

## Smart contract architecture:

#### AddressProxy:

Terra contract deployed to represent some cross-chain address
Interfaces with anchor contracts directly

#### Core:

Takes all the user facing messages that anchor takes, except they also include information about the cross-chain sender
Proxies messages to the appropriate AddressProxy

#### Wormhole Bridge Endpoints:

Receives data and wrapped tokens from various bridges
Parses data into messages readable by Core, and forwards---

## Compiling contracts:

```shell
npm run compile_eth
npm run compile_terra
```

## Deploying contracts:

`env.json` contains all relevant information for a testnet deployment. To deploy xAnchor to testnet:

```shell
npm run deploy
```

This will also rewrite the contract addresses in `env.json` with the most recently deployed set

## Testing:

```
npm run test
```

This will run tests against the deployed contracts in `env.json`

## Migration

Deploying everything takes a while. Since both of ETH and Terra side contracts are upgradable, we can update them in place to speed up our iteration time.

```
npm run migrate_terra
npm run migrate_eth
```

## OP Code Specification

Opcodes are specified as 8-bit unsigned integers. We split the opcode into two parts:

- The first 6 least significant bits are the opcode itself
- The last 2 bits are the flag bits

We have the following flags defined:

- `0b10`: `FLAG_INCOMING_TRANSFER`
- `0b01`: `FLAG_OUTGOING_TRANSFER`
- `0b11`: `FLAG_BOTH_TRANSFERS`
- `0b00`: `FLAG_NO_ASSC_TRANSFERS`

We have the following opcodes:
| Opcode | Flags | Full Opcode | Decimal |
|:-------|:------|:------------|:-------|
| Deposit Stable | `FLAG_BOTH_TRANSFERS` | `0b110000` | `192` |
| Redeem Stable | `FLAG_BOTH_TRANSFERS` | `0b110001` | `193` |
| Repay Stable | `FLAG_INCOMING_TRANSFER` | `0b1000000` | `64` |
| Lock Collateral | `FLAG_INCOMING_TRANSFER` | `0b1000001` | `65` |
| Unlock Collateral | `FLAG_OUTGOING_TRANSFER` | `0b0100000` | `32` |
| Borrow Stable | `FlAG_OUTGOING_TRANSFER` | `0b0100001` | `33` |
| Claim Rewards | `FLAG_OUTGOING_TRANSFER` | `0b0100010` | `34` |
