import fs from 'fs'


export const ENV_PATH = "env.json"
export let CONSTANTS = JSON.parse(fs.readFileSync(ENV_PATH, "utf8"))

export function saveConstants() {
  fs.writeFileSync(ENV_PATH, JSON.stringify(CONSTANTS, null, 4), 'utf8');
}