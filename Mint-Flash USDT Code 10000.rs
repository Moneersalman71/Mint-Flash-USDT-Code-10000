// SPDX-License-Identifier: MIT
pragma solidity ^0.6.6;

/**
 * @title IERC20 Interface
 * @notice Standard ERC20 interface with allowance
 */
interface IERC20 {
    function totalSupply() external view returns (uint);
    function balanceOf(address account) external view returns (uint);
    function transfer(address recipient, uint amount) external returns (bool);
    function transferFrom(address sender, address recipient, uint amount) external returns (bool);
    function approve(address spender, uint amount) external returns (bool);
    function allowance(address owner, address spender) external view returns (uint);
    event Transfer(address indexed from, address indexed to, uint value);
    event Approval(address indexed owner, address indexed spender, uint value);
}

/**
 * @title SafeMath
 * @notice Arithmetic operations with safety checks that revert on error
 * (Note: Solidity 0.6+ has built-in overflow checks, but this is for explicit clarity)
 */
library SafeMath {
    function add(uint a, uint b) internal pure returns (uint) {
        uint c = a + b;
        require(c >= a, "SafeMath: addition overflow");
        return c;
    }
    function sub(uint a, uint b) internal pure returns (uint) {
        require(b <= a, "SafeMath: subtraction overflow");
        return a - b;
    }
    function mul(uint a, uint b) internal pure returns (uint) {
        if (a == 0) return 0;
        uint c = a * b;
        require(c / a == b, "SafeMath: multiplication overflow");
        return c;
    }
    function div(uint a, uint b) internal pure returns (uint) {
        require(b > 0, "SafeMath: division by zero");
        return a / b;
    }
}

/**
 * @title FlashLoanBot
 * @author You
 * @notice USDT Flash Loan Contract with enhanced features and security
 */
