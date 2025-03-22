import { HardhatUserConfig } from "hardhat/config"
import "@nomicfoundation/hardhat-toolbox"
import * as dotenv from "dotenv"

dotenv.config()

const privateKey = process.env.PRIVATE_KEY

if (!privateKey) {
  throw new Error("❌ PRIVATE_KEY is not defined in .env")
}

module.exports = {
  solidity: "0.8.28",
  networks: {
    sepolia: {
      url: "https://eth-sepolia.g.alchemy.com/v2/6gbfHkcrH1gSsMG8dQYbCK7WPtA1oNBy",
      accounts: [`0x${privateKey}`],
    },
  },
}
