import {saveConstants} from "../constants";
import {deployTerra, registerETHBridge} from "../utils/terra_utils";
import {deployEth} from "../utils/eth_utils";
import {bridgeAsset} from "../utils/wormhole_utils";


async function main() {

  await deployTerra()
  await deployEth()
  await registerETHBridge();
  await bridgeAsset({"info": {"native_token": {"denom": "uusd"}}, "amount": "100000000"})
  saveConstants()
}

// We recommend this pattern to be able to use async/await everywhere
// and properly handle errors.
main()
  .then(() => process.exit(0))
  .catch((error) => {
    console.error(error);
    process.exit(1);
  });
