// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.28;

import "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import "@openzeppelin/contracts/utils/cryptography/MerkleProof.sol";

contract SoloPatty {
    address public owner;
    bytes32 public merkleRoot; // Latest TEE state root
    mapping(bytes32 => bool) public hasClaimed;

    event Deposited(
        address indexed user,
        address indexed token,
        uint256 amount
    );
    event Withdrawn(
        address indexed user,
        address indexed token,
        uint256 amount
    );
    event MerkleRootUpdated(bytes32 newRoot);

    modifier onlyOwner() {
        require(msg.sender == owner, "Not authorized");
        _;
    }

    constructor() {
        owner = msg.sender;
    }

    /// @notice User deposits tokens into the contract (TEE reads this off-chain)
    function depositTokens(address token, uint256 amount) external {
        require(amount > 0, "Invalid amount");
        IERC20(token).transferFrom(msg.sender, address(this), amount);
        emit Deposited(msg.sender, token, amount);
    }

    /// @notice TEE posts a new root after matching logic + balance updates
    function updateMerkleRoot(
        bytes32 newRoot,
        bytes memory attestation
    ) external onlyOwner {
        // TODO: Add attestation verification for production (SGX or signature check)
        merkleRoot = newRoot;
        emit MerkleRootUpdated(newRoot);
    }

    /// @notice Users withdraw funds via Merkle proof (TEE must provide them a proof)
    function withdrawTokensWithAttestation(
        address token,
        uint256 amount,
        bytes32[] calldata proof
    ) external {
        bytes32 leaf = keccak256(abi.encodePacked(msg.sender, token, amount));
        require(MerkleProof.verify(proof, merkleRoot, leaf), "Invalid proof");
        require(!hasClaimed[leaf], "Already claimed");

        hasClaimed[leaf] = true;
        IERC20(token).transfer(msg.sender, amount);

        emit Withdrawn(msg.sender, token, amount);
    }
}
