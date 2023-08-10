//! Benchmarking setup for pallet-pool

use super::*;
#[allow(unused)]
use crate::Pallet as PalletGame;
use crate::{pallet::BenchmarkHelper as GameBenchmarkHelper, Call, Config};
use frame_benchmarking::{account, benchmarks_instance_pallet, Box, Zero};
use frame_support::{assert_ok, dispatch::UnfilteredDispatchable, traits::Currency};
use frame_system::RawOrigin;
use gafi_support::game::{Bundle, Loot, MintSettings, MintType, NFT};
use pallet_nfts::BenchmarkHelper;
use scale_info::prelude::{format, string::String};
use sp_core::Get;
use sp_std::vec;

const UNIT: u128 = 1_000_000_000_000_000_000u128;
const MAX: u32 = 10_u32;

macro_rules! bvec {
	($( $x:tt )*) => {
		vec![$( $x )*].try_into().unwrap()
	}
}

fn string_to_static_str(s: String) -> &'static str {
	Box::leak(s.into_boxed_str())
}

fn new_funded_account<T: Config<I>, I: 'static>(
	value: u32,
	seed: u32,
	amount: u128,
) -> T::AccountId {
	let balance_amount = amount.try_into().ok().unwrap();
	let name: String = format!("{}{}", value, seed);
	let user = account(string_to_static_str(name), value, seed);
	<T as pallet::Config<I>>::Currency::make_free_balance_be(&user, balance_amount);
	return user
}

fn default_item_config() -> ItemConfig {
	ItemConfig::default()
}

fn default_mint_config<T: Config<I>, I: 'static>() -> MintSettingsFor<T, I> {
	MintSettings {
		mint_type: MintType::Public,
		price: <T as pallet::Config<I>>::Currency::minimum_balance(),
		start_block: None,
		end_block: None,
	}
}

fn assert_last_event<T: Config<I>, I: 'static>(generic_event: <T as Config<I>>::RuntimeEvent) {
	let events = frame_system::Pallet::<T>::events();
	let system_event: <T as frame_system::Config>::RuntimeEvent = generic_event.into();
	// compare to the last event record
	let frame_system::EventRecord { event, .. } = &events[events.len() - 1];
	assert_eq!(event, &system_event);
}

fn do_create_game<T: Config<I>, I: 'static>() -> (T::AccountId, T::AccountId) {
	let caller = new_funded_account::<T, I>(0, 0, 1000_000_000u128 * UNIT);
	let admin = new_funded_account::<T, I>(1, 1, 1000_000_000u128 * UNIT);
	assert_ok!(PalletGame::<T, I>::create_game(
		RawOrigin::Signed(caller.clone()).into(),
		T::Lookup::unlookup(admin.clone()),
	));
	(caller, admin)
}

fn do_create_game_collection<T: Config<I>, I: 'static>() -> (T::AccountId, T::AccountId) {
	let (owner, admin) = do_create_game::<T, I>();
	assert_ok!(PalletGame::<T, I>::create_game_collection(
		RawOrigin::Signed(admin.clone()).into(),
		<T as pallet::Config<I>>::Helper::game(0),
	));
	(owner, admin)
}

fn do_create_collection<T: Config<I>, I: 'static>() -> (T::AccountId, T::AccountId) {
	let owner = new_funded_account::<T, I>(0, 0, 1000_000_000u128 * UNIT);
	let admin = new_funded_account::<T, I>(1, 1, 1000_000_000u128 * UNIT);

	assert_ok!(PalletGame::<T, I>::create_collection(
		RawOrigin::Signed(owner.clone()).into(),
		T::Lookup::unlookup(admin.clone()),
	));
	(owner, admin)
}

fn do_create_item<T: Config<I>, I: 'static>(
	admin: &T::AccountId,
	collection: u16,
	item: u16,
	supply: Option<u32>,
) {
	assert_ok!(PalletGame::<T, I>::create_item(
		RawOrigin::Signed(admin.clone()).into(),
		<T as pallet_nfts::Config>::Helper::collection(collection),
		<T as pallet_nfts::Config>::Helper::item(item),
		supply,
	));
}

fn new_account_with_item<T: Config<I>, I: 'static>(
	collection: u16,
) -> (T::AccountId, T::AccountId, T::AccountId) {
	let (owner, admin) = do_create_collection::<T, I>();
	do_create_item::<T, I>(&admin, collection, 0, Some(1000));
	do_create_item::<T, I>(&admin, collection, 1, Some(1000));
	do_create_item::<T, I>(&admin, collection, 2, Some(1000));

	let player = new_funded_account::<T, I>(0, 0, 1000_000_000u128 * UNIT);
	for i in 0..3 {
		assert_ok!(PalletGame::<T, I>::transfer(
			RawOrigin::Signed(owner.clone()).into(),
			<T as pallet_nfts::Config>::Helper::collection(collection),
			<T as pallet_nfts::Config>::Helper::item(i),
			T::Lookup::unlookup(player.clone()),
			1000,
		));
	}
	(player, owner, admin)
}

