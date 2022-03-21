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
use sp_runtime::traits::Hash;
use verifier::VK;

use crate::num::{NativeU256, U256};
pub use crate::operator::OperatorManager;

mod alt_bn128;
mod error;
mod maybestd;
pub mod num;
mod tx_decoder;
mod verifier;
mod operator;

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
    static ref DENOMINATOR: U256 = U256::from(1000u64);
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
        tx_decoder::{TxDecoder, TxType},
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
    use sp_core::crypto::Public;
    use sp_runtime::traits::Verify;

    /// Configure the pallet by specifying the parameters and types on which it depends.
    #[pallet::config]
    pub trait Config: frame_system::Config {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        #[pallet::constant]
        type PalletId: Get<PalletId>;
        type Currency: Currency<Self::AccountId>;

        type OperatorManager: OperatorManager<Self::AccountId>;

        #[pallet::constant]
        type InitialOwner: Get<Self::AccountId>;

        #[pallet::constant]
        type PoolId: Get<NativeU256>;

        #[pallet::constant]
        type FirstRoot: Get<NativeU256>;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::storage]
    pub type Nullifiers<T> = StorageMap<_, Blake2_128Concat, NativeU256, NativeU256>;

    #[pallet::type_value]
    pub fn FirstRoot<T: Config>() -> NativeU256 {
        T::FirstRoot::get()
    }

    #[pallet::storage]
    pub type Roots<T> =
        StorageMap<_, Blake2_128Concat, NativeU256, NativeU256, ValueQuery, FirstRoot<T>>;

    #[pallet::storage]
    pub type PoolIndex<T> = StorageValue<_, NativeU256, ValueQuery>;

    #[pallet::storage]
    pub type AllMessagesHash<T> = StorageValue<_, NativeU256, ValueQuery>;

    #[pallet::storage]
    pub type TransferVk<T> = StorageValue<_, VK>;

    #[pallet::storage]
    pub type TreeVk<T> = StorageValue<_, VK>;

    #[pallet::type_value]
    pub fn DefaultOwner<T: Config>() -> T::AccountId {
        T::InitialOwner::get()
    }

    #[pallet::storage]
    pub type Owner<T: Config> = StorageValue<_, T::AccountId, ValueQuery, DefaultOwner<T>>;

    #[pallet::storage]
    pub type Operator<T: Config> = StorageValue<_, T::AccountId>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// [pool_index, all_messages_hash, commitment, memo]
        Message(NativeU256, NativeU256, NativeU256, Vec<u8>),
        TransferVkSet,
        TreeVkSet,
    }

    #[pallet::error]
    pub enum Error<T> {
        AltBn128DeserializationError,
        AltBn128SerializationError,
        NotConsistentGroth16InputsError,

        IndexOutOfBounds,
        InsufficientBalance,
        InvalidTxFormat,
        DoubleSpend,
        InvalidDepositSignature,
        Deserialization,
        IncorrectAmount,
        TransferVkNotSet,
        TreeVkNotSet,
        NotOwner,
        NotOperator,
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

        fn operator() -> Result<T::AccountId, DispatchError> {
            <<T as Config>::OperatorManager>::operator().ok_or(Error::<T>::NotOperator.into())
        }

        fn owner() -> T::AccountId {
            <Owner<T>>::get()
        }

        fn check_operator(origin: OriginFor<T>) -> Result<T::AccountId, DispatchError> {
            let who = ensure_signed(origin)?;
            let operator = Self::operator()?;

            if who != operator {
                log::warn!("Failed to enforce an operator");
                return Err(Error::<T>::NotOperator.into())
            }

            Ok(who)
        }

        fn check_owner(origin: OriginFor<T>) -> Result<T::AccountId, DispatchError> {
            let who = ensure_signed(origin)?;
            let owner = Self::owner();

            if who != owner {
                return Err(Error::<T>::NotOwner.into())
            }

            Ok(who)
        }
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(1000)]
        pub fn set_owner(origin: OriginFor<T>, address: T::AccountId) -> DispatchResult {
            Self::check_owner(origin)?;

            <Operator<T>>::put(address);

            Ok(())
        }

        // #[pallet::weight(1000)]
        // pub fn set_operator(origin: OriginFor<T>, address: T::AccountId) -> DispatchResult {
        //     Self::check_owner(origin)?;

        //     <Operator<T>>::put(address.clone());

        //     Self::deposit_event(Event::OperatorSet(address));

        //     Ok(())
        // }

        #[pallet::weight(1000)]
        pub fn set_transfer_vk(origin: OriginFor<T>, data: Vec<u8>) -> DispatchResult {
            Self::check_owner(origin)?;

            let vk = VK::try_from_slice(&data)
                .map_err(|_err| Into::<DispatchError>::into(Error::<T>::Deserialization))?;
            <TransferVk<T>>::put(vk);

            Self::deposit_event(Event::TransferVkSet);

            Ok(())
        }

        #[pallet::weight(1000)]
        pub fn set_tree_vk(origin: OriginFor<T>, data: Vec<u8>) -> DispatchResult {
            Self::check_owner(origin)?;

            let vk = VK::try_from_slice(&data)
                .map_err(|_err| Into::<DispatchError>::into(Error::<T>::Deserialization))?;
            <TreeVk<T>>::put(vk);

            Self::deposit_event(Event::TreeVkSet);

            Ok(())
        }

        // TODO: Split into separate methods?
        // TODO: Weight
        #[pallet::weight(1000)]
        pub fn transact(origin: OriginFor<T>, data: Vec<u8>) -> DispatchResult {
            let operator = Self::check_operator(origin)?;

            log::info!("Processing ZeroPool transaction");

            let tx = TxDecoder::new(data.as_slice());
            let message_hash = keccak_256(tx.memo_message()); // FIXME: REDUCE!!!
            let message_hash_num = U256::from_little_endian(&message_hash).unchecked_rem(*R);
            let mut pool_index: U256 = <PoolIndex<T>>::get().into();
            let root_before: U256 = <Roots<T>>::get::<NativeU256>(pool_index.into()).into();

            // Verify transfer proof
            log::debug!("Verifying transfer proof:");
            let transfer_vk = <TransferVk<T>>::get().ok_or(Error::<T>::TransferVkNotSet)?;
            let pool_id: U256 = T::PoolId::get().into();
            const DELTA_SIZE: u32 = 256;
            let delta = tx.delta().unchecked_add(pool_id.unchecked_shr(DELTA_SIZE));
            log::debug!("    Preparing data");
            let transact_inputs =
                [root_before, tx.nullifier().into(), tx.out_commit(), delta, message_hash_num];
            log::debug!("    Verification");
            alt_bn128_groth16verify(&transfer_vk, &tx.transact_proof(), &transact_inputs)
                .map_err(|err| Into::<Error<T>>::into(err))?;

            if <Nullifiers<T>>::contains_key::<NativeU256>(tx.nullifier().into()) {
                log::warn!("Double spend");
                return Err(Error::<T>::DoubleSpend.into())
            }

            if tx.transfer_index() > pool_index.into() {
                log::warn!("Index out of bounds");
                return Err(Error::<T>::IndexOutOfBounds.into())
            }

            // Verify tree proof
            log::debug!("Verifying tree proof:");
            let tree_vk = <TreeVk<T>>::get().ok_or(Error::<T>::TreeVkNotSet)?;
            log::debug!("    Preparing data");
            let tree_inputs = [root_before, tx.root_after(), tx.out_commit()];
            log::debug!("    Verification");
            alt_bn128_groth16verify(&tree_vk, &tx.tree_proof(), &tree_inputs)
                .map_err(|err| Into::<Error<T>>::into(err))?;

            // Set the nullifier
            log::debug!("Updating state");
            let mut elements = [0u8; core::mem::size_of::<U256>() * 2];
            tx.out_commit().using_encoded(|data| {
                elements[..core::mem::size_of::<U256>()].copy_from_slice(data);
            });
            tx.delta().using_encoded(|data| {
                elements[core::mem::size_of::<U256>()..].copy_from_slice(data);
            });
            let hash = U256::from_big_endian(&keccak_256(&elements));

            pool_index = U256::from(pool_index).unchecked_add(U256::from(128u8));

            // Calculate all_messages_hash
            log::debug!("Updating all_messages_hash");
            let mut hashes = [0u8; 32 * 2];
            let all_messages_hash = <AllMessagesHash<T>>::get();
            all_messages_hash.using_encoded(|data| hashes[..32].copy_from_slice(data));
            hashes[32..].copy_from_slice(&message_hash);
            let new_all_messages_hash = U256::from_big_endian(&keccak_256(&hashes));

            let fee = tx.memo_fee();
            let token_amount = tx.token_amount().overflowing_add(fee).0;
            let energy_amount = tx.energy_amount();

            match tx.tx_type() {
                TxType::Transfer => {
                    log::debug!("Processing transfer");
                    if token_amount != U256::ZERO || energy_amount != U256::ZERO {
                        return Err(Error::<T>::IncorrectAmount.into())
                    }
                },
                TxType::Deposit => {
                    log::debug!("Processing deposit:");
                    if token_amount > U256::MAX.unchecked_div(U256::from(2u32)) ||
                        energy_amount != U256::ZERO
                    {
                        return Err(Error::<T>::IncorrectAmount.into())
                    }

                    log::debug!("    Extracting address");
                    let src = T::AccountId::decode(&mut tx.deposit_address())
                        .map_err(|_err| Into::<DispatchError>::into(Error::<T>::Deserialization))?;

                    log::debug!("    Verifying signature");
                    let sig_result =
                        match sp_core::sr25519::Signature::try_from(tx.deposit_signature()) {
                            Ok(signature) => {
                                let signer =
                                    sp_core::sr25519::Public::from_slice(tx.deposit_address());
                                signature.verify(tx.nullifier_bytes(), &signer)
                            },
                            _ => false,
                        };

                    if !sig_result {
                        return Err(Error::<T>::InvalidDepositSignature.into())
                    }

                    log::debug!("    Preparing amounts");
                    let encoded_amount = (token_amount.unchecked_mul(*DENOMINATOR)).encode();
                    let native_amount = <BalanceOf<T>>::decode(&mut &encoded_amount[..])
                        .map_err(|_err| Into::<DispatchError>::into(Error::<T>::Deserialization))?;

                    log::debug!("    Transferring to the pool");
                    T::Currency::transfer(
                        &src,
                        &Self::account_id(),
                        native_amount,
                        ExistenceRequirement::AllowDeath,
                    )?;
                },
                TxType::Withdraw => {
                    log::debug!("Processing withdraw:");
                    // if token_amount < U256::MAX.unchecked_div(U256::from(2u32)) ||
                    //     energy_amount < U256::MAX.unchecked_div(U256::from(2u32))
                    // {
                    //     return Err(Error::<T>::IncorrectAmount.into())
                    // }

                    log::debug!("    Extracting the destination address");
                    let dest = T::AccountId::decode(&mut tx.memo_address())
                        .map_err(|_err| Into::<DispatchError>::into(Error::<T>::Deserialization))?;

                    log::debug!("    Preparing amounts");
                    let encoded_amount =
                        (token_amount.overflowing_neg().0.unchecked_mul(*DENOMINATOR)).encode();
                    let native_amount = <BalanceOf<T>>::decode(&mut &encoded_amount[..])
                        .map_err(|_err| Into::<DispatchError>::into(Error::<T>::Deserialization))?;

                    log::debug!("    Transferring to the destination address");
                    T::Currency::transfer(
                        &Self::account_id(),
                        &dest,
                        native_amount,
                        ExistenceRequirement::AllowDeath,
                    )?;
                },
            }

            // TODO: Find a less irritating way to created an indexed event.
            log::debug!("Emitting event");
            let event = Event::Message(
                pool_index.into(),
                new_all_messages_hash.into(),
                tx.out_commit().into(),
                tx.ciphertext().to_vec(),
            );

            let event = <<T as Config>::Event as From<Event<T>>>::from(event);

            let event =
                <<T as Config>::Event as Into<<T as frame_system::Config>::Event>>::into(event);

            frame_system::Pallet::<T>::deposit_event_indexed(
                &[T::Hashing::hash(b"ZeropoolMessage")],
                event,
            );

            if fee > U256::ZERO {
                log::debug!("    Processing fee");
                let encoded_fee = (fee.unchecked_mul(*DENOMINATOR).overflowing_neg().0).encode();
                let native_fee = <BalanceOf<T>>::decode(&mut &encoded_fee[..])
                    .map_err(|_err| Into::<DispatchError>::into(Error::<T>::Deserialization))?;

                T::Currency::transfer(
                    &Self::account_id(),
                    &operator,
                    native_fee,
                    ExistenceRequirement::AllowDeath,
                )?;
            }

            <PoolIndex<T>>::put::<NativeU256>(pool_index.into());
            <Roots<T>>::insert::<NativeU256, NativeU256>(pool_index.into(), tx.root_after().into());
            <Nullifiers<T>>::insert::<NativeU256, NativeU256>(tx.nullifier().into(), hash.into());
            <AllMessagesHash<T>>::put::<NativeU256>(new_all_messages_hash.into());

            log::info!("Transaction processed successfully");

            Ok(())
        }
    }
}
