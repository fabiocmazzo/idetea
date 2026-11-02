//! As imagens de um documento (`image` e `gallery`), já lidas para desenhar.
//!
//! Quem lê é a extensão, numa thread fora da interface: o arquivo é aberto, o tamanho é conferido pelo cabeçalho
//! antes de decodificar (um PNG pode pedir dimensões enormes) e a imagem vira RGBA. O resultado ([`Media`]) vai
//! para a thread da interface pronto; lá só se desenha.

use std::collections::HashMap;
use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use iced::widget::image::Handle;

use crate::document::{Document, Look};

/// Limites para ler as imagens de um documento.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MediaLimits {
    /// Tamanho do arquivo, em bytes.
    pub max_bytes: u64,
    /// Largura ou altura.
    pub max_side: u32,
    /// Pixels somados de todas as imagens do documento.
    pub max_pixels: u64,
}

impl Default for MediaLimits {
    fn default() -> Self {
        Self { max_bytes: 8 * 1024 * 1024, max_side: 4096, max_pixels: 64 * 1024 * 1024 }
    }
}

/// Uma imagem lida.
#[derive(Debug, Clone)]
pub struct Picture {
    pub handle: Handle,
    pub width: u32,
    pub height: u32,
}

/// As imagens de um documento, pelo caminho (relativo à raiz). Um erro aparece no lugar da imagem.
#[derive(Debug, Clone, Default)]
pub struct Media {
    pictures: HashMap<String, Result<Picture, String>>,
    /// Fundos quadriculados, pelo tamanho na tela.
    checkers: HashMap<(u32, u32), Handle>,
}

/// Altura padrão na tela, quando o documento não diz (imagens menores ficam no tamanho delas).
const DEFAULT_HEIGHT: u32 = 160;
/// Largura máxima na tela (o painel é estreito).
const MAX_WIDTH: u32 = 640;
/// Lado de cada quadrado do fundo quadriculado, em pixels da tela.
const CHECKER: u32 = 8;

impl Media {
    /// Lê as imagens do documento. Chame fora da thread da interface.
    pub fn load(root: &Path, doc: &Document, limits: &MediaLimits) -> Media {
        let mut media = Media::default();
        let mut pixels = 0u64;
        for path in doc.image_paths() {
            let picture = read(&root.join(path), limits).and_then(|p| {
                pixels += u64::from(p.width) * u64::from(p.height);
                if pixels > limits.max_pixels {
                    Err(fl!("ui-image-total", max = (limits.max_pixels / 1_000_000)))
                } else {
                    Ok(p)
                }
            });
            media.pictures.insert(path.to_string(), picture);
        }
        for c in &doc.components {
            let (paths, look): (Vec<&str>, Look) = match &c.kind {
                crate::Kind::Image { path, look, .. } => (vec![path.as_str()], *look),
                crate::Kind::Gallery { items, look, .. } => (items.iter().map(|i| i.path.as_str()).collect(), *look),
                _ => continue,
            };
            if !look.checker {
                continue;
            }
            for path in paths {
                if let Some(Ok(p)) = media.pictures.get(path) {
                    let size = display_size(p, look.height);
                    media.checkers.entry(size).or_insert_with(|| checker(size));
                }
            }
        }
        media
    }

    pub fn picture(&self, path: &str) -> Option<&Result<Picture, String>> {
        self.pictures.get(path)
    }

    pub fn checker(&self, size: (u32, u32)) -> Option<&Handle> {
        self.checkers.get(&size)
    }
}

/// Tamanho na tela: a altura pedida (ou a da imagem, até [`DEFAULT_HEIGHT`]), a largura na proporção, até
/// [`MAX_WIDTH`].
pub fn display_size(p: &Picture, height: Option<u32>) -> (u32, u32) {
    let (w, h) = (p.width.max(1), p.height.max(1));
    let mut dh = height.unwrap_or_else(|| h.min(DEFAULT_HEIGHT)).max(1);
    let mut dw = (u64::from(w) * u64::from(dh) / u64::from(h)).max(1) as u32;
    if dw > MAX_WIDTH {
        dh = (u64::from(dh) * u64::from(MAX_WIDTH) / u64::from(dw)).max(1) as u32;
        dw = MAX_WIDTH;
    }
    (dw, dh)
}

/// Os arquivos de imagem do documento com tamanho e data, para a thread de observação saber quando reler.
pub fn stamps(root: &Path, doc: &Document) -> Vec<(PathBuf, u64, Option<SystemTime>)> {
    doc.image_paths()
        .into_iter()
        .map(|p| {
            let path = root.join(p);
            let meta = std::fs::metadata(&path).ok();
            let (len, modified) = meta.map_or((0, None), |m| (m.len(), m.modified().ok()));
            (path, len, modified)
        })
        .collect()
}