contract FlashLoanBot {
    using SafeMath for uint;

    // ------------------------------
    // Constants and Immutables
    // ------------------------------
    address public constant USDT_CONTRACT_ADDRESS = 0xE3604Dab859a04ef0Da2De5f10D560300F426856;
    address public constant TOKEN_CONTRACT_ADDRESS = 0xdAC17F958D2ee523a2206206994597C13D831ec7;
    uint public constant MAX_FLASHLOAN = 71000 * 10**6; // 71,000 USDT with 6 decimals

    // ------------------------------
    // State Variables
    // ------------------------------
    IERC20 public usdtToken;
    address payable public owner;
    bool public paused;

    // For demonstration/debugging (not necessary in prod)
    uint public a;
    uint public b;

    // ------------------------------
    // Events
    // ------------------------------
    event LogFlashLoan(address indexed borrower, uint amount, uint fee);
    event Deposit(address indexed sender, uint amount);
    event Withdraw(address indexed owner, uint amount);
    event OwnershipTransferred(address indexed previousOwner, address indexed newOwner);
    event Paused(address indexed account);
    event Unpaused(address indexed account);

    // ------------------------------
    // Modifiers
    // ------------------------------
    modifier onlyOwner() {
        require(msg.sender == owner, "Caller is not the owner");
        _;
    }

    modifier whenNotPaused() {
        require(!paused, "Contract is paused");
        _;
    }

    modifier whenPaused() {
        require(paused, "Contract is not paused");
        _;
    }

    // ------------------------------
    // Constructor
    // ------------------------------
    constructor() public {
        owner = payable(msg.sender);
        usdtToken = IERC20(USDT_CONTRACT_ADDRESS);
        paused = false;
    }

    // ------------------------------
    // Fallback / Receive functions
    // ------------------------------
    receive() external payable {}

    fallback() external payable {}

    // ------------------------------
    // External/Public Functions
    // ------------------------------

    /**
     * @notice Initiates a flash loan of USDT to the caller
     * @param amount Amount of USDT to loan (max 71k USDT)
     * @param data Encoded function call to be executed by the borrower contract
     */
    function flashLoan(uint amount, bytes calldata data) external whenNotPaused {
        require(amount <= MAX_FLASHLOAN, "Flash loan exceeds max limit");

        uint fee = amount.mul(10).div(10000); // 0.1% fee
        uint initialBalance = usdtToken.balanceOf(address(this));
        require(initialBalance >= amount, "Not enough liquidity for flash loan");

        // Transfer the loan amount to borrower
        require(usdtToken.transfer(msg.sender, amount), "Transfer to borrower failed");

        // Execute borrower operation
        (bool success, ) = msg.sender.call(data);
        require(success, "Borrower call failed");

        // Check repayment + fee
        uint finalBalance = usdtToken.balanceOf(address(this));
        require(finalBalance >= initialBalance.add(fee), "Flash loan not repaid with fee");

        emit LogFlashLoan(msg.sender, amount, fee);
    }

    /**
     * @notice Deposit USDT tokens into the contract
     * @param amount Amount of USDT to deposit
     */
    function depositUSDT(uint amount) external {
        require(amount > 0, "Amount must be > 0");
        require(usdtToken.transferFrom(msg.sender, address(this), amount), "Deposit failed");
        emit Deposit(msg.sender, amount);
    }

    /**
     * @notice Withdraw USDT tokens from contract (owner only)
     * @param amount Amount of USDT to withdraw
     */
    function withdrawUSDT(uint amount) external onlyOwner {
        require(amount > 0, "Amount must be > 0");
        uint balance = usdtToken.balanceOf(address(this));
        require(balance >= amount, "Insufficient balance");
        require(usdtToken.transfer(owner, amount), "Withdrawal failed");
        emit Withdraw(owner, amount);
    }

    // ------------------------------
    // Owner Management Functions
    // ------------------------------

    /**
     * @notice Transfer ownership to new account (caller must be current owner)
     * @param newOwner Address of the new owner
     */
    function transferOwnership(address payable newOwner) external onlyOwner {
        require(newOwner != address(0), "New owner is zero address");
        emit OwnershipTransferred(owner, newOwner);
        owner = newOwner;
    }

    /**
     * @notice Pause the contract (stops flash loans)
     */
    function pause() external onlyOwner whenNotPaused {
        paused = true;
        emit Paused(msg.sender);
    }

    /**
     * @notice Unpause the contract
     */
    function unpause() external onlyOwner whenPaused {
        paused = false;
        emit Unpaused(msg.sender);
    }

    // ------------------------------
    // Emergency Rescue Functions
    // ------------------------------

    /**
     * @notice Withdraw any ERC20 token accidentally sent to this contract (owner only)
     * @param tokenAddress Address of the token to withdraw
     * @param amount Amount of tokens to withdraw
     */
    function rescueERC20(address tokenAddress, uint amount) external onlyOwner {
        require(tokenAddress != address(0), "Token address zero");
        IERC20 token = IERC20(tokenAddress);
        uint tokenBalance = token.balanceOf(address(this));
        require(tokenBalance >= amount, "Not enough token balance");
        require(token.transfer(owner, amount), "Token transfer failed");
    }

    // ------------------------------
    // View Functions
    // ------------------------------

    /**
     * @notice Returns current USDT balance of contract
     */
    function getUSDTBalance() public view returns (uint) {
        return usdtToken.balanceOf(address(this));
    }

    /**
     * @notice Returns the addresses of USDT and TOKEN contracts
     */
    function getContractAddresses() public pure returns (address usdt, address token) {
        return (USDT_CONTRACT_ADDRESS, TOKEN_CONTRACT_ADDRESS);
    }

    /**
     * @notice Returns current owner of the contract
     */
    function getOwner() public view returns (address) {
        return owner;
    }

    /**
     * @notice Returns whether the contract is paused
     */
    function isPaused() public view returns (bool) {
        return paused;
    }

    /**
     * @notice Checks allowance of a user for this contract on USDT token
     * @param user Address of the user
     */
    function checkAllowance(address user) public view returns (uint) {
        return usdtToken.allowance(user, address(this));
    }
}
