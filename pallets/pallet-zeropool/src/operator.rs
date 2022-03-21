pub trait OperatorManager<AccountId> {
    fn operator() -> Option<AccountId>;
}

impl<AccountId: Default> OperatorManager<AccountId> for () {
    fn operator() -> Option<AccountId> {
        Some(AccountId::default())
    }
}