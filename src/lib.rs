//! QR Forge core: payload builders and QR rendering.
//!
//! Everything runs client-side. The `*_core` functions are pure Rust and
//! unit-tested natively; thin wrappers expose them to JavaScript through
//! WebAssembly.


use wasm_bindgen::prelude::*;

fn ec_level(level: &str) -> Result<qrcode::EcLevel, String> {
    match level.trim().to_ascii_uppercase().as_str() {
        "L" => Ok(qrcode::EcLevel::L),
        "M" => Ok(qrcode::EcLevel::M),
        "Q" => Ok(qrcode::EcLevel::Q),
        "H" => Ok(qrcode::EcLevel::H),
        other => Err(format!("unknown error correction level: {other}")),
    }
}

fn make_qr(payload: &str, level: &str) -> Result<qrcode::QrCode, String> {
    let payload = payload.trim();
    if payload.is_empty() {
        return Err("nothing to encode yet".to_string());
    }
    qrcode::QrCode::with_error_correction_level(payload.as_bytes(), ec_level(level)?)
        .map_err(|e| format!("could not encode QR code: {e}"))
}

/// Minimal percent-encoding for query values. Keeps RFC 3986 unreserved
/// characters plus `@`, which UPI apps expect literally inside `pa`.
fn url_encode(value: &str) -> String {
    let mut out = String::new();
    for byte in value.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' | b'@' => {
                out.push(byte as char)
            }
            _ => out.push_str(&format!("%{byte:02X}")),
        }
    }
    out
}

/// Escape special characters inside a WIFI: payload (WiFi QR spec).
fn wifi_escape(value: &str) -> String {
    let mut out = String::new();
    for ch in value.chars() {
        match ch {
            '\\' | ';' | ',' | ':' | '"' => {
                out.push('\\');
                out.push(ch);
            }
            _ => out.push(ch),
        }
    }
    out
}

/// Plain text / URL payload. Whitespace-trimmed passthrough.
pub fn payload_text_core(text: &str) -> String {
    text.trim().to_string()
}

/// WiFi network payload: WIFI:T:WPA;S:ssid;P:password;H:false;;
/// `security` is one of WPA, WEP or nopass.
pub fn payload_wifi_core(
    ssid: &str,
    password: &str,
    security: &str,
    hidden: bool,
) -> Result<String, String> {
    let ssid = ssid.trim();
    if ssid.is_empty() {
        return Err("enter the network name (SSID)".to_string());
    }
    let security = match security.trim().to_ascii_uppercase().as_str() {
        "WPA" | "WPA2" | "WPA3" => "WPA",
        "WEP" => "WEP",
        "NOPASS" | "NONE" | "" => "nopass",
        other => return Err(format!("unsupported WiFi security: {other}")),
    };
    let mut payload = format!("WIFI:T:{security};S:{}", wifi_escape(ssid));
    if security != "nopass" {
        payload.push_str(&format!(";P:{}", wifi_escape(password.trim())));
    }
    if hidden {
        payload.push_str(";H:true");
    }
    payload.push_str(";;");
    Ok(payload)
}

/// UPI payment payload: upi://pay?pa=vpa&pn=name&am=12.50&cu=INR&tn=note
pub fn payload_upi_core(
    vpa: &str,
    payee_name: &str,
    amount: &str,
    note: &str,
) -> Result<String, String> {
    let vpa = vpa.trim();
    if vpa.is_empty() || !vpa.contains('@') || vpa.starts_with('@') || vpa.ends_with('@') {
        return Err("enter a valid UPI ID like name@bank".to_string());
    }
    let name = payee_name.trim();
    if name.is_empty() {
        return Err("enter the payee name".to_string());
    }
    let mut query = format!("pa={}&pn={}", url_encode(vpa), url_encode(name));
    let amount = amount.trim();
    if !amount.is_empty() {
        let value: f64 = amount
            .parse()
            .map_err(|_| "amount must be a number".to_string())?;
        if !(0.0..=1_000_000.0).contains(&value) || value == 0.0 {
            return Err("amount must be between 0 and 1,000,000".to_string());
        }
        query.push_str(&format!("&am={value:.2}&cu=INR"));
    }
    let note = note.trim();
    if !note.is_empty() {
        query.push_str(&format!("&tn={}", url_encode(note)));
    }
    Ok(format!("upi://pay?{query}"))
}

/// WhatsApp click-to-chat payload: https://wa.me/<digits>?text=<encoded>
pub fn payload_whatsapp_core(phone: &str, message: &str) -> Result<String, String> {
    let digits: String = phone.chars().filter(|c| c.is_ascii_digit()).collect();
    if !(7..=15).contains(&digits.len()) {
        return Err("enter the phone with country code, digits only (7-15)".to_string());
    }
    let mut url = format!("https://wa.me/{digits}");
    let message = message.trim();
    if !message.is_empty() {
        url.push_str(&format!("?text={}", url_encode(message)));
    }
    Ok(url)
}

