use std::fs;
use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::Result;
use crate::error::AppError;

pub const MAX_REPORTED_DIFFERENCES: usize = 100;
const MAX_PIXELS: u64 = 16_777_216;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
pub struct Tolerances {
    pub max_channel_delta: u32,
    pub max_different_pixels: u64,
    pub max_different_percent_milli: u64,
}

#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct Difference {
    pub x: u32,
    pub y: u32,
    pub deltas: Vec<u32>,
}

#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct Rational {
    pub numerator: u64,
    pub denominator: u64,
}

#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct VisualDiffReport {
    pub schema: &'static str,
    pub baseline: String,
    pub candidate: String,
    pub width: u32,
    pub height: u32,
    pub encoding: String,
    pub channels: u8,
    pub bit_depth: u8,
    pub tolerances: Tolerances,
    pub total_pixels: u64,
    pub different_pixels: u64,
    pub different_percent_milli: Rational,
    pub maximum_observed_channel_delta: u32,
    pub sum_absolute_error: u64,
    pub mean_absolute_error: Rational,
    pub passed: bool,
    pub first_differences: Vec<Difference>,
}

#[derive(Debug, PartialEq, Eq)]
struct Image {
    width: u32,
    height: u32,
    encoding: &'static str,
    channels: u8,
    bit_depth: u8,
    samples: Vec<u32>,
}

pub fn compare_files(
    baseline: &Path,
    candidate: &Path,
    tolerances: Tolerances,
) -> Result<VisualDiffReport> {
    let baseline_image = decode_file(baseline)?;
    let candidate_image = decode_file(candidate)?;
    compare_images(
        baseline_image,
        candidate_image,
        normalized(baseline),
        normalized(candidate),
        tolerances,
    )
}

pub fn outcome(report: VisualDiffReport, report_path: Option<&Path>) -> Result<String> {
    let json = serde_json::to_string_pretty(&report)
        .map_err(|error| format!("sérialisation du visual diff: {error}"))?;
    if let Some(path) = report_path {
        write_atomic(path, format!("{json}\n").as_bytes())?;
    }
    if report.passed {
        Ok(json)
    } else {
        Err(AppError::outcome(
            "la comparaison visuelle dépasse les tolérances",
            1,
            json,
        ))
    }
}

fn compare_images(
    baseline: Image,
    candidate: Image,
    baseline_path: String,
    candidate_path: String,
    tolerances: Tolerances,
) -> Result<VisualDiffReport> {
    if baseline.width != candidate.width || baseline.height != candidate.height {
        return Err(format!(
            "dimensions incompatibles: {}x{} contre {}x{}",
            baseline.width, baseline.height, candidate.width, candidate.height
        )
        .into());
    }
    if baseline.channels != candidate.channels || baseline.bit_depth != candidate.bit_depth {
        return Err(format!(
            "formats de canaux incompatibles: {}x{} bits contre {}x{} bits",
            baseline.channels, baseline.bit_depth, candidate.channels, candidate.bit_depth
        )
        .into());
    }
    let total_pixels = u64::from(baseline.width) * u64::from(baseline.height);
    let channels = usize::from(baseline.channels);
    let mut different_pixels = 0u64;
    let mut maximum_observed_channel_delta = 0u32;
    let mut sum_absolute_error = 0u64;
    let mut first_differences = Vec::new();
    for pixel in 0..total_pixels as usize {
        let mut deltas = Vec::with_capacity(channels);
        let mut different = false;
        for channel in 0..channels {
            let index = pixel * channels + channel;
            let delta = baseline.samples[index].abs_diff(candidate.samples[index]);
            maximum_observed_channel_delta = maximum_observed_channel_delta.max(delta);
            sum_absolute_error = sum_absolute_error
                .checked_add(u64::from(delta))
                .ok_or("somme des erreurs hors limite")?;
            different |= delta > tolerances.max_channel_delta;
            deltas.push(delta);
        }
        if different {
            different_pixels += 1;
            if first_differences.len() < MAX_REPORTED_DIFFERENCES {
                first_differences.push(Difference {
                    x: pixel as u32 % baseline.width,
                    y: pixel as u32 / baseline.width,
                    deltas,
                });
            }
        }
    }
    let percent_numerator = different_pixels
        .checked_mul(100_000)
        .ok_or("pourcentage hors limite")?;
    let percent_within = percent_numerator
        <= tolerances
            .max_different_percent_milli
            .checked_mul(total_pixels)
            .ok_or("tolérance de pourcentage hors limite")?;
    let passed = different_pixels <= tolerances.max_different_pixels && percent_within;
    let sample_count = total_pixels
        .checked_mul(u64::from(baseline.channels))
        .ok_or("nombre d'échantillons hors limite")?;
    Ok(VisualDiffReport {
        schema: "aetherion.visual-diff/v1",
        baseline: baseline_path,
        candidate: candidate_path,
        width: baseline.width,
        height: baseline.height,
        encoding: if baseline.encoding == candidate.encoding {
            baseline.encoding.into()
        } else {
            format!("{}+{}", baseline.encoding, candidate.encoding)
        },
        channels: baseline.channels,
        bit_depth: baseline.bit_depth,
        tolerances,
        total_pixels,
        different_pixels,
        different_percent_milli: Rational {
            numerator: percent_numerator,
            denominator: total_pixels,
        },
        maximum_observed_channel_delta,
        sum_absolute_error,
        mean_absolute_error: Rational {
            numerator: sum_absolute_error,
            denominator: sample_count,
        },
        passed,
        first_differences,
    })
}

