# pallet-zeropool

An implementation of the ZeroPool pool pallet.

## Configuration

```rust
construct_runtime!(
    pub enum Runtime where
        Block = Block,
        NodeBlock = opaque::Block,
        UncheckedExtrinsic = UncheckedExtrinsic
    {
        System: frame_system,
        Balances: pallet_balances,
        /* ... */
        // Add both the ZeroPool pallet and the operator manager.
        Zeropool: pallet_zeropool,
        // This is the default implementation of the operator manager.
        ZeropoolOperatorManager: pallet_zeropool::operator,
    }
);

parameter_types! {
    pub const ZeropoolPalletId: PalletId = PalletId(*b"zeropool");
    pub const PoolId: U256 = U256::ZERO;
    
    // Initial owner of the pool and the operator manager.
    pub const InitialOwner: AccountId = AccountId::new(hex_literal::hex!("..."));
}

impl pallet_zeropool::Config for Runtime {
    type Event = Event;
    type PalletId = ZeropoolPalletId;
    type Currency = Balances;
    type InitialOwner = InitialOwner;
    type PoolId = PoolId;
    type OperatorManager = ZeropoolOperatorManager;
}

impl pallet_zeropool::operator::Config for Runtime {
    type Event = Event;
    type InitialOwner = InitialOwner;
}
```

## Custom operator manager
It's possible to implement a custom operator manager (e.g. an auction or something more sophisticated):
```rust
impl<T: Config> OperatorManager<T::AccountId> for YourCustomPallet<T> {
    fn operator() -> Option<T::AccountId> {
        Operator::<T>::get()
    }

    // This method is called by the main ZeroPool pallet.
    fn set_owner(new_owner: T::AccountId) -> DispatchResult {
        <Owner<T>>::put(new_owner.clone());

        Ok(())
    }
}
```
