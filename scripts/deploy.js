async function main() {
  const [deployer] = await ethers.getSigners()

  console.log("Deploying contracts with the account:", deployer.address)

  const SoloPatty = await ethers.getContractFactory("SoloPatty")
  const soloPatty = await SoloPatty.deploy()

  console.log("SoloPatty deployed to:", soloPatty.address)
}

main()
  .then(() => process.exit(0))
  .catch((error) => {
    console.error(error)
    process.exit(1)
  })
