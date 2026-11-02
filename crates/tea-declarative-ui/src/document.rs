//! O documento de uma interface (`format: "idetea-ui"`, versão 1): leitura e validação.
//!
//! A leitura é feita à mão sobre o `serde_json::Value` (e não com `#[derive(Deserialize)]`) para que o erro
//! diga o campo exato (`components[3].label`), para que campos e componentes desconhecidos virem avisos em vez
//! de recusar o documento inteiro, e para checar os limites.

use std::fmt;

use serde_json::{Map, Value};

use crate::pointer;

pub const FORMAT: &str = "idetea-ui";
pub const VERSION: u64 = 1;

/// Limites de um documento: nada além disso é desenhado.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Limits {
    /// Tamanho do arquivo, em bytes.
    pub max_bytes: usize,
    pub max_components: usize,
    /// Tamanho de cada texto, em bytes.
    pub max_text: usize,
    /// Opções de um `select`.
    pub max_options: usize,
    /// `data` inteiro, em bytes (como JSON compacto).
    pub max_data_bytes: usize,
    /// Imagens de um documento (`image` + itens de `gallery`).
    pub max_images: usize,
    /// Profundidade de `data`.
    pub max_depth: usize,
}

impl Default for Limits {
    fn default() -> Self {
        Self {
            max_bytes: 256 * 1024,
            max_components: 200,
            max_text: 16 * 1024,
            max_options: 500,
            max_data_bytes: 64 * 1024,
            max_images: 32,
            max_depth: 16,
        }
    }
}

/// Uma interface: o que o agente escreveu, já validado.
#[derive(Debug, Clone, PartialEq)]
pub struct Document {
    /// Nome dado pelo agente (opcional; quem manda é o nome do arquivo, conferido pela extensão).
    pub surface: Option<String>,
    /// Sobe a cada versão nova do documento.
    pub revision: u64,
    pub title: Option<String>,
    /// Quem escreveu (ex.: "agente de migração"), para mostrar.
    pub source: Option<String>,
    /// Valores iniciais dos campos (sempre um objeto).
    pub data: Value,
    pub components: Vec<Component>,
    pub submit: Submit,
    /// Mostra o campo "comentário para o agente" antes dos botões (padrão: sim).
    pub comment: bool,
    /// Coisas ignoradas (campo desconhecido, estilo que não existe...), para mostrar ao agente.
    pub warnings: Vec<String>,
}

