use crate::{PublicKey, Signature, TokenAddress};
use keccak_hash::keccak;
use num_bigint::BigUint;
use serde::{Deserialize, Serialize};
use solina::structured_hash::StructuredHashInterface;

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[allow(dead_code)]
pub enum TradeDirection {
    Buy,
    Sell,
}

impl TradeDirection {
    pub fn to_bool(self) -> bool {
        match self {
            Self::Buy => false,
            Self::Sell => true,
        }
    }

    pub fn from_bool(value: bool) -> Self {
        if value {
            return Self::Sell;
        }
        return Self::Buy;
    }
}

/// Inputs for a swap
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct IntentInputs {
    /// quote token
    pub quote_token: TokenAddress,
    /// base token
    pub base_token: TokenAddress,
    /// quote amount
    pub quote_amount: BigUint,
    /// trade direction
    pub direction: TradeDirection,
}

impl StructuredHashInterface for IntentInputs {
    fn type_encode() -> String {
        "IntentInputs(BigUint from,BigUint quote_token,BigUint base_token,BigUint quote_amount)"
            .to_string()
    }
    fn data_encode(&self) -> Vec<u8> {
        let quote_token_hash = keccak(&self.quote_token).to_fixed_bytes();
        let base_token_hash = keccak(&self.base_token).to_fixed_bytes();
        let quote_amount_hash = keccak(&self.quote_amount.to_bytes_be()).to_fixed_bytes();
        let direction = keccak(&[self.direction as u8]).to_fixed_bytes();

        [
            quote_token_hash,
            base_token_hash,
            quote_amount_hash,
            direction,
        ]
        .concat()
    }
}

impl IntentInputs {
    pub fn new(
        quote_token: TokenAddress,
        base_token: TokenAddress,
        quote_amount: BigUint,
        direction: TradeDirection,
    ) -> Self {
        Self {
            quote_token,
            base_token,
            quote_amount,
            direction,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct IntentConstraints {
    /// max slippage amount
    pub min_base_token_amount: BigUint,
}

impl StructuredHashInterface for IntentConstraints {
    fn type_encode() -> String {
        "IntentConstraints(BigUint min_base_token_amount)".to_string()
    }
    fn data_encode(&self) -> Vec<u8> {
        keccak(&self.min_base_token_amount.to_bytes_be())
            .as_fixed_bytes()
            .to_vec()
    }
}

impl IntentConstraints {
    pub fn new(min_base_token_amount: BigUint) -> Self {
        Self {
            min_base_token_amount,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Intent {
    pub public_key: PublicKey,
    pub inputs: IntentInputs,
    pub constraints: IntentConstraints,
    pub signature: Signature,
}

impl Intent {
    pub fn new(
        public_key: PublicKey,
        inputs: IntentInputs,
        constraints: IntentConstraints,
        signature: Signature,
    ) -> Self {
        Self {
            public_key,
            inputs,
            constraints,
            signature,
        }
    }

    pub fn public_key(&self) -> PublicKey {
        self.public_key
    }

    pub fn signature(&self) -> &Signature {
        &self.signature
    }
}

impl StructuredHashInterface for Intent {
    fn type_encode() -> String {
        let input_type_encoding = IntentInputs::type_encode();
        let constraints_type_encoding = IntentConstraints::type_encode();
        format!(
            "Intent(IntentInputs inputs,IntentConstraints constraints){}{}",
            constraints_type_encoding, input_type_encoding
        )
    }

    fn data_encode(&self) -> Vec<u8> {
        let input_data_encoding = self.inputs.structured_hash();
        let constraints_data_encoding = self.constraints.structured_hash();
        [input_data_encoding, constraints_data_encoding].concat()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works_swap_inputs_type_encoding() {
        assert_eq!(
            IntentInputs::type_encode().as_str(),
            "IntentInputs(BigUint from,BigUint quote_token,BigUint base_token,BigUint quote_amount)"
        );
    }

    #[test]
    fn it_works_swap_constraints_type_encoding() {
        assert_eq!(
            IntentConstraints::type_encode().as_str(),
            "IntentConstraints(BigUint min_base_token_amount)"
        );
    }

    #[test]
    fn it_works_swap_intent_type_encoding() {
        assert_eq!(
            Intent::type_encode(),
            format!(
                "Intent(IntentInputs inputs,IntentConstraints constraints){}{}",
                IntentConstraints::type_encode(),
                IntentInputs::type_encode(),
            )
        );
    }

    #[test]
    fn it_works_swap_inputs_struct_hash() {
        let mut quote_token = [0u8; 32];
        quote_token[0] = 255;

        let mut base_token = [0u8; 32];
        base_token[0] = 64;

        let inputs = IntentInputs {
            quote_amount: BigUint::from(1_000_000_000_000_u64),
            quote_token,
            base_token,
            direction: TradeDirection::Buy,
        };

        let hash = inputs.structured_hash();
        assert_eq!(
            hash,
            [
                119, 55, 80, 220, 96, 145, 112, 131, 19, 95, 179, 83, 51, 195, 143, 188, 34, 228,
                10, 94, 79, 250, 104, 82, 141, 53, 135, 224, 160, 126, 74, 147
            ]
        );
    }

    #[test]
    fn it_works_swap_constraints_struct_hash() {
        let constraints = IntentConstraints {
            min_base_token_amount: BigUint::from(64_u8),
        };

        let hash = constraints.structured_hash();
        assert_eq!(
            hash,
            [
                59, 101, 217, 87, 50, 192, 198, 116, 99, 54, 249, 12, 244, 246, 21, 0, 55, 46, 126,
                117, 95, 93, 84, 185, 227, 193, 93, 71, 156, 125, 114, 5
            ]
        );
    }

    #[test]
    fn it_works_swap_intent_struct_hash() {
        let mut quote_token = [0u8; 32];
        quote_token[0] = 255;

        let mut base_token = [0u8; 32];
        base_token[0] = 64;

        let intent = Intent {
            public_key: [0u8; 32],
            signature: Signature([0u8; 64]),
            inputs: IntentInputs {
                quote_amount: BigUint::from(1_000_000_000_000_u64),
                quote_token,
                base_token,
                direction: TradeDirection::Buy,
            },
            constraints: IntentConstraints {
                min_base_token_amount: BigUint::from(64_u8),
            },
        };

        let hash = intent.structured_hash();
        assert_eq!(
            hash,
            [
                13, 34, 246, 195, 58, 50, 128, 107, 108, 146, 1, 227, 198, 214, 220, 240, 97, 224,
                195, 42, 99, 23, 54, 92, 247, 146, 24, 108, 207, 175, 86, 231
            ]
        );
    }
}
