#![cfg_attr(not(feature = "std"), no_std)]
#![allow(dead_code)]

#[cfg(not(feature = "std"))]
extern crate alloc;

use core::str::FromStr;

use borsh::BorshDeserialize;
use ff_uint::Uint;
use frame_support::traits::Currency;
use lazy_static::lazy_static;
use maybestd::vec::Vec;
pub use pallet::*;
use sp_io::hashing::keccak_256;
use verifier::VK;

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

lazy_static! {
    static ref R: U256 = U256::from_str(
        "21888242871839275222246405745257275088548364400416034343698204186575808495617"
    )
    .unwrap();
}

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
        sp_runtime::traits::AccountIdConversion,
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
    pub type Nullifiers<T> = StorageMap<_, Blake2_128Concat, U256, U256>;

    #[pallet::storage]
    pub type Roots<T> = StorageMap<_, Blake2_128Concat, U256, U256, ValueQuery>;

    #[pallet::storage]
    pub type PoolIndex<T> = StorageValue<_, U256, ValueQuery>;

    #[pallet::storage]
    pub type AllMessagesHash<T> = StorageValue<_, U256, ValueQuery>;

    #[pallet::storage]
    pub type TransferVk<T> = StorageValue<_, VK>;

    #[pallet::storage]
    pub type TreeVk<T> = StorageValue<_, VK>;

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

        // TODO: Find a way to transform codec errors into DispatchError
        Deserialization,
        IncorrectAmount,

        TransferVkNotSet,
        TreeVkNotSet,
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
        #[pallet::weight(1000)]
        pub fn set_transfer_vk(origin: OriginFor<T>, data: VK) -> DispatchResult {
            let who = ensure_signed(origin)?;
            // TODO: Ensure operator

            <TransferVk<T>>::put(data);

            Ok(())
        }

        #[pallet::weight(1000)]
        pub fn set_tree_vk(origin: OriginFor<T>, data: VK) -> DispatchResult {
            let who = ensure_signed(origin)?;
            // TODO: Ensure operator

            <TreeVk<T>>::put(data);

            Ok(())
        }

        // TODO: Use SCALE codec for transaction data
        // TODO: Split into separate methods?
        // TODO: Weight
        #[pallet::weight(1000)]
        pub fn transact(origin: OriginFor<T>, data: Vec<u8>) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let tx = EvmTxDecoder::new(data.as_slice());

            let message_hash = keccak_256(tx.memo_message());
            let mut pool_index = <PoolIndex<T>>::get();
            let root_before = <Roots<T>>::get(pool_index);

            // Verify transfer proof
            let transfer_vk = <TransferVk<T>>::get().ok_or(Error::<T>::TransferVkNotSet)?;
            // FIXME: delta + (_pool_id()<<(transfer_delta_size*8));
            let transact_inputs = [
                root_before,
                tx.nullifier(),
                tx.out_commit(),
                tx.delta(),
                U256::from_big_endian(&message_hash),
            ];
            alt_bn128_groth16verify(&transfer_vk, &tx.transact_proof(), &transact_inputs)
                .map_err(|err| Into::<Error<T>>::into(err))?;

            if <Nullifiers<T>>::contains_key(tx.nullifier()) {
                return Err(Error::<T>::DoubleSpend.into())
            }

            if tx.transfer_index() > pool_index {
                return Err(Error::<T>::IndexOutOfBounds.into())
            }

            // Verify tree proof
            let tree_vk = <TreeVk<T>>::get().ok_or(Error::<T>::TreeVkNotSet)?;
            let tree_inputs = [root_before, tx.root_after(), tx.out_commit()];
            alt_bn128_groth16verify(&tree_vk, &tx.tree_proof(), &tree_inputs)
                .map_err(|err| Into::<Error<T>>::into(err))?;

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
            let mut hashes = [0u8; 32 * 2];
            let all_messages_hash = <AllMessagesHash<T>>::get();
            all_messages_hash.using_encoded(|data| hashes[..32].copy_from_slice(data));
            hashes[32..].copy_from_slice(&message_hash);
            let new_all_messages_hash = U256::from_big_endian(&keccak_256(&hashes));
            <AllMessagesHash<T>>::put(new_all_messages_hash);

            Self::deposit_event(Event::Message(
                pool_index,
                new_all_messages_hash,
                tx.memo_message().to_vec(),
            ));

            // FIXME: use signed integers
            let fee = tx.memo_fee();
            let token_amount = tx.token_amount().unchecked_add(fee);
            let energy_amount = tx.energy_amount();
            // TODO: Use a denominator to prevent overflow
            let encoded_amount = token_amount.encode();
            let native_amount = <BalanceOf<T>>::decode(&mut &encoded_amount[..])
                .map_err(|_err| Into::<DispatchError>::into(Error::<T>::Deserialization))?;

            match tx.tx_type() {
                TxType::Deposit => {
                    if token_amount > U256::MAX.unchecked_div(U256::from(2u32)) ||
                        energy_amount != U256::ZERO
                    {
                        return Err(Error::<T>::IncorrectAmount.into())
                    }

                    T::Currency::transfer(
                        &who,
                        &Self::account_id(),
                        native_amount,
                        ExistenceRequirement::KeepAlive,
                    )?;
                },
                TxType::Transfer =>
                    if token_amount != U256::ZERO || energy_amount != U256::ZERO {
                        return Err(Error::<T>::IncorrectAmount.into())
                    },
                TxType::Withdraw => {
                    if token_amount < U256::MAX.unchecked_div(U256::from(2u32)) ||
                        energy_amount < U256::MAX.unchecked_div(U256::from(2u32))
                    {
                        return Err(Error::<T>::IncorrectAmount.into())
                    }

                    // TODO: Voucher token

                    T::Currency::transfer(
                        &Self::account_id(),
                        &who, // FIXME: get receiver from memo
                        native_amount,
                        ExistenceRequirement::KeepAlive,
                    )?;
                },
                // TODO: Fee for operator
            }

            Ok(())
        }
    }
}
