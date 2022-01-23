use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok};

#[test]
fn test_lock_release() {
	new_test_ext().execute_with(|| {
		// Dispatch a signed extrinsic.
		assert_ok!(Zeropool::lock(Origin::signed(1), 42));
		// Read pallet storage and assert an expected result.
		assert_ok!(Zeropool::release(Origin::signed(1), 42));
	});
}

#[test]
fn test_release_release_without_lock() {
	new_test_ext().execute_with(|| {
		// Ensure the expected error is thrown when no value is present.
		assert_noop!(
			Zeropool::release(Origin::signed(123), 42),
			Error::<Test>::InsufficientBalance
		);
	});
}
