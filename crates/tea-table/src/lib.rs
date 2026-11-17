//! CSV, TSV e JSONL como tabela (RFC 0020).
//!
//! A tabela não copia o texto: cada célula guarda onde está no texto (bytes). Assim ler um arquivo grande custa
//! pouco, e editar uma célula troca só os bytes dela ([`Table::edit`]): as aspas das outras células, os fins de
//! linha (`\n` ou `\r\n`) e a última linha sem quebra ficam como estavam.

use std::borrow::Cow;
use std::ops::Range;
use std::path::Path;

/// O formato do arquivo.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    /// RFC 4180, com o separador dado (`,`, `;`, `|`).
    Csv(u8),
    /// Separado por tabulação, sem aspas; `\t`, `\n`, `\r` e `\\` como escapes.
    Tsv,
    /// Um objeto JSON por linha.
    Jsonl,
}

impl Format {
    /// O formato pela extensão (`csv`, `tsv`, `jsonl`/`ndjson`); no CSV, o separador vem das primeiras linhas.
    pub fn detect(path: &Path, text: &str) -> Option<Self> {
        let ext = path.extension()?.to_string_lossy().to_lowercase();
        match ext.as_str() {
            "csv" => Some(Self::Csv(sniff_delimiter(text))),
            "tsv" | "tab" => Some(Self::Tsv),
            "jsonl" | "ndjson" => Some(Self::Jsonl),
            _ => None,
        }
    }

    fn delimiter(self) -> u8 {
        match self {
            Self::Csv(d) => d,
            Self::Tsv => b'\t',
            Self::Jsonl => b',',
        }
    }
}

/// O separador de um CSV: entre `,`, `;` e `|`, o que aparece o mesmo número de vezes (e mais de zero) no maior
/// número das primeiras 20 linhas, fora das aspas. Empate ou nada: vírgula.
pub fn sniff_delimiter(text: &str) -> u8 {
    let lines: Vec<&str> = text.lines().filter(|l| !l.trim().is_empty()).take(20).collect();
    let mut best = (b',', 0usize);
    for &d in b",;|" {
        let counts: Vec<usize> = lines.iter().map(|l| count_outside_quotes(l, d)).collect();
        let Some(&first) = counts.first() else { break };
        if first == 0 {
            continue;
        }
        let same = counts.iter().filter(|&&c| c == first).count();
        if same > best.1 {
            best = (d, same);
        }
    }
    best.0
}

fn count_outside_quotes(line: &str, d: u8) -> usize {
    let mut quoted = false;
    line.bytes()
        .filter(|&b| {
            if b == b'"' {
                quoted = !quoted;
            }
            !quoted && b == d
        })
        .count()
}

/// Uma célula: onde está no texto.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cell {
    /// Os bytes do campo no texto, com as aspas (no CSV) ou o valor JSON inteiro (no JSONL).
    pub span: Range<usize>,
    /// O campo estava entre aspas (CSV).
    pub quoted: bool,
}

/// Uma linha da tabela (um registro).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Row {
    /// A linha do arquivo onde o registro começa (0 é a primeira). Um campo com quebra de linha faz um registro
    /// ocupar mais de uma.
    pub line: usize,
    /// As células, uma por coluna; `None` onde o registro não tem a coluna (linha curta no CSV, chave ausente no
    /// JSONL).
    pub cells: Vec<Option<Cell>>,
    /// Onde o registro termina, antes do fim de linha: é onde entram as células que faltam. No JSONL, o `}`.
    pub end: usize,
    /// O registro não pôde ser lido (JSONL que não é um objeto, ou não é JSON): a mensagem.
    pub error: Option<String>,
}

/// O arquivo como tabela.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Table {
    pub format: Format,
    /// Os nomes das colunas: o cabeçalho, as chaves (JSONL) ou `A`, `B`, `C`… sem cabeçalho.
    pub columns: Vec<String>,
    /// O cabeçalho do CSV/TSV, se a primeira linha é ele.
    pub header: Option<Row>,
    pub rows: Vec<Row>,
}

/// Uma troca no texto: os bytes `range` viram `text`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Edit {
    pub range: Range<usize>,
    pub text: String,
}

/// Por que uma célula não pôde ser trocada.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EditError {
    /// Não existe essa linha ou coluna.
    OutOfRange,
    /// O registro não foi lido (ver [`Row::error`]).
    BrokenRow,
    /// No JSONL, o valor novo não é JSON (a mensagem do erro).
    InvalidJson(String),
}

