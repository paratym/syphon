use crate::{
    formats::FormatTag, Format, FormatChunk, FormatReader, FormatWriter, Stream, StreamSpec,
};
use std::{
    io::{self, Read, Write},
    ops::DerefMut,
};
use syphon_core::SyphonError;

pub struct StreamSelector<F: Format> {
    inner: F,
    stream_i: usize,
}

impl<F: Format> StreamSelector<F> {
    pub fn new(inner: F, stream_i: usize) -> Result<Self, SyphonError> {
        if inner.data().streams.len() <= stream_i {
            return Err(SyphonError::NotFound);
        }

        Ok(Self { inner, stream_i })
    }
}

impl<F: Format> Stream for StreamSelector<F> {
    type Tag = <F::Tag as FormatTag>::Codec;

    fn spec(&self) -> &StreamSpec<Self::Tag> {
        &self.inner.data().streams[self.stream_i]
    }
}

impl<T> Read for StreamSelector<T>
where
    T: DerefMut,
    T::Target: FormatReader,
{
    fn read(&mut self, buffer: &mut [u8]) -> Result<usize, io::Error> {
        loop {
            match self.inner.read(buffer)? {
                FormatChunk::Stream { stream_i, buf } if stream_i == self.stream_i => {
                    return Ok(buf.len());
                }
                _ => {}
            }
        }
    }
}

impl<T> Write for StreamSelector<T>
where
    T: DerefMut,
    T::Target: FormatWriter,
{
    fn write(&mut self, buf: &[u8]) -> Result<usize, io::Error> {
        let chunk = FormatChunk::Stream {
            stream_i: self.stream_i,
            buf,
        };

        self.inner.write(chunk)?;
        Ok(buf.len())
    }

    fn flush(&mut self) -> Result<(), io::Error> {
        Ok(self.inner.flush()?)
    }
}