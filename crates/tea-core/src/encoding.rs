//! Ler e gravar texto em outros encodings além de UTF-8 (RFC 0021).
//!
//! Ao abrir, nesta ordem: o BOM; o `charset` do `.editorconfig` (`hint`); UTF-8 válido; bytes nulos (binário,
//! recusado); o encoding de reserva (`[editor] fallback_encoding`, padrão Windows-1252). Ao gravar, o mesmo encoding
//! e o mesmo BOM; um caractere que não cabe é um erro, nunca um `?`.

use std::path::Path;

pub use encoding_rs::Encoding;
use encoding_rs::{UTF_8, UTF_16BE, UTF_16LE, WINDOWS_1252};

/// Bytes lidos para decidir se o arquivo é binário (um byte nulo neles).
const BINARY_SNIFF: usize = 8 * 1024;

/// O encoding de um texto e se ele tem BOM (só UTF-8 e UTF-16).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TextEncoding {
    pub encoding: &'static Encoding,
    pub bom: bool,
}

impl Default for TextEncoding {
    fn default() -> Self {
        Self::UTF8
    }
}

impl TextEncoding {
    pub const UTF8: Self = Self { encoding: UTF_8, bom: false };

    /// O nome na barra de status: `UTF-8`, `UTF-8 BOM`, `windows-1252`, `UTF-16LE`, `Shift_JIS`…
    pub fn label(&self) -> String {
        if self.bom && self.encoding == UTF_8 { "UTF-8 BOM".to_string() } else { self.encoding.name().to_string() }
    }

    pub fn is_utf8(&self) -> bool {
        self.encoding == UTF_8 && !self.bom
    }

    /// Um nome do `.editorconfig` (`latin1`, `utf-8`, `utf-8-bom`, `utf-16le`, `utf-16be`) ou do WHATWG
    /// (`windows-1252`, `shift_jis`, `gbk`…), sem diferenciar maiúsculas. UTF-16 vem com BOM (o que os editores
    /// gravam).
    pub fn from_label(label: &str) -> Option<Self> {
        let l = label.trim().to_ascii_lowercase();
        match l.as_str() {
            "utf-8-bom" | "utf8bom" | "utf-8 bom" => Some(Self { encoding: UTF_8, bom: true }),
            "utf-16le" | "utf16le" => Some(Self { encoding: UTF_16LE, bom: true }),
            "utf-16be" | "utf16be" => Some(Self { encoding: UTF_16BE, bom: true }),
            _ => {
                let encoding = Encoding::for_label(l.as_bytes())?;
                // `replacement` e `x-user-defined` não são encodings de arquivo.
                if encoding == encoding_rs::REPLACEMENT || encoding == encoding_rs::X_USER_DEFINED {
                    return None;
                }
                Some(Self { encoding, bom: false })
            }
        }
    }

    /// Os que se pode escolher em "Reabrir com…" e "Salvar com…", os mais comuns primeiro.
    pub fn choices() -> Vec<Self> {
        let mut out = vec![Self::UTF8, Self { encoding: UTF_8, bom: true }, Self { encoding: UTF_16LE, bom: true }, Self {
            encoding: UTF_16BE,
            bom: true,
        }];
        for e in [
            WINDOWS_1252,
            encoding_rs::ISO_8859_15,
            encoding_rs::ISO_8859_2,
            encoding_rs::WINDOWS_1250,
            encoding_rs::WINDOWS_1251,
            encoding_rs::KOI8_R,
            encoding_rs::WINDOWS_1253,
            encoding_rs::WINDOWS_1254,
            encoding_rs::WINDOWS_1255,
            encoding_rs::WINDOWS_1256,
            encoding_rs::WINDOWS_1257,
            encoding_rs::WINDOWS_1258,
            encoding_rs::WINDOWS_874,
            encoding_rs::SHIFT_JIS,
            encoding_rs::EUC_JP,
            encoding_rs::ISO_2022_JP,
            encoding_rs::GBK,
            encoding_rs::GB18030,
            encoding_rs::BIG5,
            encoding_rs::EUC_KR,
        ] {
            out.push(Self { encoding: e, bom: false });
        }
        out
    }
}

/// Por que os bytes não viraram texto.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecodeError {
    /// Tem bytes nulos e não tem BOM de UTF-16: é binário.
    Binary,
    /// Não é válido no encoding pedido (ou no de reserva).
    Invalid(&'static str),
}

