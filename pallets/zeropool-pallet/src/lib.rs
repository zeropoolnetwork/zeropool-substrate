#![cfg_attr(not(feature = "std"), no_std)]
#![allow(dead_code)]

#[cfg(not(feature = "std"))]
extern crate alloc;

use borsh::BorshDeserialize;
use frame_support::traits::Currency;
use maybestd::vec::Vec;
pub use pallet::*;
use sp_core::{hashing::keccak_256, U256};

mod alt_bn128;
mod error;
mod maybestd;
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
    use crate::{tx_decoder::EvmTxDecoder, verifier::alt_bn128_groth16verify};

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
    pub type PoolIndex<T> = StorageValue<_, U256>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// [who, amount]
        Lock(T::AccountId, BalanceOf<T>),
        /// [who, amount]
        Release(T::AccountId, BalanceOf<T>),
        Deposit,
        Transfer,
        Withdraw,
    }

    #[pallet::error]
    pub enum Error<T> {
        InsufficientBalance,
        InvalidTxFormat,
        DoubleSpend,
    }

    impl<T: Config> Pallet<T> {
        pub fn account_id() -> T::AccountId {
            T::PalletId::get().into_account()
        }
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        // TODO: Use SCALE codec instead of borsh
        #[pallet::weight(1000)]
        pub fn transact(origin: OriginFor<T>, data: Vec<u8>) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let tx = EvmTxDecoder::new(data.as_slice());

            let vk = (); // FIXME: load VK
            let inputs = [];
            let res = alt_bn128_groth16verify(&vk, &tx.transact_proof(), &inputs);

            let hash = [tx.out_commit().encode()];
            <Nullifiers<T>>::insert(tx.nullifier(), ());

            // {
            //     uint256 _pool_index = pool_index;

            //     require(transfer_verifier.verifyProof(_transfer_pub(), _transfer_proof()), "bad
            // transfer proof");     require(nullifiers[_transfer_nullifier()]==0,"
            // doublespend detected");     require(_transfer_index() <= _pool_index,
            // "transfer index out of bounds");     require(tree_verifier.
            // verifyProof(_tree_pub(), _tree_proof()), "bad tree proof");

            //     nullifiers[_transfer_nullifier()] =
            // uint256(keccak256(abi.encodePacked(_transfer_out_commit(), _transfer_delta())));
            //     _pool_index +=128;
            //     roots[_pool_index] = _tree_root_after();
            //     pool_index = _pool_index;
            //     bytes memory message = _memo_message();
            //     bytes32 message_hash = keccak256(message);
            //     bytes32 _all_messages_hash = keccak256(abi.encodePacked(all_messages_hash,
            // message_hash));     all_messages_hash = _all_messages_hash;
            //     emit Message(_pool_index, _all_messages_hash, message);
            // }

            // uint256 fee = _memo_fee();
            // int256 token_amount = _transfer_token_amount() + int256(fee);
            // int256 energy_amount = _transfer_energy_amount();

            // require(token_amount>=0 && energy_amount==0 && msg.value == 0, "incorrect deposit
            // amounts"); token.safeTransferFrom(_deposit_spender(), address(this),
            // uint256(token_amount) * denominator);

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
