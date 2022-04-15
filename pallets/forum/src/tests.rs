use crate::{mock::*};

use frame_support::{assert_ok};

use crate as pallet_forum;
use crate::mock as mock;

#[test]
fn should_fn_create_forum_work() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);

		let sender = 1u64;
		
		// ensure funtion work
		assert_ok!(ForumModule::create_forum(Origin::signed(sender)));
		
		// ensure storage value
		assert_eq!(ForumModule::forum_cnt(), 1);

		let event = <frame_system::Pallet<Test>>::events().pop().expect("Expected event").event;
		
		let forum_id = ForumModule::forum_owned(sender).last().unwrap().to_owned();

		assert_eq!(event, mock::Event::from(pallet_forum::Event::Created(sender, forum_id)));
	});
}

#[test]
fn shoud_fn_mint_generate_forum_and_save_to_storage_success() {
	new_test_ext().execute_with(|| {
		let account_id = 1;
		let forum_id = ForumModule::mint(&account_id, None, None).unwrap();

		assert_eq!(ForumModule::forum_cnt(), 1);
		
		let forum = ForumModule::forum(forum_id);
		
		assert_eq!(forum.is_some(), true);
		
		let bounded_vec = ForumModule::forum_owned(account_id);

		assert_eq!(bounded_vec.len(), 1);

		assert_eq!(bounded_vec.last(), Some(&forum_id));

		System::set_block_number(2);

		let forum_id_2 = ForumModule::mint(&account_id, None, None).unwrap();

		assert_eq!(ForumModule::forum_cnt(), 2);
		
		let forum_2 = ForumModule::forum(forum_id_2);
		
		assert_eq!(forum_2.is_some(), true);
		
		let bounded_vec_2 = ForumModule::forum_owned(account_id);

		assert_eq!(bounded_vec_2.len(), 2);

		assert_eq!(bounded_vec_2.last(), Some(&forum_id_2));
	})
}

#[test]
fn should_fn_set_price_work() {
	new_test_ext().execute_with(|| {
		let account_id = 1;
		let owner = Origin::signed(account_id);
		let forum_id = ForumModule::mint(&account_id, None, None).unwrap();
		let new_price = Some(10);
		assert_ok!(ForumModule::set_price(owner, forum_id, new_price));

		let event = <frame_system::Pallet<Test>>::events().pop().expect("Expected set price event").event;

		assert_eq!(event, mock::Event::from(pallet_forum::Event::PriceSet(account_id, forum_id, new_price)));
	});
}

#[test]
fn shoud_fn_transfer_work() {
	new_test_ext().execute_with(|| {
		let account_id_1 = 1;
		let account_id_2 = 2;
		let onwer_1 = Origin::signed(account_id_1);
		let forum_id = ForumModule::mint(&account_id_1, None, None).unwrap();
		assert_ok!(ForumModule::transfer(onwer_1, account_id_2, forum_id));
	});
}

#[test]
fn should_fn_buy_forum_work() {
	new_test_ext().execute_with(|| {
		let account_id_1 = 1;
		let account_id_2 = 2;
		let owner_1 = Origin::signed(account_id_1);
		let owner_2 = Origin::signed(account_id_2);
		let forum_id = ForumModule::mint(&account_id_1, None, None).unwrap();
		let _ = ForumModule::set_price(owner_1, forum_id, Some(10));
		assert_ok!(ForumModule::buy_forum(owner_2, forum_id, 10));
	});
}

#[test]
fn should_fn_bread_forum_work() {
	new_test_ext().execute_with(|| {
		let account_id_1 = 1;
		let owner_1 = Origin::signed(account_id_1);
		let forum_id_1 = ForumModule::mint(&account_id_1, None, None).unwrap();
		System::set_block_number(2);
		let forum_id_2 = ForumModule::mint(&account_id_1, None, None).unwrap();
		assert_ok!(ForumModule::breed_forum(owner_1, forum_id_1, forum_id_2));
	});
}