use crate::{
    io::{Stream, StreamSpec, StreamSpecBuilder, SyphonCodec},
    Sample, Signal, SignalReader, SignalSpec, SignalWriter, SyphonError,
};
use byte_slice_cast::{
    AsByteSlice, AsMutByteSlice, AsMutSliceOf, AsSliceOf, FromByteSlice, ToByteSlice,
    ToMutByteSlice,
};
use std::{
    io::{self, Read, Write},
    marker::PhantomData,
    mem::{align_of, size_of},
};

pub fn fill_pcm_stream_spec(mut spec: StreamSpecBuilder) -> Result<StreamSpecBuilder, SyphonError> {
    if spec.codec.get_or_insert(SyphonCodec::Pcm) != &SyphonCodec::Pcm {
        return Err(SyphonError::InvalidData);
    }

    spec.with_compression_ratio(1.0)
}

pub struct PcmCodec<T, S: Sample> {
    inner: T,
    spec: StreamSpec,
    _sample: PhantomData<S>,
}

impl<T, S: Sample> PcmCodec<T, S> {
    pub fn new(inner: T, mut spec: StreamSpecBuilder) -> Result<Self, SyphonError> {
        spec = fill_pcm_stream_spec(spec)?;

        Ok(Self {
            inner,
            spec: spec.build()?,
            _sample: PhantomData,
        })
    }

    pub fn as_inner(&self) -> &T {
        &self.inner
    }

    pub fn into_inner(self) -> T {
        self.inner
    }
}

impl<T, S: Sample> Signal for PcmCodec<T, S> {
    type Sample = S;

    fn spec(&self) -> &SignalSpec {
        &self.spec.decoded_spec
    }
}

impl<T, S> SignalReader for PcmCodec<T, S>
where
    T: Read,
    S: Sample + ToMutByteSlice,
{
    fn read(&mut self, buf: &mut [Self::Sample]) -> Result<usize, SyphonError> {
        let byte_buf = buf.as_mut_byte_slice();
        let n = self.inner.read(byte_buf)?;

        let bytes_per_sample = byte_buf.len() / buf.len();
        if n % bytes_per_sample != 0 {
            todo!()
        }

        Ok(n / bytes_per_sample)
    }
}

impl<T, S> SignalWriter for PcmCodec<T, S>
where
    T: Write,
    S: Sample + ToByteSlice,
{
    fn write(&mut self, buf: &[Self::Sample]) -> Result<usize, SyphonError> {
        let byte_buf = buf.as_byte_slice();
        let n = self.inner.write(byte_buf)?;

        let bytes_per_sample = byte_buf.len() / buf.len();
        if n % bytes_per_sample != 0 {
            todo!()
        }

        Ok(n / bytes_per_sample)
    }

    fn flush(&mut self) -> Result<(), SyphonError> {
        self.inner.flush().map_err(Into::into)
    }
}

impl<T, S: Sample> Stream for PcmCodec<T, S> {
    fn spec(&self) -> &StreamSpec {
        &self.spec
    }
}

impl<T, S> Read for PcmCodec<T, S>
where
    T: SignalReader<Sample = S>,
    S: Sample + FromByteSlice,
{
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let start_i = size_of::<S>() - (buf.as_ptr() as usize % align_of::<S>());
        let aligned_len = buf.len() - start_i;
        let usable_len = aligned_len - (aligned_len % size_of::<S>());

        let sample_buf = match buf[start_i..start_i + usable_len].as_mut_slice_of::<S>() {
            Ok(buf) => buf,
            _ => return Err(io::ErrorKind::InvalidData.into()),
        };

        let n = self.inner.read(sample_buf)?;
        buf.rotate_left(start_i);
        Ok(n * size_of::<S>())
    }
}

impl<T, S> Write for PcmCodec<T, S>
where
    T: SignalWriter<Sample = S>,
    S: Sample + FromByteSlice,
{
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let start_i = size_of::<S>() - (buf.as_ptr() as usize % align_of::<S>());
        let aligned_len = buf.len() - start_i;
        let usable_len = aligned_len - (aligned_len % size_of::<S>());

        let sample_buf = match buf[start_i..start_i + usable_len].as_slice_of::<S>() {
            Ok(buf) => buf,
            _ => return Err(io::ErrorKind::InvalidData.into()),
        };

        let n = self.inner.write(sample_buf)?;
        Ok(n * size_of::<S>())
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush().map_err(Into::into)
    }
}