/// Que linha da tabela: o cabeçalho ou um registro.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RowRef {
    Header,
    Row(usize),
}

impl Table {
    /// Lê `text`. `header`: a primeira linha do CSV/TSV são os nomes das colunas (ignorado no JSONL).
    pub fn parse(text: &str, format: Format, header: bool) -> Self {
        match format {
            Format::Jsonl => parse_jsonl(text),
            _ => {
                let mut rows = parse_delimited(text, format);
                let header = if header && !rows.is_empty() { Some(rows.remove(0)) } else { None };
                let width = rows.iter().chain(&header).map(|r| r.cells.len()).max().unwrap_or(0);
                let mut columns: Vec<String> = match &header {
                    Some(h) => (0..h.cells.len()).map(|c| cell_text(text, format, h, c).into_owned()).collect(),
                    None => Vec::new(),
                };
                for c in columns.len()..width {
                    columns.push(column_letter(c));
                }
                Self { format, columns, header, rows }
            }
        }
    }

    fn row(&self, row: RowRef) -> Option<&Row> {
        match row {
            RowRef::Header => self.header.as_ref(),
            RowRef::Row(i) => self.rows.get(i),
        }
    }

    /// O valor de uma célula para mostrar: sem as aspas e os escapes; no JSONL, um texto sem as aspas e os outros
    /// valores como JSON. `None` onde a célula não existe.
    pub fn value<'t>(&self, text: &'t str, row: RowRef, col: usize) -> Option<Cow<'t, str>> {
        let r = self.row(row)?;
        r.cells.get(col)?.as_ref()?;
        Some(cell_text(text, self.format, r, col))
    }

    /// O texto de uma célula como ela é editada: o valor (CSV/TSV) ou o JSON (JSONL).
    pub fn raw<'t>(&self, text: &'t str, row: RowRef, col: usize) -> Option<Cow<'t, str>> {
        let r = self.row(row)?;
        let cell = r.cells.get(col)?.as_ref()?;
        match self.format {
            Format::Jsonl => Some(Cow::Borrowed(&text[cell.span.clone()])),
            _ => Some(cell_text(text, self.format, r, col)),
        }
    }

    /// No JSONL, o tipo do valor (para alinhar números e mostrar `null`); `None` nos outros formatos.
    pub fn json_kind(&self, text: &str, row: RowRef, col: usize) -> Option<JsonKind> {
        if self.format != Format::Jsonl {
            return None;
        }
        let cell = self.row(row)?.cells.get(col)?.as_ref()?;
        Some(JsonKind::of(&text[cell.span.clone()]))
    }

    /// A troca no texto que põe `value` na célula, mexendo só nela. No CSV/TSV, `value` é o texto da célula (as
    /// aspas e os escapes são postos aqui); no JSONL, um valor JSON. Uma célula que falta é acrescentada no fim do
    /// registro (com os separadores das que faltam antes dela, no CSV/TSV).
    pub fn edit(&self, text: &str, row: RowRef, col: usize, value: &str) -> Result<Edit, EditError> {
        let r = self.row(row).ok_or(EditError::OutOfRange)?;
        if r.error.is_some() {
            return Err(EditError::BrokenRow);
        }
        if col >= self.columns.len() {
            return Err(EditError::OutOfRange);
        }
        if self.format == Format::Jsonl {
            return self.edit_json(text, r, col, value);
        }
        match r.cells.get(col).and_then(Option::as_ref) {
            Some(cell) => Ok(Edit { range: cell.span.clone(), text: encode(self.format, value, cell.quoted) }),
            None => {
                let missing = col + 1 - r.cells.len();
                let mut out = String::new();
                // Um registro vazio (uma linha em branco no meio) ainda tem a primeira célula, vazia.
                let has_any = !r.cells.is_empty();
                for i in 0..missing {
                    if has_any || i > 0 {
                        out.push(self.format.delimiter() as char);
                    }
                }
                out.push_str(&encode(self.format, value, false));
                Ok(Edit { range: r.end..r.end, text: out })
            }
        }
    }

    fn edit_json(&self, text: &str, r: &Row, col: usize, value: &str) -> Result<Edit, EditError> {
        let value = value.trim();
        serde_json::from_str::<serde_json::Value>(value).map_err(|e| EditError::InvalidJson(e.to_string()))?;
        match r.cells.get(col).and_then(Option::as_ref) {
            Some(cell) => Ok(Edit { range: cell.span.clone(), text: value.to_string() }),
            None => {
                // `r.end` é o `}`: a chave nova entra antes dele, com vírgula se o objeto já tem alguma.
                let key = serde_json::to_string(&self.columns[col]).unwrap_or_default();
                let empty = r.cells.iter().all(Option::is_none);
                let start = text[..r.end].trim_end().len();
                let sep = if empty { "" } else { ", " };
                Ok(Edit { range: start..start, text: format!("{sep}{key}: {value}") })
            }
        }
    }
}

