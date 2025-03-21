// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.28;

import "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import "@openzeppelin/contracts/utils/cryptography/MerkleProof.sol";

contract SoloPatty {
    address public owner;
    mapping(address => mapping(address => uint256)) public balances; // user => token => amount
    bytes32 public merkleRoot; // Root of Merkle Tree for compressed balances
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

    /// @notice User deposits tokens into the contract
    function depositTokens(address token, uint256 amount) external {
        require(amount > 0, "Invalid amount");
        IERC20(token).transferFrom(msg.sender, address(this), amount);
        balances[msg.sender][token] += amount;

        emit Deposited(msg.sender, token, amount);
    }

    /// @notice Updates the Merkle root from TEE attestation
    function updateMerkleRoot(
        bytes32 newRoot,
        bytes memory attestation
    ) external onlyOwner {
        // TODO: Verify attestation (e.g., using Intel SGX remote attestation verification)
        merkleRoot = newRoot;
        emit MerkleRootUpdated(newRoot);
    }

    /// @notice Withdraw tokens (normal exit)
    function withdrawTokens(address token, uint256 amount) external {
        require(balances[msg.sender][token] >= amount, "Insufficient balance");
        balances[msg.sender][token] -= amount;
        IERC20(token).transfer(msg.sender, amount);

        emit Withdrawn(msg.sender, token, amount);
    }

    /// @notice Permissionless withdrawal using Merkle proof
    function withdrawWithMerkleProof(
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
