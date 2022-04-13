# Upfront Pool Module

One of the ways provides upfront-charge services is to reduce transaction fees, and enhance the security of the network.

## Overview

The Upfront Pool Module provides functionality for upfront-charge service. It's must implement with pallet_pool, and controlled by pallet_pool

To use it in your runtime, you need to implement the Config assets

The supported dispatchable functions are documented in the comment

### Terminology

* **Refund amount:**

* **New Players:**

* **Ingame Players:**

### Goals

The upfront_pool module in Gafi is designed to make the following possible:

* Join Pool with charge double service fee
* Leave Pool with return refund amount
* Charge service fee in every specific period of time
* Verify ECDSA signature

## Interface

### Dispatchable Functions
* `set_max_player` Root only

### Public Functions


## Usage

Please visit the [unittest](https://github.com/cryptoviet/gafi/blob/master/pallets/upfront-pool/src/tests.rs)

### Prerequisites

Import the Upfront Pool module and types and derive your runtime's configuration traits from the Upfront Pool module trait.

License: Apache-2.0

