
# Pallet-Game

### Goals
The pallet-game in Gafi is designed to help developers(blockchain or non-blockchain) can use blockchain technologies to improve in-game finances.

## Introduction
The pallet-game is the place that handles all in-game assets, where the game owner can set up collection logic, and define rules for in-game financials.

## Overview

Key functions on pallet-game:
Create collections and items
Allow NFTs can be transferred with/without transfer fees
Allow swap NFT with/without swap fees
Allow NFTs can upgrade their attribute with/without upgrade fees
Set Upgrade Rules
Allow Auto Mint NFTs with/without mining fees

All NFT's function fees transfer directly to the collection owner. 

### Terminology

* **Upgade:** Change NFT attribute (usually better) with or without fees, game owner must set upgrading rules before using this function
* **Mint:** Mint specific items in the collection, usually done by the collection admin/owner
* **Pubic Mint:** Random mine any item in the collection, anyone can mint but with a mining fee


## Interface

### Dispatchable Functions
* `Create Collection`
* `Create Items`
* `Allow Transfer`
* `Allow Swap`
* `Allow Upgrade`
* `Set Upgrade Atributes`
* `Mint`
* `Public Mint`

### Public Functions

## Usage

Please visit the [unittest]()

## Testing
`$ cargo test -p pallet-game`

### Prerequisites


License: Apache-2.0