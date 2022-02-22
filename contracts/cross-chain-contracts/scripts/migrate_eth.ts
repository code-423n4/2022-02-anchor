import {CONSTANTS} from "../constants";
import { ethers, upgrades } from "hardhat";


async function main() {
  const [deployer] = await ethers.getSigners();
  console.log(`Deploying contract with the account: ${deployer.address}`);

  const balance = await deployer.getBalance();
  console.log(`Account Balance: ${balance}`);

  /**
   * ENTER OR EDIT PROXY ADDRESS
   */

  const BridgeContract = await ethers.getContractFactory("CrossAnchorBridge");
  const bridgeContract = await upgrades.upgradeProxy(CONSTANTS.eth_xanchor_bridge, BridgeContract, { unsafeAllow: ['delegatecall'] });
  await bridgeContract.deployed();
  console.log(`Contract Address: ${bridgeContract.address}`);
}

// We recommend this pattern to be able to use async/await everywhere
// and properly handle errors.
main()
  .then(() => process.exit(0))
  .catch((error) => {
    console.error(error);
    process.exit(1);
  });
