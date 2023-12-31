use crate::{Sample, SignalReader, SignalWriter, SyphonError};

pub fn copy<S: Sample>(
    reader: &mut impl SignalReader<Sample = S>,
    writer: &mut impl SignalWriter<Sample = S>,
    mut buffer: &mut [S],
) -> Result<(), SyphonError> {
    let spec = reader.spec();
    if spec != writer.spec() {
        return Err(SyphonError::SignalMismatch);
    }

    let mut n;
    loop {
        n = match reader.read(&mut buffer) {
            Ok(0) => return Ok(()),
            Ok(n) => n,
            Err(SyphonError::EndOfStream) => return Ok(()),
            Err(SyphonError::Interrupted) | Err(SyphonError::NotReady) => continue,
            Err(e) => return Err(e),
        };

        writer.write_exact(&buffer[..n])?;
    }
}
