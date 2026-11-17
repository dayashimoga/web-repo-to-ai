//! # Bounded Decompressor
//!
//! Safe, bounded decompression for Gzip, Deflate, Brotli, and Zstandard with compression bomb ratio guards.

use std::io::Read;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use crate::compression::encoding::ContentEncoding;
use crate::compression::limits::BodyLimits;
use crate::error::WebExtractorError;

/// Reader wrapper that tracks the cumulative compressed bytes read from the underlying stream.
struct CountingReader<R> {
    inner: R,
    bytes_read: Arc<AtomicUsize>,
}

impl<R: Read> CountingReader<R> {
    fn new(inner: R, bytes_read: Arc<AtomicUsize>) -> Self {
        Self { inner, bytes_read }
    }
}

impl<R: Read> Read for CountingReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let n = self.inner.read(buf)?;
        self.bytes_read.fetch_add(n, Ordering::Relaxed);
        Ok(n)
    }
}

/// Provides streaming bounded decompression enforcing size limits and compression bomb ratio ceilings.
pub struct BoundedDecompressor;

impl BoundedDecompressor {
    /// Decompresses raw bytes using the specified content encoding under configured safety limits.
    pub fn decompress(
        raw_bytes: &[u8],
        encoding: ContentEncoding,
        limits: &BodyLimits,
    ) -> Result<Vec<u8>, WebExtractorError> {
        // Enforce raw body limit
        if raw_bytes.len() > limits.max_raw_body_bytes {
            return Err(WebExtractorError::PayloadTooLarge {
                actual: raw_bytes.len(),
                maximum: limits.max_raw_body_bytes,
            });
        }

        match encoding {
            ContentEncoding::Identity => {
                if raw_bytes.len() > limits.max_decompressed_bytes {
                    return Err(WebExtractorError::PayloadTooLarge {
                        actual: raw_bytes.len(),
                        maximum: limits.max_decompressed_bytes,
                    });
                }
                Ok(raw_bytes.to_vec())
            }
            ContentEncoding::Gzip => {
                let counter = Arc::new(AtomicUsize::new(0));
                let counting = CountingReader::new(raw_bytes, Arc::clone(&counter));
                let decoder = flate2::read::GzDecoder::new(counting);
                Self::decode_bounded(decoder, &counter, encoding, limits)
            }
            ContentEncoding::Deflate => {
                // HTTP deflate may be zlib-wrapped (RFC 1950) or raw deflate (RFC 1951).
                // Try zlib first; fallback to raw deflate if header is not present.
                let counter_zlib = Arc::new(AtomicUsize::new(0));
                let counting_zlib = CountingReader::new(raw_bytes, Arc::clone(&counter_zlib));
                let zlib_decoder = flate2::read::ZlibDecoder::new(counting_zlib);

                match Self::decode_bounded(zlib_decoder, &counter_zlib, encoding, limits) {
                    Ok(data) => Ok(data),
                    Err(WebExtractorError::DecompressionBomb { .. })
                    | Err(WebExtractorError::PayloadTooLarge { .. }) => {
                        let counter_zlib2 = Arc::new(AtomicUsize::new(0));
                        let counting_zlib2 =
                            CountingReader::new(raw_bytes, Arc::clone(&counter_zlib2));
                        let zlib_decoder2 = flate2::read::ZlibDecoder::new(counting_zlib2);
                        Self::decode_bounded(zlib_decoder2, &counter_zlib2, encoding, limits)
                    }
                    Err(_) => {
                        let counter_raw = Arc::new(AtomicUsize::new(0));
                        let counting_raw = CountingReader::new(raw_bytes, Arc::clone(&counter_raw));
                        let raw_decoder = flate2::read::DeflateDecoder::new(counting_raw);
                        Self::decode_bounded(raw_decoder, &counter_raw, encoding, limits)
                    }
                }
            }
            ContentEncoding::Brotli => {
                let counter = Arc::new(AtomicUsize::new(0));
                let counting = CountingReader::new(raw_bytes, Arc::clone(&counter));
                let decoder = brotli::Decompressor::new(counting, 4096);
                Self::decode_bounded(decoder, &counter, encoding, limits)
            }
            ContentEncoding::Zstd => {
                let counter = Arc::new(AtomicUsize::new(0));
                let counting = CountingReader::new(raw_bytes, Arc::clone(&counter));
                let decoder = zstd::stream::read::Decoder::new(counting).map_err(|e| {
                    WebExtractorError::DecompressionFailed {
                        encoding: encoding.to_string(),
                        details: e.to_string(),
                    }
                })?;
                Self::decode_bounded(decoder, &counter, encoding, limits)
            }
        }
    }

