pub trait UpgradeItem<CollectionId, ItemId, AccountId> {
	/// Upgrade Item
	///
	/// Upgrade an amount of the item by transfer to new item attribute
	///
	///  Parameters:
	/// - `collection`: collection id
	/// - `item`: item id
	/// - `amount`: amount of the item
	fn upgrade_item(
		collection: CollectionId,
		item: ItemId,
		new_item: ItemId,
		amount: u32,
		fee: u128,
	) -> Result<(), ()>;

	/// Approve Upgrade Item
	///
	/// Approve upgrade the amount of item by transfer to new item attribute
	///
	///  Parameters:
	/// - `owner`: item owner
	/// - `collection`: collection id
	/// - `item`: item id
	/// - `amount`: amount of the item
	fn approve_upgrade(
		owner: AccountId,
		collection: CollectionId,
		item: ItemId,
		new_item: ItemId,
		amount: u32,
		fee: u128,
	) -> Result<(), ()>;
}

pub trait TransferItem<CollectionId, ItemId, AccountId> {
	/// Transfer Item
	///
	/// Transfer the amount of item from owner to `destination`
	///
	///  Parameters:
	/// - `collection`: collection id
	/// - `item`: item id
	/// - `amount`: amount of the item
	/// - `destination`: account receive item
	/// - `to`: to account
	fn transfer(
		collection: CollectionId,
		item: ItemId,
		amount: u32,
		destination: AccountId,
	) -> Result<(), ()>;
}

pub trait MutateItem<CollectionId, ItemId, AccountId> {
	/// Issue New Item
	///
	/// Issue new item from collection
	///
	///  Parameters:
	/// - `collection`: collection id
	/// - `item`: item id
	fn issue(collection: CollectionId, item: ItemId, who: &AccountId) -> Result<(), ()>;

	/// Mint Items
	///
	/// Mint the amount of item
	///
	///  Parameters:
	/// - `collection`: collection id
	/// - `item`: item id
	/// - `who`: account receive item
	/// - `amount`: amount of the item
	fn mint(collection: CollectionId, item: ItemId, who: &AccountId, amount: u32)
		-> Result<(), ()>;

	/// Burn Items
	///
	/// Burn the amount of item
	///
	///  Parameters:
	/// - `collection`: collection id
	/// - `item`: item id
	/// - `amount`: amount of the item
	fn burn(collection: CollectionId, item: ItemId, amount: u32) -> Result<(), ()>;
}
