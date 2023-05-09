use crate::utils::hash_data;

pub(crate) const ENCODED_BYTES_LEN: usize = 66;
pub(crate) const ENCODED_U32_LEN: usize = 17;
pub(crate) const ENCODING_PREFIX: [u8; 2] = [0x19, 0x01];

pub(crate) type DomainSeparator = [u8; 32];
/// Structured hash little-endian bytes type.
pub(crate) struct StructuredEncodedBytes(pub(crate) [u8; ENCODED_BYTES_LEN]);

impl StructuredEncodedBytes {
    pub(crate) fn to_array(self) -> [u8; ENCODED_BYTES_LEN] {
        self.0
    }
}

/// Structured hash, represented as u32 bytes array.
pub(crate) struct StructuredEncodedU32(pub(crate) [u32; 17]);

impl StructuredEncodedU32 {
    pub(crate) fn to_array(self) -> [u32; ENCODED_U32_LEN] {
        self.0
    }
}

impl From<StructuredEncodedU32> for StructuredEncodedBytes {
    fn from(value: StructuredEncodedU32) -> Self {
        let inner = value
            .0
            .iter()
            .flat_map(|u| u.to_le_bytes())
            .collect::<Vec<_>>();
        let mut bytes = [0u8; ENCODED_BYTES_LEN];
        bytes.copy_from_slice(&inner);
        Self(bytes)
    }
}

impl From<StructuredEncodedBytes> for StructuredEncodedU32 {
    fn from(value: StructuredEncodedBytes) -> Self {
        let mut inner = [0u8; 68];
        inner.copy_from_slice(&value.0);
        let mut u32_inner = [0u32; 17];

        (0..3).for_each(|i| {
            let mut val = [0u8; 4];
            val.copy_from_slice(&inner[32 * i..32 * (i + 1)]);
            u32_inner[i] = u32::from_le_bytes(val);
        });
        Self(u32_inner)
    }
}

pub(crate) trait EIP712Encoding {
    fn eip712_type_encoding() -> String;
    fn eip712_data_encoding(&self) -> Vec<u8>;
}

/// Structured SHA-256 hasher, as defined in EIP712, see https://eips.ethereum.org/EIPS/eip-712.
pub(crate) struct StructuredKeccakHasher;

impl StructuredKeccakHasher {
    /// Hashes a type T, satisfying `EIP712Encoding` trait bound, see https://eips.ethereum.org/EIPS/eip-712.
    pub(crate) fn hash_type_of<T: EIP712Encoding>() -> Vec<u8> {
        let eip712_encoded_type = T::eip712_type_encoding();
        hash_data(eip712_encoded_type.as_bytes())
    }

    /// Structured SHA-256 hasher, see https://eips.ethereum.org/EIPS/eip-712.
    pub(crate) fn hash_structured_data<T: EIP712Encoding>(message: T) -> Vec<u8> {
        let hash_of_type_t = Self::hash_type_of::<T>();
        let encoded_message = message.eip712_data_encoding();
        let data = [hash_of_type_t, encoded_message].concat();
        hash_data(&data)
    }
}
