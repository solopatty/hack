# 🧪 Hardhat Smart Contracts - SoloPatty Project

This project is a Solidity-based token system using the Hardhat development environment. The contracts are deployed on the **Sepolia** testnet and represent components of a modular token architecture — each contract corresponds to a different ingredient in the SoloPatty system.

## 📜 Contracts Deployed

### 🍔 Patty Token
- **Address:** [`0xCAdaFeDf40140C8eBCa3A0E802dfC4dD72869c9F`](https://sepolia.etherscan.io/address/0xCAdaFeDf40140C8eBCa3A0E802dfC4dD72869c9F)
- **Description:** Core token in the SoloPatty system, representing the foundational layer of the patty structure.

### 🥬 Lettuce Token
- **Address:** [`0xa966bdf941ea2eccc8ADC453B977FFeE27bC2f55`](https://sepolia.etherscan.io/address/0xa966bdf941ea2eccc8ADC453B977FFeE27bC2f55)
- **Description:** Auxiliary token representing freshness or a feature enhancer — like lettuce in a burger analogy.

### 🧀 Cheese Token
- **Address:** [`0xC9EbB17FC1f5101Db84EA345693194c520b411bb`](https://sepolia.etherscan.io/address/0xC9EbB17FC1f5101Db84EA345693194c520b411bb)
- **Description:** Complementary token for added richness — cheese symbolizes premium features or additional value.

### 🔄 SoloPatty Contract
- **Address:** [`0x5De6e7cAE4b30d4CbF744B6Dd78c6418F5750570`](https://sepolia.etherscan.io/address/0x5De6e7cAE4b30d4CbF744B6Dd78c6418F5750570)
- **Description:** The orchestrator smart contract that interacts with Patty, Lettuce, and Cheese tokens to enable composability, swaps, or bundled logic within the SoloPatty ecosystem.

---

## 🛠 Getting Started

Make sure you have the following dependencies installed:

```bash
npm install


```shell
npx hardhat help
npx hardhat test
REPORT_GAS=true npx hardhat test
npx hardhat node
npx hardhat ignition deploy ./ignition/modules/Lock.ts
```


