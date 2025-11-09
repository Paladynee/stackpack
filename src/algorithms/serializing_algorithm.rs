// use std::{convert::Infallible, marker::PhantomData};

// use anyhow::Result;

// use crate::algorithms::DynMutator;

// struct X {
//     val: usize,
// }

// impl ByteCompatible for X {
//     type Error = Infallible;
//     fn try_from_bytes<'a>(bytes: &'a [u8]) -> Result<X, Infallible> {
//         Ok(X { val: 0 })
//     }
// }

// struct GenericMutator<X, E: Copy, D: Copy> {
//     e: E,
//     d: D,
//     _marker: PhantomData<fn(X)>,
// }

// trait ByteCompatible: Sized {
//     type Error: std::error::Error + Send + Sync + 'static;
//     fn try_from_bytes<'a>(bytes: &'a [u8]) -> Result<Self, Self::Error>;
// }

// impl<X, E, D> GenericMutator<X, E, D>
// where
//     X: ByteCompatible,
//     E: Fn(X) -> Result<()> + Copy,
//     D: Fn(X) -> Result<()> + Copy,
// {
//     pub const fn new(encoder: E, decoder: D) -> Self {
//         GenericMutator {
//             e: encoder,
//             d: decoder,
//             _marker: PhantomData,
//         }
//     }

//     pub const fn encoder(&self) -> fn(&[u8], &mut Vec<u8>) -> Result<()> {
//         |data: &[u8], out: &mut Vec<u8>| -> Result<()> {
//             let x = X::try_from_bytes(data)?;
//             (self.e)(x)?;
//             Ok(())
//         }
//     }

//     pub const fn decoder(&self) -> fn(&[u8], &mut Vec<u8>) -> Result<()> {
//         |data: &[u8], out: &mut Vec<u8>| -> Result<()> { todo!() }
//     }

//     pub const fn dyn_mutator(self) -> DynMutator {
//         DynMutator {
//             drive_mutation: self.encoder(),
//             revert_mutation: self.encoder(),
//         }
//     }
// }

// pub const SerializingCompressor: DynMutator = GenericMutator::new(x_encoder, x_decoder).dyn_mutator();

// fn x_encoder(custom: X) -> Result<()> {
//     todo!()
// }

// fn x_decoder(custom: X) -> Result<()> {
//     todo!()
// }
