//! # Internal Utility Helpers
//!
//! Shared helpers for stream counting and byte formatting across format exporters.

use std::io::Write;

/// A wrapper around any [`Write`] implementation tracking bytes and lines written.
#[derive(Debug)]
pub(crate) struct CountingWriter<W> {
    inner: W,
    pub(crate) bytes_written: usize,
    pub(crate) lines_written: usize,
}

impl<W: Write> CountingWriter<W> {
    pub(crate) fn new(inner: W) -> Self {
        Self {
            inner,
            bytes_written: 0,
            lines_written: 0,
        }
    }
}

impl<W: Write> Write for CountingWriter<W> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let n = self.inner.write(buf)?;
        self.bytes_written += n;
        self.lines_written += buf[..n].iter().filter(|&&b| b == b'\n').count();
        Ok(n)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.inner.flush()
    }
}

/// Formats a byte quantity into a human-readable string (B, KB, MB, GB).
pub(crate) fn format_bytes(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{bytes} B")
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else if bytes < 1024 * 1024 * 1024 {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.2} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    }
}
