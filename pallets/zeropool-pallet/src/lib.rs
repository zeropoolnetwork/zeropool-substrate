#![cfg_attr(not(feature = "std"), no_std)]
#![allow(dead_code)]

#[cfg(not(feature = "std"))]
extern crate alloc;

use borsh::BorshDeserialize;
use ff_uint::Uint;
use frame_support::traits::Currency;
use maybestd::vec::Vec;
pub use pallet::*;
use sp_core::hashing::keccak_256;

use crate::num::U256;

mod alt_bn128;
mod error;
mod maybestd;
mod num;
mod tx_decoder;
mod verifier;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
type BalanceOf<T> = <<T as Config>::Currency as Currency<AccountIdOf<T>>>::Balance;

#[derive(Debug, BorshDeserialize)]
pub struct MerkleProof<const L: usize> {
    pub sibling: [U256; L],
    pub path: [bool; L],
}

#[frame_support::pallet]
pub mod pallet {
    use crate::{
        error::ZeroPoolError,
        tx_decoder::{EvmTxDecoder, TxType},
        verifier::alt_bn128_groth16verify,
    };

    use super::*;
    use frame_support::{
        pallet_prelude::*,
        sp_runtime::traits::{AccountIdConversion, CheckedSub},
        traits::{Currency, ExistenceRequirement},
        PalletId,
    };
    use frame_system::pallet_prelude::*;

