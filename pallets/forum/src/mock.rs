use crate as pallet_forum;
use frame_support::{traits::{ConstU16, ConstU64, ConstU32, ConstU128}};
use frame_system as system;
use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;


type Balance = u128;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
		ForumModule: pallet_forum::{Pallet, Call, Storage, Event<T>},
		Balances: pallet_balances::{Pallet, Call, Config<T>, Storage, Event<T>},
		Randomness: pallet_randomness_collective_flip::{Pallet},
		Timestamp: pallet_timestamp::{Pallet, Call, Storage},
	}
);

impl system::Config for Test {
	type BaseCallFilter = frame_support::traits::Everything;
	type BlockWeights = ();
	type BlockLength = ();
	type DbWeight = ();
	type Origin = Origin;
	type Call = Call;
	type Index = u64;
	type BlockNumber = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = u64;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = Event;
	type BlockHashCount = ConstU64<250>;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<Balance>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ConstU16<42>;
	type OnSetCode = ();
	type MaxConsumers = frame_support::traits::ConstU32<16>;
}

impl pallet_forum::Config for Test {
	type Event = Event;

	type Currency = Balances;

	type MaxForumOwned = ConstU32<100>;

	type ForumRandomness = Randomness;

	type ForumTime = Timestamp;

	type WeightInfo = pallet_forum::weights::SubstrateWeightInfo<Test>;

}

impl pallet_balances::Config for Test {
	type MaxLocks = ConstU32<50>;
	type MaxReserves = ();
	type ReserveIdentifier = [u8; 8];
	/// The type for recording an account's balance.
	type Balance = Balance;
	/// The ubiquitous event type.
	type Event = Event;
	type DustRemoval = ();
	type ExistentialDeposit = ConstU128<5>;
	type AccountStore = System;
	type WeightInfo = pallet_balances::weights::SubstrateWeight<Test>;
}

impl pallet_timestamp::Config for Test {
    type Moment = u64;


    type OnTimestampSet = ();


    type MinimumPeriod = ();


    type WeightInfo = ();

}

impl pallet_randomness_collective_flip::Config for Test {
	
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut storage = system::GenesisConfig::default().build_storage::<Test>().unwrap();
	pallet_balances::GenesisConfig::<Test> {
		balances: vec![
			(1u64, 10u128),
			(2u64, 20u128),
			(3u64, 30u128),
			(4u64, 40u128),
			(5u64, 50u128),
			(6u64, 60u128)
		],
	}
		.assimilate_storage(&mut storage)
		.unwrap();
	let mut ext = sp_io::TestExternalities::new(storage);
	ext.execute_with(|| System::set_block_number(1));
	ext
}	

