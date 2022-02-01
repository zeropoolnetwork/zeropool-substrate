use ff_uint::{construct_uint, Uint};
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
