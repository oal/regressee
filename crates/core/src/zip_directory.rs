// SPDX-License-Identifier: MPL-2.0
//! Inspect RAW central-directory entries before zip 2.4's name deduplication.
use anyhow::{Context, Result, ensure};
use std::{
    collections::HashSet,
    io::{Read, Seek, SeekFrom},
};

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
            safe_name(name) && seen.insert(name.to_lowercase()),
            "duplicate, case-colliding or unsafe ZIP entry"
        );
        f.seek(SeekFrom::Current(
            i64::from(field(30)) + i64::from(field(32)),
        ))?;
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
