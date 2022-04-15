#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/v3/runtime/frame>
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::inherent::Vec;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub (super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	pub type Voters<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, bool>;

	#[pallet::storage]
	pub type Candidates<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, u32>;

	#[pallet::event]
	#[pallet::generate_deposit(pub (super) fn deposit_event)]
	pub enum Event<T: Config> {
		VoteSuccess(T::AccountId),
		Winner((Vec<T::AccountId>, u32)),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		AlreadyVoted,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn vote(origin: OriginFor<T>, candidate: T::AccountId) -> DispatchResult {
			let who = ensure_signed(origin)?;

			if <Voters<T>>::contains_key(&who) {
				return Err(Error::<T>::AlreadyVoted).unwrap();
			}

			<Voters<T>>::insert(&who, true);

			let mut candidate_vote = 0;

			if <Candidates<T>>::contains_key(&candidate) {
				candidate_vote = <Candidates<T>>::get(&candidate).unwrap() + 1;
			} else {
				candidate_vote = 1;
			}
			<Candidates<T>>::insert(&candidate, candidate_vote);

			Self::deposit_event(Event::VoteSuccess(candidate));
			Ok(())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1, 1))]
		pub fn get_winner(origin: OriginFor<T>) -> DispatchResult {

			let mut max = 0;
			let mut winners = Vec::new();
			for candidate_value in <Candidates<T>>::iter() {
				if candidate_value.1 > max {
					winners.clear();
					max = candidate_value.1;
					winners.push(candidate_value.0);
				} else if candidate_value.1 == max {
					winners.push(candidate_value.0);
				}
			}
			Self::deposit_event(Event::Winner((winners, max)));
			Ok(())
		}
	}
}