/// O tipo de um valor JSON, pelo primeiro caractere.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JsonKind {
    String,
    Number,
    Bool,
    Null,
    Object,
    Array,
}

impl JsonKind {
    fn of(raw: &str) -> Self {
        match raw.as_bytes().first() {
            Some(b'"') => Self::String,
            Some(b'{') => Self::Object,
            Some(b'[') => Self::Array,
            Some(b't' | b'f') => Self::Bool,
            Some(b'n') => Self::Null,
            _ => Self::Number,
        }
    }
}

/// `A`, `B`, … `Z`, `AA`, `AB`…
fn column_letter(mut i: usize) -> String {
    let mut out = Vec::new();
    loop {
        out.push(b'A' + (i % 26) as u8);
        if i < 26 {
            break;
        }
        i = i / 26 - 1;
    }
    out.reverse();
    String::from_utf8(out).unwrap_or_default()
}

fn cell_text<'t>(text: &'t str, format: Format, row: &Row, col: usize) -> Cow<'t, str> {
    let Some(Some(cell)) = row.cells.get(col) else { return Cow::Borrowed("") };
    let raw = &text[cell.span.clone()];
    match format {
        Format::Csv(_) if cell.quoted => {
            let inner = &raw[1..raw.len().saturating_sub(1).max(1)];
            if inner.contains("\"\"") { Cow::Owned(inner.replace("\"\"", "\"")) } else { Cow::Borrowed(inner) }
        }
        Format::Csv(_) => Cow::Borrowed(raw),
        Format::Tsv => unescape_tsv(raw),
        Format::Jsonl => match serde_json::from_str::<serde_json::Value>(raw) {
            Ok(serde_json::Value::String(s)) => Cow::Owned(s),
            _ => Cow::Borrowed(raw),
        },
    }
}

fn unescape_tsv(raw: &str) -> Cow<'_, str> {
    if !raw.contains('\\') {
        return Cow::Borrowed(raw);
    }
    let mut out = String::with_capacity(raw.len());
    let mut chars = raw.chars();
    while let Some(c) = chars.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }
        match chars.next() {
            Some('t') => out.push('\t'),
            Some('n') => out.push('\n'),
            Some('r') => out.push('\r'),
            Some('\\') => out.push('\\'),
            Some(other) => {
                out.push('\\');
                out.push(other);
            }
            None => out.push('\\'),
        }
    }
    Cow::Owned(out)
}

/// O texto de uma célula como vai no arquivo.
fn encode(format: Format, value: &str, was_quoted: bool) -> String {
    match format {
        Format::Tsv => {
            let mut out = String::with_capacity(value.len());
            for c in value.chars() {
                match c {
                    '\t' => out.push_str("\\t"),
                    '\n' => out.push_str("\\n"),
                    '\r' => out.push_str("\\r"),
                    '\\' => out.push_str("\\\\"),
                    c => out.push(c),
                }
            }
            out
        }
        Format::Csv(d) => {
            let needs = value.bytes().any(|b| b == d || b == b'"' || b == b'\n' || b == b'\r');
            if was_quoted || needs { format!("\"{}\"", value.replace('"', "\"\"")) } else { value.to_string() }
        }
        Format::Jsonl => value.to_string(),
    }
}

