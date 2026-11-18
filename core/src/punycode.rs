//! RFC 3492 Punycode ve IDNA etiket dönüşümleri.

const BASE: u32 = 36;
const T_MIN: u32 = 1;
const T_MAX: u32 = 26;
const SKEW: u32 = 38;
const DAMP: u32 = 700;
const INITIAL_BIAS: u32 = 72;
const INITIAL_N: u32 = 128;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PunycodeError;

fn adapt(mut delta: u32, num_points: u32, first: bool) -> u32 {
    delta = if first { delta / DAMP } else { delta / 2 };
    delta += delta / num_points;
    let mut k = 0;
    while delta > ((BASE - T_MIN) * T_MAX) / 2 {
        delta /= BASE - T_MIN;
        k += BASE;
    }
    k + ((BASE - T_MIN + 1) * delta) / (delta + SKEW)
}

fn threshold(k: u32, bias: u32) -> u32 {
    if k <= bias {
        T_MIN
    } else if k >= bias + T_MAX {
        T_MAX
    } else {
        k - bias
    }
}

fn decode_digit(c: u8) -> Option<u32> {
    match c {
        b'0'..=b'9' => Some((c - b'0') as u32 + 26),
        b'a'..=b'z' => Some((c - b'a') as u32),
        b'A'..=b'Z' => Some((c - b'A') as u32),
        _ => None,
    }
}

fn encode_digit(d: u32) -> char {
    (if d < 26 { b'a' + d as u8 } else { b'0' + (d - 26) as u8 }) as char
}

/// `"pple-43d"` → `"аpple"`
pub fn decode(input: &str) -> Result<String, PunycodeError> {
    let bytes = input.as_bytes();
    let mut out: Vec<char> = Vec::new();
    let mut pos = 0;
    if let Some(b) = input.rfind('-') {
        for &c in &bytes[..b] {
            if c >= 0x80 {
                return Err(PunycodeError);
            }
            out.push(c as char);
        }
        pos = b + 1;
    }
    let (mut n, mut i, mut bias) = (INITIAL_N, 0u32, INITIAL_BIAS);
    while pos < bytes.len() {
        let old_i = i;
        let mut w = 1u32;
        let mut k = BASE;
        loop {
            let digit = decode_digit(*bytes.get(pos).ok_or(PunycodeError)?).ok_or(PunycodeError)?;
            pos += 1;
            i = i
                .checked_add(digit.checked_mul(w).ok_or(PunycodeError)?)
                .ok_or(PunycodeError)?;
            let t = threshold(k, bias);
            if digit < t {
                break;
            }
            w = w.checked_mul(BASE - t).ok_or(PunycodeError)?;
            k += BASE;
        }
        let len = out.len() as u32 + 1;
        bias = adapt(i - old_i, len, old_i == 0);
        n = n.checked_add(i / len).ok_or(PunycodeError)?;
        i %= len;
        let ch = char::from_u32(n).ok_or(PunycodeError)?;
        out.insert(i as usize, ch);
        i += 1;
    }
    Ok(out.into_iter().collect())
}

/// `"аpple"` → `"pple-43d"`
pub fn encode(input: &str) -> String {
    let cps: Vec<u32> = input.chars().map(|c| c as u32).collect();
    let mut out: String = input.chars().filter(|c| c.is_ascii()).collect();
    let b = out.len() as u32;
    let mut h = b;
    if b > 0 {
        out.push('-');
    }
    let (mut n, mut delta, mut bias) = (INITIAL_N, 0u32, INITIAL_BIAS);
    while (h as usize) < cps.len() {
        let m = *cps.iter().filter(|&&c| c >= n).min().unwrap();
        delta = delta.saturating_add((m - n).saturating_mul(h + 1));
        n = m;
        for &c in &cps {
            if c < n {
                delta += 1;
            }
            if c == n {
                let mut q = delta;
                let mut k = BASE;
                loop {
                    let t = threshold(k, bias);
                    if q < t {
                        break;
                    }
                    out.push(encode_digit(t + (q - t) % (BASE - t)));
                    q = (q - t) / (BASE - t);
                    k += BASE;
                }
                out.push(encode_digit(q));
                bias = adapt(delta, h + 1, h == b);
                delta = 0;
                h += 1;
            }
        }
        delta += 1;
        n += 1;
    }
    out
}

/// `"xn--pple-43d.com"` → `"аpple.com"`
pub fn to_unicode(host: &str) -> Result<String, PunycodeError> {
    let mut labels = Vec::new();
    for l in host.split('.') {
        match l.strip_prefix("xn--") {
            Some(rest) => labels.push(decode(rest)?),
            None => labels.push(l.to_string()),
        }
    }
    Ok(labels.join("."))
}

/// `"аpple.com"` → `"xn--pple-43d.com"`
pub fn to_ascii(host: &str) -> String {
    host.split('.')
        .map(|l| if l.is_ascii() { l.to_string() } else { format!("xn--{}", encode(l)) })
        .collect::<Vec<_>>()
        .join(".")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rfc_examples() {
        assert_eq!(encode("bücher"), "bcher-kva");
        assert_eq!(decode("bcher-kva").unwrap(), "bücher");
        assert_eq!(encode("аpple"), "pple-43d");
        assert_eq!(to_ascii("аpple.com"), "xn--pple-43d.com");
        assert_eq!(to_unicode("xn--pple-43d.com").unwrap(), "аpple.com");
    }

    #[test]
    fn roundtrip() {
        for s in ["аррӏе", "türkiye", "ışıkçğü", "日本語", "παράδειγμα", "münchen-2"] {
            assert_eq!(decode(&encode(s)).unwrap(), s);
        }
    }

    #[test]
    fn invalid() {
        assert!(decode("!!!").is_err());
        assert!(decode("zzzzzzzzzzzzzzzzzzzz").is_err());
    }
}