fn do_create_dynamic_pool<T: Config<I>, I: 'static>() -> (T::AccountId, T::AccountId) {
	let (who, _, _) = new_account_with_item::<T, I>(0);

	let table: LootTable<T::CollectionId, T::ItemId> = vec![
		Loot {
			maybe_nft: Some(NFT {
				collection: <T as pallet_nfts::Config>::Helper::collection(0),
				item: <T as pallet_nfts::Config>::Helper::item(0),
			}),
			weight: 10,
		},
		Loot {
			maybe_nft: Some(NFT {
				collection: <T as pallet_nfts::Config>::Helper::collection(0),
				item: <T as pallet_nfts::Config>::Helper::item(1),
			}),
			weight: 10,
		},
		Loot {
			maybe_nft: Some(NFT {
				collection: <T as pallet_nfts::Config>::Helper::collection(0),
				item: <T as pallet_nfts::Config>::Helper::item(2),
			}),
			weight: 10,
		},
	];

	assert_ok!(PalletGame::<T, I>::create_dynamic_pool(
		RawOrigin::Signed(who.clone()).into(),
		table.clone(),
		T::Lookup::unlookup(who.clone()),
		default_mint_config::<T, I>(),
	));

	(who.clone(), who)
}

fn do_create_stable_pool<T: Config<I>, I: 'static>() -> (T::AccountId, T::AccountId) {
	let (owner, admin) = do_create_collection::<T, I>();
	do_create_item::<T, I>(&admin, 0, 0, None);
	do_create_item::<T, I>(&admin, 0, 1, None);
	do_create_item::<T, I>(&admin, 0, 2, None);

	let table: LootTable<T::CollectionId, T::ItemId> = vec![
		Loot {
			maybe_nft: Some(NFT {
				collection: <T as pallet_nfts::Config>::Helper::collection(0),
				item: <T as pallet_nfts::Config>::Helper::item(0),
			}),
			weight: 10,
		},
		Loot {
			maybe_nft: Some(NFT {
				collection: <T as pallet_nfts::Config>::Helper::collection(0),
				item: <T as pallet_nfts::Config>::Helper::item(1),
			}),
			weight: 10,
		},
		Loot {
			maybe_nft: Some(NFT {
				collection: <T as pallet_nfts::Config>::Helper::collection(0),
				item: <T as pallet_nfts::Config>::Helper::item(2),
			}),
			weight: 10,
		},
	];

	assert_ok!(PalletGame::<T, I>::create_stable_pool(
		RawOrigin::Signed(owner.clone()).into(),
		table.clone(),
		T::Lookup::unlookup(admin.clone()),
		default_mint_config::<T, I>(),
	));

	(owner, admin)
}

fn do_set_upgrade_item<T: Config<I>, I: 'static>(who: &T::AccountId) {
	assert_ok!(PalletGame::<T, I>::set_upgrade_item(
		RawOrigin::Signed(who.clone()).into(),
		<T as pallet_nfts::Config>::Helper::collection(0),
		<T as pallet_nfts::Config>::Helper::item(0),
		<T as pallet_nfts::Config>::Helper::item(100),
		default_item_config(),
		bvec![0u8; 50],
		1,
		<T as pallet::Config<I>>::Currency::minimum_balance(),
	));
}

fn do_set_price<T: Config<I>, I: 'static>(who: &T::AccountId) {
	let package = Package {
		collection: <T as pallet_nfts::Config>::Helper::collection(0),
		item: <T as pallet_nfts::Config>::Helper::item(0),
		amount: 5,
	};

	assert_ok!(PalletGame::<T, I>::set_price(
		RawOrigin::Signed(who.clone()).into(),
		package,
		<T as pallet::Config<I>>::Currency::minimum_balance(),
		None,
		None,
	));
}

fn do_set_bundle<T: Config<I>, I: 'static>(who: &T::AccountId) {
	let bundle = vec![
		Package {
			collection: <T as pallet_nfts::Config>::Helper::collection(0),
			item: <T as pallet_nfts::Config>::Helper::item(0),
			amount: 10,
		},
		Package {
			collection: <T as pallet_nfts::Config>::Helper::collection(0),
			item: <T as pallet_nfts::Config>::Helper::item(1),
			amount: 10,
		},
	];

	assert_ok!(PalletGame::<T, I>::set_bundle(
		RawOrigin::Signed(who.clone()).into(),
		bundle.clone(),
		<T as pallet::Config<I>>::Currency::minimum_balance(),
		None,
		None,
	));
}

