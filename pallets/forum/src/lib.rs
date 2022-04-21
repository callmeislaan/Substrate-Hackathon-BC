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
    use crate::pallet;
    use crate::types::{Answer, Priority, Rank, Status, Thread, ThreadDto, Topic, Verify, Vote};

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
        ThreadClosed,
        ThreadNotFound,
        TopicNotFound,
        NotPermission,
        VoteCntOverflow,
        TopicVoterAlreadyAdded,
        TopicVoterNotExists,
        ThreadNotAnswered,
        RankingOverflow,
    }

    // Events.
    #[pallet::event]
    #[pallet::generate_deposit(pub (super) fn deposit_event)]
    pub enum Event<T> where T: Config + Debug {
        /// A new Thread was successfully created. \[sender, thread_id\]
        Created(AccountOf<T>, HashOf<T>),
        /// Thread price was successfully set. \[sender, thread_id, new_price\]
        PriceSet(AccountOf<T>, HashOf<T>, Option<BalanceOf<T>>),
        /// A Thread was successfully transferred. \[from, to, thread_id\]
        Transferred(AccountOf<T>, AccountOf<T>, HashOf<T>),
        /// A Thread was successfully bought. \[buyer, seller, thread_id, bid_price\]
        Bought(AccountOf<T>, AccountOf<T>, HashOf<T>, BalanceOf<T>),
        PoolBalances(AccountOf<T>, BalanceOf<T>),
        AuthorBalances(AccountOf<T>, BalanceOf<T>),
        RootTransferPool(AccountOf<T>, BalanceOf<T>),
        AnswerPushed(AccountOf<T>, HashOf<T>, HashOf<T>),
        Voted(AccountOf<T>, HashOf<T>, HashOf<T>),
        TopicVoterAdded(Topic, AccountOf<T>),
        TopicVoterAlreadyAdded(Topic, AccountOf<T>),
        ThreadAnswerVerified(HashOf<T>, HashOf<T>, AccountOf<T>),
        ThreadAnswered(HashOf<T>, HashOf<T>),
        AnswerForAccount(HashOf<T>, AccountOf<T>),
        RankingUpdated(Topic, AccountOf<T>, u128),
        // A Thread has been closed with no reply
        ThreadClosedWithOutAnswer(HashOf<T>),
        // A Thread has been closed with reply and executed transfer function
        ThreadCloseWithAnswer(HashOf<T>, AccountOf<T>, AccountOf<T>, BalanceOf<T>)

    }

    // Storage items.

    #[pallet::storage]
    #[pallet::getter(fn thread_cnt)]
    /// Keeps track of the number of Threads in existence.
    pub(super) type ThreadCnt<T: Config> = StorageValue<_, u64, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn threads)]
    /// Stores Map of Threads.
    pub(super) type Threads<T: Config> = StorageMap<_, Twox64Concat, HashOf<T>, Thread<AccountOf<T>, BalanceOf<T>, TimeOf<T>, HashOf<T>>>;

    #[pallet::storage]
    #[pallet::getter(fn threads_owned)]
    /// Keeps track of what accounts own what Threads.
    pub(super) type ThreadsOwned<T: Config> =
    StorageMap<_, Twox64Concat, AccountOf<T>, Vec<HashOf<T>>, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn answers)]
    /// answer id => answer
    pub(super) type Answers<T: Config> = StorageMap<_, Twox64Concat, HashOf<T>, Answer<AccountOf<T>, TimeOf<T>, HashOf<T>>>;

    #[pallet::storage]
    #[pallet::getter(fn thread_answers)]
    /// thread id => list of answer id of this thread
    pub(super) type ThreadAnswers<T: Config> = StorageMap<_, Twox64Concat, HashOf<T>, Vec<HashOf<T>>, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn thread_best_answer)]
    /// thread id => best answer id of this thread
    pub(super) type ThreadBestAnswer<T: Config> = StorageMap<_, Twox64Concat, HashOf<T>, Verify<AccountOf<T>, TimeOf<T>, HashOf<T>>>;

    #[pallet::storage]
    #[pallet::getter(fn answer_owner)]
    /// answer id => owner of answer
    pub(super) type AnswerOwner<T: Config> = StorageMap<_, Twox64Concat, HashOf<T>, AccountOf<T>>;

    #[pallet::storage]
    #[pallet::getter(fn topic_voter)]
    /// topic => list of account id has permission to vote
    pub(super) type TopicVoter<T: Config> = StorageMap<_, Twox64Concat, Topic, Vec<AccountOf<T>>, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn answer_vote_cnt)]
    /// answer id => vote number of this answer
    pub(super) type AnswerVoteCnt<T: Config> = StorageMap<_, Twox64Concat, HashOf<T>, u64, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn answer_votes)]
    /// answer id => list of voter vote for this answer
    pub(super) type AnswerVoters<T: Config> = StorageMap<_, Twox64Concat, HashOf<T>, Vec<Vote<AccountOf<T>, TimeOf<T>>>, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn topic_verifiers)]
    /// topic => list of verifier has permission to verify
    pub(super) type TopicVerifiers<T: Config> = StorageMap<_, Twox64Concat, Topic, Vec<AccountOf<T>>, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn topic_ranking)]
    /// topic => list of account id and best answer number of this account
    pub(super) type TopicRanking<T: Config> = StorageDoubleMap<_, Twox64Concat, Topic, Twox64Concat, AccountOf<T>, u128, ValueQuery>;

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
                .ok_or(Error::<T>::ThreadCntOverflow)?;

            <ThreadCnt<T>>::put(new_cnt);

            let mut total_fee = thread.price;

            // increase total fee if priority is high
            let high_fee: u32 = 1_00_000;
            if Priority::High.eq(&thread.priority) {
                total_fee += high_fee.into();
            }

            // check the creator has enough free balance
            ensure!(T::Currency::free_balance(&author) >= total_fee, Error::<T>::NotEnoughBalance);

            let account_id = Self::account_id();

            // transfer amount from creator to thread amount pool
            T::Currency::transfer(&author, &account_id, total_fee, ExistenceRequirement::KeepAlive)?;

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

            ensure!(T::Currency::free_balance(&Self::account_id()) >= value.into(), Error::<T>::NotEnoughBalance);

            // transfer amount from creator to thread amount pool
            T::Currency::transfer(&Self::account_id(), &receiver, value.into(), ExistenceRequirement::KeepAlive)?;

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

        #[pallet::weight(1000)]
        pub fn push_answer(origin: OriginFor<T>, answer_content: Vec<u8>, thread_id: HashOf<T>) -> DispatchResult {
            let answer_author = ensure_signed(origin)?;

            let thread: Thread<AccountOf<T>, BalanceOf<T>, TimeOf<T>, HashOf<T>> =
                Self::threads(&thread_id).ok_or(Error::<T>::ThreadNotFound)?;

            ensure!(Status::Closed.ne(&thread.status), Error::<T>::ThreadClosed);

            let mut answer = Answer {
                id: None,
                thread_id,
                answer: answer_content,
                author: answer_author.clone(),
                created: T::ThreadTime::now(),
            };

            let answer_hash = T::Hashing::hash_of(&answer);

            answer.id = Some(answer_hash.clone());

            <Answers<T>>::insert(answer_hash, answer.clone());

            <ThreadAnswers<T>>::mutate(&thread_id, |answer_vec| {
                answer_vec.push(answer_hash)
            });

            AnswerOwner::<T>::insert(answer_hash.clone(), answer_author.clone());

            Self::deposit_event(Event::AnswerPushed(answer_author, thread_id, answer_hash));

            Ok(())
        }

        #[pallet::weight(1000)]
        pub fn vote(origin: OriginFor<T>, thread_id: HashOf<T>, answer_id: HashOf<T>) -> DispatchResult {
            let voter = ensure_signed(origin)?;

            let thread: Thread<AccountOf<T>, BalanceOf<T>, TimeOf<T>, HashOf<T>> =
                Self::threads(&thread_id).ok_or(Error::<T>::ThreadNotFound)?;

            let topic_voter: Vec<AccountOf<T>> = Self::topic_voter(thread.topic);

            if !topic_voter.contains(&voter) {
                return Err(Error::<T>::NotPermission.into());
            }

            let new_vote = Self::answer_vote_cnt(answer_id.clone()).checked_add(1)
                .ok_or(Error::<T>::VoteCntOverflow)?;

            <AnswerVoteCnt<T>>::insert(answer_id.clone(), new_vote);

            let vote = Vote {
                voter: voter.clone(),
                vote_time: T::ThreadTime::now(),
            };

            <AnswerVoters<T>>::mutate(&answer_id, |voter_vec| { voter_vec.push(vote) });

            Self::deposit_event(Event::Voted(voter, thread_id, answer_id));

            Ok(())
        }

        #[pallet::weight(1000)]
        pub fn add_topic_voter(origin: OriginFor<T>, topic: Topic, new_voter: AccountOf<T>) -> DispatchResult {
            // let _ = ensure_root(origin)?;

            let _ = ensure_signed(origin)?;
            // todo ensure root

            <TopicVoter<T>>::mutate(&topic, |voter_vec| {
                match voter_vec.binary_search(&new_voter) {
                    Ok(_) => Err(Error::<T>::TopicVoterAlreadyAdded.into()),
                    Err(_) => {
                        voter_vec.push(new_voter.clone());
                        Self::deposit_event(Event::TopicVoterAdded(topic.clone(), new_voter));
                        Ok(())
                    }
                }
            })
        }

        #[pallet::weight(1000)]
        pub fn remove_topic_voter(origin: OriginFor<T>, topic: Topic, voter: AccountOf<T>) -> DispatchResult {
            // let _ = ensure_root(origin)?;

            let _ = ensure_signed(origin)?;
            // todo ensure root

            <TopicVoter<T>>::mutate(&topic, |voter_vec| {
                match voter_vec.binary_search(&voter) {
                    Ok(index) => {
                        voter_vec.remove(index);
                        Self::deposit_event(Event::TopicVoterAdded(topic.clone(), voter));
                        Ok(())
                    }
                    Err(_) => Err(Error::<T>::TopicVoterAlreadyAdded.into()),
                }
            })
        }

        #[pallet::weight(1000)]
        pub fn verify(origin: OriginFor<T>, thread_id: HashOf<T>, answer_id: HashOf<T>) -> DispatchResult {
            let verifier = ensure_signed(origin)?;

            let thread: Thread<AccountOf<T>, BalanceOf<T>, TimeOf<T>, HashOf<T>> =
                Self::threads(&thread_id).ok_or(Error::<T>::ThreadNotFound)?;

            let verifiers: Vec<AccountOf<T>> = Self::topic_verifiers(thread.topic.clone());

            match verifiers.binary_search(&verifier) {
                Ok(_) => {
                    let verify = Verify {
                        verifier: verifier.clone(),
                        verify_time: T::ThreadTime::now(),
                        best_answer_id: answer_id.clone(),
                    };
                    ThreadBestAnswer::<T>::insert(thread_id.clone(), verify);
                    Self::deposit_event(Event::ThreadAnswerVerified(thread_id, answer_id, verifier));
                    Threads::<T>::mutate(&thread_id, |thread_option| {
                        if let Some(thread) = thread_option {
                            thread.status = Status::Answered;
                            Self::deposit_event(Event::ThreadAnswered(thread_id.clone(), answer_id.clone()));
                        }
                    });
                    let best_answer_owner: AccountOf<T> = Self::answer_owner(answer_id.clone()).unwrap();

                    // increase account ranking
                    let new_ranking = Self::topic_ranking(thread.topic.clone(), best_answer_owner.clone()).checked_add(1)
                        .ok_or(Error::<T>::RankingOverflow)?;

                    TopicRanking::<T>::insert(thread.topic.clone(), best_answer_owner.clone(), new_ranking);

                    Self::deposit_event(Event::RankingUpdated(thread.topic.clone(), best_answer_owner.clone(), new_ranking));

                    // todo payout for answerer and voter
                    Ok(())
                }
                Err(_) => Err(Error::<T>::NotPermission.into()),
            }
        }

        #[pallet::weight(1000)]
        pub fn add_topic_verifier(origin: OriginFor<T>, topic: Topic, new_verifier: AccountOf<T>) -> DispatchResult {
            // let _ = ensure_root(origin)?;

            let _ = ensure_signed(origin)?;
            // todo ensure root

            <TopicVerifiers<T>>::mutate(&topic, |verifier_vec| {
                match verifier_vec.binary_search(&new_verifier) {
                    Ok(_) => Err(Error::<T>::TopicVoterAlreadyAdded.into()),
                    Err(_) => {
                        verifier_vec.push(new_verifier.clone());
                        Self::deposit_event(Event::TopicVoterAdded(topic.clone(), new_verifier));
                        Ok(())
                    }
                }
            })
        }

        #[pallet::weight(1000)]
        pub fn remove_topic_verifier(origin: OriginFor<T>, topic: Topic, verifier: AccountOf<T>) -> DispatchResult {
            // let _ = ensure_root(origin)?;

            let _ = ensure_signed(origin)?;
            // todo ensure root

            <TopicVerifiers<T>>::mutate(&topic, |verifier_vec| {
                match verifier_vec.binary_search(&verifier) {
                    Ok(index) => {
                        verifier_vec.remove(index);
                        Self::deposit_event(Event::TopicVoterAdded(topic.clone(), verifier));
                        Ok(())
                    }
                    Err(_) => Err(Error::<T>::TopicVoterAlreadyAdded.into()),
                }
            })
        }

        /// Phần này làm trên blockchain không được, tại blockchain tất cả thông tin đề public.
        /// Đây cũng là điểm cần lưu ý khi ứng dụng blockchain vào dự án thực thế.
        /// Hiện tại cứ làm trong đây để có bản demo
        #[pallet::weight(1000)]
        pub fn watch_answer(origin: OriginFor<T>, thread_id: HashOf<T>) -> DispatchResult{
            let watcher = ensure_signed(origin)?;

            let thread: Thread<AccountOf<T>, BalanceOf<T>, TimeOf<T>, HashOf<T>> =
                Self::threads(&thread_id).ok_or(Error::<T>::ThreadNotFound)?;

            if Status::Answered.ne(&thread.status) {
                return Err(Error::<T>::ThreadNotAnswered.into());
            }

            let thread_author: AccountOf<T> = thread.author;
            let thread_best_answer: Verify<AccountOf<T>, TimeOf<T>, HashOf<T>> =
                Self::thread_best_answer(thread_id.clone()).ok_or(Error::<T>::ThreadNotAnswered)?;
            let best_answer_id = thread_best_answer.best_answer_id;
            let thread_answer_author: AccountOf<T>  = Self::answer_owner(best_answer_id.clone()).unwrap();
            let thread_answer_voter: Vec<Vote<AccountOf<T>, TimeOf<T>>> = Self::answer_votes(best_answer_id.clone());
            let fee: u32 = 1_00_000;
            // todo transfer fee to thread author and thread answerer and voter for this answer

            Self::deposit_event(Event::AnswerForAccount(best_answer_id, watcher));
            Ok(())
        }

        #[pallet::weight(0)]
        pub fn user_close_thread(origin: OriginFor<T>, thread_id: HashOf<T>) -> DispatchResult {
            let caller = ensure_signed(origin)?;
            if !Self::threads_owned(caller).contains(&thread_id) {
                return Err(Error::<T>::NotThreadOwner.into());
            }
            Self::close_thread(thread_id)
        }

        // TODO: add auto close thread when thread_close_time is over
    }

    impl<T: Config> Pallet<T> {
        pub fn account_id() -> AccountOf<T> {
            T::PalletId::get().into_account()
        }

        pub fn close_thread(thread_id: HashOf<T>) -> DispatchResult {
            let mut thread: Thread<AccountOf<T>, BalanceOf<T>, TimeOf<T>, HashOf<T>> =
                Self::threads(&thread_id).ok_or(Error::<T>::ThreadNotFound)?;

            let get_best_answer = Self::get_currently_best_answer(thread_id);

            if get_best_answer.len() == 0 {
                Self::deposit_event(Event::ThreadClosedWithOutAnswer(thread_id));
            }
            else if get_best_answer.len() == 1 {
                let best_ans_author = get_best_answer[0].author.clone();
                T::Currency::transfer(&thread.author, &best_ans_author, thread.price, ExistenceRequirement::KeepAlive)?;
                Self::deposit_event(Event::ThreadCloseWithAnswer(thread_id, thread.author, best_ans_author, thread.price));
            }
            // TODO: add to needed verify answer list
            // else {}

            thread.status = pallet::Status::Closed;
            
            Ok(())
        }

        // get a list of best answer frome a thread id
        pub fn get_currently_best_answer(thread_id: HashOf<T>) ->Vec<Answer<AccountOf<T>, TimeOf<T>, HashOf<T>>> {
            let answer_list = Self::thread_answers(thread_id);
            let mut highest_vote = 0;
            let mut best_answer_list = Vec::<Answer<AccountOf<T>, TimeOf<T>, HashOf<T>>>::new();
            for n in 0..answer_list.len() {
                let index_vote_cnt = Self::answer_vote_cnt(answer_list[n]);
                if highest_vote < index_vote_cnt {
                    highest_vote = index_vote_cnt;
                    best_answer_list.clear();
                }
                if highest_vote == index_vote_cnt {
                    best_answer_list.push(Self::answers(answer_list[n]).unwrap());
                }
            }
            best_answer_list
        }
    }
}
//