/// Render as an SVG string (no built-in quiet zone; the page applies padding
/// so the margin slider controls both the preview and the exports).
pub fn render_svg_core(
    payload: &str,
    level: &str,
    dark: &str,
    light: &str,
) -> Result<String, String> {
    let code = make_qr(payload, level)?;
    Ok(code
        .render::<qrcode::render::svg::Color>()
        .min_dimensions(512, 512)
        .dark_color(qrcode::render::svg::Color(dark))
        .light_color(qrcode::render::svg::Color(light))
        .quiet_zone(false)
        .build())
}

/// Render the module matrix as "width;bits" ('1' = dark) for canvas export.
pub fn render_matrix_core(payload: &str, level: &str) -> Result<String, String> {
    let code = make_qr(payload, level)?;
    let width = code.width();
    let mut out = String::with_capacity(width * width + 8);
    out.push_str(&width.to_string());
    out.push(';');
    for color in code.into_colors() {
        out.push(if color == qrcode::Color::Dark { '1' } else { '0' });
    }
    Ok(out)
}


#[wasm_bindgen]
pub fn payload_text(text: &str) -> String {
    payload_text_core(text)
}

#[wasm_bindgen]
pub fn payload_wifi(ssid: &str, password: &str, security: &str, hidden: bool) -> Result<String, JsError> {
    payload_wifi_core(ssid, password, security, hidden).map_err(|e| JsError::new(&e))
}

#[wasm_bindgen]
pub fn payload_upi(vpa: &str, payee_name: &str, amount: &str, note: &str) -> Result<String, JsError> {
    payload_upi_core(vpa, payee_name, amount, note).map_err(|e| JsError::new(&e))
}

#[wasm_bindgen]
pub fn payload_whatsapp(phone: &str, message: &str) -> Result<String, JsError> {
    payload_whatsapp_core(phone, message).map_err(|e| JsError::new(&e))
}

#[wasm_bindgen]
pub fn render_svg(payload: &str, level: &str, dark: &str, light: &str) -> Result<String, JsError> {
    render_svg_core(payload, level, dark, light).map_err(|e| JsError::new(&e))
}

#[wasm_bindgen]
pub fn render_matrix(payload: &str, level: &str) -> Result<String, JsError> {
    render_matrix_core(payload, level).map_err(|e| JsError::new(&e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wifi_payload_escapes_specials() {
        let p = payload_wifi_core("my;net", "p:a\\s", "WPA2", false).unwrap();
        assert_eq!(p, "WIFI:T:WPA;S:my\\;net;P:p\\:a\\\\s;;");
    }

    #[test]
    fn wifi_open_network_has_no_password() {
        let p = payload_wifi_core("cafe", "", "nopass", true).unwrap();
        assert_eq!(p, "WIFI:T:nopass;S:cafe;H:true;;");
    }

    #[test]
    fn wifi_requires_ssid() {
        assert!(payload_wifi_core("  ", "x", "WPA", false).is_err());
    }

    #[test]
    fn upi_payload_builds_and_encodes() {
        let p = payload_upi_core("shop@okhdfcbank", "Nadan Store", "125.5", "Tea & snacks").unwrap();
        assert_eq!(
            p,
            "upi://pay?pa=shop@okhdfcbank&pn=Nadan%20Store&am=125.50&cu=INR&tn=Tea%20%26%20snacks"
        );
    }

    #[test]
    fn upi_rejects_bad_vpa_and_amount() {
        assert!(payload_upi_core("no-at-sign", "A", "", "").is_err());
        assert!(payload_upi_core("a@b", "A", "abc", "").is_err());
        assert!(payload_upi_core("a@b", "A", "-5", "").is_err());
    }

    #[test]
    fn whatsapp_payload_strips_non_digits() {
        let p = payload_whatsapp_core("+91 88483 52895", "hi there").unwrap();
        assert_eq!(p, "https://wa.me/918848352895?text=hi%20there");
    }

    #[test]
    fn whatsapp_rejects_short_numbers() {
        assert!(payload_whatsapp_core("123", "").is_err());
    }

    #[test]
    fn matrix_is_square() {
        let m = render_matrix_core("https://denelli.in", "M").unwrap();
        let (width, bits) = m.split_once(';').unwrap();
        let width: usize = width.parse().unwrap();
        assert_eq!(bits.len(), width * width);
        assert!(bits.chars().all(|c| c == '0' || c == '1'));
    }

    #[test]
    fn svg_renders() {
        let svg = render_svg_core("hello", "Q", "#0b0e14", "#ffffff").unwrap();
        assert!(svg.contains("<svg"));
    }

    #[test]
    fn empty_payload_errors() {
        assert!(render_matrix_core("   ", "M").is_err());
        assert!(render_svg_core("", "M", "#000", "#fff").is_err());
        assert!(ec_level("Z").is_err());
    }

}