    /// Configure the pallet by specifying the parameters and types on which it depends.
    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// Because this pallet emits events, it depends on the runtime's definition of an event.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        #[pallet::constant]
        type PalletId: Get<PalletId>;
        /// The staking balance.
        type Currency: Currency<Self::AccountId>;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::storage]
    pub type Balances<T> =
        StorageMap<_, Blake2_128Concat, <T as frame_system::Config>::AccountId, BalanceOf<T>>;

    #[pallet::storage]
    pub type Nullifiers<T> = StorageMap<_, Blake2_128Concat, U256, U256>;

    #[pallet::storage]
    pub type Roots<T> = StorageMap<_, Blake2_128Concat, U256, U256>;

    #[pallet::storage]
    pub type PoolIndex<T> = StorageValue<_, U256, ValueQuery>;

    #[pallet::storage]
    pub type AllMessagesHash<T> = StorageValue<_, U256, ValueQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// [who, amount]
        Lock(T::AccountId, BalanceOf<T>),
        /// [who, amount]
        Release(T::AccountId, BalanceOf<T>),
        /// [pool_index, all_messages_hash, memo]
        Message(U256, U256, Vec<u8>),
    }

    #[pallet::error]
    pub enum Error<T> {
        InsufficientBalance,
        InvalidTxFormat,
        DoubleSpend,
        IndexOutOfBounds,
        AltBn128DeserializationError,
        AltBn128SerializationError,
        NotConsistentGroth16InputsError,
    }

    impl<T> From<ZeroPoolError> for Error<T> {
        fn from(err: ZeroPoolError) -> Self {
            match err {
                ZeroPoolError::AltBn128DeserializationError => Error::AltBn128DeserializationError,
                ZeroPoolError::AltBn128SerializationError => Error::AltBn128SerializationError,
                ZeroPoolError::NotConsistentGroth16InputsError =>
                    Error::NotConsistentGroth16InputsError,
            }
        }
    }

    impl<T: Config> Pallet<T> {
        pub fn account_id() -> T::AccountId {
            T::PalletId::get().into_account()
        }
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        // TODO: Use SCALE codec instead of borsh
        // TODO: Split into separate methods?
        #[pallet::weight(1000)]
        pub fn transact(origin: OriginFor<T>, data: Vec<u8>) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let tx = EvmTxDecoder::new(data.as_slice());

            // Verify transfer proof
            let vk = (); // FIXME: load VK
            let transact_inputs = [];
            alt_bn128_groth16verify(&vk, &tx.transact_proof(), &transact_inputs)
                .map_err(|err| err.into())?;

            if <Nullifiers<T>>::contains_key(tx.nullifier()) {
                return Err(Error::DoubleSpend.into())
            }

            let mut pool_index = <PoolIndex<T>>::get();
            if tx.transfer_index() > pool_index {
                return Err(Error::IndexOutOfBounds.into())
            }

            // Verify tree proof
            let vk = (); // FIXME: load VK
            let tree_inputs = [];
            alt_bn128_groth16verify(&vk, &tx.tree_proof(), &tree_inputs)
                .map_err(|err| err.into())?;

            // Set the nullifier
            let mut elements = [0u8; core::mem::size_of::<U256>() * 2]; // FIXME: Proper size
            tx.out_commit().using_encoded(|data| {
                elements[..core::mem::size_of::<U256>()].copy_from_slice(data);
            });
            tx.delta().using_encoded(|data| {
                elements[core::mem::size_of::<U256>()..].copy_from_slice(data);
            });
            let hash = U256::from_big_endian(&keccak_256(&elements));
            <Nullifiers<T>>::insert(tx.nullifier(), hash);

            pool_index = pool_index.unchecked_add(U256::from(128u8));
            <PoolIndex<T>>::put(pool_index);
            <Roots<T>>::insert(pool_index, tx.root_after());

            // Calculate all_messages_hash
            let message_hash = keccak_256(tx.memo_message());
            let mut hashes = [0u8; 32 * 2];
            let all_messages_hash = <AllMessagesHash<T>>::get();
            all_messages_hash.using_encoded(|data| hashes[..32].copy_from_slice(data));
            hashes[32..].copy_from_slice(&message_hash);
            let new_all_messages_hash = U256::from_big_endian(&keccak_256(&hashes));
            <AllMessagesHash<T>>::put(new_all_messages_hash);

            Self::deposit_event(Event::Message(
                pool_index,
                new_all_messages_hash,
                tx.memo_message().to_owned(),
            ));

            // FIXME: use signed integers
            let fee = tx.memo_fee();
            let token_amount = tx.token_amount().unchecked_add(fee);
            let energy_amount = tx.energy_amount();
            // TODO: Use a denominator to prevent overflow
            let native_amount = <BalanceOf<T>>::decode(&mut token_amount.encode().into()).into()?;

            match tx.tx_type() {
                TxType::Deposit => {
                    // TODO: Check amounts
                    T::Currency::transfer(
                        &who,
                        &Self::account_id(),
                        native_amount,
                        ExistenceRequirement::KeepAlive,
                    )?;
                },
                TxType::Transfer => {
                    // TODO: Check amounts
                },
                TxType::Withdraw => {
                    // TODO: Check amounts
                    T::Currency::transfer(
                        &Self::account_id(),
                        &who, // FIXME: get receiver from memo
                        native_amount,
                        ExistenceRequirement::KeepAlive,
                    )?;
                },
            }

            Ok(())
        }

        #[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
        pub fn lock(origin: OriginFor<T>, amount: BalanceOf<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let total = <Balances<T>>::get(who.clone())
                .map(|balance| balance + amount)
                .unwrap_or(amount);

            <Balances<T>>::insert(who.clone(), total);

            T::Currency::transfer(
                &who,
                &Self::account_id(),
                amount,
                ExistenceRequirement::KeepAlive,
            )?;

            Self::deposit_event(Event::Lock(who, amount));
            Ok(())
        }

        #[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
        pub fn release(origin: OriginFor<T>, amount: BalanceOf<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;

            if let Some(balance) = <Balances<T>>::get(who.clone()) {
                let new_balance =
                    balance.checked_sub(&amount).ok_or(Error::<T>::InsufficientBalance)?;
                if new_balance == 0u32.into() {
                    <Balances<T>>::remove(who.clone());
                } else {
                    <Balances<T>>::insert(who.clone(), new_balance);
                }

                T::Currency::transfer(
                    &Self::account_id(),
                    &who,
                    amount,
                    ExistenceRequirement::KeepAlive,
                )?;
            } else {
                return Err(Error::<T>::InsufficientBalance.into())
            }

            Self::deposit_event(Event::Release(who, amount));
            Ok(())
        }
    }
}
