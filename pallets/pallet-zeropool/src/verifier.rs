use crate::{
    error::ZeroPoolError,
    maybestd::{vec, vec::Vec},
    num::U256,
};
use borsh::{BorshDeserialize, BorshSerialize};
use frame_support::codec::{Decode, Encode};
use scale_info::TypeInfo;

pub type G1 = [U256; 2];
pub type G2 = [U256; 4];

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize, Encode, Decode, PartialEq, TypeInfo)]
pub struct VK {
    pub alpha: G1,
    pub beta: G2,
    pub gamma: G2,
    pub delta: G2,
    pub ic: Vec<G1>,
}

#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize)]
pub struct Proof {
    pub a: G1,
    pub b: G2,
    pub c: G1,
}

#[inline]
pub fn alt_bn128_g1_multiexp(v: &[(G1, U256)]) -> core::result::Result<G1, ZeroPoolError> {
    let data = v.try_to_vec().unwrap();
    let res = crate::alt_bn128::alt_bn128_g1_multiexp(&data)?;
    let mut res_ptr = &res[..];
    Ok(<G1 as BorshDeserialize>::deserialize(&mut res_ptr).unwrap())
}

#[inline]
pub fn alt_bn128_g1_sum(v: &[(bool, G1)]) -> core::result::Result<G1, ZeroPoolError> {
    let data = v.try_to_vec().unwrap();
    let res = crate::alt_bn128::alt_bn128_g1_sum(&data)?;
    let mut res_ptr = &res[..];
    Ok(<G1 as BorshDeserialize>::deserialize(&mut res_ptr).unwrap())
}

#[inline]
pub fn alt_bn128_g1_neg(p: G1) -> core::result::Result<G1, ZeroPoolError> {
    alt_bn128_g1_sum(&[(true, p)])
}

#[inline]
pub fn alt_bn128_pairing_check(v: &[(G1, G2)]) -> core::result::Result<bool, ZeroPoolError> {
    let data = v.try_to_vec().unwrap();
    crate::alt_bn128::alt_bn128_pairing_check(&data)
}

pub fn alt_bn128_groth16verify(
    vk: &VK,
    proof: &Proof,
    input: &[U256],
) -> core::result::Result<bool, ZeroPoolError> {
    if vk.ic.len() != input.len() + 1 {
        return Err(ZeroPoolError::NotConsistentGroth16InputsError)
    }
    let neg_a = alt_bn128_g1_neg(proof.a)?;
    let acc_expr = vk
        .ic
        .iter()
        .zip([U256::from(0u32)].iter().chain(input.iter()))
        .map(|(&base, &exp)| (base, exp))
        .collect::<Vec<_>>();
    let acc = alt_bn128_g1_multiexp(&acc_expr)?;

    let pairing_expr =
        vec![(neg_a, proof.b), (vk.alpha, vk.beta), (acc, vk.gamma), (proof.c, vk.delta)];

    alt_bn128_pairing_check(&pairing_expr)
}
