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
    English
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
        }
    }
}