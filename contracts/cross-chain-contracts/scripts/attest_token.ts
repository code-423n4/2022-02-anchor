import { ethers } from "hardhat";
import { CONSTANTS } from "../constants";
import { LCDClient, MnemonicKey } from "@terra-money/terra.js";
import { signAndBroadcast } from "../utils/terra_utils";
import {
    CHAIN_ID_TERRA,
    getEmitterAddressEth,
    getEmitterAddressTerra,
    getForeignAssetEth,
    hexToUint8Array,
    createWrappedOnEth,
    attestFromTerra
} from "@certusone/wormhole-sdk";
import {
    getSignedVAAWithRetry
  } from "../utils/wormhole_utils";

async function main() {

    // bluna address
    let terraAddress = "terra1u0t35drzyy0mujj8rkdyzhe264uls4ug3wdp3x";

    let provider = new ethers.providers.JsonRpcProvider(CONSTANTS.eth_node_url);
    provider.getGasPrice().then((gasPrice) => {
        // gasPrice is a BigNumber; convert it to a decimal string
        let gasPriceString = gasPrice.toString();
        console.log("Current gas price: " + gasPriceString);
    });

    let emitterAddressTerra = await getEmitterAddressTerra(terraAddress);
    console.log("Emitter address: " + emitterAddressTerra);

    const ethAddress = await getForeignAssetEth(CONSTANTS.eth_token_bridge, provider, CHAIN_ID_TERRA, hexToUint8Array(emitterAddressTerra));

    console.log("Trying to lookup address (if exists, if not will initiate attestation process):", ethAddress);

    if (!ethAddress || ethAddress === "0x0000000000000000000000000000000000000000") {
        console.log("Starting provisioning");
        let mnemonic = CONSTANTS.eth_mnemonic;

        const signer = ethers.Wallet.fromMnemonic(mnemonic).connect(provider);
        console.log("ETH Signer: " + signer);

        let ethTokenBridgeEmitterAddress = getEmitterAddressEth(
            CONSTANTS.eth_token_bridge
        );


        let terraTokenBridgeEmitterAddress = await getEmitterAddressTerra(
            CONSTANTS.terra_token_bridge
        );

        console.log("[log] ethTokenBridgeEmitterAddress :" + ethTokenBridgeEmitterAddress);

        const terraLCD = new LCDClient({
            URL: CONSTANTS.terra_lcd_url,
            chainID: CONSTANTS.terra_chain_id
        });

        const terra_wallet = terraLCD.wallet(
            new MnemonicKey({
                mnemonic: CONSTANTS.terra_mnemonic,
            })
        );

        console.log("[1/2] Executing Attestation from Terra");
        const transaction = await attestFromTerra(
            CONSTANTS.terra_token_bridge,
            terra_wallet.key.accAddress,
            terraAddress
        );


        console.log("Broadcasting tx to terra");
        let resp = await signAndBroadcast([transaction]);

        // @ts-ignore
        let sequence = resp.logs[0].eventsByType["wasm"]["message.sequence"][0]

        console.log("Sequence: ", sequence);

        const signedVAA = await getSignedVAAWithRetry(
            CHAIN_ID_TERRA,
            terraTokenBridgeEmitterAddress,
            sequence
        );

        console.log("Attempting to retrieve VAA: " + signedVAA)

        console.log("[2/2] Executing createWrappedOnEth");
        await createWrappedOnEth(CONSTANTS.eth_token_bridge, signer, signedVAA);
        const ethAddress = await getForeignAssetEth(CONSTANTS.eth_token_bridge, provider, CHAIN_ID_TERRA, hexToUint8Array(emitterAddressTerra));
        console.log("Remote Address ", ethAddress)
    }
}

// We recommend this pattern to be able to use async/await everywhere
// and properly handle errors.
main()
    .then(() => process.exit(0))
    .catch((error) => {
        console.error(error);
        process.exit(1);
    });
