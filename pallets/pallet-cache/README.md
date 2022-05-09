# Pallet Cache

The Pallet Cache provides functions to store data temporary

## Overview

The Pallet Cache is used to store data temporarily, the data in Cache will automatically delete after 'CleanTime'.

To use it in your runtime, you need to implement the Config

The supported dispatchable functions and public functions are documented in the comment

### Terminology

* **CleanTime:** The period of time that data is available on Pallet Cache, after that time data will be removed

### Goals

The pallet_cache module in Gafi is designed to make the following possible:

* Insert data by AccountId and Action name
* Get data by AccountId and Action name

## Interface

### Dispatchable Functions
* `set_clean_time` Root only

### Public Functions
* `insert` Insert data
* `get` Get data


## Usage

Please visit the [unittest](https://github.com/cryptoviet/gafi/blob/master/pallets/pallet-cache/src/tests.rs)

### Prerequisites

Import the Pallet Cache module and types and derive your runtime's configuration traits from the Pallet Cache module trait.

License: Apache-2.0