/// Quantas vezes dá para responder a uma revisão.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Submit {
    /// Depois da primeira resposta, os botões ficam desligados até chegar uma revisão nova.
    #[default]
    Once,
    Many,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Component {
    pub id: String,
    pub kind: Kind,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Kind {
    Text { text: String, style: TextStyle },
    /// `requires`: caminho de um `steps`; o botão só liga com os passos obrigatórios dele marcados.
    Button { label: String, style: ButtonStyle, action: Action, requires: Option<String> },
    /// Passos numerados que o usuário marca. `value`: caminho da lista dos ids marcados.
    Steps { items: Vec<Step>, value: String },
    /// `value`: caminho (ponteiro JSON) do valor em `data`.
    Checkbox { label: String, value: String },
    Select { label: Option<String>, options: Vec<Choice>, value: String },
    Input { label: Option<String>, placeholder: String, value: String },
    /// Um arquivo do projeto: um clique abre na linha e coluna (começando em 1; 0 = início). `path` é relativo à
    /// raiz e não sai dela. `open`: abre sozinho quando a revisão chega.
    File { path: String, line: u64, column: u64, label: Option<String>, open: bool },
    /// Texto em Markdown (listas, código, ênfase, links). Imagens não são carregadas. O texto já lido para
    /// desenhar fica em [`crate::Rendered`].
    Markdown { text: String },
    /// Uma imagem do projeto dentro do painel. Quem desenha recebe a imagem já lida ([`crate::Media`]).
    Image { path: String, label: Option<String>, look: Look },
    /// Imagens lado a lado. Com `pick` `One` ou `Many`, o usuário escolhe; `value`: o id escolhido (ou a lista).
    Gallery { items: Vec<GalleryItem>, pick: Pick, value: Option<String>, look: Look },
    /// Componente que esta versão não conhece: um aviso aparece no lugar dele.
    Unknown { component: String },
}

impl Kind {
    /// Caminho do valor que o componente mostra e muda.
    pub fn binding(&self) -> Option<&str> {
        match self {
            Kind::Checkbox { value, .. }
            | Kind::Select { value, .. }
            | Kind::Input { value, .. }
            | Kind::Steps { value, .. } => Some(value),
            Kind::Gallery { value, .. } => value.as_deref(),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TextStyle {
    #[default]
    Normal,
    Muted,
    Warning,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ButtonStyle {
    #[default]
    Primary,
    Secondary,
    Danger,
}

/// Como uma imagem aparece.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Look {
    /// Vizinho mais próximo ao ampliar (pixel art sem borrar).
    pub pixelated: bool,
    /// Fundo quadriculado (mostra a transparência).
    pub checker: bool,
    /// Altura na tela, em pixels (a largura segue a proporção). `None`: a da imagem, até um máximo.
    pub height: Option<u32>,
}

/// Quantas imagens de uma `gallery` dá para escolher.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Pick {
    #[default]
    None,
    One,
    Many,
}

/// Uma imagem de uma `gallery`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GalleryItem {
    pub id: String,
    pub path: String,
    pub label: Option<String>,
}

impl Document {
    /// Os caminhos das imagens do documento, na ordem, sem repetir.
    pub fn image_paths(&self) -> Vec<&str> {
        let mut out: Vec<&str> = Vec::new();
        for c in &self.components {
            let paths: Vec<&str> = match &c.kind {
                Kind::Image { path, .. } => vec![path],
                Kind::Gallery { items, .. } => items.iter().map(|i| i.path.as_str()).collect(),
                _ => Vec::new(),
            };
            for p in paths {
                if !out.contains(&p) {
                    out.push(p);
                }
            }
        }
        out
    }
}

/// Um passo de um `steps`. `text`: Markdown de uma linha. `optional`: não conta para o `requires`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Step {
    pub id: String,
    pub text: String,
    pub optional: bool,
}

/// Uma opção de um `select`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Choice {
    pub value: String,
    pub label: String,
}

impl fmt::Display for Choice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.label)
    }
}

/// O que um botão faz: gera uma resposta com este nome e os valores destes caminhos.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Action {
    pub name: String,
    pub context: Vec<String>,
}

