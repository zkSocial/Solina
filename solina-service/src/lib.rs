use serde::{Deserialize, Deserializer, Serialize, Serializer};

pub mod intents;
pub mod types;
pub mod worker;

// Big endian byte representation
// TODO: refactor this directly
pub type PublicKey = [u8; 32];
pub type TokenAddress = [u8; 32];

#[derive(Clone, Debug)]
pub struct Signature([u8; 64]);

impl<'de> Deserialize<'de> for Signature {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let bytes: Vec<u8> = Vec::deserialize(deserializer)?;

        if bytes.len() != 64 {
            return Err(serde::de::Error::custom("Invalid signature length"));
        }

        let mut buffer = [0u8; 64];
        buffer.copy_from_slice(&bytes);

        Ok(Self(buffer))
    }
}

impl Serialize for Signature {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let bytes = self.0.to_vec();
        bytes.serialize(serializer)
    }
}