    /// Decompresses a chain of applied encodings in reverse order per RFC 9110 § 8.4.
    pub fn decompress_chain(
        raw_bytes: &[u8],
        encodings: &[ContentEncoding],
        limits: &BodyLimits,
    ) -> Result<Vec<u8>, WebExtractorError> {
        if encodings.is_empty()
            || (encodings.len() == 1 && encodings[0] == ContentEncoding::Identity)
        {
            return Self::decompress(raw_bytes, ContentEncoding::Identity, limits);
        }

        let mut current = raw_bytes.to_vec();
        for &encoding in encodings.iter().rev() {
            current = Self::decompress(&current, encoding, limits)?;
        }
        Ok(current)
    }

    /// Streams data through any `Read` decoder into an output buffer, enforcing memory and ratio limits.
    fn decode_bounded<D: Read>(
        mut decoder: D,
        counter: &AtomicUsize,
        encoding: ContentEncoding,
        limits: &BodyLimits,
    ) -> Result<Vec<u8>, WebExtractorError> {
        let mut decompressed = Vec::new();
        let mut chunk = [0u8; 4096];

        loop {
            let n = match decoder.read(&mut chunk) {
                Ok(0) => break,
                Ok(n) => n,
                Err(err) => {
                    return Err(WebExtractorError::DecompressionFailed {
                        encoding: encoding.to_string(),
                        details: err.to_string(),
                    });
                }
            };

            decompressed.extend_from_slice(&chunk[..n]);

            // 1. Enforce max decompressed memory limit
            if decompressed.len() > limits.max_decompressed_bytes {
                return Err(WebExtractorError::PayloadTooLarge {
                    actual: decompressed.len(),
                    maximum: limits.max_decompressed_bytes,
                });
            }

            // 2. Enforce compression bomb ratio ceiling once threshold is reached
            if decompressed.len() >= limits.min_ratio_check_bytes {
                let compressed_so_far = counter.load(Ordering::Relaxed).max(1);
                let current_ratio = decompressed.len() as f64 / compressed_so_far as f64;

                if current_ratio > limits.max_expansion_ratio {
                    return Err(WebExtractorError::DecompressionBomb {
                        ratio: current_ratio,
                        max_ratio: limits.max_expansion_ratio,
                        compressed_bytes: compressed_so_far,
                        decompressed_bytes: decompressed.len(),
                    });
                }
            }
        }

        Ok(decompressed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_gzip_normal_roundtrip() {
        let data = b"Hello, this is a test of Gzip compression in web-extractor!".repeat(10);
        let mut encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
        encoder.write_all(&data).unwrap();
        let compressed = encoder.finish().unwrap();

        let limits = BodyLimits::default();
        let decompressed =
            BoundedDecompressor::decompress(&compressed, ContentEncoding::Gzip, &limits).unwrap();
        assert_eq!(decompressed, data);
    }

    #[test]
    fn test_deflate_zlib_roundtrip() {
        let data = b"Deflate test with zlib headers for web extraction".repeat(10);
        let mut encoder =
            flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::default());
        encoder.write_all(&data).unwrap();
        let compressed = encoder.finish().unwrap();

        let limits = BodyLimits::default();
        let decompressed =
            BoundedDecompressor::decompress(&compressed, ContentEncoding::Deflate, &limits)
                .unwrap();
        assert_eq!(decompressed, data);
    }

    #[test]
    fn test_deflate_raw_roundtrip() {
        let data = b"Raw deflate test without zlib header".repeat(10);
        let mut encoder =
            flate2::write::DeflateEncoder::new(Vec::new(), flate2::Compression::default());
        encoder.write_all(&data).unwrap();
        let compressed = encoder.finish().unwrap();

        let limits = BodyLimits::default();
        let decompressed =
            BoundedDecompressor::decompress(&compressed, ContentEncoding::Deflate, &limits)
                .unwrap();
        assert_eq!(decompressed, data);
    }

    #[test]
    fn test_brotli_normal_roundtrip() {
        let data = b"Brotli test payload for HTTP content decompression".repeat(10);
        let mut compressed = Vec::new();
        {
            let mut writer = brotli::CompressorWriter::new(&mut compressed, 4096, 6, 22);
            writer.write_all(&data).unwrap();
        }

        let limits = BodyLimits::default();
        let decompressed =
            BoundedDecompressor::decompress(&compressed, ContentEncoding::Brotli, &limits).unwrap();
        assert_eq!(decompressed, data);
    }

    #[test]
    fn test_zstd_normal_roundtrip() {
        let data = b"Zstandard compression test in web-extractor".repeat(10);
        let compressed = zstd::encode_all(&data[..], 3).unwrap();

        let limits = BodyLimits::default();
        let decompressed =
            BoundedDecompressor::decompress(&compressed, ContentEncoding::Zstd, &limits).unwrap();
        assert_eq!(decompressed, data);
    }

    #[test]
    fn test_compression_bomb_detected_and_aborted() {
        // Create 100 KB of zeros, which compresses into very few bytes (~100 bytes)
        let zeros = vec![0u8; 100 * 1024];
        let mut encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::best());
        encoder.write_all(&zeros).unwrap();
        let compressed = encoder.finish().unwrap();

        // Configure limits: ratio limit 5.0, threshold 1024 bytes
        let limits = BodyLimits::default()
            .with_max_expansion_ratio(5.0)
            .with_min_ratio_check_bytes(1024);

        let result = BoundedDecompressor::decompress(&compressed, ContentEncoding::Gzip, &limits);
        match result {
            Err(WebExtractorError::DecompressionBomb {
                ratio, max_ratio, ..
            }) => {
                assert!(
                    ratio > max_ratio,
                    "Ratio {ratio} should exceed max {max_ratio}"
                );
            }
            other => panic!("Expected DecompressionBomb error, got: {other:?}"),
        }
    }