fn do_set_wishlist<T: Config<I>, I: 'static>(who: &T::AccountId) {
	let bundle = vec![
		Package {
			collection: <T as pallet_nfts::Config>::Helper::collection(0),
			item: <T as pallet_nfts::Config>::Helper::item(0),
			amount: 10,
		},
		Package {
			collection: <T as pallet_nfts::Config>::Helper::collection(0),
			item: <T as pallet_nfts::Config>::Helper::item(1),
			amount: 10,
		},
	];

	assert_ok!(PalletGame::<T, I>::set_wishlist(
		RawOrigin::Signed(who.clone()).into(),
		bundle,
		<T as pallet::Config<I>>::Currency::minimum_balance(),
		None,
		None,
	));
}

fn do_set_auction<T: Config<I>, I: 'static>() -> T::AccountId {
	let (who, _, _) = new_account_with_item::<T, I>(0);

	let source = vec![
		Package {
			collection: <T as pallet_nfts::Config>::Helper::collection(0),
			item: <T as pallet_nfts::Config>::Helper::item(0),
			amount: 10,
		},
		Package {
			collection: <T as pallet_nfts::Config>::Helper::collection(0),
			item: <T as pallet_nfts::Config>::Helper::item(1),
			amount: 10,
		},
	];

	assert_ok!(PalletGame::<T, I>::set_auction(
		RawOrigin::Signed(who.clone()).into(),
		source,
		Some(<T as pallet::Config<I>>::Currency::minimum_balance()),
		Some(<T as pallet::Config<I>>::Helper::block(0)),
		<T as pallet::Config<I>>::Helper::block(10),
	));
	who
}

