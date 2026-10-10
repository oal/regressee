// SPDX-License-Identifier: MPL-2.0
//! Inspect RAW central-directory entries before zip 2.4's name deduplication.
use anyhow::{Context, Result, ensure};
use std::{
    collections::HashSet,
    io::{Read, Seek, SeekFrom},
};

// Only inert timestamp/Unix metadata fields are supported. In particular do not
// let zip's Unicode Path, ZIP64 or AES interpretation change names, sizes,
// offsets or methods after the raw guard has validated them. Check BOTH headers.
fn extra_fields(bytes: &[u8]) -> Result<()> {
    let mut pos = 0;
    let mut seen = HashSet::new();
    while pos < bytes.len() {
        ensure!(bytes.len() - pos >= 4, "truncated ZIP extra-field header");
        let kind = u16::from_le_bytes([bytes[pos], bytes[pos + 1]]);
        let len = u16::from_le_bytes([bytes[pos + 2], bytes[pos + 3]]) as usize;
        ensure!(
            matches!(kind, 0x000a | 0x000d | 0x5455 | 0x5855 | 0x7855 | 0x7875)
                && seen.insert(kind),
            "unsupported or duplicate ZIP extra field"
        );
        pos += 4;
        ensure!(
            len <= bytes.len() - pos,
            "truncated ZIP extra-field payload"
        );
        pos += len;
    }
    Ok(())
}

