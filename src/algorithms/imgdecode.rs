use anyhow::{Result, anyhow};

use crate::{algorithms::DynMutator, registered::RegisteredCompressor};

pub const ImgDecoder: RegisteredCompressor = RegisteredCompressor::new_dyn(
    DynMutator {
        drive_mutation: img_encode,
        revert_mutation: img_decode,
    },
    "img_decode",
    Some(DESCRIPTION),
);
const DESCRIPTION: &str = "General Image Decoding";

const ARCODE_PRECISION: u64 = 48;
fn img_encode(data: &[u8], buf: &mut Vec<u8>) -> Result<()> {
    //     if_tracing! {{
    //         tracing::debug!(target = "img_decode", input_len = data.len(), "img encode start");
    //     }}

    //     // Ensure the output buffer is empty so writing via a Cursor will not leave
    //     // trailing bytes from a previous contents. The BitWriter writes into the
    //     // provided Vec starting at position 0 and may overwrite but not shrink
    //     // the vector, so we must clear it first.
    //     buf.clear();

    //     let mut model = get_model();
    //     let encode_result = encode_data_with_model(data, &mut model, buf, ARCODE_PRECISION);
    //     if_tracing! {{
    //         if let Err(ref err) = encode_result {
    //             tracing::error!(target = "arcode", error = %err, "arcode encode failed");
    //         }
    //     }}

    //     encode_result.unwrap_or_else(|e| {
    //         // BitWriter<Cursor<&mut Vec<u8>>> uses Cursor's implementation of write.
    //         // Cursor's implementation of write is specialized for Vec.
    //         // Vec write operation fails only if the write would exceed the maximum size for vec.
    //         panic!("OoM: {}", e)
    //     });

    //     if_tracing! {{
    //         tracing::info!(target = "arcode", input_len = data.len(), output_len = buf.len(), precision = ARCODE_PRECISION, "arcode encode complete");
    //     }}
    //     Ok(())
    // }

    // fn encode_data_with_model(data: &[u8], model: &mut Model, buf: &mut Vec<u8>, precision: u64) -> Result<(), String> {
    //     if_tracing! {{
    //         tracing::debug!(target = "arcode", input_len = data.len(), precision = precision, "encode_data_with_model start");
    //     }}

    //     let mut encoder = ArithmeticEncoder::new(precision);
    //     let cursor = Cursor::new(&mut *buf);
    //     let mut compressed_scratch = BitWriter::new(cursor);

    //     if_tracing! {{
    //         tracing::debug!(target = "arcode", input_len = data.len(), precision = precision, "encoding loop start");
    //     }}

    //     for &sym in data.iter() {
    //         encoder
    //             .encode(sym as u32, model, &mut compressed_scratch)
    //             .map_err(|_| format!("Error encoding symbol {}", sym))?;
    //         model.update_symbol(sym as u32);
    //     }

    //     if_tracing! {{
    //         tracing::debug!(target = "arcode", processed = data.len(), "encoding loop complete");
    //     }}

    //     if_tracing! {{
    //         tracing::debug!(target = "arcode", eof_symbol = model.eof(), "encoding EOF symbol");
    //     }}
    //     encoder.encode(model.eof(), model, &mut compressed_scratch).map_err(|_| {
    //         if_tracing! {{
    //             tracing::error!(target = "arcode", "Error encoding EOF");
    //         }}
    //         "Error encoding EOF".to_string()
    //     })?;
    //     encoder.finish_encode(&mut compressed_scratch).map_err(|_| {
    //         if_tracing! {{
    //             tracing::error!(target = "arcode", "Error finishing encoding");
    //         }}
    //         "Error finishing encoding".to_string()
    //     })?;
    //     compressed_scratch.pad_to_byte().map_err(|_| {
    //         if_tracing! {{
    //             tracing::error!(target = "arcode", "Error padding to byte");
    //         }}
    //         "Error padding to byte".to_string()
    //     })?;

    //     if_tracing! {{
    //         tracing::debug!(target = "arcode", output_len = buf.len(), "encode_data_with_model complete");
    //     }}

    if_tracing! {{
        tracing::error!("image decoder cannot be used to encode images yet");
    }}

    Err(anyhow!("image decoder cannot be used to encode images yet"))
}

fn img_decode(data: &[u8], buf: &mut Vec<u8>) -> Result<()> {
    if_tracing! {{
        tracing::debug!(target = "img_decode", input_len = data.len(), "image decode start");
    }}

    if data.is_empty() {
        if_tracing! {{
            tracing::warn!(target = "img_decode", "image decode error: input empty");
        }}
        return Err(anyhow!("data was empty"));
    }

    todo!()
}