/// Lê os bytes de um arquivo (ver o começo do módulo). `hint`: o `charset` do `.editorconfig`, que só vale se os bytes
/// forem válidos nele. O texto volta sem o BOM.
pub fn decode(bytes: &[u8], hint: Option<TextEncoding>, fallback: &'static Encoding) -> Result<(String, TextEncoding), DecodeError> {
    if let Some((encoding, len)) = Encoding::for_bom(bytes) {
        let enc = TextEncoding { encoding, bom: true };
        return decode_as(&bytes[len..], enc).map(|t| (t, enc));
    }
    if let Some(enc) = hint
        && let Ok(text) = decode_as(bytes, TextEncoding { bom: false, ..enc })
    {
        return Ok((text, TextEncoding { bom: false, ..enc }));
    }
    if let Ok(text) = std::str::from_utf8(bytes) {
        return Ok((text.to_string(), TextEncoding::UTF8));
    }
    if bytes[..bytes.len().min(BINARY_SNIFF)].contains(&0) {
        return Err(DecodeError::Binary);
    }
    let enc = TextEncoding { encoding: fallback, bom: false };
    decode_as(bytes, enc).map(|t| (t, enc))
}

/// Os bytes (sem BOM) num encoding escolhido, sem trocar nada por `�`: inválido é erro (salvar depois corromperia).
pub fn decode_as(bytes: &[u8], enc: TextEncoding) -> Result<String, DecodeError> {
    // Um BOM do próprio encoding no começo (reabrir com…) não entra no texto.
    let bytes = match Encoding::for_bom(bytes) {
        Some((e, len)) if e == enc.encoding => &bytes[len..],
        _ => bytes,
    };
    enc.encoding
        .decode_without_bom_handling_and_without_replacement(bytes)
        .map(|t| t.into_owned())
        .ok_or(DecodeError::Invalid(enc.encoding.name()))
}

/// Como [`decode_as`], mas sem falhar: um byte inválido vira `U+FFFD`. Para comparar (a margem do git), nunca
/// para editar.
pub fn decode_lossy(bytes: &[u8], enc: TextEncoding) -> String {
    let bytes = match Encoding::for_bom(bytes) {
        Some((e, len)) if e == enc.encoding => &bytes[len..],
        _ => bytes,
    };
    enc.encoding.decode_without_bom_handling(bytes).0.into_owned()
}

/// Um caractere que o encoding não tem: qual e onde (linha e coluna, começando em 1).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Unmappable {
    pub ch: char,
    pub line: usize,
    pub column: usize,
}

/// O texto nos bytes do encoding, com o BOM se ele tinha.
pub fn encode(text: &str, enc: TextEncoding) -> Result<Vec<u8>, Unmappable> {
    let e = enc.encoding;
    if e == UTF_8 {
        let mut out = Vec::with_capacity(text.len() + 3);
        if enc.bom {
            out.extend_from_slice(b"\xEF\xBB\xBF");
        }
        out.extend_from_slice(text.as_bytes());
        return Ok(out);
    }
    if e == UTF_16LE || e == UTF_16BE {
        // O `encoding_rs` não grava UTF-16 (o padrão da web não pede): à mão.
        let le = e == UTF_16LE;
        let mut out = Vec::with_capacity(text.len() * 2 + 2);
        let mut push = |u: u16| out.extend_from_slice(&if le { u.to_le_bytes() } else { u.to_be_bytes() });
        if enc.bom {
            push(0xFEFF);
        }
        text.encode_utf16().for_each(&mut push);
        return Ok(out);
    }
    let (bytes, _, had_errors) = e.encode(text);
    if !had_errors {
        return Ok(bytes.into_owned());
    }
    // Só no caminho do erro: acha o primeiro caractere que não cabe.
    let mut buf = [0u8; 4];
    for (n, line) in text.split('\n').enumerate() {
        for (c, ch) in line.chars().enumerate() {
            if e.encode(ch.encode_utf8(&mut buf)).2 {
                return Err(Unmappable { ch, line: n + 1, column: c + 1 });
            }
        }
    }
    Err(Unmappable { ch: '\u{FFFD}', line: 1, column: 1 })
}

/// O encoding de reserva por nome (`[editor] fallback_encoding`); um nome desconhecido, ou UTF-8/UTF-16 (que não
/// servem de reserva), fica no Windows-1252.
pub fn fallback(label: &str) -> &'static Encoding {
    match TextEncoding::from_label(label) {
        Some(e) if e.encoding != UTF_8 && e.encoding != UTF_16LE && e.encoding != UTF_16BE => e.encoding,
        _ => WINDOWS_1252,
    }
}

