//! Benchmarking setup for pallet-pool

use super::*;
#[allow(unused)]
use crate::Pallet as PalletGame;
use crate::{pallet::BenchmarkHelper as GameBenchmarkHelper, Call, Config};
use frame_benchmarking::{account, benchmarks_instance_pallet, Box, Zero};
use frame_support::{assert_ok, dispatch::UnfilteredDispatchable, traits::Currency};
use frame_system::RawOrigin;
use gafi_support::game::Bundle;
use pallet_nfts::BenchmarkHelper;
use scale_info::prelude::{format, string::String};
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

fn new_funded_account<T: Config<I>, I: 'static>(s: u32, seed: u32, amount: u128) -> T::AccountId {
	let balance_amount = amount.try_into().ok().unwrap();
	let name: String = format!("{}{}", s, seed);
	let user = account(string_to_static_str(name), s, seed);
	<T as pallet::Config<I>>::Currency::make_free_balance_be(&user, balance_amount);
	return user
}

fn default_item_config() -> ItemConfig {
	ItemConfig::default()
}

fn assert_last_event<T: Config<I>, I: 'static>(generic_event: <T as Config<I>>::RuntimeEvent) {
	let events = frame_system::Pallet::<T>::events();
	let system_event: <T as frame_system::Config>::RuntimeEvent = generic_event.into();
	// compare to the last event record
	let frame_system::EventRecord { event, .. } = &events[events.len() - 1];
	assert_eq!(event, &system_event);
}

fn do_create_game<T: Config<I>, I: 'static>(s: u32) -> (T::AccountId, AccountIdLookupOf<T>) {
	let caller = new_funded_account::<T, I>(s, s, 1000_000_000u128 * UNIT);
	let admin = T::Lookup::unlookup(caller.clone());
	assert_ok!(PalletGame::<T, I>::create_game(
		RawOrigin::Signed(caller.clone()).into(),
		admin.clone(),
	));
	(caller, admin)
}

fn do_create_collection<T: Config<I>, I: 'static>(
	s: u32,
	game: u16,
) -> (T::AccountId, AccountIdLookupOf<T>) {
	let (caller, admin) = do_create_game::<T, I>(s);

	assert_ok!(PalletGame::<T, I>::create_game_collection(
		RawOrigin::Signed(caller.clone()).into(),
		<T as pallet::Config<I>>::Helper::game(game),
		<T as pallet::Config<I>>::Currency::minimum_balance(),
	));
	(caller, admin)
}

fn do_create_item<T: Config<I>, I: 'static>(
	s: u32,
	game: u16,
	collection: u16,
	item: u16,
	amount: u32,
) -> (T::AccountId, AccountIdLookupOf<T>) {
	let (caller, admin) = do_create_collection::<T, I>(s, game);

	assert_ok!(PalletGame::<T, I>::create_item(
		RawOrigin::Signed(caller.clone()).into(),
		<T as pallet_nfts::Config>::Helper::collection(collection),
		<T as pallet_nfts::Config>::Helper::item(item),
		default_item_config(),
		amount,
	));
	(caller, admin)
}

fn do_mint_item<T: Config<I>, I: 'static>(
	s: u32,
	miner: &T::AccountId,
	collection: u16,
	amount: u32,
) {
	assert_ok!(PalletGame::<T, I>::mint(
		RawOrigin::Signed(miner.clone()).into(),
		<T as pallet_nfts::Config>::Helper::collection(collection),
		T::Lookup::unlookup(miner.clone()),
		amount,
	));
}

fn do_set_upgrade_item<T: Config<I>, I: 'static>(s: u32, who: &T::AccountId) {
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

fn do_set_price<T: Config<I>, I: 'static>(s: u32) -> T::AccountId {
	let (_, _) = do_create_item::<T, I>(s, 0, 0, 0, 10);
	let seller = new_funded_account::<T, I>(s, s, 1000_000_000u128 * UNIT);

	do_mint_item::<T, I>(s, &seller, 0, 10);

	let package = Package {
		collection: <T as pallet_nfts::Config>::Helper::collection(0),
		item: <T as pallet_nfts::Config>::Helper::item(0),
		amount: 5,
	};

	assert_ok!(PalletGame::<T, I>::set_price(
		RawOrigin::Signed(seller.clone()).into(),
		package,
		<T as pallet::Config<I>>::Currency::minimum_balance(),
	));
	seller
}