fn decode_file(path: &Path) -> Result<Image> {
    let bytes =
        fs::read(path).map_err(|error| format!("lecture de {}: {error}", path.display()))?;
    if bytes.starts_with(b"P6") {
        decode_netpbm(&bytes, b"P6")
    } else if bytes.starts_with(b"P5") {
        decode_netpbm(&bytes, b"P5")
    } else if bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
        decode_png(&bytes)
    } else {
        Err(format!("format d'image non pris en charge: {}", path.display()).into())
    }
}

pub fn decode_rgb_ids(path: &Path) -> Result<(u32, u32, Vec<u32>)> {
    let image = decode_file(path)?;
    if image.channels != 3 || image.bit_depth != 8 {
        return Err("la segmentation doit être une image RGB8".into());
    }
    let ids = image
        .samples
        .chunks_exact(3)
        .map(|pixel| (pixel[0] << 16) | (pixel[1] << 8) | pixel[2])
        .collect();
    Ok((image.width, image.height, ids))
}

fn decode_netpbm(bytes: &[u8], magic: &[u8; 2]) -> Result<Image> {
    let mut offset = 0usize;
    let found_magic = token(bytes, &mut offset)?;
    if found_magic != magic {
        return Err("signature Netpbm invalide".into());
    }
    let width = parse_u32(token(bytes, &mut offset)?, "largeur")?;
    let height = parse_u32(token(bytes, &mut offset)?, "hauteur")?;
    validate_dimensions(width, height)?;
    let maximum = parse_u32(token(bytes, &mut offset)?, "valeur maximale")?;
    if offset >= bytes.len() || !bytes[offset].is_ascii_whitespace() {
        return Err("séparateur Netpbm manquant".into());
    }
    offset += 1;
    let (channels, bit_depth, encoding, bytes_per_sample) = match (magic, maximum) {
        (b"P6", 255) => (3u8, 8u8, "ppm-p6-rgb8", 1usize),
        (b"P5", 65_535) => (1u8, 16u8, "pgm-p5-u16be", 2usize),
        _ => return Err("variante Netpbm non prise en charge".into()),
    };
    let sample_count = checked_sample_count(width, height, channels)?;
    let expected = sample_count
        .checked_mul(bytes_per_sample)
        .ok_or("image trop grande")?;
    if bytes.len() - offset != expected {
        return Err("taille des pixels Netpbm incohérente".into());
    }
    let samples = if bytes_per_sample == 1 {
        bytes[offset..]
            .iter()
            .map(|value| u32::from(*value))
            .collect()
    } else {
        bytes[offset..]
            .chunks_exact(2)
            .map(|value| u32::from(u16::from_be_bytes([value[0], value[1]])))
            .collect()
    };
    Ok(Image {
        width,
        height,
        encoding,
        channels,
        bit_depth,
        samples,
    })
}