/// Lê um arquivo de texto do disco como o editor lê (para quem precisa do texto sem abrir a aba: recarregar depois
/// de um `git pull`, trocar no projeto, buscar).
pub fn read_text(path: &Path, hint: Option<TextEncoding>, fallback: &'static Encoding) -> std::io::Result<(String, TextEncoding)> {
    let bytes = std::fs::read(path)?;
    decode(&bytes, hint, fallback).map_err(|e| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            match e {
                DecodeError::Binary => "binary".to_string(),
                DecodeError::Invalid(name) => format!("invalid {name}"),
            },
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn roundtrip(bytes: &[u8]) -> TextEncoding {
        let (text, enc) = decode(bytes, None, WINDOWS_1252).unwrap();
        assert_eq!(encode(&text, enc).unwrap(), bytes, "ida e volta byte a byte em {}", enc.label());
        enc
    }

    #[test]
    fn every_encoding_round_trips_byte_for_byte() {
        assert_eq!(roundtrip("ação\n".as_bytes()), TextEncoding::UTF8);
        assert_eq!(roundtrip(b"\xEF\xBB\xBFa\xC3\xA7\xC3\xA3o\r\n").label(), "UTF-8 BOM");
        assert_eq!(roundtrip(b"a\xE7\xE3o \x80\n").label(), "windows-1252", "ç, ã e o € do Windows");
        assert_eq!(roundtrip(b"\xFF\xFEa\x00\xE7\x00").label(), "UTF-16LE");
        assert_eq!(roundtrip(b"\xFE\xFF\x00a\x00\xE7").label(), "UTF-16BE");
        let (text, _) = decode(b"\xEF\xBB\xBFoi", None, WINDOWS_1252).unwrap();
        assert_eq!(text, "oi", "o BOM não entra no texto");
    }

    #[test]
    fn binary_is_still_refused_and_hints_must_fit() {
        assert_eq!(decode(b"\x89PNG\r\n\x1a\n\x00\x00", None, WINDOWS_1252), Err(DecodeError::Binary));
        // Shift_JIS de "こんにちは".
        let sjis = b"\x82\xb1\x82\xf1\x82\xc9\x82\xbf\x82\xcd";
        let hint = TextEncoding::from_label("shift_jis");
        assert_eq!(decode(sjis, hint, WINDOWS_1252).unwrap(), ("こんにちは".to_string(), hint.unwrap()));
        // Sem a dica, a reserva lê (errado, mas sem perder bytes: reabrir com Shift_JIS conserta).
        let (wrong, enc) = decode(sjis, None, WINDOWS_1252).unwrap();
        assert_eq!(enc.encoding, WINDOWS_1252);
        assert_eq!(decode_as(&encode(&wrong, enc).unwrap(), hint.unwrap()).unwrap(), "こんにちは");
        // Uma dica em que os bytes não valem é ignorada (no ISO-2022-JP, nenhum byte passa de 0x7F).
        assert_eq!(decode("ação".as_bytes(), TextEncoding::from_label("iso-2022-jp"), WINDOWS_1252).unwrap().1, TextEncoding::UTF8);
    }

    #[test]
    fn a_character_that_does_not_fit_is_an_error() {
        let enc = TextEncoding::from_label("latin1").unwrap();
        assert_eq!(enc.encoding, WINDOWS_1252);
        assert_eq!(encode("ok\nsó 🙂 aqui", enc), Err(Unmappable { ch: '🙂', line: 2, column: 4 }));
        assert!(decode_as(b"\x82", TextEncoding::from_label("shift_jis").unwrap()).is_err(), "byte solto não é texto");
    }

    #[test]
    fn labels_from_editorconfig_and_the_fallback() {
        assert_eq!(TextEncoding::from_label("UTF-8-BOM").unwrap().label(), "UTF-8 BOM");
        assert!(TextEncoding::from_label("utf-16le").unwrap().bom);
        assert_eq!(TextEncoding::from_label("klingon"), None);
        assert_eq!(fallback("windows-1251"), encoding_rs::WINDOWS_1251);
        assert_eq!(fallback("utf-8"), WINDOWS_1252, "UTF-8 não serve de reserva");
        assert!(TextEncoding::choices().iter().all(|e| TextEncoding::from_label(&e.label()) == Some(*e) || e.encoding == UTF_16LE || e.encoding == UTF_16BE));
    }

    #[test]
    fn the_git_base_decodes_in_the_tab_encoding() {
        let latin1 = TextEncoding { encoding: WINDOWS_1252, bom: false };
        assert_eq!(decode_lossy(b"a\xe7\xe3o\n", latin1), "ação\n");
        // UTF-8 com BOM: o BOM não entra no texto, como ao abrir.
        let bom = TextEncoding { encoding: UTF_8, bom: true };
        assert_eq!(decode_lossy(b"\xef\xbb\xbfok", bom), "ok");
        // Um byte inválido vira U+FFFD em vez de apagar as marcas.
        assert_eq!(decode_lossy(b"a\xe7", TextEncoding::UTF8), "a\u{FFFD}");
        let utf16 = TextEncoding { encoding: UTF_16LE, bom: true };
        assert_eq!(decode_lossy(b"\xff\xfeo\0k\0", utf16), "ok");
    }
}