/// Lê e valida um documento. O erro já vem pronto para mostrar (e diz o campo).
pub fn parse(bytes: &[u8], limits: &Limits) -> Result<Document, String> {
    if bytes.len() > limits.max_bytes {
        return Err(fl!("ui-too-big", size = bytes.len(), max = limits.max_bytes));
    }
    let value: Value = serde_json::from_slice(bytes).map_err(|e| fl!("ui-not-json", error = e.to_string()))?;
    let Value::Object(obj) = value else { return Err(fl!("ui-not-object")) };
    let mut r = Reader { limits: *limits, warnings: Vec::new() };
    r.known_fields(
        &obj,
        &["$schema", "format", "version", "surface", "revision", "title", "source", "data", "components", "submit", "comment"],
        "",
    );

    match obj.get("format") {
        Some(Value::String(f)) if f == FORMAT => {}
        Some(other) => return Err(fl!("ui-wrong-format", found = other.to_string())),
        None => return Err(fl!("ui-missing", field = "format")),
    }
    let version = r.number(&obj, "version", "version")?.ok_or_else(|| fl!("ui-missing", field = "version"))?;
    if version != VERSION {
        return Err(fl!("ui-unsupported-version", version = version, supported = VERSION));
    }

    let data = match obj.get("data") {
        None | Some(Value::Null) => Value::Object(Map::new()),
        Some(v @ Value::Object(_)) => {
            let size = serde_json::to_string(v).map_or(usize::MAX, |s| s.len());
            if size > limits.max_data_bytes {
                return Err(fl!("ui-data-too-big", max = limits.max_data_bytes));
            }
            if depth(v) > limits.max_depth {
                return Err(fl!("ui-data-too-deep", max = limits.max_depth));
            }
            v.clone()
        }
        Some(_) => return Err(fl!("ui-wrong-type", field = "data", expected = "object")),
    };

    let submit = match r.string(&obj, "submit", "submit")?.as_deref() {
        None | Some("once") => Submit::Once,
        Some("many") => Submit::Many,
        Some(other) => {
            r.warnings.push(fl!("ui-unknown-value", field = "submit", value = other));
            Submit::Once
        }
    };

    let comment = match obj.get("comment") {
        None | Some(Value::Null) => true,
        Some(Value::Bool(b)) => *b,
        Some(_) => return Err(fl!("ui-wrong-type", field = "comment", expected = "boolean")),
    };

    let list = match obj.get("components") {
        Some(Value::Array(list)) => list,
        Some(_) => return Err(fl!("ui-wrong-type", field = "components", expected = "array")),
        None => return Err(fl!("ui-missing", field = "components")),
    };
    if list.len() > limits.max_components {
        return Err(fl!("ui-too-many", field = "components", max = limits.max_components));
    }
    let mut components = Vec::with_capacity(list.len());
    for (i, item) in list.iter().enumerate() {
        let field = format!("components[{i}]");
        let component = r.component(item, &field)?;
        if components.iter().any(|c: &Component| c.id == component.id) {
            return Err(fl!("ui-duplicate-id", id = component.id));
        }
        components.push(component);
    }
    let images: usize = components
        .iter()
        .map(|c| match &c.kind {
            Kind::Image { .. } => 1,
            Kind::Gallery { items, .. } => items.len(),
            _ => 0,
        })
        .sum();
    if images > limits.max_images {
        return Err(fl!("ui-too-many-images", max = limits.max_images));
    }
    // `requires` só vale para o caminho de um `steps`; outro caminho vira aviso e o botão não espera nada.
    let steps: Vec<String> = components
        .iter()
        .filter_map(|c| match &c.kind {
            Kind::Steps { value, .. } => Some(value.clone()),
            _ => None,
        })
        .collect();
    for (i, c) in components.iter_mut().enumerate() {
        if let Kind::Button { requires: requires @ Some(_), .. } = &mut c.kind
            && !steps.contains(requires.as_ref().unwrap())
        {
            r.warnings.push(fl!("ui-requires-not-steps", field = format!("components[{i}].requires")));
            *requires = None;
        }
    }

    Ok(Document {
        surface: r.string(&obj, "surface", "surface")?,
        revision: r.number(&obj, "revision", "revision")?.unwrap_or(0),
        title: r.string(&obj, "title", "title")?,
        source: r.string(&obj, "source", "source")?,
        data,
        components,
        submit,
        comment,
        warnings: r.warnings,
    })
}

fn depth(v: &Value) -> usize {
    match v {
        Value::Object(m) => 1 + m.values().map(depth).max().unwrap_or(0),
        Value::Array(a) => 1 + a.iter().map(depth).max().unwrap_or(0),
        _ => 0,
    }
}

/// `src/main.rs` sim; `/etc/x`, `../x`, `C:\x` e vazio, não.
pub(crate) fn relative_inside(path: &str) -> bool {
    use std::path::{Component, Path};
    !path.is_empty()
        && !path.starts_with(['/', '\\'])
        && path.as_bytes().get(1) != Some(&b':')
        && Path::new(path).components().all(|c| matches!(c, Component::Normal(_) | Component::CurDir))
        && !path.split(['/', '\\']).any(|part| part == "..")
}

/// Id de componente: letras, números, `_` e `-`, até 64.
fn valid_id(id: &str) -> bool {
    !id.is_empty() && id.len() <= 64 && id.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
}

struct Reader {
    limits: Limits,
    warnings: Vec<String>,
}

