use std::{
    collections::HashMap,
    io::{BufReader, Write},
    sync::RwLock,
};

use brotli::enc::BrotliEncoderParams;
use flate2::Compression;
use lazy_static::lazy_static;
use regex::Regex;

/// When should the server try sending a compressed response?
#[derive(Default)]
pub enum Compress {
    /// Never compress responses, even if a precompressed response is available.
    Never,
    ///  Only use a compressed response if a precompressed response is available.
    ///
    /// With this option, compression won't be performed "on-the-fly".
    /// This significantly reduces the CPU usage, but will increase the amount of data transferred.
    ///
    /// This option will only work with `rust-embed-for-web` and only if compression has not been disabled.
    /// With `rust-embed`, or if the `rust-embed-for-web` resource is tagged with `#[gzip = "false"]` this is equivalent to Never.
    ///
    #[default]
    IfPrecompressed,
    /// Perform on-the-fly compression if the file mime type is well known to be compressible.
    ///
    /// This option allows you to use compression with `rust-embed-for-web` when the resource is tagged with `#[gzip = "false"]`.
    /// This will use some CPU to compress the file on the fly before responding. Compressed versions are cached in memory.
    ///
    IfWellKnown,
    /// With this option set, the file is always compressed (as long as the client supports it).
    ///
    /// This is usually not a good idea unless you know that all the files embedded are compressible.
    /// File formats that are already compressed will not compress any further (such as image or video files),
    /// in which case trying to use compression is just a waste of CPU time.
    ///
    Always,
}

