use std::io::Cursor;

use anyhow::{Result, anyhow};
use arcode::{
    ArithmeticDecoder, ArithmeticEncoder, Model,
    bitbit::{BitReader, BitWriter, MSB},
};

use crate::compressor::{Compressor, DecompressionError};

const ARCODE_PRECISION: u64 = 48;

pub struct ArithmeticCoding;

impl Compressor for ArithmeticCoding {
    fn compress_bytes(&mut self, data: &[u8]) -> Vec<u8> {
        arith_encode(data)
    }

    fn decompress_bytes(&mut self, data: &[u8]) -> Result<Vec<u8>> {
        arith_decode(data)
    }

    fn compressor_name(&self) -> String {
        "Arithmetic Coding".into()
    }
}

impl ArithmeticCoding {
    fn get_model() -> Model {
        Model::builder().num_symbols(256).eof(arcode::EOFKind::EndAddOne).build()
    }
}

fn arith_encode(data: &[u8]) -> Vec<u8> {
    let mut model = ArithmeticCoding::get_model();
    encode_data_with_model(data, &mut model, ARCODE_PRECISION).unwrap_or_else(|e| {
        // BitWriter<Cursor<&mut Vec<u8>>> uses Cursor's implementation of write.
        // Cursor's implementation of write is specialized for Vec.
        // Vec write operation fails only if the write would exceed the maximum size for vec.
        panic!(
            "arithmetic encoder could not encode the given byte slice: {}
variants upheld by the Compressor trait could not be upheld,
there is no vector to return that could decompress to the original data",
            e
        )
    })
}

fn encode_data_with_model(data: &[u8], model: &mut Model, precision: u64) -> Result<Vec<u8>, String> {
    let mut encoder = ArithmeticEncoder::new(precision);
    let mut buffer = vec![];
    let cursor = Cursor::new(&mut buffer);
    let mut compressed_scratch = BitWriter::new(cursor);

    for &sym in data {
        encoder
            .encode(sym as u32, model, &mut compressed_scratch)
            .map_err(|_| format!("Error encoding symbol {}", sym))?;
        model.update_symbol(sym as u32);
    }

    encoder
        .encode(model.eof(), model, &mut compressed_scratch)
        .map_err(|_| "Error encoding EOF".to_string())?;
    encoder
        .finish_encode(&mut compressed_scratch)
        .map_err(|_| "Error finishing encoding".to_string())?;
    compressed_scratch.pad_to_byte().map_err(|_| "Error padding to byte".to_string())?;

    Ok(buffer)
}

fn arith_decode(data: &[u8]) -> Result<Vec<u8>> {
    if data.is_empty() {
        return Err(anyhow!(DecompressionError::InvalidInput(
            "arithmetic decoder error: data was empty".to_string()
        )));
    }

    let mut model = ArithmeticCoding::get_model();
    decode_data_with_model(data, &mut model, ARCODE_PRECISION).map_err(|e| {
        anyhow!(DecompressionError::InvalidInput(format!(
            "arithmetic decoder error from arcode crate: {}",
            e
        )))
    })
}

fn decode_data_with_model(data: &[u8], model: &mut Model, precision: u64) -> Result<Vec<u8>, String> {
    let mut input_reader = BitReader::<_, MSB>::new(data);
    let mut decoder = ArithmeticDecoder::new(precision);
    let mut decompressed_data = vec![];

    while !decoder.finished() {
        let sym = decoder
            .decode(model, &mut input_reader)
            .map_err(|_| "Error decoding symbol".to_string())?;
        model.update_symbol(sym);
        decompressed_data.push(sym as u8);
    }

    // Remove EOF marker
    if decompressed_data.is_empty() {
        return Err("Empty decoded data".to_string());
    }
    decompressed_data.pop();
    Ok(decompressed_data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_tests() {
        crate::tests::roundtrip_test(ArithmeticCoding);
    }
}
