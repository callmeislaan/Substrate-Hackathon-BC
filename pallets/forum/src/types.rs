use frame_support::{pallet_prelude::*};
use frame_system::pallet_prelude::*;
use frame_support::{
    traits::{Randomness, Currency, tokens::ExistenceRequirement, Time}
};
use sp_io::hashing::blake2_128;
use scale_info::TypeInfo;
use core::fmt::Debug;
use frame_support::inherent::Vec;
use sp_runtime::traits::{Hash, Zero};

#[cfg(feature = "std")]
use frame_support::serde::{Deserialize, Serialize};

// Struct for holding Thread information.
#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
#[scale_info(skip_type_params(T))]
#[codec(mel_bound())]
pub struct Thread<AccountOf, BalanceOf, TimeOf, HashOf> {
    pub id: Option<HashOf>,
    pub topic: Topic,
    pub title: Vec<u8>,
    pub content: Vec<u8>,
    pub priority: Priority,
    pub author: AccountOf,
    pub price: BalanceOf,
    pub created: TimeOf,
    pub close_time: TimeOf,
    pub status: Status,
}

#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
#[scale_info(skip_type_params(T))]
#[codec(mel_bound())]
pub struct ThreadDto<BalanceOf, TimeOf> {
    pub topic: Topic,
    pub title: Vec<u8>,
    pub content: Vec<u8>,
    pub priority: Priority,
    pub price: BalanceOf,
    pub close_time: TimeOf,
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
    English,
    IT,
}

#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum Status {
    Opening,
    Closed,
    Canceled,
    Answered,
}


impl <AccountOf, BalanceOf, TimeOf, HashOf> Thread<AccountOf, BalanceOf, TimeOf, HashOf> {
    pub fn new(dto: ThreadDto<BalanceOf, TimeOf>, author: AccountOf, created: TimeOf) -> Thread<AccountOf, BalanceOf, TimeOf, HashOf> {
        Thread {
            id: None,
            topic: dto.topic,
            title: dto.title,
            content: dto.content,
            priority: dto.priority,
            author,
            price: dto.price,
            created,
            close_time: dto.close_time,
            status: Status::Opening,
        }
    }
}

#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
#[scale_info(skip_type_params(T))]
#[codec(mel_bound())]
pub struct Answer<AccountOf, TimeOf, HashOf> {
    pub id: Option<HashOf>,
    pub thread_id: HashOf,
    pub answer: Vec<u8>,
    pub author: AccountOf,
    pub created: TimeOf,
}

#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
#[scale_info(skip_type_params(T))]
#[codec(mel_bound())]
pub struct Vote<AccountOf, TimeOf> {
    pub voter: AccountOf,
    pub vote_time: TimeOf,
}

#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
#[scale_info(skip_type_params(T))]
#[codec(mel_bound())]
pub struct Verify<AccountOf, TimeOf, HashOf> {
    pub verifier: AccountOf,
    pub verify_time: TimeOf,
    pub best_answer_id: HashOf,
}

#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
#[scale_info(skip_type_params(T))]
#[codec(mel_bound())]
pub struct Rank<AccountOf> {
    pub account: AccountOf,
    pub number_of_best_answer: u128,
}