/// This is basically a list of text mime types, plus javascript, json, and xml.
pub(crate) fn is_well_known_compressible_mime_type(mime_type: &str) -> bool {
    lazy_static! {
        static ref RE: Regex =
            Regex::new(r#"^text/.*|application/(javascript|json5?|(ld|jsonml)[+]json|xml)$"#)
                .unwrap();
    }
    RE.is_match(mime_type)
}

// Putting the data into cache could potentially fail. That's okay if it does
// happen, we have no way of handling that and we might as well just keep
// serving files.
#[allow(unused_must_use)]
/// Compresses data with gzip encoding.
///
/// The compressed files are cached based on the hash values provided.
/// Since we already have the hashes precomputed in rust-embed and rust-embed-for-web,
/// we just reuse that instead of trying to hash the data this function gets.
pub(crate) fn compress_data_gzip(hash: &str, data: &[u8]) -> Vec<u8> {
    lazy_static! {
        static ref CACHED_GZIP_DATA: RwLock<HashMap<String, Vec<u8>>> = RwLock::new(HashMap::new());
    }

    if let Some(data_gzip) = CACHED_GZIP_DATA
        .read()
        .ok()
        .and_then(|cached| cached.get(hash).map(ToOwned::to_owned))
    {
        return data_gzip;
    }

    let mut compressed: Vec<u8> = Vec::new();
    flate2::write::GzEncoder::new(&mut compressed, Compression::default())
        .write_all(data)
        .unwrap();
    CACHED_GZIP_DATA
        .write()
        .map(|mut cached| cached.insert(hash.to_string(), compressed.clone()));
    compressed
}

// Putting the data into cache could potentially fail. That's okay if it does
// happen, we have no way of handling that and we might as well just keep
// serving files.
#[allow(unused_must_use)]
/// Compresses data with gzip encoding.
///
/// The compressed files are cached based on the hash values provided.
/// Since we already have the hashes precomputed in rust-embed and rust-embed-for-web,
/// we just reuse that instead of trying to hash the data this function gets.
pub(crate) fn compress_data_br(hash: &str, data: &[u8]) -> Vec<u8> {
    lazy_static! {
        static ref CACHED_BR_DATA: RwLock<HashMap<String, Vec<u8>>> = RwLock::new(HashMap::new());
    }

    if let Some(data_gzip) = CACHED_BR_DATA
        .read()
        .ok()
        .and_then(|cached| cached.get(hash).map(ToOwned::to_owned))
    {
        return data_gzip;
    }

    let mut data_read = BufReader::new(data);
    let mut compressed: Vec<u8> = Vec::new();
    brotli::BrotliCompress(
        &mut data_read,
        &mut compressed,
        &BrotliEncoderParams::default(),
    )
    .expect("Failed to compress br data");
    CACHED_BR_DATA
        .write()
        .map(|mut cached| cached.insert(hash.to_string(), compressed.clone()));
    compressed
}

// Putting the data into cache could potentially fail. That's okay if it does
// happen, we have no way of handling that and we might as well just keep
// serving files.
#[allow(unused_must_use)]
/// Compresses data with zstd encoding.
///
/// The compressed files are cached based on the hash values provided.
/// Since we already have the hashes precomputed in rust-embed and rust-embed-for-web,
/// we just reuse that instead of trying to hash the data this function gets.
#[cfg(feature = "compression-zstd")]
pub(crate) fn compress_data_zstd(hash: &str, data: &[u8]) -> Vec<u8> {
    lazy_static! {
        static ref CACHED_ZSTD_DATA: RwLock<HashMap<String, Vec<u8>>> = RwLock::new(HashMap::new());
    }

    if let Some(data_zstd) = CACHED_ZSTD_DATA
        .read()
        .ok()
        .and_then(|cached| cached.get(hash).map(ToOwned::to_owned))
    {
        return data_zstd;
    }

    let compressed = zstd::encode_all(data, 0).expect("Failed to compress zstd data");
    CACHED_ZSTD_DATA
        .write()
        .map(|mut cached| cached.insert(hash.to_string(), compressed.clone()));
    compressed
}

#[allow(unused_imports)]
mod test {
    use crate::compress::is_well_known_compressible_mime_type;
    use crate::compress_data_gzip;
    use std::io::Write;
    use std::time::Instant;

    #[test]
    fn html_file_is_compressible() {
        assert!(is_well_known_compressible_mime_type("text/html"))
    }

    #[test]
    fn css_file_is_compressible() {
        assert!(is_well_known_compressible_mime_type("text/css"))
    }

    #[test]
    fn javascript_file_is_compressible() {
        assert!(is_well_known_compressible_mime_type(
            "application/javascript"
        ))
    }

    #[test]
    fn json_file_is_compressible() {
        assert!(is_well_known_compressible_mime_type("application/json"))
    }

    #[test]
    fn xml_file_is_compressible() {
        assert!(is_well_known_compressible_mime_type("application/xml"))
    }

    #[test]
    fn jpg_file_not_compressible() {
        assert!(!is_well_known_compressible_mime_type("image/jpeg"))
    }

    #[test]
    fn zip_file_not_compressible() {
        assert!(!is_well_known_compressible_mime_type("application/zip"))
    }

    #[test]
    fn gzip_roundtrip() {
        let source = b"x123";
        let compressed = compress_data_gzip("foo", source);
        let mut decompressed = Vec::new();
        flate2::write::GzDecoder::new(&mut decompressed)
            .write_all(&compressed)
            .unwrap();
        assert_eq!(source, &decompressed[..]);
    }

    #[test]
    fn compression_is_cached() {
        let source = b"Et quos non sed magnam reiciendis praesentium quod libero. Architecto optio tempora iure aspernatur rerum voluptatem quas. Eos ut atque quas perspiciatis dolorem quidem. Cum et quo et. Voluptatum ut est id eligendi illum inventore. Est non rerum vel rem. Molestiae similique alias nihil harum qui. Consectetur et dolores autem. Magnam et saepe ad reprehenderit. Repellendus vel excepturi eaque esse error. Deserunt est impedit totam nostrum sunt. Eligendi magnam distinctio odit iste molestias est id. Deserunt odit similique magnam repudiandae aut saepe. Dolores laboriosam consectetur quos dolores ea. Non quod veniam quisquam molestias aut deserunt tempora. Mollitia consequuntur facilis doloremque provident eligendi similique possimus. Deleniti facere quam fugiat porro. Tenetur cupiditate eum consequatur beatae dolorum. Veniam voluptatem qui eum quasi corrupti. Quis necessitatibus maxime eum numquam ipsam ducimus expedita maiores. Aliquid voluptas non aut. Tempore dicta ut aperiam ipsum ut et esse explicabo.";

        let first_start = Instant::now();
        compress_data_gzip("lorem", source);
        let first = first_start.elapsed();
        let second_start = Instant::now();
        compress_data_gzip("lorem", source);
        let second = second_start.elapsed();

        // Check that the second call was faster
        assert!(first > second);
    }

    #[test]
    #[cfg(feature = "compression-zstd")]
    fn zstd_roundtrip() {
        let source = b"x123";
        let compressed = crate::compress_data_zstd("foo", source);
        let decompressed = zstd::decode_all(&compressed[..]).unwrap();
        assert_eq!(source, &decompressed[..]);
    }

    #[test]
    #[cfg(feature = "compression-zstd")]
    fn zstd_compression_is_cached() {
        let source = b"Et quos non sed magnam reiciendis praesentium quod libero. Architecto optio tempora iure aspernatur rerum voluptatem quas. Eos ut atque quas perspiciatis dolorem quidem. Cum et quo et. Voluptatum ut est id eligendi illum inventore. Est non rerum vel rem. Molestiae similique alias nihil harum qui. Consectetur et dolores autem. Magnam et saepe ad reprehenderit. Repellendus vel excepturi eaque esse error. Deserunt est impedit totam nostrum sunt. Eligendi magnam distinctio odit iste molestias est id. Deserunt odit similique magnam repudiandae aut saepe. Dolores laboriosam consectetur quos dolores ea. Non quod veniam quisquam molestias aut deserunt tempora. Mollitia consequuntur facilis doloremque provident eligendi similique possimus. Deleniti facere quam fugiat porro. Tenetur cupiditate eum consequatur beatae dolorum. Veniam voluptatem qui eum quasi corrupti. Quis necessitatibus maxime eum numquam ipsam ducimus expedita maiores. Aliquid voluptas non aut. Tempore dicta ut aperiam ipsum ut et esse explicabo.";

        let first_start = Instant::now();
        crate::compress_data_zstd("lorem-zstd", source);
        let first = first_start.elapsed();
        let second_start = Instant::now();
        crate::compress_data_zstd("lorem-zstd", source);
        let second = second_start.elapsed();

        // Check that the second call was faster
        assert!(first > second);
    }
}
