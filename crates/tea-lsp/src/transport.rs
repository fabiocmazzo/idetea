//! Enquadramento JSON-RPC do LSP: `Content-Length: N\r\n\r\n{json}`.

use std::io::{self, BufRead, Write};

use serde_json::Value;

/// Limite de uma mensagem, para um servidor com defeito não estourar a memória.
const MAX_MESSAGE_BYTES: usize = 64 * 1024 * 1024;

pub fn write_message(w: &mut impl Write, msg: &Value) -> io::Result<()> {
    let body = serde_json::to_vec(msg)?;
    write!(w, "Content-Length: {}\r\n\r\n", body.len())?;
    w.write_all(&body)?;
    w.flush()
}

/// Lê a próxima mensagem. `Ok(None)` = fim do stream (servidor saiu).
pub fn read_message(r: &mut impl BufRead) -> io::Result<Option<Value>> {
    let mut length: Option<usize> = None;
    let mut line = String::new();
    loop {
        line.clear();
        if r.read_line(&mut line)? == 0 {
            return Ok(None);
        }
        let header = line.trim_end();
        if header.is_empty() {
            if length.is_some() {
                break;
            }
            continue; // linhas vazias soltas antes do cabeçalho
        }
        if let Some((name, value)) = header.split_once(':')
            && name.trim().eq_ignore_ascii_case("content-length")
        {
            let n: usize =
                value.trim().parse().map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Content-Length inválido"))?;
            if n > MAX_MESSAGE_BYTES {
                return Err(io::Error::new(io::ErrorKind::InvalidData, "mensagem grande demais"));
            }
            length = Some(n);
        }
        // Outros cabeçalhos (Content-Type) são ignorados.
    }
    let mut body = vec![0; length.unwrap_or(0)];
    r.read_exact(&mut body)?;
    serde_json::from_slice(&body).map(Some).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn round_trip_and_eof() {
        let mut buf = Vec::new();
        write_message(&mut buf, &json!({"jsonrpc": "2.0", "id": 1, "result": "ç"})).unwrap();
        write_message(&mut buf, &json!({"jsonrpc": "2.0", "method": "x"})).unwrap();
        let mut r = io::BufReader::new(&buf[..]);
        assert_eq!(read_message(&mut r).unwrap().unwrap()["result"], "ç");
        assert_eq!(read_message(&mut r).unwrap().unwrap()["method"], "x");
        assert!(read_message(&mut r).unwrap().is_none());
    }

    #[test]
    fn tolerates_extra_headers_and_case() {
        let raw = b"content-length: 2\r\nContent-Type: application/vscode-jsonrpc\r\n\r\n{}";
        let mut r = io::BufReader::new(&raw[..]);
        assert_eq!(read_message(&mut r).unwrap().unwrap(), json!({}));
    }

    #[test]
    fn rejects_garbage_length() {
        let mut r = io::BufReader::new(&b"Content-Length: abc\r\n\r\n"[..]);
        assert!(read_message(&mut r).is_err());
    }
}