fn at(field: &str, key: &str) -> String {
    if field.is_empty() { key.to_string() } else { format!("{field}.{key}") }
}

impl Reader {
    /// Campos que não estão em `known` viram aviso.
    fn known_fields(&mut self, obj: &Map<String, Value>, known: &[&str], field: &str) {
        for key in obj.keys().filter(|k| !known.contains(&k.as_str())) {
            self.warnings.push(fl!("ui-unknown-field", field = at(field, key)));
        }
    }

    /// Texto opcional (ausente ou `null`: `None`).
    fn string(&self, obj: &Map<String, Value>, key: &str, field: &str) -> Result<Option<String>, String> {
        match obj.get(key) {
            None | Some(Value::Null) => Ok(None),
            Some(Value::String(s)) if s.len() > self.limits.max_text => {
                Err(fl!("ui-too-long", field = at(field, key), max = self.limits.max_text))
            }
            Some(Value::String(s)) => Ok(Some(s.clone())),
            Some(_) => Err(fl!("ui-wrong-type", field = at(field, key), expected = "string")),
        }
    }

    fn required(&self, obj: &Map<String, Value>, key: &str, field: &str) -> Result<String, String> {
        self.string(obj, key, field)?.ok_or_else(|| fl!("ui-missing", field = at(field, key)))
    }

    fn boolean(&self, obj: &Map<String, Value>, key: &str, field: &str) -> Result<bool, String> {
        match obj.get(key) {
            None | Some(Value::Null) => Ok(false),
            Some(Value::Bool(b)) => Ok(*b),
            Some(_) => Err(fl!("ui-wrong-type", field = at(field, key), expected = "boolean")),
        }
    }

    /// Caminho de arquivo relativo à raiz do projeto, sem `..` (não sai dela).
    fn file_path(&self, obj: &Map<String, Value>, field: &str) -> Result<String, String> {
        let path = self.required(obj, "path", field)?;
        if relative_inside(&path) {
            Ok(path)
        } else {
            Err(fl!("ui-bad-file-path", field = at(field, "path"), path = path))
        }
    }

    fn number(&self, obj: &Map<String, Value>, key: &str, field: &str) -> Result<Option<u64>, String> {
        match obj.get(key) {
            None | Some(Value::Null) => Ok(None),
            Some(v) => v
                .as_u64()
                .map(Some)
                .ok_or_else(|| fl!("ui-wrong-type", field = at(field, key), expected = "integer >= 0")),
        }
    }

    /// Caminho de um valor: `{"path": "/x"}` (ou só `"/x"`).
    fn path(&self, obj: &Map<String, Value>, key: &str, field: &str) -> Result<String, String> {
        let path = match obj.get(key) {
            Some(Value::Object(o)) => match o.get("path") {
                Some(Value::String(p)) => p.clone(),
                _ => return Err(fl!("ui-missing", field = at(&at(field, key), "path"))),
            },
            Some(Value::String(p)) => p.clone(),
            Some(_) => return Err(fl!("ui-wrong-type", field = at(field, key), expected = "{\"path\": \"/...\"}")),
            None => return Err(fl!("ui-missing", field = at(field, key))),
        };
        if pointer::is_valid(&path) {
            Ok(path)
        } else {
            Err(fl!("ui-bad-path", field = at(field, key), path = path))
        }
    }

