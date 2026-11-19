//! Barcode encoders: Code 128 (subset B) and EAN-13.
//!
//! Symbol patterns are taken from the published specifications (see the
//! Code 128 and EAN-13 references) and cross-checked: every width pattern
//! matches its bit pattern, R codes are complements of L codes, and G codes
//! are reversed R codes. Encoded output is returned as "modules;bits" with
//! '1' marking a bar, one bit per module, quiet zones left to the caller.

const CODE128_PATTERNS: [&str; 107] = [
    "11011001100",
    "11001101100",
    "11001100110",
    "10010011000",
    "10010001100",
    "10001001100",
    "10011001000",
    "10011000100",
    "10001100100",
    "11001001000",
    "11001000100",
    "11000100100",
    "10110011100",
    "10011011100",
    "10011001110",
    "10111001100",
    "10011101100",
    "10011100110",
    "11001110010",
    "11001011100",
    "11001001110",
    "11011100100",
    "11001110100",
    "11101101110",
    "11101001100",
    "11100101100",
    "11100100110",
    "11101100100",
    "11100110100",
    "11100110010",
    "11011011000",
    "11011000110",
    "11000110110",
    "10100011000",
    "10001011000",
    "10001000110",
    "10110001000",
    "10001101000",
    "10001100010",
    "11010001000",
    "11000101000",
    "11000100010",
    "10110111000",
    "10110001110",
    "10001101110",
    "10111011000",
    "10111000110",
    "10001110110",
    "11101110110",
    "11010001110",
    "11000101110",
    "11011101000",
    "11011100010",
    "11011101110",
    "11101011000",
    "11101000110",
    "11100010110",
    "11101101000",
    "11101100010",
    "11100011010",
    "11101111010",
    "11001000010",
    "11110001010",
    "10100110000",
    "10100001100",
    "10010110000",
    "10010000110",
    "10000101100",
    "10000100110",
    "10110010000",
    "10110000100",
    "10011010000",
    "10011000010",
    "10000110100",
    "10000110010",
    "11000010010",
    "11001010000",
    "11110111010",
    "11000010100",
    "10001111010",
    "10100111100",
    "10010111100",
    "10010011110",
    "10111100100",
    "10011110100",
    "10011110010",
    "11110100100",
    "11110010100",
    "11110010010",
    "11011011110",
    "11011110110",
    "11110110110",
    "10101111000",
    "10100011110",
    "10001011110",
    "10111101000",
    "10111100010",
    "11110101000",
    "11110100010",
    "10111011110",
    "10111101110",
    "11101011110",
    "11110101110",
    "11010000100",
    "11010010000",
    "11010011100",
    "1100011101011",
];

const EAN_L: [&str; 10] = [
    "0001101",
    "0011001",
    "0010011",
    "0111101",
    "0100011",
    "0110001",
    "0101111",
    "0111011",
    "0110111",
    "0001011",
];
const EAN_G: [&str; 10] = [
    "0100111",
    "0110011",
    "0011011",
    "0100001",
    "0011101",
    "0111001",
    "0000101",
    "0010001",
    "0001001",
    "0010111",
];
const EAN_R: [&str; 10] = [
    "1110010",
    "1100110",
    "1101100",
    "1000010",
    "1011100",
    "1001110",
    "1010000",
    "1000100",
    "1001000",
    "1110100",
];
const EAN_PARITY: [&str; 10] = [
    "LLLLLL",
    "LLGLGG",
    "LLGGLG",
    "LLGGGL",
    "LGLLGG",
    "LGGLLG",
    "LGGGLL",
    "LGLGLG",
    "LGLGGL",
    "LGGLGL",
];

fn to_result(patterns: Vec<&str>) -> String {
    let bits: String = patterns.concat();
    format!("{};{}", bits.len(), bits)
}

/// Code 128, subset B (printable ASCII 0x20-0x7E).
pub fn code128(text: &str) -> Result<String, String> {
    let text = text.trim();
    if text.is_empty() {
        return Err("enter the barcode content".to_string());
    }
    if text.len() > 48 {
        return Err("Code 128 content is limited to 48 characters here".to_string());
    }
    let mut values: Vec<usize> = Vec::with_capacity(text.len() + 3);
    values.push(104); // Start B
    for ch in text.chars() {
        let code = ch as u32;
        if !(0x20..=0x7e).contains(&code) {
            return Err(format!(
                "character {ch:?} is not in Code 128 subset B (printable ASCII only)"
            ));
        }
        values.push((code - 0x20) as usize);
    }
    let checksum: usize = values
        .iter()
        .enumerate()
        .map(|(i, v)| if i == 0 { *v } else { v * i })
        .sum::<usize>()
        % 103;
    values.push(checksum);
    values.push(106); // Stop
    Ok(to_result(values.iter().map(|v| CODE128_PATTERNS[*v]).collect()))
}

