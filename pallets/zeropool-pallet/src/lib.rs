#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/v3/runtime/frame>
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{pallet_prelude::*, traits::Currency, PalletId};
	use frame_system::pallet_prelude::*;

	// const ZeropoolPalletId: PalletId = PalletId(*b"zeropool_test");

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	pub type Balance = u128;

	// The pallet's runtime storage items.
	// https://docs.substrate.io/v3/runtime/storage
	#[pallet::storage]
	#[pallet::getter(fn something)]
	// Learn more about declaring storage items:
	// https://docs.substrate.io/v3/runtime/storage#declaring-storage-items
	pub type Balances<T> =
		StorageMap<_, Blake2_128Concat, <T as frame_system::Config>::AccountId, Balance>;

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/v3/runtime/events-and-errors
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// [amount, who]
		Lock(Balance, T::AccountId),
		/// [amount, who]
		Release(Balance, T::AccountId),
	}

	#[pallet::error]
	pub enum Error<T> {
		InsufficientBalance,
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		// pub fn pallet_account_id() -> T::AccountId {
		// 	T::PalletId::get().into_account()
		// }

		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
		pub fn lock(origin: OriginFor<T>, amount: Balance) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let total = <Balances<T>>::get(who.clone())
				.map(|balance| balance + amount)
				.unwrap_or(amount);

			<Balances<T>>::insert(who.clone(), total);

			// <Self as Currency<_>>::deposit_creating(
			// 	&who,
			// 	amount
			// )?;

			Self::deposit_event(Event::Lock(amount, who));
			Ok(())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
		pub fn release(origin: OriginFor<T>, amount: Balance) -> DispatchResult {
			let who = ensure_signed(origin)?;

			if let Some(balance) = <Balances<T>>::get(who.clone()) {
				let new_balance =
					balance.checked_sub(amount).ok_or(Error::<T>::InsufficientBalance)?;
				if new_balance == 0 {
					<Balances<T>>::remove(who.clone());
				} else {
					<Balances<T>>::insert(who.clone(), new_balance);
				}
			} else {
				return Err(Error::<T>::InsufficientBalance.into())
			}

			Self::deposit_event(Event::Release(amount, who));
			Ok(())
		}
	}
}