    fn component(&mut self, item: &Value, field: &str) -> Result<Component, String> {
        let Value::Object(obj) = item else {
            return Err(fl!("ui-wrong-type", field = field, expected = "object"));
        };
        let id = self.required(obj, "id", field)?;
        if !valid_id(&id) {
            return Err(fl!("ui-bad-id", field = at(field, "id"), id = id));
        }
        let component = self.required(obj, "component", field)?;
        let (known, kind): (&[&str], Kind) = match component.as_str() {
            "text" => (
                &["text", "style"],
                Kind::Text {
                    text: self.required(obj, "text", field)?,
                    style: match self.string(obj, "style", field)?.as_deref() {
                        None | Some("normal") => TextStyle::Normal,
                        Some("muted") => TextStyle::Muted,
                        Some("warning") => TextStyle::Warning,
                        Some("error") => TextStyle::Error,
                        Some(other) => {
                            self.warnings.push(fl!("ui-unknown-value", field = at(field, "style"), value = other));
                            TextStyle::Normal
                        }
                    },
                },
            ),
            "button" => (
                &["label", "style", "action", "requires"],
                Kind::Button {
                    label: self.required(obj, "label", field)?,
                    style: self.button_style(obj, field)?,
                    action: self.action(obj, field)?,
                    requires: match obj.get("requires") {
                        None | Some(Value::Null) => None,
                        Some(_) => Some(self.path(obj, "requires", field)?),
                    },
                },
            ),
            "image" => (
                &["path", "label", "pixelated", "background", "height"],
                Kind::Image { path: self.file_path(obj, field)?, label: self.string(obj, "label", field)?, look: self.look(obj, field)? },
            ),
            "gallery" => {
                let pick = match self.string(obj, "select", field)?.as_deref() {
                    None | Some("none") => Pick::None,
                    Some("one") => Pick::One,
                    Some("many") => Pick::Many,
                    Some(other) => {
                        self.warnings.push(fl!("ui-unknown-value", field = at(field, "select"), value = other));
                        Pick::None
                    }
                };
                let value = match pick {
                    Pick::None => None,
                    _ => Some(self.path(obj, "value", field)?),
                };
                (
                    &["items", "select", "value", "pixelated", "background", "height"],
                    Kind::Gallery { items: self.gallery(obj, field)?, pick, value, look: self.look(obj, field)? },
                )
            }
            "steps" => (&["items", "value"], Kind::Steps { items: self.steps(obj, field)?, value: self.path(obj, "value", field)? }),
            "checkbox" => (
                &["label", "value"],
                Kind::Checkbox { label: self.required(obj, "label", field)?, value: self.path(obj, "value", field)? },
            ),
            "select" => (
                &["label", "options", "value"],
                Kind::Select {
                    label: self.string(obj, "label", field)?,
                    options: self.options(obj, field)?,
                    value: self.path(obj, "value", field)?,
                },
            ),
            "markdown" => (&["text"], Kind::Markdown { text: self.required(obj, "text", field)? }),
            "file" => (
                &["path", "line", "column", "label", "open"],
                Kind::File {
                    path: self.file_path(obj, field)?,
                    line: self.number(obj, "line", field)?.unwrap_or(0),
                    column: self.number(obj, "column", field)?.unwrap_or(0),
                    label: self.string(obj, "label", field)?,
                    open: self.boolean(obj, "open", field)?,
                },
            ),
            "input" => (
                &["label", "placeholder", "value"],
                Kind::Input {
                    label: self.string(obj, "label", field)?,
                    placeholder: self.string(obj, "placeholder", field)?.unwrap_or_default(),
                    value: self.path(obj, "value", field)?,
                },
            ),
            _ => (&[], Kind::Unknown { component: component.clone() }),
        };
        if !matches!(kind, Kind::Unknown { .. }) {
            let mut all = vec!["id", "component"];
            all.extend_from_slice(known);
            self.known_fields(obj, &all, field);
        }
        Ok(Component { id, kind })
    }

    fn button_style(&mut self, obj: &Map<String, Value>, field: &str) -> Result<ButtonStyle, String> {
        Ok(match self.string(obj, "style", field)?.as_deref() {
            None | Some("primary") => ButtonStyle::Primary,
            Some("secondary") => ButtonStyle::Secondary,
            Some("danger") => ButtonStyle::Danger,
            Some(other) => {
                self.warnings.push(fl!("ui-unknown-value", field = at(field, "style"), value = other));
                ButtonStyle::Primary
            }
        })
    }

