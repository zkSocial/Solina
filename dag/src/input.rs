use anyhow::anyhow;
use libipld::{
    prelude::{Decode, Encode},
    IpldCodec,
};
use plonky2::{field::extension::Extendable, hash::hash_types::RichField};

use crate::U64_BYTES_LEN;

#[derive(Clone, Debug)]
pub struct Input<F: RichField + Extendable<D>, const D: usize> {
    field_values: Vec<F>,
}

impl<F: RichField + Extendable<D>, const D: usize> Input<F, D> {
    pub fn new(field_values: Vec<F>) -> Self {
        Self { field_values }
    }
}

// For now we just encode the underlying field element bytes, does not depend on the
// actual IPLD field value
impl<F: RichField + Extendable<D>, const D: usize> Encode<IpldCodec> for Input<F, D> {
    fn encode<W: std::io::Write>(&self, c: IpldCodec, w: &mut W) -> libipld::Result<()> {
        let field_elements_in_bytes = self
            .field_values
            .iter()
            .flat_map(|f| f.to_canonical_u64().to_le_bytes())
            .collect::<Vec<_>>();
        w.write(&field_elements_in_bytes)?;
        Ok(())
    }
}

// For now we just decode the underlying field element bytes, does not depend on the
// actual IPLD field value
impl<F: RichField + Extendable<D>, const D: usize> Decode<IpldCodec> for Input<F, D> {
    fn decode<R: std::io::Read + std::io::Seek>(c: IpldCodec, r: &mut R) -> libipld::Result<Self> {
        let mut values = vec![];
        r.read_to_end(&mut values)?;
        if values.len() % U64_BYTES_LEN != 0 {
            return Err(anyhow!("Number of bytes is not a multiple of 4"));
        }
        let field_values = (0..(values.len() / 8))
            .map(|i| {
                let mut byte_slice = [0u8; U64_BYTES_LEN];
                byte_slice.copy_from_slice(&values[i..i + U64_BYTES_LEN]);
                F::from_canonical_u64(u64::from_le_bytes(byte_slice))
            })
            .collect::<Vec<F>>();
        Ok(Input { field_values })
    }
}
