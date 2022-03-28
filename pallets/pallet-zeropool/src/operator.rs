use frame_support::dispatch::DispatchResult;

pub trait OperatorManager<AccountId> {
    fn operator() -> Option<AccountId>;
    fn set_owner(new_owner: AccountId) -> DispatchResult;
}

impl<AccountId: Default> OperatorManager<AccountId> for () {
    fn operator() -> Option<AccountId> {
        Some(AccountId::default())
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
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn set_operator(origin: OriginFor<T>, address: T::AccountId) -> DispatchResult {
            Self::check_owner(origin)?;

            <Operator<T>>::put(address.clone());

            Self::deposit_event(Event::OperatorChanged(address));

            Ok(())
        }
    }

    impl<T: Config> OperatorManager<T::AccountId> for Pallet<T> {
        fn operator() -> Option<T::AccountId> {
            Operator::<T>::get()
        }

        fn set_owner(new_owner: T::AccountId) -> DispatchResult {
            <Owner<T>>::put(new_owner.clone());

            Self::deposit_event(Event::<T>::OwnerChanged(new_owner));

            Ok(())
        }
    }
}
