use ff_uint::construct_uint;

construct_uint! {
    pub struct _U256(4);
}

pub type U256 = _U256;
