pragma solidity ^0.8.10;

import "@openzeppelin/contracts/token/ERC20/ERC20.sol";
import "@openzeppelin/contracts/token/ERC20/extensions/ERC20Capped.sol";

contract GAKI is ERC20Capped {
    constructor(uint256 initialSupply) ERC20("Gaki Network", "GAKI") ERC20Capped(1000*10**18) {
        ERC20._mint(msg.sender, initialSupply);
    }
}