fn decode_png(bytes: &[u8]) -> Result<Image> {
    if bytes.len() < 8 || &bytes[..8] != b"\x89PNG\r\n\x1a\n" {
        return Err("signature PNG invalide".into());
    }
    let mut offset = 8usize;
    let mut dimensions = None;
    let mut idat = Vec::new();
    let mut ended = false;
    while offset < bytes.len() {
        if bytes.len() - offset < 12 {
            return Err("chunk PNG tronqué".into());
        }
        let length = u32::from_be_bytes(bytes[offset..offset + 4].try_into().unwrap()) as usize;
        let end = offset
            .checked_add(12)
            .and_then(|value| value.checked_add(length))
            .ok_or("chunk PNG hors limite")?;
        if end > bytes.len() {
            return Err("chunk PNG tronqué".into());
        }
        let kind = &bytes[offset + 4..offset + 8];
        let data = &bytes[offset + 8..offset + 8 + length];
        let expected_crc = u32::from_be_bytes(bytes[offset + 8 + length..end].try_into().unwrap());
        let mut crc_data = Vec::with_capacity(4 + length);
        crc_data.extend_from_slice(kind);
        crc_data.extend_from_slice(data);
        if crc32(&crc_data) != expected_crc {
            return Err("CRC PNG invalide".into());
        }
        match kind {
            b"IHDR" => {
                if dimensions.is_some() || length != 13 {
                    return Err("IHDR PNG invalide".into());
                }
                let width = u32::from_be_bytes(data[..4].try_into().unwrap());
                let height = u32::from_be_bytes(data[4..8].try_into().unwrap());
                validate_dimensions(width, height)?;
                if data[8..] != [8, 2, 0, 0, 0] {
                    return Err("seul le PNG RGB8 non entrelacé est pris en charge".into());
                }
                dimensions = Some((width, height));
            }
            b"IDAT" => idat.extend_from_slice(data),
            b"IEND" => {
                if length != 0 || end != bytes.len() {
                    return Err("IEND PNG invalide".into());
                }
                ended = true;
            }
            _ => return Err("chunk PNG non pris en charge".into()),
        }
        offset = end;
        if ended {
            break;
        }
    }
    let (width, height) = dimensions.ok_or("IHDR PNG manquant")?;
    if !ended || idat.is_empty() {
        return Err("PNG incomplet".into());
    }
    let raw = inflate_stored_zlib(&idat)?;
    let stride = checked_sample_count(width, 1, 3)?;
    let expected = (stride + 1)
        .checked_mul(height as usize)
        .ok_or("image trop grande")?;
    if raw.len() != expected {
        return Err("taille des pixels PNG incohérente".into());
    }
    let mut samples = Vec::with_capacity(stride * height as usize);
    for row in raw.chunks_exact(stride + 1) {
        if row[0] != 0 {
            return Err("seul le filtre PNG 0 est pris en charge".into());
        }
        samples.extend(row[1..].iter().map(|value| u32::from(*value)));
    }
    Ok(Image {
        width,
        height,
        encoding: "png-rgb8",
        channels: 3,
        bit_depth: 8,
        samples,
    })
}

fn inflate_stored_zlib(bytes: &[u8]) -> Result<Vec<u8>> {
    if bytes.len() < 6 || bytes[0] != 0x78 || bytes[1] != 0x01 {
        return Err("flux zlib PNG non pris en charge".into());
    }
    let data_end = bytes.len() - 4;
    let mut offset = 2usize;
    let mut output = Vec::new();
    let mut final_seen = false;
    while offset < data_end {
        let header = bytes[offset];
        offset += 1;
        if header & 0xfe != 0 {
            return Err("seuls les blocs deflate stockés sont pris en charge".into());
        }
        let final_block = header & 1 != 0;
        if offset + 4 > data_end {
            return Err("bloc deflate tronqué".into());
        }
        let length = u16::from_le_bytes([bytes[offset], bytes[offset + 1]]);
        let complement = u16::from_le_bytes([bytes[offset + 2], bytes[offset + 3]]);
        offset += 4;
        if complement != !length {
            return Err("longueur deflate invalide".into());
        }
        let end = offset + usize::from(length);
        if end > data_end {
            return Err("bloc deflate tronqué".into());
        }
        output.extend_from_slice(&bytes[offset..end]);
        offset = end;
        if final_block {
            final_seen = true;
            break;
        }
    }
    if !final_seen || offset != data_end {
        return Err("flux deflate invalide".into());
    }
    let expected = u32::from_be_bytes(bytes[data_end..].try_into().unwrap());
    if adler32(&output) != expected {
        return Err("Adler-32 PNG invalide".into());
    }
    Ok(output)
}

