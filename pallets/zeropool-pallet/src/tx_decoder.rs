use crate::verifier::Proof;
use borsh::BorshDeserialize;
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use sp_core::U256;

const NUM_SIZE_BYTES: usize = 32;
const PROOF_SIZE: usize = NUM_SIZE_BYTES * 8;

// Offsets
// const SELECTOR: usize = 0;
const NULLIFIER: usize = 4;
const TRANSFER_INDEX: usize = NULLIFIER + NUM_SIZE_BYTES;
const OUT_COMMIT: usize = TRANSFER_INDEX + NUM_SIZE_BYTES;
const ENERGY_AMOUNT: usize = OUT_COMMIT + 6;
const TOKEN_AMOUNT: usize = ENERGY_AMOUNT + 14;
const TRANSACT_PROOF: usize = TOKEN_AMOUNT + 8;
const ROOT_AFTER: usize = TRANSACT_PROOF + PROOF_SIZE;
const TREE_PROOF: usize = ROOT_AFTER + NUM_SIZE_BYTES;
const TX_TYPE: usize = TREE_PROOF + PROOF_SIZE;
const MEMO: usize = TX_TYPE + 2;

#[derive(Debug, BorshDeserialize, FromPrimitive)]
#[repr(u16)]
pub enum TxType {
    Deposit = 0,
    Transfer = 1,
    Withdraw = 2,
}
pub struct EvmTxDecoder<'a> {
    data: &'a [u8],
}

impl<'a> EvmTxDecoder<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        EvmTxDecoder { data }
    }

    #[inline]
    pub fn nullifier(&self) -> U256 {
        U256::from_big_endian(&self.data[NULLIFIER..(NULLIFIER + NUM_SIZE_BYTES)])
    }

    #[inline]
    pub fn out_commit(&self) -> U256 {
        U256::from_big_endian(&self.data[NULLIFIER..(NULLIFIER + NUM_SIZE_BYTES)])
    }

    #[inline]
    pub fn transfer_index(&self) -> U256 {
        U256::from_big_endian(&self.data[TRANSFER_INDEX..(TRANSFER_INDEX + 6)])
    }

    #[inline]
    pub fn energy_amount(&self) -> U256 {
        U256::from_big_endian(&self.data[ENERGY_AMOUNT..(ENERGY_AMOUNT + 14)])
    }

    #[inline]
    pub fn token_amount(&self) -> U256 {
        U256::from_big_endian(&self.data[TOKEN_AMOUNT..(TOKEN_AMOUNT + 8)])
    }

    #[inline]
    pub fn transact_proof(&self) -> Proof {
        decode_proof(&self.data[TRANSACT_PROOF..(TRANSACT_PROOF + PROOF_SIZE)])
    }

    #[inline]
    pub fn root_after(&self) -> U256 {
        U256::from_big_endian(&self.data[ROOT_AFTER..(ROOT_AFTER + NUM_SIZE_BYTES)])
    }

    #[inline]
    pub fn tree_proof(&self) -> Proof {
        decode_proof(&self.data[TREE_PROOF..(TREE_PROOF + PROOF_SIZE)])
    }

    #[inline]
    pub fn tx_type(&self) -> TxType {
        let bytes = [self.data[ROOT_AFTER], self.data[ROOT_AFTER + 1]];
        let num = u16::from_be_bytes(bytes);
        TxType::from_u16(num).unwrap()
    }

    #[inline]
    pub fn memo(&self) -> &'a [u8] {
        &self.data[MEMO..]
    }
}

fn decode_proof(data: &[u8]) -> Proof {
    let a = decode_point(data);
    let b = decode_point(&data[NUM_SIZE_BYTES * 2..]);
    let c = decode_point(&data[NUM_SIZE_BYTES * 6..]);

    Proof { a, b, c }
}

fn decode_point<const N: usize>(data: &[u8]) -> [U256; N] {
    let mut buf = [U256::zero(); N];

    for i in 0..N {
        let offset = i * NUM_SIZE_BYTES;
        buf[i] = U256::from_big_endian(&data[offset..(offset + NUM_SIZE_BYTES)]);
    }

    buf
}