benchmarks_instance_pallet! {

	create_game {
		let caller = new_funded_account::<T, I>(0, 0, 1000_000_000u128 * UNIT);
		let admin =  T::Lookup::unlookup(new_funded_account::<T, I>(0, 0, 1000_000_000u128 * UNIT));

		let call = Call::<T, I>::create_game { admin };
	}: { call.dispatch_bypass_filter(RawOrigin::Signed(caller.clone()).into())? }
	verify {
		assert_last_event::<T, I>(Event::GameCreated { who: caller, game: <T as pallet::Config<I>>::Helper::game(0) }.into());
	}


	create_game_collection {
		let (caller, admin) = do_create_game::<T, I>();

		let call = Call::<T, I>::create_game_collection { game: <T as pallet::Config<I>>::Helper::game(0)};
	}: { call.dispatch_bypass_filter(RawOrigin::Signed(admin.clone()).into())? }
	verify {
		assert_last_event::<T, I>(Event::CollectionCreated { who: admin.clone(), collection: <T as pallet_nfts::Config>::Helper::collection(0) }.into());
	}

	create_item {
		let (caller, admin) = do_create_collection::<T, I>();
		let call = Call::<T, I>::create_item { collection: <T as pallet_nfts::Config>::Helper::collection(0),
			item: <T as pallet_nfts::Config>::Helper::item(0),
			maybe_supply: Some(10) };
	}: { call.dispatch_bypass_filter(RawOrigin::Signed(admin.clone()).into())? }
	verify {
		assert_last_event::<T, I>(Event::ItemCreated { who: admin,
			collection: <T as pallet_nfts::Config>::Helper::collection(0),
			item: <T as pallet_nfts::Config>::Helper::item(0), maybe_supply: Some(10) }.into());
	}

	add_supply {
		let (owner, admin) = do_create_collection::<T, I>();
		do_create_item::<T, I>(&admin, 0, 0, Some(10));

		let call = Call::<T, I>::add_supply { collection: <T as pallet_nfts::Config>::Helper::collection(0),
			 item: <T as pallet_nfts::Config>::Helper::item(0), amount: 10 };
	}: { call.dispatch_bypass_filter(RawOrigin::Signed(admin.clone()).into())? }
	verify {
		assert_last_event::<T, I>(Event::ItemAdded { who: admin,
			collection: <T as pallet_nfts::Config>::Helper::collection(0),
			item: <T as pallet_nfts::Config>::Helper::item(0), amount: 10 }.into());
	}

	burn {
		let (who, _, _) = new_account_with_item::<T, I>(0);

		let call = Call::<T, I>::burn { collection: <T as pallet_nfts::Config>::Helper::collection(0),
			item: <T as pallet_nfts::Config>::Helper::item(0),
			amount: 1 };
	}: { call.dispatch_bypass_filter(RawOrigin::Signed(who.clone()).into())? }
	verify {
		assert_last_event::<T, I>(Event::Burned { who: who.clone(),
			collection: <T as pallet_nfts::Config>::Helper::collection(0),
			item: <T as pallet_nfts::Config>::Helper::item(0),
			amount: 1,
			}.into());
	}


	transfer {
		let (who, _, _) = new_account_with_item::<T, I>(0);

		let dest =  T::Lookup::unlookup(new_funded_account::<T, I>(0, 0, 1000_000_000u128 * UNIT));

		let call = Call::<T, I>::transfer { collection: <T as pallet_nfts::Config>::Helper::collection(0),
			item: <T as pallet_nfts::Config>::Helper::item(0),
			dest: dest.clone(),
			amount: 10 };
	}: { call.dispatch_bypass_filter(RawOrigin::Signed(who.clone()).into())? }
	verify {
		assert_last_event::<T, I>(Event::Transferred {
			from: who.clone(),
			collection: <T as pallet_nfts::Config>::Helper::collection(0),
			item: <T as pallet_nfts::Config>::Helper::item(0),
			dest:  T::Lookup::lookup(dest.clone()).unwrap(),
			amount: 10,
		}.into() );
	}

	set_upgrade_item {
		let s in 0 .. <T as pallet_nfts::Config>::StringLimit::get();
		let (_, _, who) = new_account_with_item::<T, I>(0);


		let call = Call::<T, I>::set_upgrade_item {
			collection: <T as pallet_nfts::Config>::Helper::collection(0),
			item: <T as pallet_nfts::Config>::Helper::item(0),
			new_item: <T as pallet_nfts::Config>::Helper::item(100),
			config: default_item_config(),
			data: bvec![0u8; s as usize],
			level: 0,
			fee: <T as pallet::Config<I>>::Currency::minimum_balance(),
			};
	}: { call.dispatch_bypass_filter(RawOrigin::Signed(who.clone()).into())? }
	verify {
		assert_last_event::<T, I>(Event::UpgradeSet {
			who,
			collection: <T as pallet_nfts::Config>::Helper::collection(0),
			item: <T as pallet_nfts::Config>::Helper::item(0),
			new_item: <T as pallet_nfts::Config>::Helper::item(100),
			level: 0,
		}.into() );
	}

	upgrade_item {
		let (who, _, admin) = new_account_with_item::<T, I>(0);

		do_set_upgrade_item::<T, I>(&admin);

		let call = Call::<T, I>::upgrade_item {
			collection: <T as pallet_nfts::Config>::Helper::collection(0),
			item: <T as pallet_nfts::Config>::Helper::item(0),
			amount: 10,
			};

	}: { call.dispatch_bypass_filter(RawOrigin::Signed(who.clone()).into())? }
	verify {
		assert_last_event::<T, I>(Event::Upgraded {
			who,
			collection: <T as pallet_nfts::Config>::Helper::collection(0),
			item: <T as pallet_nfts::Config>::Helper::item(0),
			new_item: <T as pallet_nfts::Config>::Helper::item(100),
			amount: 10,
		}.into() );
	}

	set_price {
		let (who, _, _) = new_account_with_item::<T, I>(0);

		let package = Package {
			collection: <T as pallet_nfts::Config>::Helper::collection(0),
			item: <T as pallet_nfts::Config>::Helper::item(0),
			amount: 10,
		};

		let call = Call::<T, I>::set_price {
			package: package,
			unit_price: <T as pallet::Config<I>>::Currency::minimum_balance(),
			start_block: None,
			end_block: None,
		};
	}: { call.dispatch_bypass_filter(RawOrigin::Signed(who.clone()).into())? }
	verify {
		assert_last_event::<T, I>(Event::PriceSet {
			trade: <T as pallet::Config<I>>::Helper::trade(0),
			who,
			collection: <T as pallet_nfts::Config>::Helper::collection(0),
			item: <T as pallet_nfts::Config>::Helper::item(0),
			amount: 10,
			unit_price:  <T as pallet::Config<I>>::Currency::minimum_balance(),
		}.into() );
	}

	buy_item {
		let (who, _, _) = new_account_with_item::<T, I>(0);

		do_set_price::<T, I>(&who);

		let call = Call::<T, I>::buy_item {
			trade: <T as pallet::Config<I>>::Helper::trade(0),
			amount: 5,
			bid_price: <T as pallet::Config<I>>::Currency::minimum_balance(),
		};
	}: { call.dispatch_bypass_filter(RawOrigin::Signed(who.clone()).into())? }
	verify {
		assert_last_event::<T, I>(Event::ItemBought {
			trade: <T as pallet::Config<I>>::Helper::trade(0),
			who,
			amount: 5,
			bid_unit_price:  <T as pallet::Config<I>>::Currency::minimum_balance(),
		}.into() );
	}

	set_bundle {
		let s in 0 .. <T as pallet::Config<I>>::MaxBundle::get();
		let (who, _, _) = new_account_with_item::<T, I>(0);
		let bundle = vec![
		Package {
			collection: <T as pallet_nfts::Config>::Helper::collection(0),
			item: <T as pallet_nfts::Config>::Helper::item(0),
			amount: 1,
		}; s as usize];

		let call = Call::<T, I>::set_bundle {
			bundle: bundle.clone(),
			price: <T as pallet::Config<I>>::Currency::minimum_balance(),
			start_block: None,
			end_block: None,
		};
	}: { call.dispatch_bypass_filter(RawOrigin::Signed(who.clone()).into())? }
	verify {
		assert_last_event::<T, I>(Event::BundleSet {
			trade: <T as pallet::Config<I>>::Helper::trade(0),
			who,
			bundle,
			price:  <T as pallet::Config<I>>::Currency::minimum_balance(),
		}.into() );
	}

	buy_bundle {
		let (who, _, _) = new_account_with_item::<T, I>(0);
		do_set_bundle::<T, I>(&who);

		let call = Call::<T, I>::buy_bundle {
			trade: <T as pallet::Config<I>>::Helper::trade(0),
			bid_price: <T as pallet::Config<I>>::Currency::minimum_balance(),
		};
	}: { call.dispatch_bypass_filter(RawOrigin::Signed(who.clone()).into())? }
	verify {
		assert_last_event::<T, I>(Event::BundleBought {
			trade: <T as pallet::Config<I>>::Helper::trade(0),
			who,
			bid_price:  <T as pallet::Config<I>>::Currency::minimum_balance(),
		}.into() );
	}

	set_wishlist {
		let s in 0 .. <T as pallet::Config<I>>::MaxBundle::get();
		let (who, _, _) = new_account_with_item::<T, I>(0);
		let bundle = vec![
		Package {
			collection: <T as pallet_nfts::Config>::Helper::collection(0),
			item: <T as pallet_nfts::Config>::Helper::item(0),
			amount: 1,
		}; s as usize];

		let call = Call::<T, I>::set_wishlist {
			bundle: bundle.clone(),
			price: <T as pallet::Config<I>>::Currency::minimum_balance(),
			start_block: None,
			end_block: None,
		};
	}: { call.dispatch_bypass_filter(RawOrigin::Signed(who.clone()).into())? }
	verify {
		assert_last_event::<T, I>(Event::WishlistSet {
			trade: <T as pallet::Config<I>>::Helper::trade(0),
			who,
			wishlist: bundle,
			price:  <T as pallet::Config<I>>::Currency::minimum_balance(),
		}.into() );
	}

	claim_wishlist {
		let player = new_funded_account::<T, I>(1, 1, 1000_000_000u128 * UNIT);
		do_set_wishlist::<T, I>(&player);

		let (who, _, _) = new_account_with_item::<T, I>(0);

		let call = Call::<T, I>::claim_wishlist {
			trade: <T as pallet::Config<I>>::Helper::trade(0),
			ask_price: <T as pallet::Config<I>>::Currency::minimum_balance(),
		};
	}: { call.dispatch_bypass_filter(RawOrigin::Signed(who.clone()).into())? }
	verify {
		assert_last_event::<T, I>(Event::WishlistFilled {
			trade: <T as pallet::Config<I>>::Helper::trade(0),
			who,
			ask_price:  <T as pallet::Config<I>>::Currency::minimum_balance(),
		}.into() );
	}

	remove_collection {
		let (owner, who) = do_create_game_collection::<T, I>();

		let call = Call::<T, I>::remove_collection {
			game: <T as pallet::Config<I>>::Helper::game(0),
			collection: <T as pallet_nfts::Config>::Helper::collection(0),
		};
	}: { call.dispatch_bypass_filter(RawOrigin::Signed(who.clone()).into())? }
	verify {
		assert_last_event::<T, I>(Event::CollectionRemoved {
			who,
			game: <T as pallet::Config<I>>::Helper::game(0),
			collection: <T as pallet_nfts::Config>::Helper::collection(0),
		}.into() );
	}

	set_swap {
		let s in 0 .. <T as pallet::Config<I>>::MaxBundle::get();
		let x in 0 .. <T as pallet::Config<I>>::MaxBundle::get();

		let (who, _, _) = new_account_with_item::<T, I>(0);
		let bundle = vec![
		Package {
			collection: <T as pallet_nfts::Config>::Helper::collection(0),
			item: <T as pallet_nfts::Config>::Helper::item(0),
			amount: 1,
		}; s as usize];

		let required = vec![
		Package {
			collection: <T as pallet_nfts::Config>::Helper::collection(0),
			item: <T as pallet_nfts::Config>::Helper::item(0),
			amount: 1,
		}; x as usize];

		let call = Call::<T, I>::set_swap {
			source: bundle.clone(),
			required: required.clone(),
			maybe_price: Some(<T as pallet::Config<I>>::Currency::minimum_balance()),
			start_block: None,
			end_block: None,
		};
	}: { call.dispatch_bypass_filter(RawOrigin::Signed(who.clone()).into())? }
	verify {
		assert_last_event::<T, I>(Event::SwapSet {
			trade: <T as pallet::Config<I>>::Helper::trade(0),
			who,
			source:  bundle,
			required: required.clone(),
			maybe_price: Some(<T as pallet::Config<I>>::Currency::minimum_balance()),
		}.into() );
	}

	claim_swap {
		let (player1, _, _) = new_account_with_item::<T, I>(0);
		let (player2, _, _) = new_account_with_item::<T, I>(1);

		let source = vec![
		Package {
			collection: <T as pallet_nfts::Config>::Helper::collection(0),
			item: <T as pallet_nfts::Config>::Helper::item(0),
			amount: 10,
		},
		Package {
			collection: <T as pallet_nfts::Config>::Helper::collection(0),
			item: <T as pallet_nfts::Config>::Helper::item(1),
			amount: 10,
		}];

		let required = vec![
		Package {
			collection: <T as pallet_nfts::Config>::Helper::collection(1),
			item: <T as pallet_nfts::Config>::Helper::item(0),
			amount: 10,
		},
		Package {
			collection: <T as pallet_nfts::Config>::Helper::collection(1),
			item: <T as pallet_nfts::Config>::Helper::item(1),
			amount: 10,
		}];

		assert_ok!(PalletGame::<T, I>::set_swap(
			RawOrigin::Signed(player1.clone()).into(),
			source.clone(),
			required.clone(),
			Some(<T as pallet::Config<I>>::Currency::minimum_balance()),
			None,
			None,
		));

		let call = Call::<T, I>::claim_swap {
			trade: <T as pallet::Config<I>>::Helper::trade(0),
			maybe_bid_price: Some(<T as pallet::Config<I>>::Currency::minimum_balance()),
		};
	}: { call.dispatch_bypass_filter(RawOrigin::Signed(player2.clone()).into())? }
	verify {
		assert_last_event::<T, I>(Event::SwapClaimed {
			trade:  <T as pallet::Config<I>>::Helper::trade(0),
			who: player2.clone(),
			maybe_bid_price: Some(<T as pallet::Config<I>>::Currency::minimum_balance()),
		}.into() );
	}

	set_auction {
		let s in 0 .. <T as pallet::Config<I>>::MaxBundle::get();
		let (who, _, _) = new_account_with_item::<T, I>(0);
		let source = vec![
		Package {
			collection: <T as pallet_nfts::Config>::Helper::collection(0),
			item: <T as pallet_nfts::Config>::Helper::item(0),
			amount: 1,
		}; s as usize];

		let call = Call::<T, I>::set_auction {
			source: source.clone(),
			maybe_price: Some(<T as pallet::Config<I>>::Currency::minimum_balance()),
			start_block: Some(<T as pallet::Config<I>>::Helper::block(0)),
			duration:  <T as pallet::Config<I>>::Helper::block(10),
		};
	}: { call.dispatch_bypass_filter(RawOrigin::Signed(who.clone()).into())? }
	verify {
		assert_last_event::<T, I>(Event::AuctionSet {
			trade: <T as pallet::Config<I>>::Helper::trade(0),
			who,
			source:  source,
			maybe_price: Some(<T as pallet::Config<I>>::Currency::minimum_balance()),
			start_block:  Some(<T as pallet::Config<I>>::Helper::block(0)),
			duration:  <T as pallet::Config<I>>::Helper::block(10),
		}.into() );
	}

	bid_auction {
		let _ = do_set_auction::<T, I>();

		let caller = new_funded_account::<T, I>(0, 0, 1000_000_000u128 * UNIT);

		let call = Call::<T, I>::bid_auction {
			trade: <T as pallet::Config<I>>::Helper::trade(0),
			bid: <T as pallet::Config<I>>::Currency::minimum_balance(),
		};
	}: { call.dispatch_bypass_filter(RawOrigin::Signed(caller.clone()).into())? }
	verify {
		assert_last_event::<T, I>(Event::Bid {
			trade: <T as pallet::Config<I>>::Helper::trade(0),
			who: caller,
			bid:<T as pallet::Config<I>>::Currency::minimum_balance(),
		}.into() );
	}

	claim_auction {
		let _ = do_set_auction::<T, I>();

		let bidder = new_funded_account::<T, I>(0, 0, 1000_000_000u128 * UNIT);
		assert_ok!(PalletGame::<T, I>::bid_auction(
			RawOrigin::Signed(bidder.clone()).into(),
			<T as pallet::Config<I>>::Helper::trade(0),
			<T as pallet::Config<I>>::Currency::minimum_balance()
		));

		let caller = new_funded_account::<T, I>(0, 0, 1000_000_000u128 * UNIT);

		frame_system::Pallet::<T>::set_block_number(<T as pallet::Config<I>>::Helper::block(10));

		let call = Call::<T, I>::claim_auction {
			trade: <T as pallet::Config<I>>::Helper::trade(0),
		};
	}: { call.dispatch_bypass_filter(RawOrigin::Signed(caller.clone()).into())? }
	verify {
		assert_last_event::<T, I>(Event::AuctionClaimed {
			trade: <T as pallet::Config<I>>::Helper::trade(0),
			maybe_bid: Some((bidder, <T as pallet::Config<I>>::Currency::minimum_balance())),
		}.into() );
	}

	set_buy {
		let caller = new_funded_account::<T, I>(0, 0, 1000_000_000u128 * UNIT);

		let package = Package {
			collection: <T as pallet_nfts::Config>::Helper::collection(0),
			item: <T as pallet_nfts::Config>::Helper::item(0),
			amount: 10,
		};

		let call = Call::<T, I>::set_buy {
			package: package.clone(),
			unit_price: <T as pallet::Config<I>>::Currency::minimum_balance(),
			start_block: None,
			end_block: None,
		};
	}: { call.dispatch_bypass_filter(RawOrigin::Signed(caller.clone()).into())? }
	verify {
		assert_last_event::<T, I>(Event::BuySet {
			trade: <T as pallet::Config<I>>::Helper::trade(0),
			who: caller,
			collection: package.collection,
			item: package.item,
			amount: package.amount,
			unit_price:  <T as pallet::Config<I>>::Currency::minimum_balance(),
		}.into() );
	}

	claim_set_buy {
		let (who, _, _) = new_account_with_item::<T, I>(0);

		let player = new_funded_account::<T, I>(0, 0, 1000_000_000u128 * UNIT);
		let package = Package {
			collection: <T as pallet_nfts::Config>::Helper::collection(0),
			item: <T as pallet_nfts::Config>::Helper::item(0),
			amount: 10,
		};
		assert_ok!(PalletGame::<T, I>::set_buy(
			RawOrigin::Signed(player.clone()).into(),
			package.clone(),
			<T as pallet::Config<I>>::Currency::minimum_balance(),
			None,
			None,
		));

		let call = Call::<T, I>::claim_set_buy {
			trade: <T as pallet::Config<I>>::Helper::trade(0),
			amount: 10,
			ask_price: <T as pallet::Config<I>>::Currency::minimum_balance(),
		};
	}: { call.dispatch_bypass_filter(RawOrigin::Signed(who.clone()).into())? }
	verify {
		assert_last_event::<T, I>(Event::SetBuyClaimed {
			trade: <T as pallet::Config<I>>::Helper::trade(0),
			who,
			amount: 10,
			ask_unit_price:  <T as pallet::Config<I>>::Currency::minimum_balance(),
		}.into() );
	}

	create_collection{
		let caller = new_funded_account::<T, I>(0, 0, 1000_000_000u128 * UNIT);

		let call = Call::<T, I>::create_collection {
			admin:  T::Lookup::unlookup(caller.clone()),
		};
	}: { call.dispatch_bypass_filter(RawOrigin::Signed(caller.clone()).into())? }
	verify {
		assert_last_event::<T, I>(Event::CollectionCreated {
			who: caller,
			collection: <T as pallet_nfts::Config>::Helper::collection(0),
		}.into() );
	}

	set_accept_adding {
		let who = new_funded_account::<T, I>(0, 0, 1000_000_000u128 * UNIT);

		assert_ok!(PalletGame::<T, I>::create_collection(RawOrigin::Signed(who.clone()).into(),
		T::Lookup::unlookup(who.clone()),
		));

		let call = Call::<T, I>::set_accept_adding {
			game: <T as pallet::Config<I>>::Helper::game(0),
			collection:<T as pallet_nfts::Config>::Helper::collection(0),
		};
	}: { call.dispatch_bypass_filter(RawOrigin::Signed(who.clone()).into())? }
	verify {
		assert_last_event::<T, I>(Event::AddingAcceptanceSet {
			who,
			game: <T as pallet::Config<I>>::Helper::game(0),
			collection: <T as pallet_nfts::Config>::Helper::collection(0),
		}.into() );
	}

	add_game_collection {
		let (_, who) = do_create_game::<T, I>();
		let player = new_funded_account::<T, I>(0, 0, 1000_000_000u128 * UNIT);

		assert_ok!(PalletGame::<T, I>::create_collection(
			RawOrigin::Signed(player.clone()).into(),
			T::Lookup::unlookup(player.clone()),
		));

		assert_ok!(PalletGame::<T, I>::set_accept_adding(
			RawOrigin::Signed(player.clone()).into(),
			<T as pallet::Config<I>>::Helper::game(0),
			<T as pallet_nfts::Config>::Helper::collection(0),
		));

		let call = Call::<T, I>::add_game_collection {
			game: <T as pallet::Config<I>>::Helper::game(0),
			collection:<T as pallet_nfts::Config>::Helper::collection(0),
		};
	}: { call.dispatch_bypass_filter(RawOrigin::Signed(who.clone()).into())? }
	verify {
		assert_last_event::<T, I>(Event::CollectionAdded {
			who,
			game: <T as pallet::Config<I>>::Helper::game(0),
			collection: <T as pallet_nfts::Config>::Helper::collection(0),
		}.into() );
	}

	add_retail_supply {
		let (who, _, _) = new_account_with_item::<T, I>(0);
		do_set_price::<T, I>(&who);
		let package = Package {
			collection: <T as pallet_nfts::Config>::Helper::collection(0),
			item: <T as pallet_nfts::Config>::Helper::item(0),
			amount: 5,
		};

		let call = Call::<T, I>::add_retail_supply {
			trade: <T as pallet::Config<I>>::Helper::trade(0),
			supply: package,
		};
	}:  { call.dispatch_bypass_filter(RawOrigin::Signed(who.clone()).into())? }

	cancel_trade {
		let (who, _, _) = new_account_with_item::<T, I>(0);
		do_set_bundle::<T, I>(&who);

		let call = Call::<T, I>::cancel_trade {
			trade: <T as pallet::Config<I>>::Helper::trade(0),
			trade_type: TradeType::Bundle
		};
	}:  { call.dispatch_bypass_filter(RawOrigin::Signed(who.clone()).into())? }
	verify {
		assert_last_event::<T, I>(Event::TradeCanceled {
			who,
			trade: <T as pallet::Config<I>>::Helper::trade(0),
		}.into() );
	}

	create_dynamic_pool {
		let s in 0 .. <T as pallet::Config<I>>::MaxLoot::get();
		let (who, _, _) = new_account_with_item::<T, I>(0);
		let table = vec![
			Loot {
				maybe_nft: Some(NFT {
					collection: <T as pallet_nfts::Config>::Helper::collection(0),
					item: <T as pallet_nfts::Config>::Helper::item(0),
				}),
				weight: 10,
		}; s as usize];

		let call = Call::<T, I>::create_dynamic_pool {
			loot_table: table.clone(),
			mint_settings: default_mint_config::<T, I>(),
			admin: T::Lookup::unlookup(who.clone()),
		};
	}:  { call.dispatch_bypass_filter(RawOrigin::Signed(who.clone()).into())? }
	verify {
		assert_last_event::<T, I>(Event::MiningPoolCreated {
			pool: <T as pallet::Config<I>>::Helper::pool(0),
			who,
			pool_type: PoolType::Dynamic,
			table,
		}.into() );
	}

	create_stable_pool {
		let s in 0 .. <T as pallet::Config<I>>::MaxLoot::get();
		let (who, admin) = do_create_collection::<T, I>();
		do_create_item::<T, I>(&admin, 0, 0, None);
		do_create_item::<T, I>(&admin, 0, 1, None);
		do_create_item::<T, I>(&admin, 0, 2, None);

		let table = vec![
			Loot {
				maybe_nft: Some(NFT {
					collection: <T as pallet_nfts::Config>::Helper::collection(0),
					item: <T as pallet_nfts::Config>::Helper::item(0),
				}),
				weight: 10,
		}; s as usize];

		let call = Call::<T, I>::create_stable_pool {
			loot_table: table.clone(),
			mint_settings: default_mint_config::<T, I>(),
			admin: T::Lookup::unlookup(who.clone()),
		};
	}:  { call.dispatch_bypass_filter(RawOrigin::Signed(who.clone()).into())? }
	verify {
		assert_last_event::<T, I>(Event::MiningPoolCreated {
			pool: <T as pallet::Config<I>>::Helper::pool(0),
			who,
			pool_type: PoolType::Stable,
			table,
		}.into() );
	}

	request_mint {
		do_create_stable_pool::<T, I>();
		let miner = new_funded_account::<T, I>(0, 0, 1000_000_000u128 * UNIT);
		let mint_to =   T::Lookup::unlookup(miner.clone());

		let call = Call::<T, I>::request_mint {
			pool: <T as pallet::Config<I>>::Helper::pool(0),
			mint_to: mint_to.clone(), amount: 10 };
	}: { call.dispatch_bypass_filter(RawOrigin::Signed(miner.clone()).into())? }
	verify {
		assert_last_event::<T, I>(Event::RequestMint {
			who: miner.clone(),
			pool: <T as pallet::Config<I>>::Helper::pool(0),
			target: T::Lookup::lookup(mint_to.clone()).unwrap(),
			block_number: <T as pallet::Config<I>>::Helper::block(3),
			// nfts: vec![ NFT{collection: <T as pallet_nfts::Config>::Helper::collection(0), item: <T as pallet_nfts::Config>::Helper::item(0)}; 10]
		}.into());
	}
}