fn do_set_bundle<T: Config<I>, I: 'static>(
	s: u32,
) -> (T::AccountId, Bundle<T::CollectionId, T::ItemId>) {
	let (_, _) = do_create_item::<T, I>(s, 0, 0, 0, 10);
	let (_, _) = do_create_item::<T, I>(s, 0, 0, 1, 10);

	let seller = new_funded_account::<T, I>(s, s, 1000_000_000u128 * UNIT);
	do_mint_item::<T, I>(s, &seller, 0, 10);
	do_mint_item::<T, I>(s, &seller, 0, 10);

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
		RawOrigin::Signed(seller.clone()).into(),
		bundle.clone(),
		<T as pallet::Config<I>>::Currency::minimum_balance(),
	));

	(seller, bundle)
}

fn do_set_wishlist<T: Config<I>, I: 'static>(s: u32) -> T::AccountId {
	let caller = new_funded_account::<T, I>(s, s, 1000_000_000u128 * UNIT);

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
		RawOrigin::Signed(caller.clone()).into(),
		bundle,
		<T as pallet::Config<I>>::Currency::minimum_balance(),
	));
	caller
}

fn do_set_auction<T: Config<I>, I: 'static>(s: u32) -> T::AccountId {
	let caller = new_funded_account::<T, I>(s, s, 1000_000_000u128 * UNIT);

	let (_, _) = do_create_item::<T, I>(s, 0, 0, 0, 10);
	let (_, _) = do_create_item::<T, I>(s, 0, 0, 1, 10);

	do_mint_item::<T, I>(s, &caller, 0, 10);
	do_mint_item::<T, I>(s, &caller, 0, 10);

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
		RawOrigin::Signed(caller.clone()).into(),
		source,
		Some(<T as pallet::Config<I>>::Currency::minimum_balance()),
		<T as pallet::Config<I>>::Helper::block(0),
		<T as pallet::Config<I>>::Helper::block(10),
	));
	caller
}