/// CSV e TSV: os registros, com a posição de cada campo. Uma linha em branco no fim do arquivo não é registro.
fn parse_delimited(text: &str, format: Format) -> Vec<Row> {
    let bytes = text.as_bytes();
    let d = format.delimiter();
    let csv = matches!(format, Format::Csv(_));
    let mut rows = Vec::new();
    let mut i = 0;
    let mut line = 0;
    while i < bytes.len() {
        let start_line = line;
        let mut cells = Vec::new();
        let end;
        loop {
            // Um campo.
            let start = i;
            let quoted = csv && bytes.get(i) == Some(&b'"');
            if quoted {
                i += 1;
                while i < bytes.len() {
                    if bytes[i] == b'"' {
                        if bytes.get(i + 1) == Some(&b'"') {
                            i += 2;
                            continue;
                        }
                        i += 1;
                        break;
                    }
                    if bytes[i] == b'\n' {
                        line += 1;
                    }
                    i += 1;
                }
                // Lixo depois das aspas de fechamento (`"a"b`) fica no mesmo campo.
                while i < bytes.len() && bytes[i] != d && bytes[i] != b'\n' {
                    i += 1;
                }
            } else {
                while i < bytes.len() && bytes[i] != d && bytes[i] != b'\n' {
                    i += 1;
                }
            }
            let mut field_end = i;
            let at_line_end = i >= bytes.len() || bytes[i] == b'\n';
            if at_line_end && field_end > start && bytes[field_end - 1] == b'\r' {
                field_end -= 1;
            }
            cells.push(Some(Cell { span: start..field_end, quoted }));
            if at_line_end {
                end = field_end;
                if i < bytes.len() {
                    i += 1;
                    line += 1;
                }
                break;
            }
            i += 1; // o separador
        }
        rows.push(Row { line: start_line, cells, end, error: None });
    }
    rows
}

/// JSONL: um objeto por linha; linhas em branco não são registros. As colunas são as chaves, na ordem em que
/// aparecem pela primeira vez.
fn parse_jsonl(text: &str) -> Table {
    let mut columns: Vec<String> = Vec::new();
    let mut parsed: Vec<(usize, Result<Object, String>)> = Vec::new();
    let mut offset = 0;
    for (line, raw) in text.split('\n').enumerate() {
        let start = offset;
        offset += raw.len() + 1;
        let content = raw.strip_suffix('\r').unwrap_or(raw);
        if content.trim().is_empty() {
            continue;
        }
        let result = match serde_json::from_str::<serde_json::Value>(content) {
            Ok(serde_json::Value::Object(_)) => scan_object(content).map(|(fields, close)| {
                (fields.into_iter().map(|(k, r)| (k, r.start + start..r.end + start)).collect::<Vec<_>>(), close + start)
            }),
            Ok(_) => Err("não é um objeto".to_string()),
            Err(e) => Err(e.to_string()),
        };
        if let Ok((fields, _)) = &result {
            for (key, _) in fields.iter() {
                if !columns.contains(key) {
                    columns.push(key.clone());
                }
            }
        }
        parsed.push((line, result));
    }
    let rows = parsed
        .into_iter()
        .map(|(line, result)| match result {
            Ok((fields, close)) => {
                let mut cells = vec![None; columns.len()];
                for (key, span) in fields {
                    if let Some(c) = columns.iter().position(|k| *k == key) {
                        cells[c] = Some(Cell { span, quoted: false });
                    }
                }
                Row { line, cells, end: close, error: None }
            }
            Err(e) => Row { line, cells: Vec::new(), end: 0, error: Some(e) },
        })
        .collect();
    Table { format: Format::Jsonl, columns, header: None, rows }
}

/// As chaves de um objeto JSON com o intervalo de cada valor, e a posição do `}`.
type Object = (Vec<(String, Range<usize>)>, usize);

/// As chaves de um objeto JSON (já validado) com o intervalo de cada valor, e a posição do `}`.
fn scan_object(s: &str) -> Result<Object, String> {
    let b = s.as_bytes();
    let mut i = skip_ws(b, 0);
    if b.get(i) != Some(&b'{') {
        return Err("não é um objeto".into());
    }
    i += 1;
    let mut fields = Vec::new();
    loop {
        i = skip_ws(b, i);
        match b.get(i) {
            Some(b'}') => return Ok((fields, i)),
            Some(b',') => {
                i += 1;
                continue;
            }
            Some(b'"') => {}
            _ => return Err("objeto malformado".into()),
        }
        let key_end = skip_string(b, i);
        let key: String = serde_json::from_str(&s[i..key_end]).map_err(|e| e.to_string())?;
        i = skip_ws(b, key_end);
        if b.get(i) != Some(&b':') {
            return Err("objeto malformado".into());
        }
        i = skip_ws(b, i + 1);
        let value_end = skip_value(b, i);
        fields.push((key, i..value_end));
        i = value_end;
    }
}

fn skip_ws(b: &[u8], mut i: usize) -> usize {
    while i < b.len() && b[i].is_ascii_whitespace() {
        i += 1;
    }
    i
}

/// `b[i]` é `"`: o índice depois da aspa de fechamento.
fn skip_string(b: &[u8], mut i: usize) -> usize {
    i += 1;
    while i < b.len() {
        match b[i] {
            b'\\' => i += 2,
            b'"' => return i + 1,
            _ => i += 1,
        }
    }
    b.len()
}

