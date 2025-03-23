# 🧪 Hardhat Smart Contracts - SoloPatty Project

This project is a Solidity-based token system using the Hardhat development environment. The contracts are deployed on the **Sepolia** testnet and represent components of a modular token architecture — each contract corresponds to a different ingredient in the SoloPatty system.

## 📜 Contracts Deployed

### 🍔 Patty Token
- **Address:** [`0x126F0c11F3e5EafE37AB143D4AA688429ef7DCB3`](https://sepolia.etherscan.io/address/0x126F0c11F3e5EafE37AB143D4AA688429ef7DCB3)
- **Description:** Core token in the SoloPatty system, representing the foundational layer of the patty structure.

### 🥬 Lettuce Token
- **Address:** [`0xF7aE103AacD84641Fa0c43860C23a8Cf7cE5DB5a`](https://sepolia.etherscan.io/address/0xF7aE103AacD84641Fa0c43860C23a8Cf7cE5DB5a)
- **Description:** Auxiliary token representing freshness or a feature enhancer — like lettuce in a burger analogy.

### 🧀 Cheese Token
- **Address:** [`0x5D7714751FAf22a96F7D2eAC15304839242cF8c0`](https://sepolia.etherscan.io/address/0x5D7714751FAf22a96F7D2eAC15304839242cF8c0)
- **Description:** Complementary token for added richness — cheese symbolizes premium features or additional value.

### 🔄 SoloPatty Contract
- **Address:** [`0xCB30D0881119bA8837A9e26E298d3b73c4c521EC`](https://sepolia.etherscan.io/address/0xCB30D0881119bA8837A9e26E298d3b73c4c521EC)
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


