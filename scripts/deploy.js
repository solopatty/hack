async function main() {
  const [deployer] = await ethers.getSigners()

  console.log("Deploying contracts with the account:", deployer.address)

  const SoloPatty = await ethers.getContractFactory("SoloPatty")
  const soloPatty = await SoloPatty.deploy(
    "0x92b9baA72387Fb845D8Fe88d2a14113F9cb2C4E7" // Make sure case matches (for readability)
  )

  await soloPatty.deployed() // ⬅️ THIS ensures it's deployed before printing

  console.log("✅ SoloPatty deployed to:", soloPatty.address)
}
