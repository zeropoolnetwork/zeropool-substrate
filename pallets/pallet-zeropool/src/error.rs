use borsh::{BorshDeserialize, BorshSerialize};

#[derive(Debug, Clone, PartialEq, Eq, BorshDeserialize, BorshSerialize)]
pub enum ZeroPoolError {
    AltBn128DeserializationError,
    AltBn128SerializationError,
    NotConsistentGroth16InputsError,
}
