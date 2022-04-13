# Upfront Pool Module

One of the ways provides upfront-charge services is to reduce transaction fees, and enhance the security of the network.

## Overview

The Upfront Pool Module provides functionality for upfront-charge service. It's must implement with pallet_pool, and controlled by pallet_pool

To use it in your runtime, you need to implement the Config assets

The supported dispatchable functions are documented in the comment

### Terminology

* **Refund amount:** The refund amount when player leave the pool early

* **TimeService:** The specific period of time to charge service fee

* **New Players:** The new players join the pool before the TimeService, whose are without charge

* **Ingame Players:** The players, who stay in the pool longer than TimeService

### Goals

The upfront_pool module in Gafi is designed to make the following possible:

* Join Pool with charge double service fee
* Leave Pool with return refund amount
* Charge service fee in every specific period of time

## Interface

### Dispatchable Functions
* `set_max_player` Root only

### Public Functions


## Usage

Please visit the [unittest](https://github.com/cryptoviet/gafi/blob/master/pallets/upfront-pool/src/tests.rs)

### Prerequisites

Import the Upfront Pool module and types and derive your runtime's configuration traits from the Upfront Pool module trait.

License: Apache-2.0

