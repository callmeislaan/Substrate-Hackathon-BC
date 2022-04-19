#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

pub mod weights;
mod types;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::{pallet_prelude::*, PalletId};
    use frame_system::pallet_prelude::*;
    use frame_support::{
        traits::{Randomness, Currency, tokens::ExistenceRequirement, Time},
        transactional,
    };
    use sp_io::hashing::blake2_128;
    use scale_info::TypeInfo;
    use core::fmt::Debug;
    use frame_support::inherent::Vec;
    use sp_runtime::traits::{AccountIdConversion, Hash, Zero};

    #[cfg(feature = "std")]
    use frame_support::serde::{Deserialize, Serialize};
    use frame_support::traits::tokens::Balance;

    use log::{info, error};
    use crate::types::{Priority, Thread, ThreadDto};

    use crate::weights::WeightInfo;

    type AccountOf<T> = <T as frame_system::Config>::AccountId;
    type BalanceOf<T> =
    <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
    type TimeOf<T> = <<T as Config>::ThreadTime as frame_support::traits::Time>::Moment;
    type HashOf<T> = <T as frame_system::Config>::Hash;

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

        #[pallet::constant]
        type PalletId: Get<PalletId>;
    }

    // Errors.
    #[pallet::error]
    pub enum Error<T> {
        /// Handles arithmetic overflow when incrementing the Thread counter.
        ThreadCntOverflow,
        FeeOverflow,
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
        PoolBalances(T::AccountId, BalanceOf<T>),
        AuthorBalances(T::AccountId, BalanceOf<T>),
        RootTransferPool(T::AccountId, BalanceOf<T>),
    }

    // Storage items.

    #[pallet::storage]
    #[pallet::getter(fn thread_cnt)]
    /// Keeps track of the number of Threads in existence.
    pub(super) type ThreadCnt<T: Config> = StorageValue<_, u64, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn threads)]
    /// Stores Map of Threads.
    pub(super) type Threads<T: Config> = StorageMap<_, Twox64Concat, T::Hash, Thread<AccountOf<T>, BalanceOf<T>, TimeOf<T>, HashOf<T>>>;

    #[pallet::storage]
    #[pallet::getter(fn threads_owned)]
    /// Keeps track of what accounts own what Threads.
    pub(super) type ThreadsOwned<T: Config> =
    StorageMap<_, Twox64Concat, T::AccountId, Vec<T::Hash>, ValueQuery>;


    #[pallet::call]
    impl<T: Config> Pallet<T> {

        #[transactional]
        #[pallet::weight(10000000)]
        pub fn create_new_thread(origin: OriginFor<T>, dto: ThreadDto<BalanceOf<T>, TimeOf<T>>) -> DispatchResult {
            let author = ensure_signed(origin)?;

            // create new thread
            let mut thread: Thread<AccountOf<T>, BalanceOf<T>, TimeOf<T>, HashOf<T>> =
                Thread::new(dto, author.clone(), T::ThreadTime::now());

            // calculate the hash of thread
            let thread_hash = T::Hashing::hash_of(&thread);

            // set the hash of thread to thread id
            thread.id = Some(thread_hash.clone());

            // insert threads storage
            <Threads<T>>::insert(&thread_hash, thread.clone());

            // insert to threads owner
            <ThreadsOwned<T>>::mutate(&author, |thread_vec| {
                thread_vec.push(thread_hash)
            });

            // increase thread count
            let new_cnt = Self::thread_cnt().checked_add(1)
                .ok_or(<Error<T>>::ThreadCntOverflow)?;

            <ThreadCnt<T>>::put(new_cnt);

            let mut total_fee = thread.price;

            // increase total fee if priority is high
            let high_fee: u32 = 1_00_000;
            if Priority::High.eq(&thread.priority) {
                total_fee +=high_fee.into();
            }

            // check the creator has enough free balance
            ensure!(T::Currency::free_balance(&author) >= total_fee, <Error<T>>::NotEnoughBalance);

            let account_id = Self::account_id();

            // transfer amount from creator to thread amount pool
            T::Currency::transfer(&author, &account_id, thread.price, ExistenceRequirement::KeepAlive);

            Self::deposit_event(Event::AuthorBalances(author.clone(), T::Currency::free_balance(&author)));
            Self::deposit_event(Event::PoolBalances(account_id.clone(), T::Currency::free_balance(&account_id)));

            // deposit created new thread event
            Self::deposit_event(Event::Created(author, thread_hash));

            Ok(())
        }

        #[transactional]
        #[pallet::weight(1000)]
        pub fn demo_pool_transfer(origin: OriginFor<T>) -> DispatchResult {
            let receiver = ensure_signed(origin)?;

            let value: u32 = 1_00_000;

            let account_id = Self::account_id();

            ensure!(T::Currency::free_balance(&Self::account_id()) >= value.into(), <Error<T>>::NotEnoughBalance);

            // transfer amount from creator to thread amount pool
            T::Currency::transfer(&Self::account_id(), &receiver, value.into(), ExistenceRequirement::KeepAlive);

            Self::deposit_event(Event::AuthorBalances(receiver.clone(), T::Currency::free_balance(&receiver)));
            Self::deposit_event(Event::PoolBalances(account_id.clone(), T::Currency::free_balance(&account_id)));


            Ok(())
        }

        #[pallet::weight(1000)]
        pub fn demo_get_pool_balance(origin: OriginFor<T>) -> DispatchResult {
            let _ = ensure_signed(origin)?;

            Self::deposit_event(Event::PoolBalances(Self::account_id(), T::Currency::free_balance(&Self::account_id())));

            Ok(())
        }
    }

    impl <T: Config> Pallet<T> {
        pub fn account_id() -> T::AccountId {
            T::PalletId::get().into_account()
        }

    }
}
