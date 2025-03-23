// test/withdraw.test.ts
import { expect } from "chai"
import { ethers } from "hardhat"
import { SoloPatty } from "../typechain-types"
import { Wallet } from "ethers"

const TEE_PRIVATE_KEY =
  "cf7a742fec9e562470e239cc976d177b9a3d91ae8603775939d35b22fbf8b46c" // Mock
const teeWallet = new Wallet(TEE_PRIVATE_KEY)

describe("SoloPatty TEE Withdrawal Test", function () {
  let contract: SoloPatty
  let token: any
  let user: any

  beforeEach(async () => {
    const [deployer, u1] = await ethers.getSigners()
    user = u1

    const Token = await ethers.getContractFactory("SimpleToken")
    token = await Token.deploy("MockToken", "MTK", 18)
    await token.deployed()
    await token.mint(user.address, 1_000_000)

    const SoloPatty = await ethers.getContractFactory("SoloPatty")
    contract = await SoloPatty.deploy(teeWallet.address)
    await contract.deployed()

    // Approve & Deposit
    await token.connect(user).approve(contract.address, 500_000)
    await contract.connect(user).depositTokens(token.address, 500_000)
  })

  it("allows withdrawal using TEE signature", async () => {
    const amount = 500_000
    const userAddr = user.address
    const tokenAddr = token.address

    const leaf = ethers.utils.solidityKeccak256(
      ["address", "address", "uint256"],
      [userAddr, tokenAddr, amount]
    )

    const signature = await teeWallet.signMessage(ethers.utils.arrayify(leaf))

    const before = await token.balanceOf(userAddr)

    const tx = await contract
      .connect(user)
      .withdrawTokensWithSignature(userAddr, tokenAddr, amount, signature)
    await tx.wait()

    const after = await token.balanceOf(userAddr)
    expect(after.sub(before)).to.equal(amount)
  })

  it("rejects reused signature", async () => {
    const amount = 500_000
    const userAddr = user.address
    const tokenAddr = token.address

    const leaf = ethers.utils.solidityKeccak256(
      ["address", "address", "uint256"],
      [userAddr, tokenAddr, amount]
    )

    const signature = await teeWallet.signMessage(ethers.utils.arrayify(leaf))

    await contract
      .connect(user)
      .withdrawTokensWithSignature(userAddr, tokenAddr, amount, signature)

    // Try again with same signature (should fail)
    await expect(
      contract.withdrawTokensWithSignature(
        userAddr,
        tokenAddr,
        amount,
        signature
      )
    ).to.be.revertedWith("Already claimed")
  })
})
