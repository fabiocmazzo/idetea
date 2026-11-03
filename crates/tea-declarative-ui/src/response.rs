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
    /// O "comentário para o agente", se o usuário escreveu um.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
    /// A IDE que gravou a resposta (o agente confere se o teste rodou na versão nova).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ide: Option<Ide>,
    /// As imagens e os sons do documento, pelo caminho: a versão que estava na tela e o que o usuário fez com
    /// eles nesta revisão. Informação, não prova (o painel não sabe se o som estava baixo).
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub media: BTreeMap<String, Seen>,
}

/// Uma imagem ou um som, na resposta.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Seen {
    /// Tamanho do arquivo lido para a tela, em bytes (`None`: não deu para ler).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<u64>,
    /// Data do arquivo lido para a tela (RFC 3339, UTC): compare com a do disco.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modified: Option<String>,
    /// De um WAV, o que o painel mostrou: duração em segundos, pico em dBFS (`None` também no silêncio) e se
    /// chega a 0 dBFS (o aviso de estouro; só aparece quando chega).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seconds: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub peak_db: Option<f64>,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub clips: bool,
    /// Vezes que entrou na tela e ficou à vista meio segundo (rolar até ela, abrir o painel). Mídia que ficou
    /// abaixo da dobra e nunca foi rolada fica sem.
    #[serde(skip_serializing_if = "is_zero")]
    pub shown: u32,
    /// Cliques que abriram o arquivo: a imagem na aba, ou um link `idetea:open/` no app do sistema.
    #[serde(skip_serializing_if = "is_zero")]
    pub opened: u32,
    /// Vezes que o som começou a tocar (cada volta de um `loop` conta).
    #[serde(skip_serializing_if = "is_zero")]
    pub played: u32,
    /// Vezes que o som tocou até o fim (sem Parar).
    #[serde(skip_serializing_if = "is_zero")]
    pub completed: u32,
    /// Vezes que o usuário clicou em Parar.
    #[serde(skip_serializing_if = "is_zero")]
    pub stopped: u32,
    /// Vezes que outro som começou por cima (um de cada vez).
    #[serde(skip_serializing_if = "is_zero")]
    pub interrupted: u32,
    /// Quadros passados à mão num `sprite` pausado.
    #[serde(skip_serializing_if = "is_zero")]
    pub stepped: u32,
}

fn is_zero(n: &u32) -> bool {
    *n == 0
}

/// Quem respondeu: o processo da IDE e se ele está desatualizado.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Ide {
    pub pid: u32,
    pub version: String,
    /// O executável mudou no disco depois que esta IDE abriu (recompilada ou atualizada): ela roda o código antigo.
    pub outdated: bool,
    /// Versão das instruções para agentes desta IDE (a linha "Instructions version" no fim do bloco do
    /// `AGENTS.md`). Diferente da do arquivo: falta rodar "Ensinar os agentes".
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub instructions: Option<String>,
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
            comment: None,
            ide: None,
            media: BTreeMap::new(),
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
        let mut r = Response::new("s", 1, 1, "e".into(), "ok".into(), "ok", "t".into(), BTreeMap::new());
        assert!(!serde_json::to_string(&r).unwrap().contains("comment"), "sem comentário, sem o campo");
        r.comment = Some("pense de novo".into());
        assert!(serde_json::to_string(&r).unwrap().contains(r#""comment":"pense de novo""#));
        r.ide = Some(Ide { pid: 42, version: "0.0.1".into(), outdated: true, instructions: None });
        let json = serde_json::to_string(&r).unwrap();
        assert!(json.contains(r#""ide":{"pid":42,"version":"0.0.1","outdated":true}"#), "{json}");
        assert_eq!(serde_json::from_str::<Response>(&json).unwrap(), r);
    }
}
