import {CONSTANTS} from "../constants";
import { migrateWithCodePath} from "../utils/terra_utils";

async function main() {
  await migrateWithCodePath(CONSTANTS.terra_xanchor_bridge, "terra/artifacts/wormhole_bridge.wasm")
  await migrateWithCodePath("terra1du0nux344pedw88r8257ff0lfvv3nrqzc93ypw", "terra/artifacts/crossanchor_core.wasm")
}

// We recommend this pattern to be able to use async/await everywhere
// and properly handle errors.
main()
  .then(() => process.exit(0))
  .catch((error) => {
    console.error(error);
    process.exit(1);
  });
