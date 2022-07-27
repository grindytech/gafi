# [Gafi TX](https://wiki.gafi.network/learn/gafi-tx)

Gafi TX is the controller to keep the balance of Gafi Network

## Overview

The Upfront Pool Module controls the transaction flow, gas_price... to keep the balancing Defi Network

To use it in your runtime, you need to implement the Config assets

The supported dispatchable functions are documented in the comment

### Terminology


### Goals

The gafi-tx module in Gafi is designed to make the following possible:

* Control gas_price to balance transaction fee
* Control tickets of players
* Handle GasWeightMaping

## Interface

### Dispatchable Functions
* `set_gas_price` Root only

### Public Functions
* `min_gas_price` Get min gas_price

## Usage

Please visit the [unittest](https://github.com/grindytech/gafi/blob/master/pallets/gafi-tx/src/tests.rs)

### Prerequisites

Import the Gafi TX module and types and derive your runtime's configuration traits from the Gafi TX module trait.

License: Apache-2.0

