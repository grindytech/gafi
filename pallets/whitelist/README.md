# Gafi Whitelist Module

Provide whitelist functionality that allows pool owners can restrict player access to their pool.

## Overview

pallet_whitelist allows pool owner set up whitelist api to verify allowed players through offchain-worker.

To use it in your runtime, you need to implement the Config Custom Pool

The supported dispatchable functions are documented in the comment

### Terminology

* **Fetch whitelist:** verify player whiteliste by offchain-worker

### Goals

The pallet_whitelist module in Gafi is designed to make the following possible:

* Allow pool owner setup pool's whitelist
* Allow players to apply whitelist to the pool
* Verify whitelist
* Join whitelisted Pool

## Interface

### Dispatchable Functions
* `approve_whitelist` pool owner approves player's whitelist request
* `approve_whitelist_unsigned` unsigned
* `apply_whitelist` player apply whitelist
* `enable_whitelist` pool owner enable whitelist to the pool
* `withdraw_whitelist` pool owner withdraw whitelist

### Public Functions


## Usage

Please visit the [unittest](https://github.com/grindytech/gafi/blob/master/pallets/whitelist/src/tests.rs)

### Prerequisites

Import the Pallet Whitelist module and types and derive your runtime's configuration traits from the Pallet Whitelist module trait.

License: Apache-2.0

