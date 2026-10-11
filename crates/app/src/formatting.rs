// SPDX-License-Identifier: MPL-2.0
//! Shared human-readable formatting for product messages and UI summaries.

/// Binary size units, with exact bytes below 1 KiB and one decimal thereafter.
pub fn human_bytes(bytes: u64) -> String {
    let units = ["B", "KiB", "MiB", "GiB", "TiB", "PiB", "EiB"];
    let mut value = bytes as f64;
    let mut unit = 0;
    while value >= 1024. && unit + 1 < units.len() {
        value /= 1024.;
        unit += 1;
    }
    if unit == 0 {
        format!("{bytes} B")
    } else {
        format!("{value:.1} {}", units[unit])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn human_size_units_are_bounded_and_unambiguous() {
        for (bytes, expected) in [
            (0, "0 B"),
            (1, "1 B"),
            (1023, "1023 B"),
            (1024, "1.0 KiB"),
            (1536, "1.5 KiB"),
            (1024_u64.pow(2), "1.0 MiB"),
            (1024_u64.pow(3), "1.0 GiB"),
            (1024_u64.pow(4), "1.0 TiB"),
            (1024_u64.pow(5), "1.0 PiB"),
            (1024_u64.pow(6), "1.0 EiB"),
            (82_350_444, "78.5 MiB"),
            (258_433_266, "246.5 MiB"),
            (u64::MAX, "16.0 EiB"),
        ] {
            assert_eq!(human_bytes(bytes), expected);
        }
    }
}
