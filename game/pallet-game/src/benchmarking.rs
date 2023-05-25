//! Benchmarking setup for pallet-pool

use super::*;
#[allow(unused)]
use crate::Pallet as PalletGame;
use crate::{pallet::BenchmarkHelper as GameBenchmarkHelper, Call, Config};
use enumflags2::{BitFlag, BitFlags};
use frame_benchmarking::{account, benchmarks, benchmarks_instance_pallet, Box};
use frame_support::{assert_ok, dispatch::UnfilteredDispatchable, traits::Currency};
use frame_system::RawOrigin;
use pallet_nfts::{
	BenchmarkHelper, CollectionSetting, CollectionSettings, ItemSettings, MintSettings,
};
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

fn make_collection_config<T: Config<I>, I: 'static>(
	disable_settings: BitFlags<CollectionSetting>,
) -> CollectionConfigFor<T, I> {
	CollectionConfig {
		settings: CollectionSettings::from_disabled(disable_settings),
		max_supply: None,
		mint_settings: MintSettings::default(),
	}
}

fn default_collection_config<T: Config<I>, I: 'static>() -> CollectionConfigFor<T, I> {
	make_collection_config::<T, I>(CollectionSetting::empty())
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

fn do_create_collection<T: Config<I>, I: 'static>(s: u32) -> (T::AccountId, AccountIdLookupOf<T>) {
	let (caller, admin) = do_create_game::<T, I>(s);

	assert_ok!(PalletGame::<T, I>::create_game_collection(
		RawOrigin::Signed(caller.clone()).into(),
		<T as pallet::Config<I>>::Helper::game(0),
		default_collection_config::<T, I>(),
	));
	(caller, admin)
}

fn do_create_item<T: Config<I>, I: 'static>(
	s: u32,
	item: u16,
	amount: u32,
) -> (T::AccountId, AccountIdLookupOf<T>) {
	let (caller, admin) = do_create_collection::<T, I>(s);

	assert_ok!(PalletGame::<T, I>::create_item(
		RawOrigin::Signed(caller.clone()).into(),
		<T as pallet_nfts::Config>::Helper::collection(0),
		<T as pallet_nfts::Config>::Helper::item(item),
		default_item_config(),
		10,
	));
	(caller, admin)
}