/// O fim de um valor JSON que começa em `i`.
fn skip_value(b: &[u8], i: usize) -> usize {
    match b.get(i) {
        Some(b'"') => skip_string(b, i),
        Some(b'{' | b'[') => {
            let mut depth = 0usize;
            let mut j = i;
            while j < b.len() {
                match b[j] {
                    b'"' => {
                        j = skip_string(b, j);
                        continue;
                    }
                    b'{' | b'[' => depth += 1,
                    b'}' | b']' => {
                        depth -= 1;
                        if depth == 0 {
                            return j + 1;
                        }
                    }
                    _ => {}
                }
                j += 1;
            }
            b.len()
        }
        _ => {
            let mut j = i;
            while j < b.len() && !matches!(b[j], b',' | b'}' | b']') && !b[j].is_ascii_whitespace() {
                j += 1;
            }
            j
        }
    }
}

/// Aplica uma [`Edit`] a um texto (para testes e para quem não tem um editor).
pub fn apply(text: &str, edit: &Edit) -> String {
    let mut out = String::with_capacity(text.len() + edit.text.len());
    out.push_str(&text[..edit.range.start]);
    out.push_str(&edit.text);
    out.push_str(&text[edit.range.end..]);
    out
}

/// A posição (linha, coluna em caracteres) de um byte do texto, começando em 0.
pub fn position(text: &str, byte: usize) -> (usize, usize) {
    let before = &text[..byte];
    let line = before.matches('\n').count();
    let line_start = before.rfind('\n').map_or(0, |i| i + 1);
    (line, before[line_start..].chars().count())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn csv(text: &str) -> Table {
        Table::parse(text, Format::Csv(b','), true)
    }

    fn values(t: &Table, text: &str, row: usize) -> Vec<String> {
        (0..t.columns.len()).map(|c| t.value(text, RowRef::Row(row), c).map(|v| v.into_owned()).unwrap_or("∅".into())).collect()
    }

    fn set(t: &Table, text: &str, row: RowRef, col: usize, value: &str) -> String {
        apply(text, &t.edit(text, row, col, value).unwrap())
    }

    #[test]
    fn csv_quotes_newlines_and_crlf() {
        let text = "nome,obs,n\r\nana,\"tem, vírgula\",1\r\n\"bia\",\"diz \"\"oi\"\"\nem duas\",2\r\nzé,,3";
        let t = csv(text);
        assert_eq!(t.columns, ["nome", "obs", "n"]);
        assert_eq!(values(&t, text, 0), ["ana", "tem, vírgula", "1"]);
        assert_eq!(values(&t, text, 1), ["bia", "diz \"oi\"\nem duas", "2"]);
        assert_eq!(values(&t, text, 2), ["zé", "", "3"]);
        assert_eq!(t.rows.iter().map(|r| r.line).collect::<Vec<_>>(), [1, 2, 4]);
    }

    #[test]
    fn csv_edit_touches_only_the_cell() {
        let text = "nome,obs\r\nana,\"a\"\r\nbia,b\r\n";
        let t = csv(text);
        // Estava entre aspas: continua.
        assert_eq!(set(&t, text, RowRef::Row(0), 1, "x"), "nome,obs\r\nana,\"x\"\r\nbia,b\r\n");
        // Não estava: só ganha aspas se precisar.
        assert_eq!(set(&t, text, RowRef::Row(1), 1, "c"), "nome,obs\r\nana,\"a\"\r\nbia,c\r\n");
        assert_eq!(set(&t, text, RowRef::Row(1), 1, "c, \"d\""), "nome,obs\r\nana,\"a\"\r\nbia,\"c, \"\"d\"\"\"\r\n");
        // O cabeçalho também.
        assert_eq!(set(&t, text, RowRef::Header, 0, "quem"), "quem,obs\r\nana,\"a\"\r\nbia,b\r\n");
    }

    #[test]
    fn csv_short_row_gets_the_missing_cells_at_the_end() {
        let text = "a,b,c\n1\n2,3,4";
        let t = csv(text);
        assert_eq!(values(&t, text, 0), ["1", "∅", "∅"]);
        assert_eq!(set(&t, text, RowRef::Row(0), 2, "z"), "a,b,c\n1,,z\n2,3,4");
        assert_eq!(set(&t, text, RowRef::Row(1), 2, "9"), "a,b,c\n1\n2,3,9", "última linha sem quebra");
    }

    #[test]
    fn csv_without_header_names_columns_by_letter() {
        let text = "1,2\n3,4,5\n";
        let t = Table::parse(text, Format::Csv(b','), false);
        assert_eq!(t.columns, ["A", "B", "C"]);
        assert_eq!(t.rows.len(), 2);
        assert_eq!(column_letter(27), "AB");
    }

    #[test]
    fn sniffs_the_delimiter() {
        assert_eq!(sniff_delimiter("a;b;c\n1;2;3\n4;\"5;6\";7\n"), b';');
        assert_eq!(sniff_delimiter("a,b\n1,2\n"), b',');
        assert_eq!(sniff_delimiter("a|b|c\n1|2|3\n"), b'|');
        assert_eq!(sniff_delimiter("sozinho\n"), b',');
        assert_eq!(Format::detect(Path::new("x.CSV"), "a;b\n1;2"), Some(Format::Csv(b';')));
        assert_eq!(Format::detect(Path::new("x.ndjson"), ""), Some(Format::Jsonl));
        assert_eq!(Format::detect(Path::new("x.txt"), ""), None);
    }

    #[test]
    fn tsv_escapes_round_trip() {
        let text = "a\tb\nlinha\\num\tc:\\\\x\n";
        let t = Table::parse(text, Format::Tsv, true);
        assert_eq!(values(&t, text, 0), ["linha\num", "c:\\x"]);
        assert_eq!(set(&t, text, RowRef::Row(0), 1, "tab\there"), "a\tb\nlinha\\num\ttab\\there\n");
    }

    #[test]
    fn jsonl_columns_kinds_and_broken_lines() {
        let text = "{\"id\": 1, \"nome\": \"ana\"}\n\n{\"id\":2,\"tags\":[\"x\",\"}\"],\"nome\":null}\nnão é json\n[1]\n";
        let t = Table::parse(text, Format::Jsonl, true);
        assert_eq!(t.columns, ["id", "nome", "tags"]);
        assert_eq!(t.rows.len(), 4);
        assert_eq!(values(&t, text, 0), ["1", "ana", "∅"]);
        assert_eq!(values(&t, text, 1), ["2", "null", "[\"x\",\"}\"]"]);
        assert_eq!(t.rows[1].line, 2);
        assert_eq!(t.raw(text, RowRef::Row(0), 1).as_deref(), Some("\"ana\""));
        assert_eq!(t.json_kind(text, RowRef::Row(1), 1), Some(JsonKind::Null));
        assert_eq!(t.json_kind(text, RowRef::Row(0), 0), Some(JsonKind::Number));
        assert!(t.rows[2].error.is_some());
        assert_eq!(t.rows[3].error.as_deref(), Some("não é um objeto"));
        assert_eq!(t.edit(text, RowRef::Row(2), 0, "1"), Err(EditError::BrokenRow));
    }

    #[test]
    fn jsonl_edit_keeps_the_rest_of_the_line() {
        let text = "{\"id\": 1,  \"nome\": \"ana\"}\r\n{\"id\": 2}\n{}\n";
        let t = Table::parse(text, Format::Jsonl, true);
        assert_eq!(set(&t, text, RowRef::Row(0), 1, "\"bia\""), "{\"id\": 1,  \"nome\": \"bia\"}\r\n{\"id\": 2}\n{}\n");
        assert_eq!(set(&t, text, RowRef::Row(0), 0, " 42 "), "{\"id\": 42,  \"nome\": \"ana\"}\r\n{\"id\": 2}\n{}\n");
        // Chave que falta: entra antes do `}`.
        assert_eq!(set(&t, text, RowRef::Row(1), 1, "null"), "{\"id\": 1,  \"nome\": \"ana\"}\r\n{\"id\": 2, \"nome\": null}\n{}\n");
        assert_eq!(set(&t, text, RowRef::Row(2), 0, "7"), "{\"id\": 1,  \"nome\": \"ana\"}\r\n{\"id\": 2}\n{\"id\": 7}\n");
        assert!(matches!(t.edit(text, RowRef::Row(0), 1, "ana"), Err(EditError::InvalidJson(_))));
        assert_eq!(t.edit(text, RowRef::Row(0), 9, "1"), Err(EditError::OutOfRange));
    }

    #[test]
    fn position_counts_characters() {
        let text = "ação\nzé,1";
        assert_eq!(position(text, text.find('1').unwrap()), (1, 3));
        assert_eq!(position(text, 0), (0, 0));
    }
}
