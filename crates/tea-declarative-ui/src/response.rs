//! A resposta gravada quando o usuário clica num botão (`format: "idetea-ui-response"`).

use std::collections::BTreeMap;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const RESPONSE_FORMAT: &str = "idetea-ui-response";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Response {
    pub format: String,
    pub version: u64,
    pub surface: String,
    /// Revisão do documento que o usuário viu ao responder.
    pub revision: u64,
    /// Único: o agente ignora um `event` repetido.
    pub event: String,
    /// Cresce a cada resposta da mesma interface: a ordem.
    pub seq: u64,
    /// O `action.name` do botão.
    pub action: String,
    /// O id do botão.
    pub source: String,
    /// Hora UTC (RFC 3339). Só informativa: a ordem é `seq`.
    pub time: String,
    /// Caminho → valor, para os caminhos do `action.context`.
    pub values: BTreeMap<String, Value>,
}

impl Response {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        surface: &str,
        revision: u64,
        seq: u64,
        event: String,
        action: String,
        source: &str,
        time: String,
        values: BTreeMap<String, Value>,
    ) -> Self {
        Self {
            format: RESPONSE_FORMAT.to_string(),
            version: crate::VERSION,
            surface: surface.to_string(),
            revision,
            event,
            seq,
            action,
            source: source.to_string(),
            time,
            values,
        }
    }
}

/// Nome do arquivo da resposta: começa pelo `seq` com zeros, então a ordem alfabética é a ordem das respostas.
pub fn file_name(seq: u64, event: &str) -> String {
    format!("{seq:06}-{event}.json")
}

/// `2026-11-01T20:41:07Z`.
pub fn timestamp(t: SystemTime) -> String {
    let secs = t.duration_since(UNIX_EPOCH).map_or(0, |d| d.as_secs());
    let (days, rest) = (secs / 86_400, secs % 86_400);
    let (y, m, d) = civil_from_days(days as i64);
    format!("{y:04}-{m:02}-{d:02}T{:02}:{:02}:{:02}Z", rest / 3600, rest % 3600 / 60, rest % 60)
}

/// Dias desde 1970-01-01 → (ano, mês, dia). Algoritmo de Howard Hinnant (calendário gregoriano).
fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    let y = yoe + era * 400 + i64::from(m <= 2);
    (y, m, d)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn timestamps_and_names() {
        assert_eq!(timestamp(UNIX_EPOCH), "1970-01-01T00:00:00Z");
        assert_eq!(timestamp(UNIX_EPOCH + Duration::from_secs(1_793_565_667)), "2026-11-01T20:41:07Z");
        assert_eq!(timestamp(UNIX_EPOCH + Duration::from_secs(951_782_400)), "2000-02-29T00:00:00Z");
        assert_eq!(file_name(7, "abc"), "000007-abc.json");
    }
}
