require("@nomicfoundation/hardhat-toolbox")
require("dotenv").config() // Load environment variables

module.exports = {
  solidity: "0.8.20",
  networks: {
    sepolia: {
      url: "https://eth-sepolia.g.alchemy.com/v2/6gbfHkcrH1gSsMG8dQYbCK7WPtA1oNBy",
      accounts: [
        "cf7a742fec9e562470e239cc976d177b9a3d91ae8603775939d35b22fbf8b46c",
      ],
    },
  },
}
