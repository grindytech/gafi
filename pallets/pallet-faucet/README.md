# Pallet Faucet

Allow user to get faucet tokens when testing the network and donate it back.

## Overview

The Faucet pallet provides functionality for faucet and donate

To use it in your runtime, you need to implement the Config assets

The supported dispatchable functions are documented in the comment

### Terminology

- **FaucetAmount:** The amount of token that will be sent to user account.

- **GenesisAccounts:** Accounts that will send faucet tokens to user.

### Goals

The faucet pallet in Gafi is designed to make the following possible:

- Faucet
- Donate

## Interface

### Dispatchable Functions

- `faucet`
- `donate`

### Public Functions

## Usage

Please visit the [unittest](https://github.com/cryptoviet/gafi/blob/master/pallets/pallet-faucet/src/tests.rs)

### Prerequisites

Import the Faucet pallet and types and derive your runtime's configuration traits from the Faucet module trait.

License: Apache-2.0
