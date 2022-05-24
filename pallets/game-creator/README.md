# Game Creator Pallet

The module for contract creators claims the ownership of their smart-contract addresses to receive the transaction fee rewarded. 

## Overview

Game Creator Pallet provides the functionality to contract creator to claim the ownership to become the game's creator
to receive the transaction fee reward

To use it in your runtime, you need to implement the Config assets

The supported dispatchable functions are documented in the comment

### Terminology

* **Contract Creator:** The H160 address that deploys the contract

* **Contract Owner:** The owner of the contract address, the owner can change the ownership


### Goals

The game_creator pallet in Gafi is designed to make the following possible:

* Claim the ownership of smart-contract address
* Change the ownership
* Withdraw the ownership

## Interface

### Dispatchable Functions
* `claim_contract`
* `change_ownership`
* `withdraw_contract`

### Public Functions

* `get_game_creator` - Get the current owner of contract

## Usage

Please visit the [unittest](https://github.com/cryptoviet/gafi/blob/master/pallets/game-creator/src/tests.rs)

### Prerequisites

Import the Game Creator pallet and types and derive your runtime's configuration traits from the Game Creator trait.

License: Apache-2.0