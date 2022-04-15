#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

pub mod weights;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::{pallet_prelude::*};
    use frame_system::pallet_prelude::*;
    use frame_support::{
		traits::{Randomness, Currency, tokens::ExistenceRequirement, Time},
		transactional,
	};
    use sp_io::hashing::blake2_128;
    use scale_info::TypeInfo;
    use core::fmt::Debug;
    use frame_support::inherent::Vec;
    use sp_runtime::traits::{Hash, Zero};

    #[cfg(feature = "std")]
    use frame_support::serde::{Deserialize, Serialize};

    use log::{info, error};

    use crate::weights::WeightInfo;


    type AccountOf<T> = <T as frame_system::Config>::AccountId;
    type BalanceOf<T> =
    <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
    type TimeOf<T> = <<T as Config>::ThreadTime as frame_support::traits::Time>::Moment;

    // Struct for holding Thread information.
    #[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
    #[scale_info(skip_type_params(T))]
    #[codec(mel_bound())]
    pub struct Thread<T: Config> {
        pub id: Option<T::Hash>,
        pub topic: Topic,
        pub title: Vec<u8>,
        pub content: Vec<u8>,
        pub priority: Priority,
        pub author: AccountOf<T>,
        pub price: BalanceOf<T>,
        pub created: TimeOf<T>,
        pub close_time: TimeOf<T>,
    }

    // Enum declaration for Gender.
    #[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
    #[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
    pub enum Priority {
        High,
        Normal,
	}

    #[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
    #[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
    pub enum Topic {
        English
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub (super) trait Store)]
    #[pallet::without_storage_info]
    pub struct Pallet<T>(_);

    /// Configure the pallet by specifying the parameters and types it depends on.
    #[pallet::config]
    pub trait Config: frame_system::Config + Debug {
        /// Because this pallet emits events, it depends on the runtime's definition of an event.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        /// The Currency handler for the Forum pallet.
        type Currency: Currency<Self::AccountId>;

        /// The type of Randomness we want to specify for this pallet.
        type ThreadRandomness: Randomness<Self::Hash, Self::BlockNumber>;

        type ThreadTime: Time;

        type WeightInfo: WeightInfo;
    }

    // Errors.
    #[pallet::error]
    pub enum Error<T> {
        /// Handles arithmetic overflow when incrementing the Thread counter.
        ThreadCntOverflow,
        /// An account cannot own more Forum than `MaxThreadCount`.
        ExceedMaxThreadOwned,
        /// Buyer cannot be the owner.
        BuyerIsThreadOwner,
        /// Cannot transfer a thread to its owner.
        TransferToSelf,
        /// Handles checking whether the Thread exists.
        ThreadNotExist,
        /// Handles checking that the Thread is owned by the account transferring, buying or setting a price for it.
        NotThreadOwner,
        /// Ensures the Thread is for sale.
        ThreadNotForSale,
        /// Ensures that the buying price is greater than the asking price.
        ThreadBidPriceTooLow,
        /// Ensures that an account has enough funds to purchase a Thread.
        NotEnoughBalance,
        ThreadExists,
    }

    // Events.
    #[pallet::event]
    #[pallet::generate_deposit(pub (super) fn deposit_event)]
    pub enum Event<T> where T: Config + Debug {
        /// A new Thread was successfully created. \[sender, thread_id\]
        Created(T::AccountId, T::Hash),
        /// Thread price was successfully set. \[sender, thread_id, new_price\]
        PriceSet(T::AccountId, T::Hash, Option<BalanceOf<T>>),
        /// A Thread was successfully transferred. \[from, to, thread_id\]
        Transferred(T::AccountId, T::AccountId, T::Hash),
        /// A Thread was successfully bought. \[buyer, seller, thread_id, bid_price\]
        Bought(T::AccountId, T::AccountId, T::Hash, BalanceOf<T>),
    }

    // Storage items.

    #[pallet::storage]
    #[pallet::getter(fn thread_cnt)]
    /// Keeps track of the number of Threads in existence.
    pub(super) type ThreadCnt<T: Config> = StorageValue<_, u64, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn threads)]
    /// Stores Map of Threads.
    pub(super) type Threads<T: Config> = StorageMap<_, Twox64Concat, T::Hash, Thread<T>>;

    #[pallet::storage]
    #[pallet::getter(fn threads_owned)]
    /// Keeps track of what accounts own what Threads.
    pub(super) type ThreadsOwned<T: Config> =
    StorageMap<_, Twox64Concat, T::AccountId, Vec<T::Hash>, ValueQuery>;


    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(10000000)]
        pub fn create_new_thread(origin: OriginFor<T>) -> DispatchResult {
            let author = ensure_signed(origin)?;

            let thread = Thread {
                id: None,
                topic: Topic::English,
                title: Vec::new(),
                content: Vec::new(),
                priority: Priority::High,
                author: author.clone(),
                price: sp_runtime::traits::Zero::zero(),
                created: T::ThreadTime::now(),
                close_time: T::ThreadTime::now(),
            };

            let thread_hash = T::Hashing::hash_of(&thread);

            <Threads<T>>::insert(&thread_hash, thread.clone());

            <ThreadsOwned<T>>::mutate(&author, |thread_vec| {
                thread_vec.push(thread_hash)
            });

            let new_cnt = Self::thread_cnt().checked_add(1)
                .ok_or(<Error<T>>::ThreadCntOverflow)?;

            <ThreadCnt<T>>::put(new_cnt);

            Self::deposit_event(Event::Created(author, thread_hash));

            Ok(())
        }
    }
}
