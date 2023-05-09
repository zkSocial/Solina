use keccak_hash::keccak256;
use std::any::type_name;

/// Gets the name of a (generic) type `T`.
fn type_of<T>() -> &'static str {
    type_name::<T>()
}

/// SHA-256 hash data.
pub(crate) fn hash_data(data: &[u8]) -> Vec<u8> {
    let mut output = [0u8; 32];
    keccak_hash::write_keccak(data, &mut output);
    output.to_vec()
}

/// A macro to be able to encode any Rust struct as a String representing
/// `STRUCT_NAME(field_name_1 field_type_1, ..., field_name_N field_type_N).
/// This is inspired in Ethereum's EIP712, see https://eips.ethereum.org/EIPS/eip-712.
/// Notice that our type encoding differs from the one sketched in EIP712, as in the later
/// the field's name comes before of the field's type.
macro_rules! type_encoding_macro {
    (struct $name:ident {
        $($field_name:ident: $field_type:ty,)*
    }) => {
        struct $name {
            $($field_name: $field_type,)*
        }

        impl $name {
            // Returns the field names and types
            fn field_names_and_types() -> Vec<String> {
                vec![$(format!("{} {}", stringify!($field_name), stringify!($field_type))),*]
            }

            fn eip712_encoding() -> String {
                format!("{}({})", stringify!($name), $name::field_names_and_types().join(","))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hex_literal::hex;

    #[test]
    fn it_works_field_names_and_types() {
        type_encoding_macro! { struct MyStruct {
                name: String,
                struct_name: String,
                data: Vec<u8>,
        }
        }

        assert_eq!(
            MyStruct::field_names_and_types(),
            vec![
                "name String".to_string(),
                "struct_name String".to_string(),
                "data Vec<u8>".to_string()
            ]
        )
    }

    #[test]
    fn it_works_eip712_encoding() {
        type_encoding_macro! { struct MyStruct {
                name: String,
                struct_name: String,
                data: Vec<u8>,
        }
        }

        assert_eq!(
            MyStruct::eip712_encoding(),
            String::from("MyStruct(name String,struct_name String,data Vec<u8>)")
        )
    }

    #[test]
    fn it_works_keccak_hash_data() {
        let mut data = *b"my_message";
        let mut hashed_data = hash_data(&data);
        assert_eq!(
            hashed_data,
            // retrieved from https://emn178.github.io/online-tools/keccak_256.html
            hex!("741667d534dd9ae3346736b96c64f3a7f5370afdae8357d7177f14ee61eb8d46")
        );
    }
}
