# Staking Pool Module

One of the ways provides staking services is to reduce transaction fees, and enhance the security of the network.

## Overview

The Staking Pool Module provides functionality for staking-charge service. It's must implement with pallet_pool, and controlled by pallet_pool

To use it in your runtime, you need to implement the Config assets

The supported dispatchable functions are documented in the comment

### Terminology

* **Staking amount:** The amount of token that will be reserved to get a discount on the transaction fee 

### Goals

The staking_pool module in Gafi is designed to make the following possible:

* Join Pool with reserve staking amount
* Leave Pool with return correct staking amount

## Interface

### Dispatchable Functions
* `set_max_player` Root only

### Public Functions


## Usage

Please visit the [unittest](https://github.com/cryptoviet/gafi/blob/master/pallets/Staking-pool/src/tests.rs)

### Prerequisites

Import the Staking Pool module and types and derive your runtime's configuration traits from the Staking Pool module trait.

License: Apache-2.0

