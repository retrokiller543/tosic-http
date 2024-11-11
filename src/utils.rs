use bytes::BufMut;
use std::io;

pub(crate) struct MutWriter<'a, B>(pub(crate) &'a mut B);

impl<'a, B> io::Write for MutWriter<'a, B>
where
    B: BufMut,
{
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0.put_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