fn token<'a>(bytes: &'a [u8], offset: &mut usize) -> Result<&'a [u8]> {
    loop {
        while *offset < bytes.len() && bytes[*offset].is_ascii_whitespace() {
            *offset += 1;
        }
        if *offset < bytes.len() && bytes[*offset] == b'#' {
            while *offset < bytes.len() && bytes[*offset] != b'\n' {
                *offset += 1;
            }
        } else {
            break;
        }
    }
    let start = *offset;
    while *offset < bytes.len() && !bytes[*offset].is_ascii_whitespace() {
        *offset += 1;
    }
    if start == *offset {
        return Err("en-tête Netpbm incomplet".into());
    }
    Ok(&bytes[start..*offset])
}

fn parse_u32(bytes: &[u8], name: &str) -> Result<u32> {
    std::str::from_utf8(bytes)
        .ok()
        .and_then(|value| value.parse().ok())
        .ok_or_else(|| format!("{name} Netpbm invalide").into())
}

fn validate_dimensions(width: u32, height: u32) -> Result<()> {
    let pixels = u64::from(width)
        .checked_mul(u64::from(height))
        .ok_or("dimensions hors limite")?;
    if width == 0 || height == 0 || pixels > MAX_PIXELS {
        return Err("dimensions d'image invalides".into());
    }
    Ok(())
}

fn checked_sample_count(width: u32, height: u32, channels: u8) -> Result<usize> {
    usize::try_from(
        u64::from(width)
            .checked_mul(u64::from(height))
            .and_then(|value| value.checked_mul(u64::from(channels)))
            .ok_or("image trop grande")?,
    )
    .map_err(|_| "image trop grande".into())
}