    fn action(&mut self, obj: &Map<String, Value>, field: &str) -> Result<Action, String> {
        let field = at(field, "action");
        let Some(Value::Object(action)) = obj.get("action") else {
            return Err(match obj.get("action") {
                None => fl!("ui-missing", field = field),
                Some(_) => fl!("ui-wrong-type", field = field, expected = "object"),
            });
        };
        self.known_fields(action, &["name", "context"], &field);
        let name = self.required(action, "name", &field)?;
        let context = match action.get("context") {
            None | Some(Value::Null) => Vec::new(),
            Some(Value::Array(paths)) => {
                let mut out = Vec::new();
                for (i, p) in paths.iter().enumerate() {
                    let item = format!("{field}.context[{i}]");
                    match p {
                        Value::String(p) if pointer::is_valid(p) => out.push(p.clone()),
                        Value::String(p) => return Err(fl!("ui-bad-path", field = item, path = p.clone())),
                        _ => return Err(fl!("ui-wrong-type", field = item, expected = "string")),
                    }
                }
                out
            }
            Some(_) => return Err(fl!("ui-wrong-type", field = at(&field, "context"), expected = "array")),
        };
        Ok(Action { name, context })
    }

    fn look(&mut self, obj: &Map<String, Value>, field: &str) -> Result<Look, String> {
        let checker = match self.string(obj, "background", field)?.as_deref() {
            None | Some("none") => false,
            Some("checker") => true,
            Some(other) => {
                self.warnings.push(fl!("ui-unknown-value", field = at(field, "background"), value = other));
                false
            }
        };
        let height = self.number(obj, "height", field)?.map(|h| h.clamp(8, 1024) as u32);
        Ok(Look { pixelated: self.boolean(obj, "pixelated", field)?, checker, height })
    }

    /// Itens de uma `gallery`: `{"id": "a", "path": "…", "label": "…"}`; ids únicos no componente.
    fn gallery(&mut self, obj: &Map<String, Value>, field: &str) -> Result<Vec<GalleryItem>, String> {
        let field = at(field, "items");
        let list = match obj.get("items") {
            Some(Value::Array(list)) => list,
            Some(_) => return Err(fl!("ui-wrong-type", field = field, expected = "array")),
            None => return Err(fl!("ui-missing", field = field)),
        };
        let mut out: Vec<GalleryItem> = Vec::with_capacity(list.len());
        for (i, item) in list.iter().enumerate() {
            let item_field = format!("{field}[{i}]");
            let Value::Object(o) = item else {
                return Err(fl!("ui-wrong-type", field = item_field, expected = "object"));
            };
            self.known_fields(o, &["id", "path", "label"], &item_field);
            let id = self.required(o, "id", &item_field)?;
            if !valid_id(&id) {
                return Err(fl!("ui-bad-id", field = at(&item_field, "id"), id = id));
            }
            if out.iter().any(|g| g.id == id) {
                return Err(fl!("ui-duplicate-id", id = id));
            }
            out.push(GalleryItem { id, path: self.file_path(o, &item_field)?, label: self.string(o, "label", &item_field)? });
        }
        Ok(out)
    }

    /// Passos: `{"id": "a", "text": "…", "optional": true}`; ids únicos no componente.
    fn steps(&mut self, obj: &Map<String, Value>, field: &str) -> Result<Vec<Step>, String> {
        let field = at(field, "items");
        let list = match obj.get("items") {
            Some(Value::Array(list)) => list,
            Some(_) => return Err(fl!("ui-wrong-type", field = field, expected = "array")),
            None => return Err(fl!("ui-missing", field = field)),
        };
        if list.len() > self.limits.max_options {
            return Err(fl!("ui-too-many", field = field, max = self.limits.max_options));
        }
        let mut out: Vec<Step> = Vec::with_capacity(list.len());
        for (i, item) in list.iter().enumerate() {
            let item_field = format!("{field}[{i}]");
            let Value::Object(o) = item else {
                return Err(fl!("ui-wrong-type", field = item_field, expected = "object"));
            };
            self.known_fields(o, &["id", "text", "optional"], &item_field);
            let id = self.required(o, "id", &item_field)?;
            if !valid_id(&id) {
                return Err(fl!("ui-bad-id", field = at(&item_field, "id"), id = id));
            }
            if out.iter().any(|s| s.id == id) {
                return Err(fl!("ui-duplicate-id", id = id));
            }
            let text = self.required(o, "text", &item_field)?;
            out.push(Step { id, text, optional: self.boolean(o, "optional", &item_field)? });
        }
        Ok(out)
    }

