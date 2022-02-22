import { ethers } from "hardhat";
import chai, { expect } from "chai";
import {
  CHAIN_ID_ETHEREUM_ROPSTEN,
  CHAIN_ID_TERRA,
  getEmitterAddressEth,
  getEmitterAddressTerra,
  getSignedVAA,
} from "@certusone/wormhole-sdk";
import { NodeHttpTransport } from "@improbable-eng/grpc-web-node-http-transport";

import { parseSequencesFromEthLogs } from "../utils/eth_utils";
import {processAnchorMessage, queryReturningSequence, signAndBroadcast, terraWallet} from "../utils/terra_utils";
import {
  bridgeAsset,
  getSignedVAAWithRetry,
  parseSequenceFromPayload,
  parseVAA,
} from "../utils/wormhole_utils";
import {CONSTANTS, saveConstants} from "../constants";
import {Coin, MsgExecuteContract} from "@terra-money/terra.js";

// These tests use the environment variables set in the .env file.
// Theoretically, this means that the tests can be run on mainnet...
// We should probably explicitly define a .env.test file for local development and testing.

// Tests the LockCollateral method on the Ethereum Cross Anchor Bridge contract.
// In general, this test takes around 2-4 minutes to run.
describe("Integration / Lock Collateral", () => {
  let globalArbitraryInfoSeq = 0;
  it('Bridge Collateral (bLUNA)', async function() {
    this.timeout(60000000)
    // bluna hub -- mint bLUNA
    let msgs = [
      new MsgExecuteContract(
        terraWallet.key.accAddress,
        CONSTANTS.terra_bluna_hub,
        {"bond": {}},
      [new Coin("uluna", "10000000")])
    ]
    await signAndBroadcast(msgs)
    await bridgeAsset(
      {"info": {"token": {"contract_addr": CONSTANTS.terra_bluna}}, "amount": "9000000"}
    )
  })

  it('Approve Collateral (bLuna)', async function() {
    this.timeout(60000000)
    // approve usage of wbLuna
    let ethbLuna = await ethers.getContractAt(
      "IERC20", CONSTANTS.eth_bluna
    );
    let approveTx = await ethbLuna.approve(CONSTANTS.eth_xanchor_bridge, 900000000);
    await approveTx.wait();
  })

  it('Ethereum -> Terra', async function() {
    this.timeout(60000000)
    // run bridge contract
    const BridgeContract = await ethers.getContractFactory("CrossAnchorBridge");

    let ethCrossAnchorBridge = await BridgeContract.attach(
      CONSTANTS.eth_xanchor_bridge
    )
    console.log("Lock Collateral (bLuna)");
    let bridgeTransaction = await ethCrossAnchorBridge.lockCollateral(
      CONSTANTS.eth_bluna,
      50000
    );

    let bridge = await bridgeTransaction.wait();

    //Fetch sequences from the deposit receipt.
    let [tokenTransferSeq, arbitraryInfoSeq] = //[685, 45];
      parseSequencesFromEthLogs(bridge);

    globalArbitraryInfoSeq = arbitraryInfoSeq;
    let ethTokenBridgeEmitterAddress = getEmitterAddressEth(
      CONSTANTS.eth_token_bridge
    );
    let ethCrossAnchorCoreEmitterAddress = getEmitterAddressEth(
      CONSTANTS.eth_xanchor_bridge
    );
    //Fetch the VAAs for the transfer
    console.log("Attempting to retrieve VAA")
    console.log("Sequences", tokenTransferSeq, arbitraryInfoSeq);
    let arbitraryInfoVAA = await getSignedVAAWithRetry(
      CHAIN_ID_ETHEREUM_ROPSTEN,
      ethCrossAnchorCoreEmitterAddress,
      arbitraryInfoSeq
    );
    let tokenTransferVAA = await getSignedVAAWithRetry(
      CHAIN_ID_ETHEREUM_ROPSTEN,
      ethTokenBridgeEmitterAddress,
      tokenTransferSeq
    );
    //Redeem the VAA for the wormhole transfer on the Terra side.
    console.log(`Terra Chain ID: ${CHAIN_ID_TERRA}`);
    await processAnchorMessage(arbitraryInfoVAA, tokenTransferVAA);
    console.log("Outgoing sequence: ", await queryReturningSequence(arbitraryInfoSeq));
  });
});
