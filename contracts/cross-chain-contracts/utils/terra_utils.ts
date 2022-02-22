import {
  Coin,
  getCodeId,
  getContractAddress,
  isTxError,
  LCDClient,
  MnemonicKey,
  MsgExecuteContract,
  MsgInstantiateContract,
  MsgMigrateContract,
  MsgStoreCode,
  TxInfo,
} from "@terra-money/terra.js";
import {
  CHAIN_ID_ETHEREUM_ROPSTEN, CHAIN_ID_TERRA,
  getEmitterAddressEth,
} from "@certusone/wormhole-sdk";
import {CONSTANTS} from "../constants";
import * as fs from "fs";
import * as path from "path";

export const terra = new LCDClient({
  URL: CONSTANTS.terra_lcd_url,
  chainID: CONSTANTS.terra_chain_id,
});

export const terraWallet = terra.wallet(
  new MnemonicKey({
    mnemonic: CONSTANTS.terra_mnemonic
  })
);

export async function signAndBroadcast(msgs) {
  await new Promise((resolve) => setTimeout(resolve, 3000));

  const tx = await terraWallet.createAndSignTx({
    msgs: msgs,
    gas: "10000000",
    // gasPrices: [new Coin("uusd", "0.2")]
  });

  const txResult = await terra.tx.broadcast(tx);
  if (isTxError(txResult)) {
    console.log(txResult);
    throw new Error(txResult.raw_log);
  }
  let txInfo = txResult as any;
  txInfo.tx = tx;
  return txInfo as TxInfo;
}

export async function executeContract(contractAddress, execute_msg, coins?) {
  let msg = new MsgExecuteContract(
    terraWallet.key.accAddress,
    contractAddress,
    execute_msg,
    coins
  );
  return await signAndBroadcast([msg]);
}

export async function registerETHBridge() {
  let ethAddress = Uint8Array.from(
    Buffer.from(getEmitterAddressEth(CONSTANTS.eth_xanchor_bridge), "hex")
  );

  return await executeContract(CONSTANTS.terra_xanchor_bridge, {
    register_wormhole_chain_info: {
      chain_id: CHAIN_ID_ETHEREUM_ROPSTEN,
      address: Array.from(ethAddress),
    },
  });
}

export async function processAnchorMessage(arbitraryInfo, tokenTransfer) {
  let msg = tokenTransfer != undefined ? new MsgExecuteContract(
    terraWallet.key.accAddress,
    CONSTANTS.terra_xanchor_bridge,
    {
      process_anchor_message: {
        instruction_vaa: Buffer.from(arbitraryInfo).toString("base64"),
        option_token_transfer_vaa:
          Buffer.from(tokenTransfer).toString("base64"),
      },
    }
  ) : new MsgExecuteContract(
    terraWallet.key.accAddress,
    CONSTANTS.terra_xanchor_bridge,
    {
      process_anchor_message: {
        instruction_vaa: Buffer.from(arbitraryInfo).toString("base64")
      },
    }
  );
  console.log("Trying to sign and broadcast");
  return await signAndBroadcast([msg]);
}

export async function storeCode(contractPath) {
  console.log("== storeCode START. contractWasm:", contractPath);
  let msg = new MsgStoreCode(
    terraWallet.key.accAddress,
    fs.readFileSync(contractPath).toString("base64")
  );

  const storeCodeTxResult = await signAndBroadcast([msg]);
  const codeId = parseInt(getCodeId(storeCodeTxResult));

  console.log(
    "== storeCode DONE. contractWasm:",
    contractPath,
    "codeId:",
    codeId
  );
  return codeId;
}

export async function instantiateContact(codeId, initMsg) {
  let msg = new MsgInstantiateContract(
    terraWallet.key.accAddress,
    terraWallet.key.accAddress,
    codeId,
    initMsg
  );
  return getContractAddress(await signAndBroadcast([msg]));
}

export async function storeTerraCodes() {
  let p = "terra/artifacts";
  let codeIds = {};
  for (let name of ["addressproxy", "wormhole_bridge", "crossanchor_core"]) {
    codeIds[name] = await storeCode(path.join(p, name + ".wasm"));
  }
  return codeIds;
}

export async function queryReturningSequence (incomingSequence) {
  console.log(CONSTANTS.terra_xanchor_bridge);
  let sequence_info = await terra.wasm.contractQuery(
    CONSTANTS.terra_xanchor_bridge,
    {
      "sequence_info": {
        chain_id: CHAIN_ID_ETHEREUM_ROPSTEN,
        sequence: parseInt(incomingSequence)
      }
    }
  )
  console.log(sequence_info);
  // @ts-ignore
  return parseInt(sequence_info.outgoing_sequence)
}

export async function migrateContract(contractAddress, newCode) {
  console.log("== Migrating. contract:", contractAddress, "newCode:", newCode);
  let msg = new MsgMigrateContract(
    terraWallet.key.accAddress,
    contractAddress,
    newCode,
    {}
  );
  return await signAndBroadcast([msg]);
}

export async function migrateWithCodePath(contractAddress, contractPath) {
  let codeId = await storeCode(contractPath);
  return await migrateContract(contractAddress, codeId);
}

export async function deployTerra() {
  let codeIds = await storeTerraCodes();

  let coreAddress = await instantiateContact(codeIds["crossanchor_core"], {
    address_proxy_code_id: codeIds["addressproxy"],
    owner: terraWallet.key.accAddress,
    // testnet anchor overseer; can hardcode
    overseer_address: "terra1qljxd0y3j3gk97025qvl3lgq8ygup4gsksvaxv",
    anc_gov_address: "terra1qljxd0y3j3gk97025qvl3lgq8ygup4gsksvaxv"
  });
  console.log("Core Address", coreAddress);
  let anchorBridge = await instantiateContact(codeIds["wormhole_bridge"], {
    wormhole_core_bridge_addr: CONSTANTS.terra_wormhole,
    wormhole_token_bridge_addr: CONSTANTS.terra_token_bridge,
    cross_anchor_core_addr: coreAddress,
    aust_cw20_addr: CONSTANTS.terra_aust,
    owner: terraWallet.key.accAddress,
  });
  console.log("Anchor bridge", anchorBridge);

  // register anchor bridge to core
  await executeContract(coreAddress, {
    add_bridges: {
      bridges: [anchorBridge],
    },
  });

  CONSTANTS.terra_xanchor_bridge = anchorBridge
}