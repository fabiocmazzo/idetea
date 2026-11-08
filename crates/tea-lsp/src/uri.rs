//! `file://` URIs ↔ caminhos (com percent-encoding de espaços, acentos etc.).

use std::path::{Path, PathBuf};

pub fn path_to_uri(path: &Path) -> String {
    let mut s = String::from("file://");
    let text = path.to_string_lossy();
    #[cfg(windows)]
    let text = {
        let t = text.replace('\\', "/");
        if t.starts_with('/') { t } else { format!("/{t}") }
    };
    for b in text.bytes() {
        let keep = b.is_ascii_alphanumeric() || b"/-_.~:".contains(&b);
        if keep {
            s.push(b as char);
        } else {
            s.push_str(&format!("%{b:02X}"));
        }
    }
    s
}

pub fn uri_to_path(uri: &str) -> Option<PathBuf> {
    let rest = uri.strip_prefix("file://")?;
    // `file://host/caminho`: só aceitamos host vazio ou localhost.
    let rest = rest.strip_prefix("localhost").unwrap_or(rest);
    let bytes = rest.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            let hex = std::str::from_utf8(&bytes[i + 1..i + 3]).ok()?;
            out.push(u8::from_str_radix(hex, 16).ok()?);
            i += 3;
        } else {
            out.push(bytes[i]);
            i += 1;
        }
    }
    let s = String::from_utf8(out).ok()?;
    #[cfg(windows)]
    let s = {
        let s = if s.starts_with('/') && s.as_bytes().get(2) == Some(&b':') { &s[1..] } else { &s };
        let mut s = s.replace('/', "\\");
        if s.as_bytes().get(1) == Some(&b':') && s.as_bytes()[0].is_ascii_lowercase() {
            s.replace_range(0..1, &s[0..1].to_ascii_uppercase());
        }
        s
    };
    Some(PathBuf::from(s))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_with_spaces_and_accents() {
        let p = Path::new("/home/fábio/meu projeto/a+b.rs");
        let uri = path_to_uri(p);
        assert_eq!(uri, "file:///home/f%C3%A1bio/meu%20projeto/a%2Bb.rs");
        assert_eq!(uri_to_path(&uri).unwrap(), p);
    }

    #[test]
    fn decodes_server_style_uris() {
        assert_eq!(uri_to_path("file:///tmp/x%3A1.rs").unwrap(), Path::new("/tmp/x:1.rs"));
        assert_eq!(uri_to_path("file://localhost/tmp/a.rs").unwrap(), Path::new("/tmp/a.rs"));
        assert!(uri_to_path("https://x").is_none());
        assert!(uri_to_path("file:///bad%zz").is_none());
    }
}
