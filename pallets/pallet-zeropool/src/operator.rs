use frame_support::dispatch::DispatchResult;

pub trait OperatorManager<AccountId>
where
    AccountId: PartialEq,
{
    fn is_operator(account: AccountId) -> bool;
    fn set_owner(new_owner: AccountId) -> DispatchResult;
}

impl<AccountId: PartialEq> OperatorManager<AccountId> for () {
    fn is_operator(_: AccountId) -> bool {
        true
    }

    fn set_owner(_new_owner: AccountId) -> DispatchResult {
        Ok(())
    }
}

pub use pallet::*;

/// A simple implementation of OperatorManager pallet.
#[frame_support::pallet]
pub mod pallet {
    use super::OperatorManager;
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        #[pallet::constant]
        type InitialOwner: Get<Self::AccountId>;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::storage]
    pub type Operator<T: Config> = StorageValue<_, T::AccountId>;

    #[pallet::type_value]
    pub fn DefaultOwner<T: Config>() -> T::AccountId {
        T::InitialOwner::get()
    }

    #[pallet::storage]
    pub type Owner<T: Config> = StorageValue<_, T::AccountId, ValueQuery, DefaultOwner<T>>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// [who]
        OperatorChanged(T::AccountId),
        /// [who]
        OwnerChanged(T::AccountId),
    }

    #[pallet::error]
    pub enum Error<T> {
        NotOwner,
    }

    impl<T: Config> Pallet<T> {
        fn owner() -> T::AccountId {
            <Owner<T>>::get()
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
        #[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1, 1))]
        pub fn set_operator(origin: OriginFor<T>, address: T::AccountId) -> DispatchResult {
            Self::check_owner(origin)?;

            <Operator<T>>::put(address.clone());

            Self::deposit_event(Event::OperatorChanged(address));

            Ok(())
        }
    }

    impl<T: Config> OperatorManager<T::AccountId> for Pallet<T> {
        fn is_operator(account: T::AccountId) -> bool {
            Operator::<T>::get().map(|op| op == account).unwrap_or(false)
        }

        fn set_owner(new_owner: T::AccountId) -> DispatchResult {
            <Owner<T>>::put(new_owner.clone());

            Self::deposit_event(Event::<T>::OwnerChanged(new_owner));

            Ok(())
        }
    }
}
