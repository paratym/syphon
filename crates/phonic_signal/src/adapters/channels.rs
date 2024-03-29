use crate::{Channels, Signal, SignalReader, SignalSpec, SignalWriter};
use phonic_core::PhonicError;

pub struct ChannelsAdapter<T: Signal> {
    signal: T,
    spec: SignalSpec,
}

impl<T: Signal> ChannelsAdapter<T> {
    pub fn new(signal: T, channels: Channels) -> Self {
        let mut spec = *signal.spec();
        spec.channels = channels;

        Self { signal, spec }
    }
}

impl<T: Signal> Signal for ChannelsAdapter<T> {
    type Sample = T::Sample;

    fn spec(&self) -> &SignalSpec {
        &self.spec
    }
}

impl<T: SignalReader> SignalReader for ChannelsAdapter<T> {
    fn read(&mut self, buffer: &mut [Self::Sample]) -> Result<usize, PhonicError> {
        todo!()
    }
}

impl<T: SignalWriter> SignalWriter for ChannelsAdapter<T> {
    fn write(&mut self, buffer: &[Self::Sample]) -> Result<usize, PhonicError> {
        todo!()
    }

    fn flush(&mut self) -> Result<(), PhonicError> {
        todo!()
    }
}
