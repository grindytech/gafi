use crate::{mock::*, Config, Error};
use frame_support::{assert_err, assert_ok, traits::Currency};
use crate::pool::{PackService};
const POOL_FEE: u64 = 10000000000000000;
const MARK_BLOCK: u64 = 30;
const MAX_PLAYER: u32 = 1000;

#[test]
fn player_join_pool_should_works() {
	ExtBuilder::default().build_and_execute(|| {
		run_to_block(10);
		let balance_before = <Test as Config>::Currency::free_balance(&ALICE);
		assert_ok!(PalletPool::join(Origin::signed(ALICE), PackService::Basic));
		let balance_after = <Test as Config>::Currency::free_balance(&ALICE);
		assert_eq!(balance_before, balance_after + POOL_FEE * 2, "charge pool fee not correct");
	});
}

#[test]
fn player_join_pool_should_fail() {
	ExtBuilder::default().build_and_execute(|| {
		run_to_block(10);
		{
			let balance_before = <Test as Config>::Currency::free_balance(&ALICE);
			assert_ok!((PalletPool::join(Origin::signed(ALICE), PackService::Basic)));
			let balance_after = <Test as Config>::Currency::free_balance(&ALICE);
			assert_eq!(balance_before, balance_after + POOL_FEE * 2, "charge pool fee not correct");
		}

		{
			let balance_before = <Test as Config>::Currency::free_balance(&ALICE);
			assert_err!((PalletPool::join(Origin::signed(ALICE), PackService::Basic)), <Error<Test>>::PlayerAlreadyJoin);
			let balance_after = <Test as Config>::Currency::free_balance(&ALICE);
			assert_eq!(balance_before, balance_after, "charge pool fee when fail not correct");
		}
	})
}

#[test]
fn should_move_newplayers_to_ingame() {
	ExtBuilder::default().build_and_execute(|| {
		run_to_block(10);
		{
			let balance_before = <Test as Config>::Currency::free_balance(&ALICE);
			assert_ok!((PalletPool::join(Origin::signed(ALICE), PackService::Basic)));
			let balance_after = <Test as Config>::Currency::free_balance(&ALICE);
			assert_eq!(balance_before, balance_after + POOL_FEE * 2, "charge pool fee not correct");
		}

		{
			let new_players_before = PalletPool::new_players();
			let ingame_players_before = PalletPool::ingame_players();

			assert_eq!(new_players_before.len(), 1, "new_players_before length not correct");
			assert_eq!(ingame_players_before.len(), 0, "ingame_players_before length not correct");
		}

		{
			let balance_before = <Test as Config>::Currency::free_balance(&ALICE);
			run_to_block((MARK_BLOCK * 2) + 1);
			let balance_after = <Test as Config>::Currency::free_balance(&ALICE);
			assert_eq!(
				balance_before,
				balance_after + POOL_FEE,
				"charge ingame players pool fee not correct"
			);
		}

		{
			let new_players_after = PalletPool::new_players();
			let ingame_players_after = PalletPool::ingame_players();

			assert_eq!(new_players_after.len(), 0, "new_players_after length not correct");
			assert_eq!(ingame_players_after.len(), 1, "ingame_players_after length not correct");
		}
	})
}

#[test]
fn newplayer_refund_should_refund_correct() {
	ExtBuilder::default().build_and_execute(|| {
		{
			let balance_before = <Test as Config>::Currency::free_balance(&ALICE);
			assert_ok!((PalletPool::join(Origin::signed(ALICE), PackService::Basic)));
			let balance_after = <Test as Config>::Currency::free_balance(&ALICE);
			assert_eq!(balance_before, balance_after + POOL_FEE * 2, "charge pool fee not correct");
		}
		run_to_block(10);
		{
			let balance_before = <Test as Config>::Currency::free_balance(&ALICE);
			assert_ok!(PalletPool::leave(Origin::signed(ALICE)));
			let balance_after = <Test as Config>::Currency::free_balance(&ALICE);
			assert_eq!(balance_before + POOL_FEE, balance_after, "newplayer refund not correct");
		}
	})
}

const INGAME_REFUNDS: [(u64, u64, u64); 3] =
	[(10, 40, POOL_FEE), (15, 81, POOL_FEE * 80 / 100), (72, 134, POOL_FEE * 14 / 15)];

#[test]
fn should_calculate_correct_refund_amount() {
	ExtBuilder::default().build_and_execute(|| {
		for refund in INGAME_REFUNDS {
			let refund_amount =
				PalletPool::calculate_ingame_refund_amount(refund.0, refund.1).unwrap();
			assert_eq!(refund_amount, refund.2, "Calculate ingame refund fee not correct");
		}
	})
}

#[test]
fn leave_pool_should_work() {
	for refund_map in INGAME_REFUNDS {
		ExtBuilder::default().build_and_execute(|| {
			run_to_block(refund_map.0);
			{
				let balance_before = <Test as Config>::Currency::free_balance(&ALICE);
				assert_ok!((PalletPool::join(Origin::signed(ALICE), PackService::Basic)));
				let balance_after = <Test as Config>::Currency::free_balance(&ALICE);
				assert_eq!(
					balance_before,
					balance_after + POOL_FEE * 2,
					"charge pool fee not correct"
				);
			}
			run_to_block(refund_map.1);
			{
				let balance_before = <Test as Config>::Currency::free_balance(&ALICE);
				assert_ok!(PalletPool::leave(Origin::signed(ALICE)));
				let balance_after = <Test as Config>::Currency::free_balance(&ALICE);
				assert_eq!(
					balance_before + refund_map.2,
					balance_after,
					"ingame refund not correct"
				);
			}
		})
	}
}

#[test]
fn leave_pool_should_fail() {
	ExtBuilder::default().build_and_execute(|| {
		run_to_block(10);
		{
			let balance_before = <Test as Config>::Currency::free_balance(&ALICE);
			assert_ok!((PalletPool::join(Origin::signed(ALICE), PackService::Basic)));
			let balance_after = <Test as Config>::Currency::free_balance(&ALICE);
			assert_eq!(balance_before, balance_after + POOL_FEE * 2, "charge pool fee not correct");
		}
		run_to_block(20);
		assert_ok!(PalletPool::leave(Origin::signed(ALICE)));
		run_to_block(20);
		assert_err!(PalletPool::leave(Origin::signed(ALICE)), <Error<Test>>::PlayerNotFound);
	})
}
