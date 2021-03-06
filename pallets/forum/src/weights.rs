
//! Autogenerated weights for `pallet_forum`
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2022-03-20, STEPS: `1`, REPEAT: 10, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("dev"), DB CACHE: 1024

// Executed Command:
// ./target/release/node-forum
// benchmark
// --chain
// dev
// --execution
// wasm
// --wasm-execution
// compiled
// --pallet
// pallet_forum
// --extrinsic
// *
// --steps
// 1
// --repeat
// 10
// --json
// --output
// ./pallets/forum/src/weights.rs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::Weight};
use sp_std::marker::PhantomData;

pub trait WeightInfo {
	fn create_forum() -> Weight;
}

/// Weight functions for `pallet_forum`.
pub struct SubstrateWeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeightInfo<T> {
	// Storage: RandomnessCollectiveFlip RandomMaterial (r:1 w:0)
	// Storage: unknown [0x3a65787472696e7369635f696e646578] (r:1 w:0)
	// Storage: Timestamp Now (r:1 w:0)
	// Storage: PalletForum ForumCnt (r:1 w:1)
	// Storage: PalletForum Forum (r:1 w:1)
	// Storage: PalletForum ForumOwned (r:1 w:1)
	fn create_forum() -> Weight {
		(39_833_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(6 as Weight))
			.saturating_add(T::DbWeight::get().writes(3 as Weight))
	}
}
