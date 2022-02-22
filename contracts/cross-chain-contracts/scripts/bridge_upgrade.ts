import {ethers, upgrades} from "hardhat";


async function main() {
  const [deployer] = await ethers.getSigners();
  console.log(`Deploying contract with the account: ${deployer.address}`);

  const balance = await deployer.getBalance();
  console.log(`Account Balance: ${balance}`);

  /**
   * ENTER OR EDIT PROXY ADDRESS
   */
  const PROXY_ADDRESS = "0xC987e573f08833637cC4657eE9158357cc640B28"

  const BridgeContract = await ethers.getContractFactory("CrossAnchorBridge");
  const bridgeContract = await upgrades.upgradeProxy(PROXY_ADDRESS, BridgeContract, { unsafeAllow: ['delegatecall'] });
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
