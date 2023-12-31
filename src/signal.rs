use crate::{Sample, SyphonError};
use std::{
    ops::{BitAnd, BitOr, BitXor, Deref, DerefMut},
    time::Duration,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Channels {
    Count(u32),
    Layout(ChannelLayout),
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct ChannelLayout {
    mask: u32,
}

/// A set of parameters that describes a pcm signal
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct SignalSpec {
    /// The number of samples per channel per second.
    pub frame_rate: u32,

    /// The layout of the channels in the signal.
    pub channels: Channels,

    /// The total number of sample blocks in the signal.
    pub n_frames: Option<u64>,
}

#[derive(Debug, Clone, Copy)]
pub struct SignalSpecBuilder {
    pub frame_rate: Option<u32>,
    pub channels: Option<Channels>,
    pub n_frames: Option<u64>,
}

impl Channels {
    pub fn count(&self) -> u32 {
        match self {
            Self::Count(n) => *n,
            Self::Layout(layout) => layout.count(),
        }
    }

    pub fn layout(&self) -> Option<&ChannelLayout> {
        match self {
            Self::Count(_) => None,
            Self::Layout(layout) => Some(layout),
        }
    }
}

impl From<u32> for Channels {
    fn from(n: u32) -> Self {
        Self::Count(n)
    }
}

impl From<ChannelLayout> for Channels {
    fn from(layout: ChannelLayout) -> Self {
        Self::Layout(layout)
    }
}

impl ChannelLayout {
    pub const fn from_bits(mask: u32) -> Self {
        Self { mask }
    }

    pub const FRONT_LEFT: Self = Self::from_bits(1 << 0);
    pub const FRONT_RIGHT: Self = Self::from_bits(1 << 1);
    pub const FRONT_CENTRE: Self = Self::from_bits(1 << 2);
    pub const LFE1: Self = Self::from_bits(1 << 3);
    pub const REAR_LEFT: Self = Self::from_bits(1 << 4);
    pub const REAR_RIGHT: Self = Self::from_bits(1 << 5);
    pub const FRONT_LEFT_CENTRE: Self = Self::from_bits(1 << 6);
    pub const FRONT_RIGHT_CENTRE: Self = Self::from_bits(1 << 7);
    pub const REAR_CENTRE: Self = Self::from_bits(1 << 8);
    pub const SIDE_LEFT: Self = Self::from_bits(1 << 9);
    pub const SIDE_RIGHT: Self = Self::from_bits(1 << 10);
    pub const TOP_CENTRE: Self = Self::from_bits(1 << 11);
    pub const TOP_FRONT_LEFT: Self = Self::from_bits(1 << 12);
    pub const TOP_FRONT_CENTRE: Self = Self::from_bits(1 << 13);
    pub const TOP_FRONT_RIGHT: Self = Self::from_bits(1 << 14);
    pub const TOP_REAR_LEFT: Self = Self::from_bits(1 << 15);
    pub const TOP_REAR_CENTRE: Self = Self::from_bits(1 << 16);
    pub const TOP_REAR_RIGHT: Self = Self::from_bits(1 << 17);
    pub const REAR_LEFT_CENTRE: Self = Self::from_bits(1 << 18);
    pub const REAR_RIGHT_CENTRE: Self = Self::from_bits(1 << 19);
    pub const FRONT_LEFT_WIDE: Self = Self::from_bits(1 << 20);
    pub const FRONT_RIGHT_WIDE: Self = Self::from_bits(1 << 21);
    pub const FRONT_LEFT_HIGH: Self = Self::from_bits(1 << 22);
    pub const FRONT_CENTRE_HIGH: Self = Self::from_bits(1 << 23);
    pub const FRONT_RIGHT_HIGH: Self = Self::from_bits(1 << 24);
    pub const LFE2: Self = Self::from_bits(1 << 25);

    pub const MONO: Self = Self::FRONT_LEFT;
    pub const STEREO: Self = Self::from_bits(Self::FRONT_LEFT.mask | Self::FRONT_RIGHT.mask);
    pub const STEREO_2_1: Self = Self::from_bits(Self::STEREO.mask | Self::LFE1.mask);
    pub const SURROUND_5_1: Self = Self::from_bits(
        Self::STEREO_2_1.mask
            | Self::FRONT_CENTRE.mask
            | Self::REAR_LEFT.mask
            | Self::REAR_RIGHT.mask,
    );

    pub const SURROUND_7_1: Self =
        Self::from_bits(Self::SURROUND_5_1.mask | Self::SIDE_LEFT.mask | Self::SIDE_RIGHT.mask);

    #[inline]
    pub fn count(&self) -> u32 {
        self.mask.count_ones()
    }
}

impl BitAnd for ChannelLayout {
    type Output = Self;

    #[inline]
    fn bitand(self, rhs: Self) -> Self::Output {
        Self::from_bits(self.mask & rhs.mask)
    }
}

impl BitOr for ChannelLayout {
    type Output = Self;

    #[inline]
    fn bitor(self, rhs: Self) -> Self::Output {
        Self::from_bits(self.mask | rhs.mask)
    }
}

impl BitXor for ChannelLayout {
    type Output = Self;

    #[inline]
    fn bitxor(self, rhs: Self) -> Self::Output {
        Self::from_bits(self.mask ^ rhs.mask)
    }
}

impl SignalSpec {
    pub fn builder() -> SignalSpecBuilder {
        SignalSpecBuilder::new()
    }

    pub fn n_samples(&self) -> Option<u64> {
        self.n_frames.map(|n| n * self.channels.count() as u64)
    }

    pub fn duration(&self) -> Option<Duration> {
        self.n_frames
            .map(|n| Duration::from_secs_f64(n as f64 / self.frame_rate as f64))
    }
}

impl TryFrom<SignalSpecBuilder> for SignalSpec {
    type Error = SyphonError;

    fn try_from(builder: SignalSpecBuilder) -> Result<Self, Self::Error> {
        Ok(Self {
            channels: builder.channels.ok_or(SyphonError::MissingData)?,
            frame_rate: builder.frame_rate.ok_or(SyphonError::MissingData)?,
            n_frames: builder.n_frames,
        })
    }
}

impl SignalSpecBuilder {
    pub fn new() -> Self {
        Self {
            frame_rate: None,
            channels: None,
            n_frames: None,
        }
    }

    pub fn n_samples(&self) -> Option<u64> {
        self.n_frames
            .zip(self.channels)
            .map(|(n_frames, channels)| n_frames * channels.count() as u64)
    }

    pub fn duration(&self) -> Option<Duration> {
        self.n_frames
            .zip(self.frame_rate)
            .map(|(n_frames, frame_rate)| {
                Duration::from_secs_f64(n_frames as f64 / frame_rate as f64)
            })
    }

    pub fn with_frame_rate(mut self, frame_rate: u32) -> Self {
        self.frame_rate = Some(frame_rate);
        self
    }

    pub fn with_channels(mut self, channels: impl Into<Channels>) -> Self {
        self.channels = Some(channels.into());
        self
    }

    pub fn with_n_frames(mut self, n_frames: impl Into<Option<u64>>) -> Self {
        self.n_frames = n_frames.into();
        self
    }

    pub fn with_duration(mut self, duration: Duration) -> Self {
        self.n_frames = self
            .frame_rate
            .map(|hz| (hz as f64 * duration.as_secs_f64()) as u64);

        self
    }

    pub fn build(self) -> Result<SignalSpec, SyphonError>
    where
        Self: TryInto<SignalSpec, Error = SyphonError>,
    {
        self.try_into()
    }
}

impl From<SignalSpec> for SignalSpecBuilder {
    fn from(spec: SignalSpec) -> Self {
        Self {
            frame_rate: Some(spec.frame_rate),
            channels: Some(spec.channels),
            n_frames: spec.n_frames,
        }
    }
}

pub trait Signal {
    type Sample: Sample;

    fn spec(&self) -> &SignalSpec;
}

pub trait SignalReader: Signal {
    fn read(&mut self, buffer: &mut [Self::Sample]) -> Result<usize, SyphonError>;

    fn read_exact(&mut self, mut buffer: &mut [Self::Sample]) -> Result<(), SyphonError> {
        while !buffer.is_empty() {
            match self.read(&mut buffer) {
                Ok(0) => break,
                Ok(n) => buffer = &mut buffer[n..],
                Err(SyphonError::Interrupted) => continue,
                Err(e) => return Err(e),
            };
        }

        if buffer.len() > 0 {
            return Err(SyphonError::EndOfStream);
        }

        Ok(())
    }
}

pub trait SignalWriter: Signal {
    fn write(&mut self, buffer: &[Self::Sample]) -> Result<usize, SyphonError>;
    fn flush(&mut self) -> Result<(), SyphonError>;

    fn write_exact(&mut self, mut buffer: &[Self::Sample]) -> Result<(), SyphonError> {
        while !buffer.is_empty() {
            match self.write(&buffer) {
                Ok(0) => break,
                Ok(n) => buffer = &buffer[n..],
                Err(SyphonError::Interrupted) => continue,
                Err(e) => return Err(e),
            };
        }

        if buffer.len() > 0 {
            return Err(SyphonError::EndOfStream);
        }

        Ok(())
    }
}

impl<T, S> Signal for T
where
    S: Sample,
    T: Deref,
    T::Target: Signal<Sample = S>,
{
    type Sample = S;

    fn spec(&self) -> &SignalSpec {
        self.deref().spec()
    }
}

impl<S, T> SignalReader for T
where
    S: Sample,
    T: DerefMut,
    T::Target: SignalReader<Sample = S>,
{
    fn read(&mut self, buffer: &mut [S]) -> Result<usize, SyphonError> {
        self.deref_mut().read(buffer)
    }
}

impl<S, T> SignalWriter for T
where
    S: Sample,
    T: DerefMut,
    T::Target: SignalWriter<Sample = S>,
{
    fn write(&mut self, buffer: &[S]) -> Result<usize, SyphonError> {
        self.deref_mut().write(buffer)
    }

    fn flush(&mut self) -> Result<(), SyphonError> {
        self.deref_mut().flush()
    }
}
