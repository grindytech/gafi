
# Pallet-Game

### Goals
The pallet game is designed to make it possible for developers (blockchain or non-blockchain) to use blockchain technologies to improve in-game finance.

## Introduction
The pallet-game is where all the assets in the game are handled, where the game owner can set up the game, collection, and NFT rules.
Pallet-game is coupled with (pallet-nfts)[https://github.com/paritytech/substrate/tree/master/frame/nfts], where pallet-game is the (ERC-1155)[https://eips.ethereum.org/EIPS/eip-1155]: Multi Token Standard version.

## Overview

Key functions on pallet-game:
Create collections and NFTs
Allow NFTs can be transferred with/without transfer fees
Allow swap NFT with/without swap fees
Allow NFTs can upgrade their attribute with/without upgrade fees
Set Upgrade Rules
Allow Auto Mint NFTs with/without mining fees

All NFT's function fees transfer directly to the collection owner. 

### Terminology
* **Game:** The game is a place to manage limited collections, where collections containing an unlimited number of ERC-1155 NFTs

* **Create game collection:** The creation of a new collection in a game.

* **Create collection:** The creation of a new collection.

* **Add collection:** Add an existing collection to a game.

* **Create item:** Create a certain number of NFTs in a collection.

* **Add item:** Add a certain number of NFTs in a collection.

* **Mint:** Randomly mint a certain amount of NFTs in the collection with a mining fee sent to the collection owner.
The rarity depends on the number of NFTs in reserve.

* **Set upgade:** Define NFT upgrade rules, any upgrade can cost a fee sent to the collection owner.

* **Upgade:** Upgrade an NFT with a new NFT id.

* **NFT transfer:** The action of sending a certain amount of an NFT from one account to another.

* **Set price:** The act of setting a price for a certain quantity of an NFT.

* **Buy NFT:** Buy a certain amount of an NFT from `Set price`.

* **Set buy:** The action of wanting to buy a certain quantity of an NFT.

* **Claim set buy:** Sell a certain number of NFTs for `Set buy`.

* **Bundle:** NFTs might be from different collections wrapped together.

* **Set bundle:** The act of setting the all-in price for a bundle.

* **Buy bundle:** The act of purchasing a bundle from `Set bundle`.

* **Set wishlist:** The act of wanting to buy a bundle at a price.

* **Fill wishlist:** The act of selling a bundle for `Set wishlist`.

* **Set swap:** The act of setting up an exchange of a bundle for a bundle that can have an additional price.

* **Claim swap:** The act of accepting an exchange from `Set swap`.

* **Set auction:** The act of setting up an auction for a bundle that can have a minimum bid.
The highest bid when the auction expired is the winner.

* **Bid auction:** The act of placing a bid in an auction, the bid must be higher than the previous bid.

* **Claim auction:** The act of ending the auction when the auction period expires.

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