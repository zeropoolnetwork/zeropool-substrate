#![cfg_attr(not(feature = "std"), no_std)]

use borsh::{BorshDeserialize, BorshSerialize};
use ff_uint::{construct_primefield_params, construct_uint, Num};
use frame_support::{inherent::Vec, traits::Currency};
pub use pallet::*;

#[derive(Debug, Clone, PartialEq, Eq, BorshDeserialize, BorshSerialize)]
pub enum ZeroPoolError {
    AltBn128DeserializationError { msg: String },
    AltBn128SerializationError { msg: String },
	NotConsistentGroth16InputsError
}

mod alt_bn128;
mod verifier;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
type BalanceOf<T> = <<T as Config>::Currency as Currency<AccountIdOf<T>>>::Balance;

construct_uint! {
	struct U256(4);
}

construct_primefield_params! {
	pub struct Fr(super::U256);

	impl PrimeFieldParams for Fr {
		type Inner = super::U256;
		const MODULUS: &'static str = "21888242871839275222246405745257275088548364400416034343698204186575808495617";
		const GENERATOR: &'static str = "7";
   }
}

#[derive(Debug, BorshSerialize, BorshDeserialize)]
#[repr(u16)]
enum TxType {
	Deposit = 0,
	Transfer = 1,
	Withdraw = 2,
}

#[derive(Debug, BorshSerialize, BorshDeserialize)]
pub struct MerkleProof<const L: usize> {
	pub sibling: [Num<Fr>; L],
	pub path: [bool; L],
}

#[derive(Debug, BorshSerialize, BorshDeserialize)]
struct Transaction {
	nullifier: Num<Fr>,
	out_commit: Num<Fr>,
	transfer_index: Num<Fr>,
	energy_amount: Num<Fr>,
	token_amount: Num<Fr>,
	transact_proof: Vec<Num<Fr>>, // FIXME
	root_after: Num<Fr>,
	tree_proof: Vec<Num<Fr>>, // FIXME
	tx_type: TxType,
	memo: Vec<u8>,
}

#[frame_support::pallet]
pub mod pallet {
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
	#[pallet::getter(fn get_balance)]
	pub type Balances<T> =
		StorageMap<_, Blake2_128Concat, <T as frame_system::Config>::AccountId, BalanceOf<T>>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// [who, amount]
		Lock(T::AccountId, BalanceOf<T>),
		/// [who, amount]
		Release(T::AccountId, BalanceOf<T>),
	}

	#[pallet::error]
	pub enum Error<T> {
		InsufficientBalance,
		InvalidTxFormat,
	}

	impl<T: Config> Pallet<T> {
		pub fn account_id() -> T::AccountId {
			T::PalletId::get().into_account()
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		// TODO: Try using SCALE codec instead of borsh
		#[pallet::weight(10_000)]
		pub fn transact(origin: OriginFor<T>, data: Vec<u8>) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let decoded =
				Transaction::try_from_slice(&data).map_err(|_| Error::<T>::InvalidTxFormat)?;

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
