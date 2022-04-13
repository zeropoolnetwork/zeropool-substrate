use crate::{self as pallet_zeropool, num::U256};
use ff_uint::Uint;
use frame_support::{parameter_types, PalletId};
use sp_core::{crypto::AccountId32, H256};
use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, IdentityLookup},
};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;
type Balance = u128;
type AccountId = AccountId32;

const OWNER: AccountId = AccountId::new(hex_literal::hex!(
    "d000ac5048ae858aca2e6aa43e00661562a47026fe88ff83992430204a159752"
));

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
    pub enum Test where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
        Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
        Zeropool: pallet_zeropool::{Pallet, Call, Storage, Event<T>},
        ZeropoolOperatorManager: pallet_zeropool::operator,
    }
);

parameter_types! {
    pub const BlockHashCount: u64 = 250;
    pub const SS58Prefix: u8 = 42;
    pub static ExistentialDeposit: Balance = 1;
}

impl frame_system::Config for Test {
    type BaseCallFilter = frame_support::traits::Everything;
    type BlockWeights = ();
    type BlockLength = ();
    type DbWeight = ();
    type Origin = Origin;
    type Call = Call;
    type Index = u64;
    type BlockNumber = u64;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = AccountId;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Header = Header;
    type Event = Event;
    type BlockHashCount = BlockHashCount;
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = pallet_balances::AccountData<Balance>;
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = SS58Prefix;
    type OnSetCode = ();
    type MaxConsumers = frame_support::traits::ConstU32<16>;
}

impl pallet_balances::Config for Test {
    type MaxLocks = ();
    type MaxReserves = ();
    type ReserveIdentifier = [u8; 8];
    type Balance = Balance;
    type Event = Event;
    type DustRemoval = ();
    type ExistentialDeposit = ExistentialDeposit;
    type AccountStore = System;
    type WeightInfo = ();
}

parameter_types! {
    pub const TestPalletId: PalletId = PalletId(*b"zeropool");
    pub const PoolId: U256 = U256::ZERO;
    pub const InitialOwner: <Test as frame_system::Config>::AccountId = OWNER;
}

impl pallet_zeropool::Config for Test {
    type Event = Event;
    type PalletId = TestPalletId;
    type Currency = Balances;

    type OperatorManager = ZeropoolOperatorManager;
    type PoolId = PoolId;
    type InitialOwner = InitialOwner;
}

impl pallet_zeropool::operator::Config for Test {
    type Event = Event;
    type InitialOwner = InitialOwner;
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
    let mut t = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();
    pallet_balances::GenesisConfig::<Test> { balances: vec![(OWNER, 1000000000000000000)] }
        .assimilate_storage(&mut t)
        .unwrap();

    let mut ext = sp_io::TestExternalities::new(t);
    ext.execute_with(|| System::set_block_number(1));
    ext
}