    #[test]
    fn test_decompressed_size_ceiling_exceeded() {
        // Use non-repeating data with reasonable compression ratio (e.g. 2:1)
        let data: Vec<u8> = (0..10 * 1024)
            .map(|i| ((i * 31 + 17) % 256) as u8)
            .collect();
        let mut encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
        encoder.write_all(&data).unwrap();
        let compressed = encoder.finish().unwrap();

        // Ceiling is 4 KB, but uncompressed data is 10 KB; set high ratio allowance to isolate size ceiling
        let limits = BodyLimits::new(1024 * 1024, 4 * 1024).with_max_expansion_ratio(100.0);

        let result = BoundedDecompressor::decompress(&compressed, ContentEncoding::Gzip, &limits);
        match result {
            Err(WebExtractorError::PayloadTooLarge { actual, maximum }) => {
                assert_eq!(maximum, 4 * 1024);
                assert!(actual > maximum);
            }
            other => panic!("Expected PayloadTooLarge error, got: {other:?}"),
        }
    }

    #[test]
    fn test_corrupt_compressed_data() {
        let corrupt = b"not a valid gzip header or stream";
        let limits = BodyLimits::default();
        let result = BoundedDecompressor::decompress(corrupt, ContentEncoding::Gzip, &limits);
        assert!(matches!(
            result,
            Err(WebExtractorError::DecompressionFailed { .. })
        ));
    }
}
