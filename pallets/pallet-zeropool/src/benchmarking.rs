//! Benchmarking setup for pallet-template

use super::*;

#[allow(unused_imports)]
use crate::Pallet as Zeropool;
use frame_benchmarking::{benchmarks, whitelisted_caller};
use frame_system::RawOrigin;
use hex_literal::hex;
use sp_core::crypto::AccountId32;

const OWNER: AccountId32 =
    AccountId32::new(hex!("d000ac5048ae858aca2e6aa43e00661562a47026fe88ff83992430204a159752"));

const TRANSFER_VK: &[u8] = include_bytes!("../../../js/params/transfer_verification_key.bin");
const TREE_VK: &[u8] = include_bytes!("../../../js/params/tree_verification_key.bin");

// fn owner() -> Origin {
//     Origin::signed(OWNER)
// }

// fn init_state() {
//     let transfer_vk = std::fs::read("../../js/params/transfer_verification_key.bin")
//         .expect("download and place the params directory into the js directory");
//     assert_ok!(Zeropool::set_transfer_vk(owner(), transfer_vk));
//
//     let tree_vk = std::fs::read("../../js/params/tree_verification_key.bin")
//         .expect("download and place the params directory into the js directory");
//     assert_ok!(Zeropool::set_tree_vk(owner(), tree_vk));
//
//     assert_ok!(ZeropoolOperatorManager::set_operator(owner(), OWNER));
// }

benchmarks! {
    set_transfer_vk {
        let origin: T::AccountId = whitelisted_caller();
    }: _(RawOrigin::Signed(origin), TRANSFER_VK.to_vec())

    set_tree_vk {
        let origin: T::AccountId = whitelisted_caller();
    }: _(RawOrigin::Signed(origin), TREE_VK.to_vec())

    impl_benchmark_test_suite!(Zeropool, crate::mock::new_test_ext(), crate::mock::Test);
}
