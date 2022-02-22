# xAnchor Deployment

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
