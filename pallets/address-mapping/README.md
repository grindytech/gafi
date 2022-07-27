# Proof Address Mapping

A simple, secure way to dealing with address mapping between Substrate(H256) address
and EVM(H160) address

## Overview

The Proof Address Mapping pallet provides functionality for address mapping

To use it in your runtime, you need to implement the Config assets

The supported dispatchable functions are documented in the comment

### Terminology

* **Bond:** Mapping Substrate(H256) address and EVM(H160) address by verified
the signature signed by EVM address

* **Unbond:** Breaking the bond of Substrate(H256) address and EVM(H160) address
so these addresses will be using the default AddressMapping after that

* **Original H160:** The H160 address that hash from AccountId32 using get_default_evm_address
function, there is no way to convert back to the AccountId32 address. 

* **Original ID:** The AccountId32 address that generated from H160 address by function into_account_id
of OriginAddressMapping, this AccountId32 contains b'evm' in the prefix and can be converted back
to H160 address which used to generate it.

### Goals

The address_mapping pallet in Gafi is designed to make the following possible:

* Bond
* Unbond
* Move assets from EVM address before making bonded to H256 address after bonding success
* Verify ECDSA signature 

## Interface

### Dispatchable Functions
* `bond`
* `unbond`

### Public Functions

* `into_account_id` - Get AccountId32 address from H160 address, in case two addresses didn't bond,
the into_account_id function of OriginAddressMapping will be returned.
* `get_evm_address` - Get H160 address from AccountId32 if that AccountId32 started with prefix b'evm',
otherwise, the None will be returned.
* `get_default_evm_address` - Get H160 Original address

## Usage

Please visit the [unittest](https://github.com/grindytech/gafi/blob/master/pallets/address-mapping/src/tests.rs)

### Prerequisites

Import the Proof Address Mapping pallet and types and derive your runtime's configuration traits from the Proof Address Mapping trait.

License: Apache-2.0