fn normalized(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn write_atomic(path: &Path, bytes: &[u8]) -> Result<()> {
    let parent = path
        .parent()
        .filter(|value| !value.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent)
        .map_err(|error| format!("création de {}: {error}", parent.display()))?;
    let name = path
        .file_name()
        .ok_or("nom de rapport invalide")?
        .to_string_lossy();
    let temporary: PathBuf = parent.join(format!(".{name}.aetherion-tmp-{}", std::process::id()));
    fs::write(&temporary, bytes)
        .map_err(|error| format!("écriture temporaire de {}: {error}", path.display()))?;
    if path.exists() {
        fs::remove_file(path)
            .map_err(|error| format!("remplacement de {}: {error}", path.display()))?;
    }
    if let Err(error) = fs::rename(&temporary, path) {
        let _ = fs::remove_file(&temporary);
        return Err(format!("publication atomique de {}: {error}", path.display()).into());
    }
    Ok(())
}

fn crc32(bytes: &[u8]) -> u32 {
    let mut crc = !0u32;
    for &byte in bytes {
        crc ^= u32::from(byte);
        for _ in 0..8 {
            crc = (crc >> 1) ^ (0xedb8_8320 & (0u32.wrapping_sub(crc & 1)));
        }
    }
    !crc
}

fn adler32(bytes: &[u8]) -> u32 {
    let (mut a, mut b) = (1u32, 0u32);
    for &byte in bytes {
        a = (a + u32::from(byte)) % 65_521;
        b = (b + a) % 65_521;
    }
    (b << 16) | a
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capture::encode_png;
    use crate::render::Image as RenderImage;

    fn rgb(width: u32, height: u32, samples: Vec<u32>) -> Image {
        Image {
            width,
            height,
            encoding: "ppm-p6-rgb8",
            channels: 3,
            bit_depth: 8,
            samples,
        }
    }

    fn compare(left: Image, right: Image, tolerances: Tolerances) -> Result<VisualDiffReport> {
        compare_images(left, right, "left".into(), "right".into(), tolerances)
    }

    #[test]
    fn exact_match_passes() {
        let report = compare(
            rgb(1, 1, vec![1, 2, 3]),
            rgb(1, 1, vec![1, 2, 3]),
            Tolerances::default(),
        )
        .unwrap();
        assert!(report.passed);
        assert_eq!(report.different_pixels, 0);
    }

    #[test]
    fn channel_delta_is_inclusive() {
        let left = rgb(1, 1, vec![10, 20, 30]);
        let right = rgb(1, 1, vec![12, 20, 30]);
        assert!(!compare(left, right, Tolerances::default()).unwrap().passed);
        assert!(
            compare(
                rgb(1, 1, vec![10, 20, 30]),
                rgb(1, 1, vec![12, 20, 30]),
                Tolerances {
                    max_channel_delta: 2,
                    ..Tolerances::default()
                }
            )
            .unwrap()
            .passed
        );
    }

    #[test]
    fn pixel_and_percentage_thresholds_are_both_required() {
        let left = rgb(2, 1, vec![0; 6]);
        let right = rgb(2, 1, vec![1, 0, 0, 0, 0, 0]);
        assert!(
            !compare(
                left,
                right,
                Tolerances {
                    max_different_pixels: 1,
                    max_different_percent_milli: 49_999,
                    ..Tolerances::default()
                }
            )
            .unwrap()
            .passed
        );
        assert!(
            compare(
                rgb(2, 1, vec![0; 6]),
                rgb(2, 1, vec![1, 0, 0, 0, 0, 0]),
                Tolerances {
                    max_different_pixels: 1,
                    max_different_percent_milli: 50_000,
                    ..Tolerances::default()
                }
            )
            .unwrap()
            .passed
        );
    }

    #[test]
    fn dimension_mismatch_is_invalid() {
        assert!(
            compare(
                rgb(1, 1, vec![0; 3]),
                rgb(2, 1, vec![0; 6]),
                Tolerances::default()
            )
            .is_err()
        );
    }

    #[test]
    fn decodes_ppm_pgm_and_internal_png() {
        let ppm = decode_netpbm(b"P6\n1 1\n255\n\x01\x02\x03", b"P6").unwrap();
        assert_eq!(ppm.samples, vec![1, 2, 3]);
        let pgm = decode_netpbm(b"P5\n1 1\n65535\n\x12\x34", b"P5").unwrap();
        assert_eq!(pgm.samples, vec![0x1234]);
        let png = encode_png(&RenderImage {
            width: 1,
            height: 1,
            pixels: vec![4, 5, 6],
        })
        .unwrap();
        assert_eq!(decode_png(&png).unwrap().samples, vec![4, 5, 6]);
    }

    #[test]
    fn depth_u16_delta_and_segmentation_exact_are_supported() {
        let depth = |value| Image {
            width: 1,
            height: 1,
            encoding: "pgm-p5-u16be",
            channels: 1,
            bit_depth: 16,
            samples: vec![value],
        };
        let report = compare(
            depth(1000),
            depth(1007),
            Tolerances {
                max_channel_delta: 7,
                ..Tolerances::default()
            },
        )
        .unwrap();
        assert!(report.passed);
        assert_eq!(report.maximum_observed_channel_delta, 7);
        let segmentation = compare(
            rgb(1, 1, vec![0, 0, 1]),
            rgb(1, 1, vec![0, 0, 2]),
            Tolerances::default(),
        )
        .unwrap();
        assert!(!segmentation.passed);
    }

    #[test]
    fn first_differences_are_row_major_and_bounded() {
        let count = MAX_REPORTED_DIFFERENCES + 5;
        let report = compare(
            rgb(count as u32, 1, vec![0; count * 3]),
            rgb(count as u32, 1, vec![1; count * 3]),
            Tolerances::default(),
        )
        .unwrap();
        assert_eq!(report.first_differences.len(), MAX_REPORTED_DIFFERENCES);
        assert_eq!(report.first_differences[0].x, 0);
        assert_eq!(report.first_differences[99].x, 99);
    }
}
