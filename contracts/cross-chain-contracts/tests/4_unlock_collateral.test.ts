import { ethers } from "hardhat";
import chai, { expect } from "chai";
import {
  CHAIN_ID_ETHEREUM_ROPSTEN,
  CHAIN_ID_TERRA,
  getEmitterAddressEth,
  getEmitterAddressTerra,
  getSignedVAA,
  hexToUint8Array
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

// Tests the UnlockCollateral method on the Ethereum Cross Anchor Bridge contract.
// In general, this test takes around 2-4 minutes to run.
describe("Integration / Unlock Collateral", () => {
  let globalArbitraryInfoSeq = 0;
  it('Ethereum -> Terra ', async function() {
    this.timeout(60000000)

    const BridgeContract = await ethers.getContractFactory("CrossAnchorBridge");

    let ethCrossAnchorBridge = await BridgeContract.attach(
      CONSTANTS.eth_xanchor_bridge
    )
    console.log("Unlock Collateral (bLuna)");
    let bridgeTransaction = await ethCrossAnchorBridge.unlockCollateral(
      hexToUint8Array(await getEmitterAddressTerra(CONSTANTS.terra_bluna)),
      5000
    );

    let bridgeReceipt = await bridgeTransaction.wait();

    // Fetch sequences from the deposit receipt.
    let [arbitraryInfoSeq] = //[685, 45];
      parseSequencesFromEthLogs(bridgeReceipt);

    globalArbitraryInfoSeq = arbitraryInfoSeq;
    let ethCrossAnchorCoreEmitterAddress = getEmitterAddressEth(
      CONSTANTS.eth_xanchor_bridge
    );
    // Fetch the VAAs for the transfer
    console.log("Attempting to retrieve VAA")
    console.log("Sequences", arbitraryInfoSeq);
    let arbitraryInfoVAA = await getSignedVAAWithRetry(
      CHAIN_ID_ETHEREUM_ROPSTEN,
      ethCrossAnchorCoreEmitterAddress,
      arbitraryInfoSeq
    );
    // Redeem the VAA for the wormhole transfer on the Terra side.
    console.log(`Terra Chain ID: ${CHAIN_ID_TERRA}`);
    await processAnchorMessage(arbitraryInfoVAA, undefined);
    console.log("Outgoing sequence: ", await queryReturningSequence(arbitraryInfoSeq));
  });

  it('Terra -> Ethereum', async function()  {
    this.timeout(60000000)
    const BridgeContract = await ethers.getContractFactory("CrossAnchorBridge");

    let ethCrossAnchorBridge = await BridgeContract.attach(
      CONSTANTS.eth_xanchor_bridge
    )

    let returningInfoSeq = await queryReturningSequence(globalArbitraryInfoSeq);

    let returningInfoVAA: Uint8Array;
    while (true) {
      try {
        const returningInfoVAA  = await getSignedVAAWithRetry(
          CHAIN_ID_TERRA,
          await getEmitterAddressTerra(CONSTANTS.terra_xanchor_bridge),
          returningInfoSeq.toString(),
        );
        if (returningInfoVAA == undefined) {
          continue;
        }
        let parsedPayload = parseVAA(returningInfoVAA);
        let returningTokenTransferSeq = parseSequenceFromPayload(parsedPayload);

        const returningTokenTransferVAA = await getSignedVAAWithRetry(
          CHAIN_ID_TERRA,
          await getEmitterAddressTerra(CONSTANTS.terra_token_bridge),
          Number(returningTokenTransferSeq)
        );
        try {
          await ethCrossAnchorBridge.processTokenTransferInstruction(
            returningInfoVAA,
            returningTokenTransferVAA
          );
          break;
        } catch (e) {
          if (
            e.error?.message ===
            "execution reverted: transfer info already processed"
          ) {
            returningInfoSeq++;
          } else {
            throw e;
          }
        }
      } catch (e) {
        returningInfoSeq++;
        if (e.message === "requested VAA not found in store") {
          throw e;
        }
      }
    }
  });
});


