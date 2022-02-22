import {ethers, upgrades} from "hardhat";
import {CONSTANTS} from "../constants";
import {getEmitterAddressTerra, hexToUint8Array} from "@certusone/wormhole-sdk";
import { Interface } from "ethers/lib/utils.js";

export async function deployEth() {
  const [deployer] = await ethers.getSigners();
  console.log(`Deploying contract with the account: ${deployer.address}`);

  const balance = await deployer.getBalance();
  console.log(`Account Balance: ${balance}`);

  const BridgeContract = await ethers.getContractFactory("CrossAnchorBridge");
  const bridgeContract = await upgrades.deployProxy(BridgeContract, [
    1,
    CONSTANTS.eth_ust,
    CONSTANTS.eth_aust,
    [],
    CONSTANTS.eth_wormhole,
    CONSTANTS.eth_token_bridge,
    hexToUint8Array(await getEmitterAddressTerra(CONSTANTS.terra_xanchor_bridge))
  ], { unsafeAllow: ['delegatecall'] });
  await bridgeContract.deployed();
  console.log(`Ethereum Bridge Address: ${bridgeContract.address}`);
  // approve usage of wUST
  ethers.getContractAt(
    "IERC20", CONSTANTS.eth_ust
  )
  let ethUst = await ethers.getContractAt(
    "IERC20", CONSTANTS.eth_ust
  );

  // approve usage of wUST
  let approveTx = await ethUst.approve(bridgeContract.address, 1000000000);
  await approveTx.wait();

  // approve usage of wAUST
  let ethAust = await ethers.getContractAt(
    "IERC20", CONSTANTS.eth_aust
  );
  approveTx = await ethAust.approve(bridgeContract.address, 1000000000);
  await approveTx.wait();

  CONSTANTS.eth_xanchor_bridge = bridgeContract.address;
}

export function parseSequencesFromEthLogs(receipt) {
  let abi = [
    "event LogMessagePublished(address indexed sender, uint64 sequence, uint32 nonce, bytes payload, uint8 consistencyLevel)",
  ];
  const bridgeLogs = receipt.logs.filter((l) => {
    return l.address === CONSTANTS.eth_wormhole;
  });
  let iface = new Interface(abi);
  let res : any[] = [];
  for (const log of bridgeLogs) {
    res.push(iface.parseLog(log).args.sequence.toString());
  }
  return res;
}