    /// Opções: `{"value": "x", "label": "X"}` ou só `"x"`.
    fn options(&mut self, obj: &Map<String, Value>, field: &str) -> Result<Vec<Choice>, String> {
        let field = at(field, "options");
        let list = match obj.get("options") {
            Some(Value::Array(list)) => list,
            Some(_) => return Err(fl!("ui-wrong-type", field = field, expected = "array")),
            None => return Err(fl!("ui-missing", field = field)),
        };
        if list.len() > self.limits.max_options {
            return Err(fl!("ui-too-many", field = field, max = self.limits.max_options));
        }
        let mut out = Vec::with_capacity(list.len());
        for (i, item) in list.iter().enumerate() {
            let item_field = format!("{field}[{i}]");
            let choice = match item {
                Value::String(v) => Choice { value: v.clone(), label: v.clone() },
                Value::Object(o) => {
                    self.known_fields(o, &["value", "label"], &item_field);
                    let value = self.required(o, "value", &item_field)?;
                    let label = self.string(o, "label", &item_field)?.unwrap_or_else(|| value.clone());
                    Choice { value, label }
                }
                _ => return Err(fl!("ui-wrong-type", field = item_field, expected = "object")),
            };
            if choice.value.len() > self.limits.max_text || choice.label.len() > self.limits.max_text {
                return Err(fl!("ui-too-long", field = item_field, max = self.limits.max_text));
            }
            out.push(choice);
        }
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn doc(v: Value) -> Result<Document, String> {
        parse(v.to_string().as_bytes(), &Limits::default())
    }

    pub(crate) fn sample() -> Value {
        json!({
            "format": "idetea-ui", "version": 1, "surface": "revisar", "revision": 3, "title": "Migração",
            "data": { "ambiente": "staging", "backup": true, "nota": "" },
            "components": [
                { "id": "intro", "component": "text", "text": "Vou aplicar 3 migrações." },
                { "id": "aviso", "component": "text", "style": "warning", "text": "A 0013 apaga uma coluna." },
                { "id": "amb", "component": "select", "label": "Ambiente",
                  "options": [{ "value": "staging", "label": "Staging" }, "producao"],
                  "value": { "path": "/ambiente" } },
                { "id": "bkp", "component": "checkbox", "label": "Backup antes", "value": { "path": "/backup" } },
                { "id": "obs", "component": "input", "label": "Observação", "value": { "path": "/nota" } },
                { "id": "ok", "component": "button", "label": "Aplicar",
                  "action": { "name": "aprovar", "context": ["/ambiente", "/backup", "/nota"] } },
                { "id": "nao", "component": "button", "label": "Não aplicar", "style": "secondary",
                  "action": { "name": "recusar" } }
            ]
        })
    }

    #[test]
    fn reads_a_full_document() {
        let d = doc(sample()).unwrap();
        assert_eq!(d.revision, 3);
        assert_eq!(d.submit, Submit::Once);
        assert_eq!(d.components.len(), 7);
        assert!(d.warnings.is_empty(), "{:?}", d.warnings);
        let Kind::Select { options, value, .. } = &d.components[2].kind else { panic!() };
        assert_eq!(options[1], Choice { value: "producao".into(), label: "producao".into() });
        assert_eq!(value, "/ambiente");
        let Kind::Button { action, style, .. } = &d.components[6].kind else { panic!() };
        assert_eq!((action.name.as_str(), action.context.len(), *style), ("recusar", 0, ButtonStyle::Secondary));
    }

    #[test]
    fn unknown_things_are_warnings() {
        let mut v = sample();
        v["cor"] = json!("azul");
        v["components"][0]["negrito"] = json!(true);
        v["components"][1]["style"] = json!("piscando");
        v["components"].as_array_mut().unwrap().push(json!({ "id": "t", "component": "table", "rows": [] }));
        let d = doc(v).unwrap();
        assert_eq!(d.warnings.len(), 3, "{:?}", d.warnings);
        assert!(d.warnings.iter().any(|w| w.contains("components[0].negrito")), "{:?}", d.warnings);
        assert_eq!(d.components.last().unwrap().kind, Kind::Unknown { component: "table".into() });
    }

    #[test]
    fn errors_name_the_field() {
        let err = |f: &dyn Fn(&mut Value)| {
            let mut v = sample();
            f(&mut v);
            doc(v).unwrap_err()
        };
        assert!(err(&|v| v["format"] = json!("a2ui")).contains("idetea-ui"));
        assert!(err(&|v| v["version"] = json!(2)).contains('2'));
        assert!(err(&|v| v["components"][3]["value"] = json!({ "path": "backup" })).contains("components[3].value"));
        assert!(err(&|v| v["components"][5]["action"]["context"] = json!(["/a", 3])).contains("context[1]"));
        assert!(err(&|v| v["components"][4]["id"] = json!("intro")).contains("intro"), "id repetido");
        assert!(err(&|v| v["components"][0]["id"] = json!("com espaço")).contains("components[0].id"));
        assert!(err(&|v| v["components"][5].as_object_mut().unwrap().remove("action").map(|_| ()).unwrap())
            .contains("components[5].action"));
        assert!(parse(b"{ quebrado", &Limits::default()).is_err());
        assert!(parse(b"[]", &Limits::default()).is_err());
    }

    #[test]
    fn file_component() {
        let mut v = sample();
        v["components"].as_array_mut().unwrap().push(json!({
            "id": "f", "component": "file", "path": "docs/nota.md", "line": 3, "column": 2, "open": true
        }));
        let d = doc(v.clone()).unwrap();
        assert!(d.warnings.is_empty(), "{:?}", d.warnings);
        assert_eq!(d.components.last().unwrap().kind, Kind::File {
            path: "docs/nota.md".into(), line: 3, column: 2, label: None, open: true
        });
        for bad in ["/etc/passwd", "../fora.txt", "a/../../b", "", "C:\\x", "\\\\srv\\x"] {
            v["components"][7]["path"] = json!(bad);
            assert!(doc(v.clone()).unwrap_err().contains("components[7].path"), "{bad}");
        }
        v["components"][7]["path"] = json!("./a.txt");
        v["components"][7]["open"] = json!("sim");
        assert!(doc(v).unwrap_err().contains("components[7].open"));
    }

    #[test]
    fn limits() {
        let small = Limits { max_bytes: 100, ..Limits::default() };
        assert!(parse(sample().to_string().as_bytes(), &small).is_err());
        let few = Limits { max_components: 3, ..Limits::default() };
        assert!(parse(sample().to_string().as_bytes(), &few).unwrap_err().contains("components"));
        let mut deep = json!(1);
        for _ in 0..20 {
            deep = json!({ "a": deep });
        }
        let mut v = sample();
        v["data"] = deep;
        assert!(doc(v).is_err(), "data fundo demais");
    }
}

#[cfg(test)]
mod examples {
    use super::*;
    use std::path::Path;

    /// Os exemplos da documentação são documentos válidos e sem avisos.
    #[test]
    fn documented_examples_parse_cleanly() {
        let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/interfaces/exemplos");
        let mut count = 0;
        for entry in std::fs::read_dir(&dir).unwrap() {
            let path = entry.unwrap().path();
            let doc = parse(&std::fs::read(&path).unwrap(), &Limits::default())
                .unwrap_or_else(|e| panic!("{}: {e}", path.display()));
            assert!(doc.warnings.is_empty(), "{}: {:?}", path.display(), doc.warnings);
            count += 1;
        }
        assert!(count >= 2);
    }
}
