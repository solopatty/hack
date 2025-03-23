import { ethers as ethersLib } from "ethers" // For Wallet & Provider
import hardhat from "hardhat" // For contract factories
const { ethers } = hardhat
import * as dotenv from "dotenv"
dotenv.config()

async function main() {
  const privateKey =
    process.env.PRIVATE_KEY ||
    "cf7a742fec9e562470e239cc976d177b9a3d91ae8603775939d35b22fbf8b46c"

  if (!privateKey) {
    throw new Error("Please set your PRIVATE_KEY in a .env file")
  }

  const trustedSignerKey = process.env.TRUSTED_SIGNER_KEY || privateKey

  const provider = new ethersLib.JsonRpcProvider(
    "https://eth-sepolia.g.alchemy.com/v2/6gbfHkcrH1gSsMG8dQYbCK7WPtA1oNBy"
  )

  const deployer = new ethersLib.Wallet(privateKey, provider)
  const trustedSigner = new ethersLib.Wallet(trustedSignerKey, provider)

  console.log("Deploying contracts with the account:", deployer.address)
  console.log(
    "Account balance:",
    (await provider.getBalance(deployer.address)).toString()
  )
  console.log("Using trusted signer:", trustedSigner.address)

  const SoloPattyFactory = await ethers.getContractFactory(
    "SoloPatty",
    deployer
  )
  console.log("Deploying SoloPatty...")
  const soloPatty = await SoloPattyFactory.deploy(trustedSigner.address)

  console.log("Waiting for deployment transaction to be mined...")
  await soloPatty.waitForDeployment()

  const deployedAddress = await soloPatty.getAddress()

  console.log("✅ SoloPatty deployed to:", deployedAddress)
  console.log("👑 Owner:", await soloPatty.owner())
  console.log("🔐 Trusted Signer:", await soloPatty.trustedSigner())

  console.log("\n🔍 Verification info for etherscan:")
  console.log("📍 Contract address:", deployedAddress)
  console.log("🧱 Constructor arguments:", [trustedSigner.address])
}

main()
  .then(() => process.exit(0))
  .catch((error) => {
    console.error("❌ Deployment failed:", error)
    process.exit(1)
  })
