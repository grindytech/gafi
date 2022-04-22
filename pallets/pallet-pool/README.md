# Pallet Pool

Manage the player flow to join and leave the pools, control the use of tickets

## Overview

To use it in your runtime, you need to implement the Config assets

The supported dispatchable functions are documented in the comment

### Terminology

* **TimeService:** The specific period of time to charge service fee for upfront pool and renew the services

### Goals

The master_pool module in Gafi is designed to make the following possible:

* Join Pool
* Leave Pool
* Use tickets
* Renew tickets

## Interface

### Dispatchable Functions
* `join` join pool
* `leave` leave pool

### Public Functions
* `renew_tickets` renew ticket in every 'TimeService'
* `use_ticket` use ticket means decreasing the tx_limit of the ticket
* `get_service` get service detail of ticket


## Usage

Please visit the [unittest](https://github.com/cryptoviet/gafi/blob/master/tests)

### Prerequisites

Import the Pallet Pool module and types and derive your runtime's configuration traits from the Pallet Pool module trait.

License: Apache-2.0