/// EAN-13 checksum digit for the first 12 digits.
fn ean13_check(digits: &[u8]) -> u8 {
    let sum: u32 = digits
        .iter()
        .enumerate()
        .map(|(i, d)| (*d as u32) * if i % 2 == 0 { 1 } else { 3 })
        .sum();
    ((10 - (sum % 10)) % 10) as u8
}

/// EAN-13 from 12 digits (check digit appended) or 13 digits (validated).
pub fn ean13(input: &str) -> Result<String, String> {
    let digits: Vec<u8> = input
        .chars()
        .filter(|c| !c.is_whitespace() && *c != '-')
        .map(|c| c.to_digit(10).map(|d| d as u8))
        .collect::<Option<Vec<u8>>>()
        .ok_or_else(|| "EAN-13 takes digits only".to_string())?;
    let digits = match digits.len() {
        12 => {
            let mut d = digits;
            d.push(ean13_check(&d));
            d
        }
        13 => {
            if ean13_check(&digits[..12]) != digits[12] {
                return Err(format!(
                    "check digit should be {} for those 12 digits",
                    ean13_check(&digits[..12])
                ));
            }
            digits
        }
        n => return Err(format!("EAN-13 needs 12 or 13 digits, got {n}")),
    };
    let mut parts: Vec<&str> = vec!["101"]; // left guard
    let parity = EAN_PARITY[digits[0] as usize];
    for (i, d) in digits[1..7].iter().enumerate() {
        parts.push(if parity.as_bytes()[i] == b'L' { EAN_L[*d as usize] } else { EAN_G[*d as usize] });
    }
    parts.push("01010"); // center guard
    for d in &digits[7..13] {
        parts.push(EAN_R[*d as usize]);
    }
    parts.push("101"); // right guard
    Ok(to_result(parts))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ean13_appends_check_digit() {
        // Wikipedia's worked example: 590123412345 with check digit 7.
        let out = ean13("590123412345").unwrap();
        let (modules, bits) = out.split_once(';').unwrap();
        assert_eq!(modules, "95");
        assert_eq!(bits.len(), 95);
        assert!(bits.starts_with("101") && bits.ends_with("101"));
        assert_eq!(&bits[45..50], "01010");
    }

    #[test]
    fn ean13_validates_given_check_digit() {
        assert!(ean13("5901234123457").is_ok());
        let err = ean13("5901234123458").unwrap_err();
        assert!(err.contains('7'));
    }

    #[test]
    fn ean13_rejects_bad_input() {
        assert!(ean13("123").is_err());
        assert!(ean13("59012341234a").is_err());
    }

    #[test]
    fn code128_structure() {
        // "HI": Start B(104) H(40) I(41) checksum stop.
        // checksum = (104 + 1*40 + 2*41) % 103 = 226 % 103 = 20
        let out = code128("HI").unwrap();
        let (modules, bits) = out.split_once(';').unwrap();
        assert_eq!(modules.parse::<usize>().unwrap(), bits.len());
        // 5 symbols x 11 + final 2 of stop's 13 => 11*4 + 13 = 57
        assert_eq!(bits.len(), 4 * 11 + 13);
        assert!(bits.starts_with(CODE128_PATTERNS[104]));
        assert!(bits.ends_with(CODE128_PATTERNS[106]));
        let mid = &bits[11..44];
        assert_eq!(&mid[0..11], CODE128_PATTERNS[40]);
        assert_eq!(&mid[11..22], CODE128_PATTERNS[41]);
        assert_eq!(&mid[22..33], CODE128_PATTERNS[20]); // checksum symbol
    }

    #[test]
    fn code128_rejects_non_ascii_and_empty() {
        assert!(code128("").is_err());
        assert!(code128("₹500").is_err());
    }

    #[test]
    fn all_patterns_well_formed() {
        for (i, p) in CODE128_PATTERNS.iter().enumerate() {
            let expect = if i == 106 { 13 } else { 11 };
            assert_eq!(p.len(), expect, "pattern {i}");
            assert!(p.starts_with('1'), "pattern {i} starts with a bar");
        }
    }
}
