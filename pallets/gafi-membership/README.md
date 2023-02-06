# Membership Pallet

The module for users to build up membership level. After reach a specific level, users will unlock some privilege.

## Overview

Membership Pallet provides the functionality for users to join the membership program that provide many advance feature and value.

To use it in your runtime, you need to implement the Config assets

The supported dispatchable functions are documented in the comment

### Terminology

* **Membership Level:** The level of user that joined membership program

* **Point:** Point will give to user that joined membership program and complete mission.

* **Achievements:** The list of mission or requirements to achieve point


### Goals

The membership pallet in Gafi is designed to make the following possible:

* Provide a list of users that actively
* Allow users to receive privilege when become an active user.
* Helping project owners to find users that active on the projects.

## Interface

### Dispatchable Functions
* `registration`
* `remove_member`

### Public Functions

* `get_level` - Get the current level of member.
* `is_registered` - Check if user registered

## Usage

Please visit the [unittest](https://github.com/grindytech/gafi/blob/master/pallets/gafi-membership/src/tests.rs)

### Prerequisites

Import the Membership pallet and types and derive your runtime's configuration traits from the Membership trait.

License: Apache-2.0
