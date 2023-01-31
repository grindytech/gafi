# Pallet Funding Pool

One more option for players to participate in Gafi Network, reduce transaction fees and help game-creator appeals to their users.

## Overview

Please visit [Funding Pool Wiki](https://wiki.gafi.network/learn/funding-pool)

To use it in your runtime, you need to implement the Config assets

The supported dispatchable functions are documented in the comment

### Terminology

* **Sponsor:** The owner of the funding-pool

* **Pool ID:** The random 32-character represents the id of the pool,
the sponsor deposit token to this id to make the pool works.

* **Targets:** The smart-contract addresses that are added to the funding-pool,
players can only interact with those contracts to get the discount.


### Goals

The funding-pool pallet in Gafi is designed to make the following possible:

* Create pool
* Withdraw pool
* Change targets

## Interface

### Dispatchable Functions
* `create_pool` sponsor create new pool
* `withdraw_pool` withdraw all the balances then destroy the pool

### Public Functions


## Usage

Please visit the [unittest](https://github.com/grindytech/gafi/blob/master/pallets/funding-pool/src/tests.rs)

### Prerequisites

Import the Funding Pool pallet and types and derive your runtime's configuration traits from the Funding Pool pallet trait.

License: Apache-2.0
