import "@nomiclabs/hardhat-waffle";
import "@nomiclabs/hardhat-ethers";
import "@openzeppelin/hardhat-upgrades";
import {CONSTANTS} from "./constants";

export default {
  networks: {
    hardhat: {
    },
    ropsten:{
      url: CONSTANTS.eth_node_url,
      accounts: [CONSTANTS.eth_account],
      gas: "auto",
      gasPrice: "auto"
    },
  },
  solidity: "0.8.4",
  paths: {
    sources: "./ethereum",
    tests: "./tests",
    cache: "./cache",
    artifacts: "./artifacts"
  },
  mocha: {
    timeout: 200000
  }
};

