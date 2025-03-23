// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.20;

import "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import {ECDSA} from "@openzeppelin/contracts/utils/cryptography/ECDSA.sol";
import "@openzeppelin/contracts/utils/cryptography/MessageHashUtils.sol";

contract SoloPatty {
    address public immutable owner;
    address public immutable trustedSigner;
    bytes32 public merkleRoot;
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

    constructor(address _trustedSigner) {
        owner = msg.sender;
        trustedSigner = _trustedSigner;
    }

    modifier onlyOwner() {
        require(msg.sender == owner, "Not authorized");
        _;
    }

    /// @notice Users deposit tokens into the contract (TEE listens off-chain)
    function depositTokens(address token, uint256 amount) external {
        require(amount > 0, "Invalid amount");
        IERC20(token).transferFrom(msg.sender, address(this), amount);
        emit Deposited(msg.sender, token, amount);
    }

    /// @notice Users withdraw funds with a signed message from the TEE
    function withdrawTokensWithSignature(
        address user,
        address token,
        uint256 amount,
        bytes memory signature
    ) external {
        bytes32 leaf = keccak256(abi.encodePacked(user, token, amount));
        bytes32 ethHash = MessageHashUtils.toEthSignedMessageHash(leaf);
        address recovered = ECDSA.recover(ethHash, signature);
        require(recovered == trustedSigner, "Invalid TEE signature");
        require(!hasClaimed[leaf], "Already claimed");

        hasClaimed[leaf] = true;
        IERC20(token).transfer(user, amount);
        emit Withdrawn(user, token, amount);
    }
}
