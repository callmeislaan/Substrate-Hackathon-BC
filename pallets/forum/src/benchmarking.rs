//! Benchmarking setup for pallet-template

use super::*;

#[allow(unused)]
use crate::Pallet as Forum;
use frame_benchmarking::{benchmarks, whitelisted_caller};
use frame_system::RawOrigin;

benchmarks! {
	create_forum {
		let s in 1 .. 1000;
		let caller: T::AccountId = whitelisted_caller();
	}: _(RawOrigin::Signed(caller))

	impl_benchmark_test_suite!(ForumModule, crate::mock::new_test_ext(), crate::mock::Test);
}