fn do_mint_item<T: Config<I>, I: 'static>(s: u32, miner: &T::AccountId, amount: u32) {
	assert_ok!(PalletGame::<T, I>::mint(
		RawOrigin::Signed(miner.clone()).into(),
		<T as pallet_nfts::Config>::Helper::collection(0),
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
	let (_, _) = do_create_item::<T, I>(s, 0, 10);
	let seller = new_funded_account::<T, I>(s, s, 1000_000_000u128 * UNIT);

	do_mint_item::<T, I>(s, &seller, 10);

	let package = Package {
		collection: <T as pallet_nfts::Config>::Helper::collection(0),
		item: <T as pallet_nfts::Config>::Helper::item(0),
		amount: 10,
	};

	assert_ok!(PalletGame::<T, I>::set_price(
		RawOrigin::Signed(seller.clone()).into(),
		package,
		<T as pallet::Config<I>>::Currency::minimum_balance(),
	));
	seller
}

fn do_set_bundle<T: Config<I>, I: 'static>(s: u32) -> T::AccountId {
	let (_, _) = do_create_item::<T, I>(s, 0, 10);
	let (_, _) = do_create_item::<T, I>(s, 1, 10);

	let seller = new_funded_account::<T, I>(s, s, 1000_000_000u128 * UNIT);
	do_mint_item::<T, I>(s, &seller, 10);
	do_mint_item::<T, I>(s, &seller, 10);

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
		bundle,
		<T as pallet::Config<I>>::Currency::minimum_balance(),
	));

	seller
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
		let call = Call::<T, I>::create_game_collection { game: <T as pallet::Config<I>>::Helper::game(0), config: default_collection_config::<T, I>() };
	}: { call.dispatch_bypass_filter(RawOrigin::Signed(caller.clone()).into())? }
	verify {
		assert_last_event::<T, I>(Event::CollectionCreated { who: caller.clone(), collection: <T as pallet_nfts::Config>::Helper::collection(0) }.into());
	}

	create_item {
		let s in 0 .. MAX as u32;
		let (caller, admin) = do_create_collection::<T, I>(s);
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
		let (caller, admin) = do_create_item::<T, I>(s, 0, 10);
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
		let (caller, admin) = do_create_item::<T, I>(s, 0, 10);

		let miner = new_funded_account::<T, I>(s, s, 1000_000_000u128 * UNIT);
		let mint_to =   T::Lookup::unlookup(miner.clone());

		let call = Call::<T, I>::mint { collection: <T as pallet_nfts::Config>::Helper::collection(0),
			mint_to: mint_to.clone(), amount: 1 };
	}: { call.dispatch_bypass_filter(RawOrigin::Signed(caller).into())? }
	verify {
		assert_last_event::<T, I>(Event::Minted { who: miner.clone(),
			target: T::Lookup::lookup(mint_to.clone()).unwrap(),
			collection: <T as pallet_nfts::Config>::Helper::collection(0),
			minted_items: vec![<T as pallet_nfts::Config>::Helper::item(0)] }.into());
	}

	burn {
		let s in 0 .. MAX as u32;
		let (owner, admin) = do_create_item::<T, I>(s, 0, 10);
		let miner = new_funded_account::<T, I>(s, s, 1000_000_000u128 * UNIT);
		do_mint_item::<T, I>(s, &miner, 10);

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
		let (owner, admin) = do_create_item::<T, I>(s, 0, 10);

		let miner = new_funded_account::<T, I>(s, s, 1000_000_000u128 * UNIT);
		do_mint_item::<T, I>(s, &miner, 10);

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
		let (caller, admin) = do_create_item::<T, I>(s, 0, 10);

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

		let (owner, admin) = do_create_item::<T, I>(s, 0, 10);
		let miner = new_funded_account::<T, I>(s, s, 1000_000_000u128 * UNIT);
		do_mint_item::<T, I>(s, &miner, 10);

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
		let (owner, admin) = do_create_item::<T, I>(s, 0, 10);

		let caller = new_funded_account::<T, I>(s, s, 1000_000_000u128 * UNIT);
		do_mint_item::<T, I>(s, &caller, 10);

		let package = Package {
			collection: <T as pallet_nfts::Config>::Helper::collection(0),
			item: <T as pallet_nfts::Config>::Helper::item(0),
			amount: 10,
		};

		let call = Call::<T, I>::set_price {
			package: package,
			price: <T as pallet::Config<I>>::Currency::minimum_balance(),
		};
	}: { call.dispatch_bypass_filter(RawOrigin::Signed(caller.clone()).into())? }
	verify {
		assert_last_event::<T, I>(Event::PriceSet {
			id: <T as pallet::Config<I>>::Helper::trade(0),
			who: caller.clone(),
			collection: <T as pallet_nfts::Config>::Helper::collection(0),
			item: <T as pallet_nfts::Config>::Helper::item(0),
			amount: 10,
			price:  <T as pallet::Config<I>>::Currency::minimum_balance(),
		}.into() );
	}

	buy_item {
		let s in 0 .. MAX as u32;

		let seller = do_set_price::<T, I>(s);
		let caller = new_funded_account::<T, I>(s, s, 1000_000_000u128 * UNIT);

		let call = Call::<T, I>::buy_item {
			id: <T as pallet::Config<I>>::Helper::trade(0),
			amount: 10,
			bid_price: <T as pallet::Config<I>>::Currency::minimum_balance(),
		};
	}: { call.dispatch_bypass_filter(RawOrigin::Signed(caller.clone()).into())? }
	verify {
		assert_last_event::<T, I>(Event::ItemBought {
			id: <T as pallet::Config<I>>::Helper::trade(0),
			seller: seller,
			buyer: caller,
			collection: <T as pallet_nfts::Config>::Helper::collection(0),
			item: <T as pallet_nfts::Config>::Helper::item(0),
			amount: 10,
			price:  <T as pallet::Config<I>>::Currency::minimum_balance(),
		}.into() );
	}

	set_bundle {
		let s in 0 .. MAX as u32;
		let (_, _) = do_create_item::<T, I>(s, 0, 10);
		let (_, _) = do_create_item::<T, I>(s, 1, 10);

		let caller = new_funded_account::<T, I>(s, s, 1000_000_000u128 * UNIT);
		do_mint_item::<T, I>(s, &caller, 10);
		do_mint_item::<T, I>(s, &caller, 10);

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
			bundle: bundle,
			price: <T as pallet::Config<I>>::Currency::minimum_balance(),
		};
	}: { call.dispatch_bypass_filter(RawOrigin::Signed(caller.clone()).into())? }
	verify {
		assert_last_event::<T, I>(Event::BundleSet {
			id: <T as pallet::Config<I>>::Helper::trade(0),
			who: caller,
			price:  <T as pallet::Config<I>>::Currency::minimum_balance(),
		}.into() );
	}

	buy_bundle {
		let s in 0 .. MAX as u32;

		let seller = do_set_bundle::<T, I>(s);
		let caller = new_funded_account::<T, I>(s, s, 1000_000_000u128 * UNIT);

		let call = Call::<T, I>::buy_bundle {
			trade_id: <T as pallet::Config<I>>::Helper::trade(0),
			bid_price: <T as pallet::Config<I>>::Currency::minimum_balance(),
		};
	}: { call.dispatch_bypass_filter(RawOrigin::Signed(caller.clone()).into())? }
	verify {
		assert_last_event::<T, I>(Event::BundleBought {
			id: <T as pallet::Config<I>>::Helper::trade(0),
			buyer: caller,
			seller: seller,
			price:  <T as pallet::Config<I>>::Currency::minimum_balance(),
		}.into() );
	}

	cancel_set_price {
		let s in 0 .. MAX as u32;

		let caller = do_set_price::<T, I>(s);

		let call = Call::<T, I>::cancel_set_price {
			trade_id: <T as pallet::Config<I>>::Helper::trade(0),
		};
	}: { call.dispatch_bypass_filter(RawOrigin::Signed(caller.clone()).into())? }
	verify {
		assert_last_event::<T, I>(Event::TradeCanceled {
			id: <T as pallet::Config<I>>::Helper::trade(0),
			who: caller,
		}.into() );
	}

	cancel_set_bundle {
		let s in 0 .. MAX as u32;

		let caller = do_set_bundle::<T, I>(s);

		let call = Call::<T, I>::cancel_set_bundle {
			trade_id: <T as pallet::Config<I>>::Helper::trade(0),
		};
	}: { call.dispatch_bypass_filter(RawOrigin::Signed(caller.clone()).into())? }
	verify {
		assert_last_event::<T, I>(Event::TradeCanceled {
			id: <T as pallet::Config<I>>::Helper::trade(0),
			who: caller,
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
			bundle: bundle,
			price: <T as pallet::Config<I>>::Currency::minimum_balance(),
		};
	}: { call.dispatch_bypass_filter(RawOrigin::Signed(caller.clone()).into())? }
	verify {
		assert_last_event::<T, I>(Event::WishlistSet {
			id: <T as pallet::Config<I>>::Helper::trade(0),
			who: caller,
			price:  <T as pallet::Config<I>>::Currency::minimum_balance(),
		}.into() );
	}

	fill_wishlist {
		let s in 0 .. MAX as u32;
		let (_, _) = do_create_item::<T, I>(s, 0, 10);
		let (_, _) = do_create_item::<T, I>(s, 1, 10);

		let wisher = do_set_wishlist::<T, I>(s);

		let filler = new_funded_account::<T, I>(s, s, 1000_000_000u128 * UNIT);
		do_mint_item::<T, I>(s, &filler, 10);
		do_mint_item::<T, I>(s, &filler, 10);

		let call = Call::<T, I>::fill_wishlist {
			trade_id: <T as pallet::Config<I>>::Helper::trade(0),
			ask_price: <T as pallet::Config<I>>::Currency::minimum_balance(),
		};
	}: { call.dispatch_bypass_filter(RawOrigin::Signed(filler.clone()).into())? }
	verify {
		assert_last_event::<T, I>(Event::WishlistFilled {
			id: <T as pallet::Config<I>>::Helper::trade(0),
			wisher: wisher,
			filler: filler,
			price:  <T as pallet::Config<I>>::Currency::minimum_balance(),
		}.into() );
	}

}
