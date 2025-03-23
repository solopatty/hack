import { ethers as ethersLib } from "ethers"
import hardhat from "hardhat"
const { ethers } = hardhat
import * as dotenv from "dotenv"
dotenv.config()

async function deployToken(initialSupply, name, symbol, deployer) {
  const TokenFactory = await ethers.getContractFactory("SimpleToken", deployer)
  const token = await TokenFactory.deploy(initialSupply, name, symbol)
  await token.waitForDeployment()
  const tokenAddress = await token.getAddress()
  console.log(`✅ ${name} token deployed to: ${tokenAddress}`)
  return tokenAddress
}

async function main() {
  const privateKey =
    process.env.PRIVATE_KEY ||
    "cf7a742fec9e562470e239cc976d177b9a3d91ae8603775939d35b22fbf8b46c"

  const provider = new ethersLib.JsonRpcProvider(
    "https://eth-sepolia.g.alchemy.com/v2/6gbfHkcrH1gSsMG8dQYbCK7WPtA1oNBy"
  )

  const deployer = new ethersLib.Wallet(privateKey, provider)
  console.log("Deploying tokens with account:", deployer.address)
  console.log(
    "Account balance:",
    (await provider.getBalance(deployer.address)).toString()
  )

  const tokens = [
    { name: "PATTY", symbol: "PATTY", supply: "1000000000000" },
    { name: "CHEESE", symbol: "CHEESE", supply: "1000000000000" },
    { name: "LETTUCE", symbol: "LETTUCE", supply: "1000000000000" },
  ]

  for (const { name, symbol, supply } of tokens) {
    await deployToken(supply, name, symbol, deployer)
  }
}

main()
  .then(() => process.exit(0))
  .catch((err) => {
    console.error("❌ Token deployment failed:", err)
    process.exit(1)
  })
