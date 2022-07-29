# Pallet Pool Names

Allow sponsor to name their sponsored pools, make it easy for user to recognize.

## Overview

To use it in your runtime, you need to implement the Config assets

The supported dispatchable functions are documented in the comment

### Terminology

### Goals

The pool names pallet in Gafi is designed to make the following possible:

- Set pool name.
- Clear pool name.

## Interface

### Dispatchable Functions

- `set_name` sponsor set name for the pool.
- `clear_name` sponsor clear name for the pool.

### Public Functions

## Usage

Please visit the [unittest](https://github.com/grindytech/gafi/blob/master/pallets/pool-names/src/tests.rs)

### Prerequisites

Import the Pool Names pallet and types and derive your runtime's configuration traits from the Pool Names pallet trait.

License: Apache-2.0
