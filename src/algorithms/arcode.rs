use std::io::Cursor;

use anyhow::{Result, anyhow};
use arcode::{
    ArithmeticDecoder, ArithmeticEncoder, Model,
    bitbit::{BitReader, BitWriter, MSB},
};

use crate::algorithms::DynMutator;

if_tracing! {
    use tracing::{debug, error, info, warn};
}

const ARCODE_PRECISION: u64 = 48;
pub const ArithmeticCoding: DynMutator = DynMutator {
    drive_mutation: arith_encode,
    revert_mutation: arith_decode,
};

pub use self::ArithmeticCoding as ThisMutator;

fn get_model() -> Model {
    Model::builder().num_symbols(256).eof(arcode::EOFKind::EndAddOne).build()
}

fn arith_encode(data: &[u8], buf: &mut Vec<u8>) -> Result<()> {
    if_tracing! {
        debug!(target = "arcode", input_len = data.len(), precision = ARCODE_PRECISION, "arcode encode start");
    }

    // Ensure the output buffer is empty so writing via a Cursor will not leave
    // trailing bytes from a previous contents. The BitWriter writes into the
    // provided Vec starting at position 0 and may overwrite but not shrink
    // the vector, so we must clear it first.
    buf.clear();

    let mut model = get_model();
    let encode_result = encode_data_with_model(data, &mut model, buf, ARCODE_PRECISION);
    if_tracing! {
        if let Err(ref err) = encode_result {
            error!(target = "arcode", error = %err, "arcode encode failed");
        }
    }

    encode_result.unwrap_or_else(|e| {
        // BitWriter<Cursor<&mut Vec<u8>>> uses Cursor's implementation of write.
        // Cursor's implementation of write is specialized for Vec.
        // Vec write operation fails only if the write would exceed the maximum size for vec.
        panic!("OoM: {}", e)
    });

    if_tracing! {
        info!(target = "arcode", input_len = data.len(), output_len = buf.len(), precision = ARCODE_PRECISION, "arcode encode complete");
    }
    Ok(())
}

fn encode_data_with_model(data: &[u8], model: &mut Model, buf: &mut Vec<u8>, precision: u64) -> Result<(), String> {
    if_tracing! {
        debug!(target = "arcode", input_len = data.len(), precision = precision, "encode_data_with_model start");
    }

    let mut encoder = ArithmeticEncoder::new(precision);
    let cursor = Cursor::new(&mut *buf);
    let mut compressed_scratch = BitWriter::new(cursor);

    if_tracing! {
        debug!(target = "arcode", input_len = data.len(), precision = precision, "encoding loop start");
    }

    for &sym in data.iter() {
        encoder
            .encode(sym as u32, model, &mut compressed_scratch)
            .map_err(|_| format!("Error encoding symbol {}", sym))?;
        model.update_symbol(sym as u32);
    }

    if_tracing! {
        debug!(target = "arcode", processed = data.len(), "encoding loop complete");
    }

    if_tracing! {
        debug!(target = "arcode", eof_symbol = model.eof(), "encoding EOF symbol");
    }
    encoder.encode(model.eof(), model, &mut compressed_scratch).map_err(|_| {
        if_tracing! {
            error!(target = "arcode", "Error encoding EOF");
        }
        "Error encoding EOF".to_string()
    })?;
    encoder.finish_encode(&mut compressed_scratch).map_err(|_| {
        if_tracing! {
            error!(target = "arcode", "Error finishing encoding");
        }
        "Error finishing encoding".to_string()
    })?;
    compressed_scratch.pad_to_byte().map_err(|_| {
        if_tracing! {
            error!(target = "arcode", "Error padding to byte");
        }
        "Error padding to byte".to_string()
    })?;

    if_tracing! {
        debug!(target = "arcode", output_len = buf.len(), "encode_data_with_model complete");
    }

    Ok(())
}

fn arith_decode(data: &[u8], buf: &mut Vec<u8>) -> Result<()> {
    if_tracing! {
        debug!(target = "arcode", input_len = data.len(), precision = ARCODE_PRECISION, "arcode decode start");
    }

    if data.is_empty() {
        if_tracing! {
            warn!(target = "arcode", "arcode decode error: input empty");
        }
        return Err(anyhow!("arithmetic decoder error: data was empty".to_string()));
    }

    let mut model = get_model();
    let decode_result = decode_data_with_model(data, &mut model, buf, ARCODE_PRECISION);

    if_tracing! {
        if let Err(ref err) = decode_result {
            error!(target = "arcode", error = %err, "arcode decode failed");
        }
    }

    let mapped = decode_result.map_err(|e| anyhow!("arithmetic decoder error from arcode crate: {}", e));

    if_tracing! {
        if mapped.is_ok() {
            info!(target = "arcode", input_len = data.len(), output_len = buf.len(), precision = ARCODE_PRECISION, "arcode decode complete");
        }
    }

    mapped
}

fn decode_data_with_model(data: &[u8], model: &mut Model, buf: &mut Vec<u8>, precision: u64) -> Result<(), String> {
    let mut input_reader = BitReader::<_, MSB>::new(data);
    let mut decoder = ArithmeticDecoder::new(precision);
    buf.clear();

    while !decoder.finished() {
        let sym = decoder
            .decode(model, &mut input_reader)
            .map_err(|_| "Error decoding symbol".to_string())?;
        model.update_symbol(sym);
        buf.push(sym as u8);
    }

    // Remove EOF marker
    if buf.is_empty() {
        if_tracing! {
            warn!(target = "arcode", "arcode decode error: EOF marker missing");
        }
        return Err("Couldn't pop EOF marker".to_string());
    }
    buf.pop();
    Ok(())
}