pub fn validate(
    mut f: impl Read + Seek,
    max_entries: u16,
    safe_name: impl Fn(&str) -> bool,
) -> Result<()> {
    let size = f.seek(SeekFrom::End(0))?;
    let len = size.min(65557) as usize;
    f.seek(SeekFrom::End(-(len as i64)))?;
    let mut tail = vec![0; len];
    f.read_exact(&mut tail)?;
    let end = (0..len.saturating_sub(21))
        .rev()
        .find(|&i| {
            tail[i..i + 4] == *b"PK\x05\x06"
                && i + 22 + u16::from_le_bytes([tail[i + 20], tail[i + 21]]) as usize == len
        })
        .context("invalid ZIP end directory")?;
    let u16at = |i| u16::from_le_bytes([tail[end + i], tail[end + i + 1]]);
    let u32at = |i| u32::from_le_bytes(tail[end + i..end + i + 4].try_into().unwrap());
    ensure!(
        u16at(4) == 0 && u16at(6) == 0 && u16at(8) == u16at(10) && u16at(10) <= max_entries,
        "unsupported ZIP directory/count (ZIP64 and multi-disk unsupported)"
    );
    let (length, offset) = (u32at(12) as u64, u32at(16) as u64);
    ensure!(
        offset + length == size - len as u64 + end as u64,
        "invalid ZIP bounds"
    );
    f.seek(SeekFrom::Start(offset))?;
    let mut seen = HashSet::new();
    let mut local_ranges = Vec::new();
    for _ in 0..u16at(10) {
        let mut h = [0; 46];
        f.read_exact(&mut h)?;
        ensure!(&h[..4] == b"PK\x01\x02", "invalid ZIP entry");
        let field = |i| u16::from_le_bytes([h[i], h[i + 1]]);
        let mut name_bytes = vec![0; field(28) as usize];
        f.read_exact(&mut name_bytes)?;
        let name = std::str::from_utf8(&name_bytes)?;
        ensure!(
            (name.is_ascii() || field(8) & 0x0800 != 0)
                && safe_name(name)
                && seen.insert(name.to_lowercase()),
            "duplicate, case-colliding or unsafe ZIP entry"
        );
        ensure!(
            f.stream_position()? + u64::from(field(30)) + u64::from(field(32)) <= offset + length,
            "invalid ZIP extra/comment bounds"
        );
        let mut extra = vec![0; field(30) as usize];
        f.read_exact(&mut extra)?;
        extra_fields(&extra)?;
        f.seek(SeekFrom::Current(i64::from(field(32))))?;
        let next = f.stream_position()?;
        ensure!(next <= offset + length, "invalid ZIP directory length");
        let word = |i| u32::from_le_bytes(h[i..i + 4].try_into().unwrap());
        let local = word(42) as u64;
        let compressed = word(20);
        let expanded = word(24);
        ensure!(
            compressed != u32::MAX
                && expanded != u32::MAX
                && local < offset
                && field(34) == 0
                && field(8) & (1 | 64 | 8192) == 0,
            "Unsupported ZIP64/encrypted/multi-disk entry"
        );
        f.seek(SeekFrom::Start(local))?;
        let mut header = [0; 30];
        f.read_exact(&mut header)?;
        let local_field = |i| u16::from_le_bytes([header[i], header[i + 1]]);
        ensure!(
            &header[..4] == b"PK\x03\x04"
                && local_field(6) == field(8)
                && local_field(8) == field(10)
                && local_field(26) == field(28),
            "ZIP local/central header mismatch"
        );
        let mut local_name = vec![0; local_field(26) as usize];
        f.read_exact(&mut local_name)?;
        ensure!(local_name == name_bytes, "ZIP local/central path mismatch");
        let mut data_end = local
            + 30
            + u64::from(local_field(26))
            + u64::from(local_field(28))
            + u64::from(compressed);
        ensure!(data_end <= offset, "ZIP local data exceeds bounds");
        let mut local_extra = vec![0; local_field(28) as usize];
        f.read_exact(&mut local_extra)?;
        extra_fields(&local_extra)?;
        if field(8) & 8 == 0 {
            ensure!(
                header[14..26] == h[16..28],
                "ZIP local/central CRC/size mismatch"
            );
        } else {
            f.seek(SeekFrom::Start(data_end))?;
            let mut first = [0; 4];
            f.read_exact(&mut first)?;
            let mut descriptor = [0; 12];
            if first == *b"PK\x07\x08" {
                f.read_exact(&mut descriptor)?;
                data_end += 16;
            } else {
                descriptor[..4].copy_from_slice(&first);
                f.read_exact(&mut descriptor[4..])?;
                data_end += 12;
            }
            ensure!(descriptor == h[16..28], "ZIP descriptor mismatch");
        }
        local_ranges.push((local, data_end));
        f.seek(SeekFrom::Start(next))?;
    }
    ensure!(
        f.stream_position()? == offset + length,
        "invalid ZIP directory length"
    );
    local_ranges.sort();
    let mut end = 0;
    for (start, next) in local_ranges {
        ensure!(
            start == end && next <= offset,
            "Overlapping/unlisted/prepended ZIP local entries"
        );
        end = next;
    }
    ensure!(end == offset, "Unlisted ZIP local payload");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    struct Entry<'a> {
        name: &'a str,
        flags: u16,
        central_extra: Vec<u8>,
        local_extra: Vec<u8>,
    }
    fn word(out: &mut Vec<u8>, value: u16) {
        out.extend_from_slice(&value.to_le_bytes());
    }
    fn dword(out: &mut Vec<u8>, value: u32) {
        out.extend_from_slice(&value.to_le_bytes());
    }
    // Inert stored, zero-byte entries, including complete local/central records.
    // No ZIP dependency is needed to construct decoder-disagreement fixtures.
    fn archive(entries: &[Entry<'_>]) -> Vec<u8> {
        let mut out = Vec::new();
        let mut central = Vec::new();
        for e in entries {
            let offset = out.len() as u32;
            dword(&mut out, 0x04034b50);
            for v in [20, e.flags, 0, 0, 0] {
                word(&mut out, v);
            }
            for _ in 0..3 {
                dword(&mut out, 0);
            }
            word(&mut out, e.name.len() as u16);
            word(&mut out, e.local_extra.len() as u16);
            out.extend_from_slice(e.name.as_bytes());
            out.extend_from_slice(&e.local_extra);
            dword(&mut central, 0x02014b50);
            for v in [20, 20, e.flags, 0, 0, 0] {
                word(&mut central, v);
            }
            for _ in 0..3 {
                dword(&mut central, 0);
            }
            word(&mut central, e.name.len() as u16);
            word(&mut central, e.central_extra.len() as u16);
            for _ in 0..3 {
                word(&mut central, 0);
            } // comment/disk/attributes
            dword(&mut central, 0);
            dword(&mut central, offset);
            central.extend_from_slice(e.name.as_bytes());
            central.extend_from_slice(&e.central_extra);
        }
        let offset = out.len() as u32;
        let length = central.len() as u32;
        out.extend_from_slice(&central);
        dword(&mut out, 0x06054b50);
        for v in [0, 0, entries.len() as u16, entries.len() as u16] {
            word(&mut out, v);
        }
        dword(&mut out, length);
        dword(&mut out, offset);
        word(&mut out, 0);
        out
    }
    fn crc32(bytes: &[u8]) -> u32 {
        let mut crc = !0u32;
        for b in bytes {
            crc ^= *b as u32;
            for _ in 0..8 {
                crc = (crc >> 1) ^ (0xedb88320 & 0u32.wrapping_sub(crc & 1));
            }
        }
        !crc
    }
    fn unicode_path(raw: &str, effective: &str) -> Vec<u8> {
        let mut out = Vec::new();
        word(&mut out, 0x7075);
        word(&mut out, (5 + effective.len()) as u16);
        out.push(1);
        dword(&mut out, crc32(raw.as_bytes()));
        out.extend_from_slice(effective.as_bytes());
        out
    }
    fn entry(name: &str) -> Entry<'_> {
        Entry {
            name,
            flags: 0,
            central_extra: Vec::new(),
            local_extra: Vec::new(),
        }
    }
    #[test]
    fn unicode_path_effective_collision_is_refused_before_decoder_dedup() -> Result<()> {
        let effective = "catalog-file";
        let mut entries = [entry("raw-one"), entry("raw-two")];
        assert_ne!(entries[0].name, entries[1].name);
        for e in &mut entries {
            e.central_extra = unicode_path(e.name, effective);
            e.local_extra = e.central_extra.clone();
            // Each valid version-1 extra uses the correct raw-name CRC and the
            // SAME effective name: zip 2.4 would deduplicate these raw entries.
            assert_eq!(&e.central_extra[9..], effective.as_bytes());
        }
        assert!(validate(Cursor::new(archive(&entries)), 10, |_| true).is_err());
        for e in &mut entries {
            e.central_extra.clear();
            e.local_extra.clear();
        }
        validate(Cursor::new(archive(&entries)), 10, |_| true)?;
        // A local-only name-changing field is unsupported too.
        entries[0].local_extra = unicode_path(entries[0].name, effective);
        assert!(validate(Cursor::new(archive(&entries)), 10, |_| true).is_err());
        Ok(())
    }
    #[test]
    fn non_ascii_names_require_utf8_flag_and_extra_fields_are_strict() -> Result<()> {
        let mut e = entry("é.txt");
        assert!(validate(Cursor::new(archive(&[e])), 10, |_| true).is_err());
        e = entry("é.txt");
        e.flags = 0x0800;
        validate(Cursor::new(archive(&[e])), 10, |_| true)?;
        let mut e = entry("ascii");
        // Harmless extended timestamp, no timestamps supplied (flag byte zero).
        e.central_extra = vec![0x55, 0x54, 1, 0, 0];
        e.local_extra = e.central_extra.clone();
        validate(Cursor::new(archive(&[e])), 10, |_| true)?;
        for extra in [
            vec![0x75, 0x70, 0, 0],
            vec![1, 0, 0, 0],
            vec![0x55, 0x54, 2, 0, 0],
            vec![0x55, 0x54, 0],
            vec![0x55, 0x54, 0, 0, 0x55, 0x54, 0, 0],
        ] {
            let mut e = entry("ascii");
            e.central_extra = extra;
            assert!(validate(Cursor::new(archive(&[e])), 10, |_| true).is_err());
        }
        Ok(())
    }
    #[test]
    fn existing_name_and_physical_range_checks_remain_enforced() -> Result<()> {
        assert!(
            validate(
                Cursor::new(archive(&[entry("same"), entry("same")])),
                10,
                |_| true
            )
            .is_err()
        );
        assert!(
            validate(
                Cursor::new(archive(&[entry("Same"), entry("same")])),
                10,
                |_| true
            )
            .is_err()
        );
        let mut bytes = archive(&[entry("name")]);
        bytes[30] = b'N'; // Local name no longer equals the central name.
        assert!(validate(Cursor::new(bytes), 10, |_| true).is_err());
        let mut bytes = archive(&[entry("name")]);
        let end = bytes.len() - 22;
        let central = u32::from_le_bytes(bytes[end + 16..end + 20].try_into().unwrap()) as usize;
        bytes[central + 42..central + 46].copy_from_slice(&1u32.to_le_bytes());
        assert!(validate(Cursor::new(bytes), 10, |_| true).is_err());
        Ok(())
    }
}
