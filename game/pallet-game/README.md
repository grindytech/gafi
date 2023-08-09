
# Pallet-Game

### Goals
A pallet designed to make it possible for developers (both blockchain and non-blockchain developers) to integrate Web3 technologies to improve in-game finance in minutes at a cost equal to zero.
The studio can publish NFTs and get the rewards from minting fees, upgrading fees, and might be from trading fees in Gafi Marketplace.

## Overview
The pallet-game is where all the assets in the game are handled, where the game owner can set up the game, collection, and NFT rules.
Pallet-game is coupled with [pallet-nfts](https://github.com/grindytech/substrate/tree/master/frame/nfts), and pallet-game is the [ERC-1155](https://eips.ethereum.org/EIPS/eip-1155): Multi Token Standard version.


### Terminology
* **NFTs:** a certain amount of NFT.

* **Bundle:** NFTs might be from different collections wrapped together.

* **Game:** The game is a place to manage limited collections, where collections containing an unlimited number of ERC-1155 NFTs

* **Create game collection:** The creation of a new collection in a game.

* **Create collection:** The creation of a new collection.

* **Add collection:** Add an existing collection to a game.

* **Create item:** Create a certain number of NFTs in a collection.

* **Add item:** Add a certain number of NFTs in a collection.

* **Mint:** Randomly mint a certain amount of NFTs in the collection with a minting fee sent to the collection owner.
The rarity depends on the number of NFTs in reserve.

* **Set upgade:** Define NFT upgrade rules, any upgrade can cost a fee sent to the collection owner.

* **Upgade:** Upgrade an NFT with a new NFT id.

* **NFT transfer:** The action of sending a certain amount of an NFT from one account to another.

* **Set price:** The act of setting a price for a certain quantity of an NFT.

* **Buy NFT:** Buy a certain amount of an NFT from `Set price`.

* **Set buy:** The action of wanting to buy a certain quantity of an NFT.

* **Claim set buy:** Sell a certain number of an NFT for `Set buy`.

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

### Permissionless dispatchables
* `create_collection`: Create a new collection.
* `mint`: Random mint NFTs.
* `burn`: Burn NFTs.
* `transfer`: Transfer NFTs.
* `upgrade_item`: Upgrade NFTs.


### Permissioned dispatchables
* `add_game_collection`: Add exists a collection to a game.
* `create_item`: Create NFTs.
* `add_supply`: Add NFTs.
* `set_upgrade_item`: Set NFT upgrade rules.
* `remove_collection`: Remove a collection from a game.
* `lock_item_transfer`: Lock NFT to prevent any trade.
* `unlock_item_transfer`: Revert the effects of a previous `lock_item_transfer`.
* `create_stable_pool`: Create a minting pool with a constant weight loot table.

### Trade dispatchables
* `set_price`: Set price for NFTs.
* `buy_item`: Buy NFTs from `set_price`.
* `add_retail_supply`: Add NFTs for `set_price`.
* `set_bundle`: Set a price for a bundle.
* `buy_bundle`: Buy a bundle from `set_bundle`.
* `set_buy`: Place a buy trade for NFTs.
* `claim_set_buy`: Sell NFTs for `set_buy`.
* `set_wishlist`: Order a buy-all for a bundle.
* `claim_wishlist`: Sell a bundle for `set_wishlist`.
* `set_swap`: Set an exchange a bundle for a bundle may have an additional cost.
* `claim_swap`: Make an exchange from `set_swap`.
* `set_auction`: Bid for a bundle, starting on a specific block with a minimum bid and duration.
* `bid_auction`: Make a bid.
* `claim_auction`: End the auction when it expires.
* `cancel_trade`: Cancel a trade with trade id.
* `create_dynamic_pool`: Create a minting pool with a dynamic weight loot table.

### Metadata (permissioned) dispatchables
* `set_attribute`: Set a metadata attribute of an item or collection.
* `clear_attribute`: Remove a metadata attribute of an item or collection.
* `set_metadata`: Set general metadata of an item (E.g. an IPFS address of an image url).
* `clear_metadata`: Remove general metadata of an item.
* `set_collection_metadata`: Set general metadata of a collection.
* `clear_collection_metadata`: Remove general metadata of a collection.

### Offchain worker
* `submit_random_seed_unsigned`: the offchain-worker submits a random seed in each block.

## Usage

Please visit the [unittest](https://github.com/grindytech/gafi/blob/master/game/pallet-game/src/tests.rs)

## Testing
`$ cargo test -p pallet-game`

### Prerequisites
[pallet-nfts](https://github.com/grindytech/substrate/tree/master/frame/nfts)
[offchain-worker](https://docs.substrate.io/reference/how-to-guides/offchain-workers/)

License: Apache-2.0