benchmarks_instance_pallet! {

	create_game {
		let s in 0 .. MAX as u32;
		let caller = new_funded_account::<T, I>(s, s, 1000_000_000u128 * UNIT);
		let admin =  T::Lookup::unlookup(new_funded_account::<T, I>(s + MAX, MAX, 1000_000_000u128 * UNIT));

		let call = Call::<T, I>::create_game { admin };
	}: { call.dispatch_bypass_filter(RawOrigin::Signed(caller.clone()).into())? }
	verify {
		assert_last_event::<T, I>(Event::GameCreated { who: caller, game: <T as pallet::Config<I>>::Helper::game(0) }.into());
	}


	create_game_collection {
		let s in 0 .. MAX as u32;
		let (caller, admin) = do_create_game::<T, I>(s);
		let call = Call::<T, I>::create_game_collection { game: <T as pallet::Config<I>>::Helper::game(0), fee: <T as pallet::Config<I>>::Currency::minimum_balance() };
	}: { call.dispatch_bypass_filter(RawOrigin::Signed(caller.clone()).into())? }
	verify {
		assert_last_event::<T, I>(Event::CollectionCreated { who: caller.clone(), collection: <T as pallet_nfts::Config>::Helper::collection(0) }.into());
	}

	create_item {
		let s in 0 .. MAX as u32;
		let (caller, admin) = do_create_collection::<T, I>(s, 0);
		let call = Call::<T, I>::create_item { collection: <T as pallet_nfts::Config>::Helper::collection(0),
			 item: <T as pallet_nfts::Config>::Helper::item(0),
			config: default_item_config(), amount: 10 };
	}: { call.dispatch_bypass_filter(RawOrigin::Signed(caller.clone()).into())? }
	verify {
		assert_last_event::<T, I>(Event::ItemCreated { who: caller,
			collection: <T as pallet_nfts::Config>::Helper::collection(0),
			item: <T as pallet_nfts::Config>::Helper::item(0), amount: 10 }.into());
	}

	add_item {
		let s in 0 .. MAX as u32;
		let (caller, admin) = do_create_item::<T, I>(s, 0, 0, 0, 10);
		let call = Call::<T, I>::add_item { collection: <T as pallet_nfts::Config>::Helper::collection(0),
			 item: <T as pallet_nfts::Config>::Helper::item(0), amount: 10 };
	}: { call.dispatch_bypass_filter(RawOrigin::Signed(caller.clone()).into())? }
	verify {
		assert_last_event::<T, I>(Event::ItemAdded { who: caller,
			collection: <T as pallet_nfts::Config>::Helper::collection(0),
			item: <T as pallet_nfts::Config>::Helper::item(0), amount: 10 }.into());
	}

	mint {
		let s in 0 .. MAX as u32;
		let (caller, admin) = do_create_item::<T, I>(s, 0, 0, 0, 10);

		let miner = new_funded_account::<T, I>(s, s, 1000_000_000u128 * UNIT);
		let mint_to =   T::Lookup::unlookup(miner.clone());

		let call = Call::<T, I>::mint { collection: <T as pallet_nfts::Config>::Helper::collection(0),
			mint_to: mint_to.clone(), amount: 1 };
	}: { call.dispatch_bypass_filter(RawOrigin::Signed(caller).into())? }
	verify {
		assert_last_event::<T, I>(Event::Minted { who: miner.clone(),
			target: T::Lookup::lookup(mint_to.clone()).unwrap(),
			collection: <T as pallet_nfts::Config>::Helper::collection(0),
			items: vec![<T as pallet_nfts::Config>::Helper::item(0)] }.into());
	}

	burn {
		let s in 0 .. MAX as u32;
		let (owner, admin) = do_create_item::<T, I>(s, 0, 0, 0, 10);
		let miner = new_funded_account::<T, I>(s, s, 1000_000_000u128 * UNIT);
		do_mint_item::<T, I>(s, &miner, 0, 10);

		let call = Call::<T, I>::burn { collection: <T as pallet_nfts::Config>::Helper::collection(0),
			item: <T as pallet_nfts::Config>::Helper::item(0),
			amount: 1 };
	}: { call.dispatch_bypass_filter(RawOrigin::Signed(miner.clone()).into())? }
	verify {
		assert_last_event::<T, I>(Event::Burned { who: miner.clone(),
			collection: <T as pallet_nfts::Config>::Helper::collection(0),
			item: <T as pallet_nfts::Config>::Helper::item(0),
			amount: 1,
			}.into());
	}


	transfer {
		let s in 0 .. MAX as u32;
		let (owner, admin) = do_create_item::<T, I>(s, 0, 0, 0, 10);

		let miner = new_funded_account::<T, I>(s, s, 1000_000_000u128 * UNIT);
		do_mint_item::<T, I>(s, &miner, 0, 10);

		let dest =  T::Lookup::unlookup(new_funded_account::<T, I>(s + MAX, s + MAX, 1000_000_000u128 * UNIT));

		let call = Call::<T, I>::transfer { collection: <T as pallet_nfts::Config>::Helper::collection(0),
			item: <T as pallet_nfts::Config>::Helper::item(0),
			dest: dest.clone(),
			amount: 10 };
	}: { call.dispatch_bypass_filter(RawOrigin::Signed(miner.clone()).into())? }
	verify {
		assert_last_event::<T, I>(Event::Transferred {
			from: miner.clone(),
			collection: <T as pallet_nfts::Config>::Helper::collection(0),
			item: <T as pallet_nfts::Config>::Helper::item(0),
			dest:  T::Lookup::lookup(dest.clone()).unwrap(),
			amount: 10,
		}.into() );
	}

	set_upgrade_item {
		let s in 0 .. MAX as u32;
		let (caller, admin) = do_create_item::<T, I>(s, 0, 0, 0, 10);

		let call = Call::<T, I>::set_upgrade_item {
			collection: <T as pallet_nfts::Config>::Helper::collection(0),
			item: <T as pallet_nfts::Config>::Helper::item(0),
			new_item: <T as pallet_nfts::Config>::Helper::item(100),
			config: default_item_config(),
			data: bvec![0u8; 50],
			level: 0,
			fee: <T as pallet::Config<I>>::Currency::minimum_balance(),
			};
	}: { call.dispatch_bypass_filter(RawOrigin::Signed(caller.clone()).into())? }
	verify {
		assert_last_event::<T, I>(Event::UpgradeSet {
			who: caller,
			collection: <T as pallet_nfts::Config>::Helper::collection(0),
			item: <T as pallet_nfts::Config>::Helper::item(0),
			new_item: <T as pallet_nfts::Config>::Helper::item(100),
			level: 0,
		}.into() );
	}

	upgrade_item {
		let s in 0 .. MAX as u32;

		let (owner, admin) = do_create_item::<T, I>(s, 0, 0, 0, 10);
		let miner = new_funded_account::<T, I>(s, s, 1000_000_000u128 * UNIT);
		do_mint_item::<T, I>(s, &miner, 0, 10);

		do_set_upgrade_item::<T, I>(s, &owner);

		let call = Call::<T, I>::upgrade_item {
			collection: <T as pallet_nfts::Config>::Helper::collection(0),
			item: <T as pallet_nfts::Config>::Helper::item(0),
			amount: 10,
			};

	}: { call.dispatch_bypass_filter(RawOrigin::Signed(miner.clone()).into())? }
	verify {
		assert_last_event::<T, I>(Event::Upgraded {
			who: miner.clone(),
			collection: <T as pallet_nfts::Config>::Helper::collection(0),
			item: <T as pallet_nfts::Config>::Helper::item(0),
			new_item: <T as pallet_nfts::Config>::Helper::item(100),
			amount: 10,
		}.into() );
	}

	set_price {
		let s in 0 .. MAX as u32;
		let (owner, admin) = do_create_item::<T, I>(s, 0, 0, 0, 10);

		let caller = new_funded_account::<T, I>(s, s, 1000_000_000u128 * UNIT);
		do_mint_item::<T, I>(s, &caller, 0, 10);

		let package = Package {
			collection: <T as pallet_nfts::Config>::Helper::collection(0),
			item: <T as pallet_nfts::Config>::Helper::item(0),
			amount: 10,
		};

		let call = Call::<T, I>::set_price {
			package: package,
			unit_price: <T as pallet::Config<I>>::Currency::minimum_balance(),
		};
	}: { call.dispatch_bypass_filter(RawOrigin::Signed(caller.clone()).into())? }
	verify {
		assert_last_event::<T, I>(Event::PriceSet {
			trade: <T as pallet::Config<I>>::Helper::trade(0),
			who: caller.clone(),
			collection: <T as pallet_nfts::Config>::Helper::collection(0),
			item: <T as pallet_nfts::Config>::Helper::item(0),
			amount: 10,
			unit_price:  <T as pallet::Config<I>>::Currency::minimum_balance(),
		}.into() );
	}

	buy_item {
		let s in 0 .. MAX as u32;

		let seller = do_set_price::<T, I>(s);
		let caller = new_funded_account::<T, I>(s, s, 1000_000_000u128 * UNIT);

		let call = Call::<T, I>::buy_item {
			trade: <T as pallet::Config<I>>::Helper::trade(0),
			amount: 5,
			bid_price: <T as pallet::Config<I>>::Currency::minimum_balance(),
		};
	}: { call.dispatch_bypass_filter(RawOrigin::Signed(caller.clone()).into())? }
	verify {
		assert_last_event::<T, I>(Event::ItemBought {
			trade: <T as pallet::Config<I>>::Helper::trade(0),
			who: caller,
			amount: 5,
			bid_unit_price:  <T as pallet::Config<I>>::Currency::minimum_balance(),
		}.into() );
	}

	set_bundle {
		let s in 0 .. MAX as u32;
		let (_, _) = do_create_item::<T, I>(s, 0, 0, 0, 10);
		let (_, _) = do_create_item::<T, I>(s, 0, 0, 1, 10);

		let caller = new_funded_account::<T, I>(s, s, 1000_000_000u128 * UNIT);
		do_mint_item::<T, I>(s, &caller, 0, 10);
		do_mint_item::<T, I>(s, &caller, 0, 10);

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
		}];

		let call = Call::<T, I>::set_bundle {
			bundle: bundle.clone(),
			price: <T as pallet::Config<I>>::Currency::minimum_balance(),
		};
	}: { call.dispatch_bypass_filter(RawOrigin::Signed(caller.clone()).into())? }
	verify {
		assert_last_event::<T, I>(Event::BundleSet {
			trade: <T as pallet::Config<I>>::Helper::trade(0),
			who: caller,
			bundle,
			price:  <T as pallet::Config<I>>::Currency::minimum_balance(),
		}.into() );
	}

	buy_bundle {
		let s in 0 .. MAX as u32;

		let (seller, bundle) = do_set_bundle::<T, I>(s);
		let caller = new_funded_account::<T, I>(s, s, 1000_000_000u128 * UNIT);

		let call = Call::<T, I>::buy_bundle {
			trade: <T as pallet::Config<I>>::Helper::trade(0),
			bid_price: <T as pallet::Config<I>>::Currency::minimum_balance(),
		};
	}: { call.dispatch_bypass_filter(RawOrigin::Signed(caller.clone()).into())? }
	verify {
		assert_last_event::<T, I>(Event::BundleBought {
			trade: <T as pallet::Config<I>>::Helper::trade(0),
			who: caller,
			bid_price:  <T as pallet::Config<I>>::Currency::minimum_balance(),
		}.into() );
	}

	set_wishlist {
		let s in 0 .. MAX as u32;
		let caller = new_funded_account::<T, I>(s, s, 1000_000_000u128 * UNIT);

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
		}];

		let call = Call::<T, I>::set_wishlist {
			bundle: bundle.clone(),
			price: <T as pallet::Config<I>>::Currency::minimum_balance(),
		};
	}: { call.dispatch_bypass_filter(RawOrigin::Signed(caller.clone()).into())? }
	verify {
		assert_last_event::<T, I>(Event::WishlistSet {
			trade: <T as pallet::Config<I>>::Helper::trade(0),
			who: caller,
			wishlist: bundle,
			price:  <T as pallet::Config<I>>::Currency::minimum_balance(),
		}.into() );
	}

	fill_wishlist {
		let s in 0 .. MAX as u32;
		let (_, _) = do_create_item::<T, I>(s, 0, 0, 0, 10);
		let (_, _) = do_create_item::<T, I>(s, 0, 0, 1, 10);

		let wisher = do_set_wishlist::<T, I>(s);

		let filler = new_funded_account::<T, I>(s, s, 1000_000_000u128 * UNIT);
		do_mint_item::<T, I>(s, &filler, 0, 10);
		do_mint_item::<T, I>(s, &filler, 0, 10);

		let call = Call::<T, I>::fill_wishlist {
			trade: <T as pallet::Config<I>>::Helper::trade(0),
			ask_price: <T as pallet::Config<I>>::Currency::minimum_balance(),
		};
	}: { call.dispatch_bypass_filter(RawOrigin::Signed(filler.clone()).into())? }
	verify {
		assert_last_event::<T, I>(Event::WishlistFilled {
			trade: <T as pallet::Config<I>>::Helper::trade(0),
			who: filler,
			ask_price:  <T as pallet::Config<I>>::Currency::minimum_balance(),
		}.into() );
	}

	remove_collection {
		let s in 0 .. MAX as u32;
		let (owner, _) = do_create_collection::<T, I>(s, 0);
		do_create_collection::<T, I>(s, 0);

		let call = Call::<T, I>::remove_collection {
			game: <T as pallet::Config<I>>::Helper::game(0),
			collection: <T as pallet_nfts::Config>::Helper::collection(0),
		};
	}: { call.dispatch_bypass_filter(RawOrigin::Signed(owner.clone()).into())? }
	verify {
		assert_last_event::<T, I>(Event::CollectionRemoved {
			who: owner.clone(),
			game: <T as pallet::Config<I>>::Helper::game(0),
			collection: <T as pallet_nfts::Config>::Helper::collection(0),
		}.into() );
	}

	lock_item_transfer {
		let s in 0 .. MAX as u32;
		let (owner, _) = do_create_item::<T, I>(s, 0, 0, 0, 10);

		let call = Call::<T, I>::lock_item_transfer {
			collection: <T as pallet_nfts::Config>::Helper::collection(0),
			item: <T as pallet_nfts::Config>::Helper::item(0),
		};
	}: { call.dispatch_bypass_filter(RawOrigin::Signed(owner.clone()).into())? }

	unlock_item_transfer {
		let s in 0 .. MAX as u32;
		let (owner, _) = do_create_item::<T, I>(s, 0, 0, 0, 10);

		assert_ok!(PalletGame::<T, I>::lock_item_transfer(
			RawOrigin::Signed(owner.clone()).into(),
			<T as pallet_nfts::Config>::Helper::collection(0),
			<T as pallet_nfts::Config>::Helper::item(0),
		));

		let call = Call::<T, I>::unlock_item_transfer {
			collection: <T as pallet_nfts::Config>::Helper::collection(0),
			item: <T as pallet_nfts::Config>::Helper::item(0),
		};
	}: { call.dispatch_bypass_filter(RawOrigin::Signed(owner.clone()).into())? }


	set_swap {
		let s in 0 .. MAX as u32;
		let (_, _) = do_create_item::<T, I>(s, 0, 0, 0, 10);
		let (_, _) = do_create_item::<T, I>(s, 0, 0, 1, 10);

		let caller = new_funded_account::<T, I>(s, s, 1000_000_000u128 * UNIT);

		do_mint_item::<T, I>(s, &caller, 0, 10);
		do_mint_item::<T, I>(s, &caller, 0, 10);

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
		}];

		let required = vec![
		Package {
			collection: <T as pallet_nfts::Config>::Helper::collection(0),
			item: <T as pallet_nfts::Config>::Helper::item(2),
			amount: 10,
		}];

		let call = Call::<T, I>::set_swap {
			source: bundle.clone(),
			required: required.clone(),
			maybe_price: Some(<T as pallet::Config<I>>::Currency::minimum_balance()),
		};
	}: { call.dispatch_bypass_filter(RawOrigin::Signed(caller.clone()).into())? }
	verify {
		assert_last_event::<T, I>(Event::SwapSet {
			trade: <T as pallet::Config<I>>::Helper::trade(0),
			who: caller,
			source:  bundle,
			required: required.clone(),
			maybe_price: Some(<T as pallet::Config<I>>::Currency::minimum_balance()),
		}.into() );
	}

	claim_swap {
		let s in 0 .. MAX as u32;
		let (player1, _) = do_create_item::<T, I>(s, 0, 0, 0, 10);
		let (player1, _) = do_create_item::<T, I>(s, 0, 0, 1, 10);
		do_mint_item::<T, I>(s, &player1, 0, 10);
		do_mint_item::<T, I>(s, &player1, 0, 10);

		let (player2, _) = do_create_item::<T, I>(s, 1, 1, 0, 10);
		let (player2, _) = do_create_item::<T, I>(s, 1, 1, 1, 10);
		do_mint_item::<T, I>(s, &player2, 1, 10);
		do_mint_item::<T, I>(s, &player2, 1, 10);

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
			Some(<T as pallet::Config<I>>::Currency::minimum_balance())
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
		let s in 0 .. MAX as u32;
		let (_, _) = do_create_item::<T, I>(s, 0, 0, 0, 10);
		let (_, _) = do_create_item::<T, I>(s, 0, 0, 1, 10);

		let caller = new_funded_account::<T, I>(s, s, 1000_000_000u128 * UNIT);

		do_mint_item::<T, I>(s, &caller, 0, 10);
		do_mint_item::<T, I>(s, &caller, 0, 10);

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

		let call = Call::<T, I>::set_auction {
			source: source.clone(),
			maybe_price: Some(<T as pallet::Config<I>>::Currency::minimum_balance()),
			start_block:  <T as pallet::Config<I>>::Helper::block(0),
			duration:  <T as pallet::Config<I>>::Helper::block(10),
		};
	}: { call.dispatch_bypass_filter(RawOrigin::Signed(caller.clone()).into())? }
	verify {
		assert_last_event::<T, I>(Event::AuctionSet {
			trade: <T as pallet::Config<I>>::Helper::trade(0),
			who: caller,
			source:  source,
			maybe_price: Some(<T as pallet::Config<I>>::Currency::minimum_balance()),
			start_block:  <T as pallet::Config<I>>::Helper::block(0),
			duration:  <T as pallet::Config<I>>::Helper::block(10),
		}.into() );
	}

	bid_auction {
		let s in 0 .. MAX as u32;
		let _ = do_set_auction::<T, I>(s);

		let caller = new_funded_account::<T, I>(s, s, 1000_000_000u128 * UNIT);

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
		let s in 0 .. MAX as u32;
		let _ = do_set_auction::<T, I>(s);

		let bidder = new_funded_account::<T, I>(s, s, 1000_000_000u128 * UNIT);
		assert_ok!(PalletGame::<T, I>::bid_auction(
			RawOrigin::Signed(bidder.clone()).into(),
			<T as pallet::Config<I>>::Helper::trade(0),
			<T as pallet::Config<I>>::Currency::minimum_balance()
		));

		let caller = new_funded_account::<T, I>(s, s, 1000_000_000u128 * UNIT);

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
		let s in 0 .. MAX as u32;
		let caller = new_funded_account::<T, I>(s, s, 1000_000_000u128 * UNIT);

		let package = Package {
			collection: <T as pallet_nfts::Config>::Helper::collection(0),
			item: <T as pallet_nfts::Config>::Helper::item(0),
			amount: 10,
		};

		let call = Call::<T, I>::set_buy {
			package: package.clone(),
			unit_price: <T as pallet::Config<I>>::Currency::minimum_balance(),
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
		let s in 0 .. MAX as u32;
		let player = new_funded_account::<T, I>(s, s, 1000_000_000u128 * UNIT);

		let package = Package {
			collection: <T as pallet_nfts::Config>::Helper::collection(0),
			item: <T as pallet_nfts::Config>::Helper::item(0),
			amount: 10,
		};

		assert_ok!(PalletGame::<T, I>::set_buy(
			RawOrigin::Signed(player.clone()).into(),
			package.clone(),
			<T as pallet::Config<I>>::Currency::minimum_balance()
		));

		let (_, _) = do_create_item::<T, I>(s, 0, 0, 0, 10);
		let caller = new_funded_account::<T, I>(s, s, 1000_000_000u128 * UNIT);
		do_mint_item::<T, I>(s, &caller, 0, 10);

		let call = Call::<T, I>::claim_set_buy {
			trade: <T as pallet::Config<I>>::Helper::trade(0),
			amount: 10,
			ask_price: <T as pallet::Config<I>>::Currency::minimum_balance(),
		};
	}: { call.dispatch_bypass_filter(RawOrigin::Signed(caller.clone()).into())? }
	verify {
		assert_last_event::<T, I>(Event::SetBuyClaimed {
			trade: <T as pallet::Config<I>>::Helper::trade(0),
			who: caller,
			amount: 10,
			ask_unit_price:  <T as pallet::Config<I>>::Currency::minimum_balance(),
		}.into() );
	}

	create_collection{
		let s in 0 .. MAX as u32;
		let caller = new_funded_account::<T, I>(s, s, 1000_000_000u128 * UNIT);

		let call = Call::<T, I>::create_collection {
			admin: caller.clone(),
			fee: <T as pallet::Config<I>>::Currency::minimum_balance(),
		};
	}: { call.dispatch_bypass_filter(RawOrigin::Signed(caller.clone()).into())? }
	verify {
		assert_last_event::<T, I>(Event::CollectionCreated {
			who: caller,
			collection: <T as pallet_nfts::Config>::Helper::collection(0),
		}.into() );
	}

	set_accept_adding {
		let s in 0 .. MAX as u32;
		let caller = new_funded_account::<T, I>(s, s, 1000_000_000u128 * UNIT);

		assert_ok!(PalletGame::<T, I>::create_collection(RawOrigin::Signed(caller.clone()).into(),
		caller.clone(),
		<T as pallet::Config<I>>::Currency::minimum_balance()));

		let call = Call::<T, I>::set_accept_adding {
			game: <T as pallet::Config<I>>::Helper::game(0),
			collection:<T as pallet_nfts::Config>::Helper::collection(0),
		};
	}: { call.dispatch_bypass_filter(RawOrigin::Signed(caller.clone()).into())? }
	verify {
		assert_last_event::<T, I>(Event::AddingAcceptanceSet {
			who: caller,
			game: <T as pallet::Config<I>>::Helper::game(0),
			collection: <T as pallet_nfts::Config>::Helper::collection(0),
		}.into() );
	}

	add_game_collection {
		let s in 0 .. MAX as u32;

		let (owner, admin) = do_create_game::<T, I>(s);
		let admin = T::Lookup::lookup(admin).unwrap();

		let caller = new_funded_account::<T, I>(s, s, 1000_000_000u128 * UNIT);

		assert_ok!(PalletGame::<T, I>::create_collection(
			RawOrigin::Signed(caller.clone()).into(),
			caller.clone(),
			<T as pallet::Config<I>>::Currency::minimum_balance())
		);

		assert_ok!(PalletGame::<T, I>::set_accept_adding(
			RawOrigin::Signed(caller.clone()).into(),
			<T as pallet::Config<I>>::Helper::game(0),
			<T as pallet_nfts::Config>::Helper::collection(0),
		));

		let call = Call::<T, I>::add_game_collection {
			game: <T as pallet::Config<I>>::Helper::game(0),
			collection:<T as pallet_nfts::Config>::Helper::collection(0),
		};
	}: { call.dispatch_bypass_filter(RawOrigin::Signed(admin.clone()).into())? }
	verify {
		assert_last_event::<T, I>(Event::CollectionAdded {
			who: admin,
			game: <T as pallet::Config<I>>::Helper::game(0),
			collection: <T as pallet_nfts::Config>::Helper::collection(0),
		}.into() );
	}

	add_retail_supply {
		let s in 0 .. MAX as u32;
		let caller = do_set_price::<T, I>(s);

		let package = Package {
			collection: <T as pallet_nfts::Config>::Helper::collection(0),
			item: <T as pallet_nfts::Config>::Helper::item(0),
			amount: 5,
		};

		let call = Call::<T, I>::add_retail_supply {
			trade: <T as pallet::Config<I>>::Helper::trade(0),
			supply: package,
		};
	}:  { call.dispatch_bypass_filter(RawOrigin::Signed(caller.clone()).into())? }

	cancel_trade {
		let s in 0 .. MAX as u32;
		let (caller, _) = do_set_bundle::<T, I>(s);

		let call = Call::<T, I>::cancel_trade {
			trade: <T as pallet::Config<I>>::Helper::trade(0),
			trade_type: TradeType::Bundle
		};
	}:  { call.dispatch_bypass_filter(RawOrigin::Signed(caller.clone()).into())? }
	verify {
		assert_last_event::<T, I>(Event::TradeCanceled {
			who: caller,
			trade: <T as pallet::Config<I>>::Helper::trade(0),
		}.into() );
	}
}
