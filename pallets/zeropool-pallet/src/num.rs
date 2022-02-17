use ff_uint::construct_uint;
pub use ff_uint::Uint;
use frame_support::codec::{Decode, Encode, EncodeLike, Input, MaxEncodedLen};
use scale_info::TypeInfo;

construct_uint! {
    pub struct _U256(4);
}

pub type U256 = _U256;

// FIXME: Temporary impls
impl TypeInfo for U256 {
    type Identity = Self;

    fn type_info() -> scale_info::Type {
        todo!()
    }
}

impl Encode for U256 {
    fn using_encoded<R, F: FnOnce(&[u8]) -> R>(&self, f: F) -> R {
        let bytes = self.to_little_endian();
        bytes.using_encoded(f)
    }
}

impl EncodeLike for U256 {}

impl Decode for U256 {
    fn decode<I: Input>(input: &mut I) -> ::core::result::Result<Self, codec::Error> {
        <[u8; 4 * 8] as Decode>::decode(input).map(|b| U256::from_little_endian(&b))
    }
}

impl MaxEncodedLen for U256 {
    fn max_encoded_len() -> usize {
        ::core::mem::size_of::<U256>()
    }
}

impl _U256 {
    pub const fn from_const_str(bytes: &[u8]) -> _U256 {
        let mut i: usize = 0;
        while i < bytes.len() {
            let b = bytes[i];
            if !(b >= 48 && b <= 57) {
                panic!("Invalid character");
            }

            i += 1;
        }

        let mut res = _U256::ZERO;

        let mut i: usize = 0;
        while i < bytes.len() {
            let b = bytes[i] - 48;

            let (r, overflow) = overflowing_mul_u64(res, 10);
            if overflow > 0 {
                panic!("Invalid length")
            }
            let (r, overflow) = overflowing_add(r, uint_from_u64(b as u64));
            if overflow {
                panic!("Invalid length")
            }

            res = r;
            i += 1;
        }

        res
    }
}

const fn uint_from_u64(v: u64) -> _U256 {
    let mut ret = [0; 4];
    ret[0] = v;
    _U256(ret)
}

const fn overflowing_mul_u64(mut lhs: _U256, rhs: u64) -> (_U256, u64) {
    let mut carry = 0u64;

    let mut i = 0;
    while i < lhs.0.len() {
        let (res, c) = mul_u64(lhs.0[i], rhs, carry);
        lhs.0[i] = res;
        carry = c;
        i += 1;
    }

    (lhs, carry)
}

const fn mul_u64(a: u64, b: u64, carry: u64) -> (u64, u64) {
    let (hi, lo) = split_u128(a as u128 * b as u128 + carry as u128);
    (lo, hi)
}

const fn split_u128(a: u128) -> (u64, u64) {
    ((a >> 64) as _, (a & 0xFFFFFFFFFFFFFFFF) as _)
}

const fn overflowing_add(lhs: _U256, rhs: _U256) -> (_U256, bool) {
    let _U256(ref me) = lhs;
    let _U256(ref you) = rhs;

    let mut ret = [0u64; 4];
    let mut carry = 0u64;

    let mut i: usize = 0;
    while i < 4 {
        if carry != 0 {
            let (res1, overflow1) = u64::overflowing_add(me[i], you[i]);
            let (res2, overflow2) = u64::overflowing_add(res1, carry);

            ret[i] = res2;
            carry = (overflow1 as u8 + overflow2 as u8) as u64;
        } else {
            let (res, overflow) = u64::overflowing_add(me[i], you[i]);

            ret[i] = res;
            carry = overflow as u64;
        }

        i += 1;
    }

    (_U256(ret), carry > 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_const_str() {
        use core::str::FromStr;

        let num = U256::from_str("12345").unwrap();
        let num_c = U256::from_const_str(b"12345");

        assert_eq!(num, num_c);
    }

    #[test]
    #[should_panic]
    fn test_from_const_str_invalid() {
        U256::from_const_str(b"a12345");
    }
}
