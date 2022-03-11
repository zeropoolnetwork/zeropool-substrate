use ff_uint::construct_uint;
pub use ff_uint::Uint;
pub use sp_core::U256 as NativeU256;

construct_uint! {
    pub struct _U256(4);
}

pub type U256 = _U256;

impl From<NativeU256> for U256 {
    fn from(num: NativeU256) -> Self {
        _U256(num.0)
    }
}

impl From<U256> for NativeU256 {
    fn from(num: U256) -> Self {
        NativeU256(num.0)
    }
}

impl U256 {
    pub const fn from_const_str(bytes: &[u8]) -> U256 {
        let mut i: usize = 0;
        while i < bytes.len() {
            let b = bytes[i];
            if !(b >= 48 && b <= 57) {
                panic!("Invalid character");
            }

            i += 1;
        }

        let mut res = U256::ZERO;

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

const fn uint_from_u64(v: u64) -> U256 {
    let mut ret = [0; 4];
    ret[0] = v;
    _U256(ret)
}

const fn overflowing_mul_u64(mut lhs: U256, rhs: u64) -> (U256, u64) {
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

const fn overflowing_add(lhs: U256, rhs: U256) -> (U256, bool) {
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
