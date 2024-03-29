use crate::KnownCodec;
use lazy_static::lazy_static;
use std::collections::HashMap;
use phonic_core::PhonicError;
use phonic_format_wave::WAVE_IDENTIFIERS;
use phonic_io_core::{
    utils::{FormatIdentifier, FormatIdentifiers},
    DynFormat, DynFormatConstructor, FormatData, FormatTag, StdIoSource,
};

#[derive(Eq, PartialEq, Copy, Clone, Hash, Debug)]
#[non_exhaustive]
pub enum KnownFormat {
    #[cfg(feature = "wave")]
    Wave,
}

lazy_static! {
    static ref KNOWN_FORMAT_IDENTIFIERS: HashMap<KnownFormat, &'static FormatIdentifiers> = {
        let mut map = HashMap::new();

        #[cfg(feature = "wave")]
        map.insert(KnownFormat::Wave, &WAVE_IDENTIFIERS);

        map
    };
}

impl FormatTag for KnownFormat {
    type Codec = KnownCodec;

    fn fill_data(data: &mut FormatData<Self>) -> Result<(), PhonicError> {
        match data.format {
            #[cfg(feature = "wave")]
            Some(Self::Wave) => crate::formats::wave::fill_wave_data(data),

            _ => return Ok(()),
        }
    }
}

impl DynFormatConstructor for KnownFormat {
    fn from_std_io<S: StdIoSource + 'static>(
        &self,
        inner: S,
    ) -> Result<Box<dyn DynFormat<Tag = Self>>, PhonicError> {
        Ok(match self {
            #[cfg(feature = "wave")]
            KnownFormat::Wave => Box::new(crate::formats::wave::WaveFormat::new(inner)?),

            _ => return Err(PhonicError::Unsupported),
        })
    }
}

impl<'a> TryFrom<&FormatIdentifier<'a>> for KnownFormat {
    type Error = PhonicError;

    fn try_from(id: &FormatIdentifier<'a>) -> Result<Self, Self::Error> {
        KNOWN_FORMAT_IDENTIFIERS
            .iter()
            .find(|(_, ids)| ids.contains(id))
            .map(|(fmt, _)| *fmt)
            .ok_or(PhonicError::NotFound)
    }
}

#[cfg(feature = "wave")]
impl From<crate::formats::wave::WaveFormatTag> for KnownFormat {
    fn from(_: crate::formats::wave::WaveFormatTag) -> Self {
        Self::Wave
    }
}

#[cfg(feature = "wave")]
impl TryFrom<KnownFormat> for crate::formats::wave::WaveFormatTag {
    type Error = PhonicError;

    fn try_from(format: KnownFormat) -> Result<Self, Self::Error> {
        match format {
            KnownFormat::Wave => Ok(Self),
            _ => Err(PhonicError::Unsupported),
        }
    }
}
