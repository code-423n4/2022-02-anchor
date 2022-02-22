import {Coin, MsgExecuteContract} from "@terra-money/terra.js";
import { signAndBroadcast, terraWallet } from "./terra_utils";
import {ethers, upgrades} from "hardhat";

import {
  CHAIN_ID_ETHEREUM_ROPSTEN,
  CHAIN_ID_TERRA,
  getEmitterAddressEth,
  getEmitterAddressTerra,
  getSignedVAA,
  parseSequenceFromLogTerra,
  redeemOnEth,
} from "@certusone/wormhole-sdk";
import {CONSTANTS} from "../constants";
import { NodeHttpTransport } from "@improbable-eng/grpc-web-node-http-transport";

export async function getSignedVAAWithRetry(
  emitterChain,
  emitterAddress,
  sequence,
  log?: boolean
) {
  if (log) process.stdout.write(`Fetching VAA...`);
  while (true) {
    try {
      const { vaaBytes } = await getSignedVAA(
        CONSTANTS.wormhole_rpc_host,
        emitterChain,
        emitterAddress,
        sequence,
        {
          transport: NodeHttpTransport(),
        }
      );
      if (vaaBytes !== undefined) {
        if (log) process.stdout.write(`✅\n`);
        return vaaBytes;
      }
    } catch (e) {}
    if (log) process.stdout.write(".");
    await new Promise((resolve) => setTimeout(resolve, 1000));
  }
}

export async function bridgeAsset(asset) {
  console.log("Bridging asset");
  const [deployer] = await ethers.getSigners();
  let msgs: any[] = [];

  if (asset.info.native_token) {
    let coins = [new Coin(asset.info.native_token.denom, asset.amount)]
    msgs.push(
      new MsgExecuteContract(
        terraWallet.key.accAddress,
        CONSTANTS.terra_token_bridge,
        { deposit_tokens: {} },
        coins
        )
    )
  } else {
    msgs.push(
      new MsgExecuteContract(
        terraWallet.key.accAddress,
        asset.info.token.contract_addr,
        {increase_allowance: {amount: asset.amount, spender: CONSTANTS.terra_token_bridge}}
      )
    )
  }

  msgs.push(
    new MsgExecuteContract(
      terraWallet.key.accAddress,
      CONSTANTS.terra_token_bridge,
      {
        initiate_transfer: {
          asset: asset,
          recipient_chain: CHAIN_ID_ETHEREUM_ROPSTEN,
          recipient: Buffer.from(
            getEmitterAddressEth(deployer.address),
            "hex"
          ).toString("base64"),
          fee: "0",
          nonce: 0,
        },
      },
      { uusd: "1000000" }
    )
  )

  console.log("Broadcasting tx to terra");
  let resp = await signAndBroadcast(msgs);
  let seq = parseSequenceFromLogTerra(resp);

  console.log("Getting VAA");
  let vaaBytes = await getSignedVAAWithRetry(
    CHAIN_ID_TERRA,
    await getEmitterAddressTerra(CONSTANTS.terra_token_bridge),
    seq
  );
  console.log("Redeeming on ETH");

  console.log(await redeemOnEth(CONSTANTS.eth_token_bridge, deployer, vaaBytes));
}

const HEADER_LEN = 6;
const SIG_LEN = 66;
const PAYLOAD_POS = 51;

export function parseVAA(vaa: Uint8Array): Uint8Array {
  let buffer = Buffer.from(vaa);
  let numSigs = buffer.readUInt8(5);
  let bodyOffset = HEADER_LEN + numSigs * SIG_LEN;
  let payload = vaa.slice(bodyOffset + PAYLOAD_POS);
  return payload;
}

const SEQ_POS = 34;
export function parseSequenceFromPayload(payload: Uint8Array): Number {
  let buffer = Buffer.from(payload);
  let sequence = buffer.readBigUInt64BE(SEQ_POS);
  return Number(sequence);
}