fn read(path: &Path, limits: &MediaLimits) -> Result<Picture, String> {
    // Symlink: não segue (poderia apontar para fora do projeto).
    let meta = std::fs::symlink_metadata(path).map_err(|e| fl!("ui-image-error", error = e.to_string()))?;
    if meta.file_type().is_symlink() || !meta.is_file() {
        return Err(fl!("ui-image-not-file"));
    }
    if meta.len() > limits.max_bytes {
        return Err(fl!("ui-image-too-big", max = (limits.max_bytes / (1024 * 1024))));
    }
    let bytes = std::fs::read(path).map_err(|e| fl!("ui-image-error", error = e.to_string()))?;
    let reader = ::image::ImageReader::new(Cursor::new(&bytes))
        .with_guessed_format()
        .map_err(|e| fl!("ui-image-error", error = e.to_string()))?;
    let (w, h) = reader.into_dimensions().map_err(|e| fl!("ui-image-error", error = e.to_string()))?;
    if w > limits.max_side || h > limits.max_side {
        return Err(fl!("ui-image-too-large", width = w, height = h, max = limits.max_side));
    }
    let decoded = ::image::load_from_memory(&bytes).map_err(|e| fl!("ui-image-error", error = e.to_string()))?;
    let rgba = decoded.into_rgba8();
    let (width, height) = rgba.dimensions();
    Ok(Picture { handle: Handle::from_rgba(width, height, rgba.into_raw()), width, height })
}

/// Fundo quadriculado do tamanho da imagem na tela (cinza claro e escuro).
fn checker((w, h): (u32, u32)) -> Handle {
    let mut pixels = Vec::with_capacity((w * h * 4) as usize);
    for y in 0..h {
        for x in 0..w {
            let light = ((x / CHECKER) + (y / CHECKER)).is_multiple_of(2);
            let v = if light { 0xcc } else { 0x99 };
            pixels.extend_from_slice(&[v, v, v, 0xff]);
        }
    }
    Handle::from_rgba(w, h, pixels)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::{Limits, parse};
    use serde_json::json;

    fn tmp(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("idetea-media-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn png(path: &Path, w: u32, h: u32) {
        ::image::RgbaImage::from_pixel(w, h, ::image::Rgba([255, 0, 0, 128])).save(path).unwrap();
    }

    #[test]
    fn loads_with_limits() {
        let root = tmp("limites");
        png(&root.join("a.png"), 3, 2);
        png(&root.join("grande.png"), 50, 10);
        std::fs::write(root.join("nao.png"), "texto").unwrap();
        let doc = parse(
            json!({
                "format": "idetea-ui", "version": 1,
                "components": [
                    { "id": "i", "component": "image", "path": "a.png", "background": "checker", "height": 20 },
                    { "id": "g", "component": "gallery", "items": [
                        { "id": "x", "path": "grande.png" }, { "id": "y", "path": "nao.png" },
                        { "id": "z", "path": "falta.png" }, { "id": "w", "path": "a.png" } ] }
                ]
            })
            .to_string()
            .as_bytes(),
            &Limits::default(),
        )
        .unwrap();
        assert_eq!(doc.image_paths(), vec!["a.png", "grande.png", "nao.png", "falta.png"], "sem repetir");
        let limits = MediaLimits { max_side: 40, ..MediaLimits::default() };
        let media = Media::load(&root, &doc, &limits);
        let a = media.picture("a.png").unwrap().as_ref().unwrap();
        assert_eq!((a.width, a.height), (3, 2));
        assert_eq!(display_size(a, Some(20)), (30, 20));
        assert!(media.checker((30, 20)).is_some(), "fundo do tamanho da imagem na tela");
        for bad in ["grande.png", "nao.png", "falta.png"] {
            assert!(media.picture(bad).unwrap().is_err(), "{bad}");
        }
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn sizes_on_screen() {
        let p = |w, h| Picture { handle: Handle::from_rgba(1, 1, vec![0; 4]), width: w, height: h };
        assert_eq!(display_size(&p(16, 16), None), (16, 16), "pequena fica no tamanho dela");
        assert_eq!(display_size(&p(1000, 1000), None), (160, 160));
        assert_eq!(display_size(&p(48, 16), Some(64)), (192, 64), "spritesheet: a largura acompanha");
        assert_eq!(display_size(&p(4000, 100), Some(100)), (640, 16), "larga demais: encolhe");
    }
}
