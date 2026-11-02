//! As imagens (`image` e `gallery`) e os sons (`audio`) de um documento, já lidos para desenhar.
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
use crate::wav::{self, WavInfo};

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

/// Os quadros de um `sprite`, já recortados (todos do mesmo tamanho).
#[derive(Debug, Clone)]
pub struct Frames {
    pub handles: Vec<Handle>,
    pub width: u32,
    pub height: u32,
}

/// Um som lido. `info` e `wave` só existem para WAV; outros formatos só tocam.
#[derive(Debug, Clone)]
pub struct Sound {
    pub info: Option<WavInfo>,
    /// Forma de onda, já desenhada ([`WAVE_WIDTH`] × [`WAVE_HEIGHT`]).
    pub wave: Option<Handle>,
    /// O WAV não pôde ser lido (formato que o leitor não conhece); ainda toca.
    pub note: Option<String>,
}

/// Extensões que o componente `audio` aceita (o tocador do sistema recebe só estas).
pub const AUDIO_EXTENSIONS: &[&str] = &["wav", "ogg", "oga", "opus", "flac", "mp3"];
/// Tamanho máximo de um som para ler duração e forma de onda (maior que isso, só toca).
const MAX_AUDIO_BYTES: u64 = 64 * 1024 * 1024;
pub const WAVE_WIDTH: u32 = 240;
pub const WAVE_HEIGHT: u32 = 36;

/// As imagens e os sons de um documento, pelo caminho (relativo à raiz). Um erro aparece no lugar.
#[derive(Debug, Clone, Default)]
pub struct Media {
    pictures: HashMap<String, Result<Picture, String>>,
    sounds: HashMap<String, Result<Sound, String>>,
    /// Tamanho e data de cada arquivo quando foi lido (a versão na tela).
    files: HashMap<String, (u64, Option<SystemTime>)>,
    /// Pelo id do componente: dois `sprite` da mesma folha podem recortar diferente.
    sprites: HashMap<String, Result<Frames, String>>,
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
        for path in doc.image_paths().into_iter().chain(doc.audio_paths()) {
            // Antes de ler: se o arquivo mudar no meio, a data mais velha faz a próxima leitura acontecer.
            if let Ok(m) = std::fs::symlink_metadata(root.join(path)) {
                media.files.insert(path.to_string(), (m.len(), m.modified().ok()));
            }
        }
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
            if let crate::Kind::Sprite { path, frames, frame, .. } = &c.kind {
                let cut = match media.pictures.get(path.as_str()) {
                    Some(Ok(p)) => cut(p, *frames, *frame).and_then(|f| {
                        // Os quadros ocupam outra cópia da folha: contam no limite de pixels.
                        pixels += u64::from(f.width) * u64::from(f.height) * f.handles.len() as u64;
                        if pixels > limits.max_pixels {
                            Err(fl!("ui-image-total", max = (limits.max_pixels / 1_000_000)))
                        } else {
                            Ok(f)
                        }
                    }),
                    Some(Err(e)) => Err(e.clone()),
                    None => Err(fl!("ui-image-not-file")),
                };
                media.sprites.insert(c.id.clone(), cut);
            }
        }
        for path in doc.audio_paths() {
            media.sounds.insert(path.to_string(), sound(&root.join(path)));
        }
        for c in &doc.components {
            let (paths, look): (Vec<&str>, Look) = match &c.kind {
                crate::Kind::Image { path, look, .. } => (vec![path.as_str()], *look),
                crate::Kind::Sprite { look, .. } => {
                    if let (true, Some(Ok(f))) = (look.checker, media.sprites.get(&c.id)) {
                        let size = fit(f.width, f.height, look.height);
                        media.checkers.entry(size).or_insert_with(|| checker(size));
                    }
                    continue;
                }
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

    /// O que não deu para ler, para mostrar fora do painel (`idetea ui check`): caminho (ou id do `sprite`) e
    /// motivo, em ordem. Um som que só toca (sem duração) também entra.
    pub fn problems(&self) -> Vec<(String, String)> {
        let mut out: Vec<(String, String)> = Vec::new();
        out.extend(self.pictures.iter().filter_map(|(p, r)| r.as_ref().err().map(|e| (p.clone(), e.clone()))));
        out.extend(self.sprites.iter().filter_map(|(id, r)| r.as_ref().err().map(|e| (format!("#{id}"), e.clone()))));
        for (p, r) in &self.sounds {
            match r {
                Err(e) => out.push((p.clone(), e.clone())),
                Ok(Sound { note: Some(n), .. }) => out.push((p.clone(), n.clone())),
                Ok(_) => {}
            }
        }
        out.sort();
        out.dedup();
        out
    }

    /// Tamanho e data do arquivo quando foi lido para a tela.
    pub fn file(&self, path: &str) -> Option<(u64, Option<SystemTime>)> {
        self.files.get(path).copied()
    }

    /// Os quadros do `sprite` com este id.
    pub fn sprite(&self, id: &str) -> Option<&Result<Frames, String>> {
        self.sprites.get(id)
    }

    pub fn sound(&self, path: &str) -> Option<&Result<Sound, String>> {
        self.sounds.get(path)
    }

    pub fn checker(&self, size: (u32, u32)) -> Option<&Handle> {
        self.checkers.get(&size)
    }
}

/// Tamanho na tela: a altura pedida (ou a da imagem, até [`DEFAULT_HEIGHT`]), a largura na proporção, até
/// [`MAX_WIDTH`].
pub fn display_size(p: &Picture, height: Option<u32>) -> (u32, u32) {
    fit(p.width, p.height, height)
}

/// [`display_size`] para uma largura e altura quaisquer (um quadro de `sprite`).
pub fn fit(w: u32, h: u32, height: Option<u32>) -> (u32, u32) {
    let (w, h) = (w.max(1), h.max(1));
    let mut dh = height.unwrap_or_else(|| h.min(DEFAULT_HEIGHT)).max(1);
    let mut dw = (u64::from(w) * u64::from(dh) / u64::from(h)).max(1) as u32;
    if dw > MAX_WIDTH {
        dh = (u64::from(dh) * u64::from(MAX_WIDTH) / u64::from(dw)).max(1) as u32;
        dw = MAX_WIDTH;
    }
    (dw, dh)
}

/// Os arquivos de imagem e de som do documento com tamanho e data, para a thread de observação saber quando
/// reler.
pub fn stamps(root: &Path, doc: &Document) -> Vec<(PathBuf, u64, Option<SystemTime>)> {
    doc.image_paths()
        .into_iter()
        .chain(doc.audio_paths())
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

/// Recorta os quadros da folha: uma fila de `frames` quadros, ou uma grade de `frame` (largura, altura).
fn cut(sheet: &Picture, frames: u32, frame: Option<(u32, u32)>) -> Result<Frames, String> {
    let Handle::Rgba { width, height, pixels, .. } = &sheet.handle else { return Err(fl!("ui-image-not-file")) };
    let (sw, sh) = (*width, *height);
    let (fw, fh, cols, count) = match frame {
        Some((fw, fh)) => {
            let (cols, rows) = (sw / fw, sh / fh);
            (fw, fh, cols, frames.min(cols * rows))
        }
        None => (sw / frames, sh, frames, frames),
    };
    if count == 0 || fw == 0 || fh == 0 {
        return Err(fl!("ui-sprite-too-small", width = sw, height = sh));
    }
    let row_bytes = sw as usize * 4;
    let handles = (0..count)
        .map(|i| {
            let (x0, y0) = ((i % cols) * fw, (i / cols) * fh);
            let mut out = Vec::with_capacity((fw * fh * 4) as usize);
            for y in y0..y0 + fh {
                let start = y as usize * row_bytes + x0 as usize * 4;
                out.extend_from_slice(&pixels[start..start + fw as usize * 4]);
            }
            Handle::from_rgba(fw, fh, out)
        })
        .collect();
    Ok(Frames { handles, width: fw, height: fh })
}

/// Confere o arquivo (sem symlink, extensão de som) e, se for WAV, lê formato e forma de onda.
fn sound(path: &Path) -> Result<Sound, String> {
    let meta = std::fs::symlink_metadata(path).map_err(|e| fl!("ui-image-error", error = e.to_string()))?;
    if meta.file_type().is_symlink() || !meta.is_file() {
        return Err(fl!("ui-image-not-file"));
    }
    let ext = path.extension().and_then(|e| e.to_str()).map(str::to_ascii_lowercase).unwrap_or_default();
    if !AUDIO_EXTENSIONS.contains(&ext.as_str()) {
        return Err(fl!("ui-audio-extension", list = AUDIO_EXTENSIONS.join(", ")));
    }
    let mut out = Sound { info: None, wave: None, note: None };
    if ext != "wav" {
        return Ok(out);
    }
    if meta.len() > MAX_AUDIO_BYTES {
        out.note = Some(fl!("ui-audio-too-big", max = (MAX_AUDIO_BYTES / (1024 * 1024))));
        return Ok(out);
    }
    let bytes = std::fs::read(path).map_err(|e| fl!("ui-image-error", error = e.to_string()))?;
    match wav::read(&bytes, WAVE_WIDTH as usize) {
        Ok((info, columns)) => {
            out.info = Some(info);
            out.wave = Some(wave(&columns));
        }
        Err(e) => out.note = Some(fl!("ui-audio-unreadable", error = e)),
    }
    Ok(out)
}

/// A forma de onda como imagem: uma barra por coluna, do menor ao maior valor, em volta da linha do meio.
fn wave(columns: &[(f32, f32)]) -> Handle {
    let (w, h) = (WAVE_WIDTH, WAVE_HEIGHT);
    let mut pixels = vec![0u8; (w * h * 4) as usize];
    let mid = (h - 1) as f32 / 2.0;
    for (x, (lo, hi)) in columns.iter().enumerate().take(w as usize) {
        // Pelo menos um pixel: o silêncio aparece como linha.
        let top = (mid - hi * mid).floor().clamp(0.0, (h - 1) as f32) as u32;
        let bottom = (mid - lo * mid).ceil().clamp(0.0, (h - 1) as f32) as u32;
        for y in top.min(bottom)..=bottom.max(top) {
            let i = ((y * w + x as u32) * 4) as usize;
            pixels[i..i + 4].copy_from_slice(&[0x5b, 0x8d, 0xef, 0xff]);
        }
    }
    Handle::from_rgba(w, h, pixels)
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
    fn cuts_sprite_frames() {
        let root = tmp("quadros");
        // Folha 6×2 com três quadros de 2×2: vermelho, verde, azul.
        let mut sheet = ::image::RgbaImage::new(6, 2);
        for (x, _, px) in sheet.enumerate_pixels_mut() {
            *px = ::image::Rgba(match x / 2 {
                0 => [255, 0, 0, 255],
                1 => [0, 255, 0, 255],
                _ => [0, 0, 255, 255],
            });
        }
        sheet.save(root.join("fila.png")).unwrap();
        let doc = parse(
            json!({
                "format": "idetea-ui", "version": 1,
                "components": [
                    { "id": "fila", "component": "sprite", "path": "fila.png", "frames": 3, "background": "checker", "height": 20 },
                    { "id": "grade", "component": "sprite", "path": "fila.png", "frames": 9, "frame_width": 2, "frame_height": 1 },
                    { "id": "grande", "component": "sprite", "path": "fila.png", "frames": 2, "frame_width": 8, "frame_height": 8 },
                    { "id": "falta", "component": "sprite", "path": "falta.png", "frames": 2 }
                ]
            })
            .to_string()
            .as_bytes(),
            &Limits::default(),
        )
        .unwrap();
        let media = Media::load(&root, &doc, &MediaLimits::default());
        let rgba = |h: &Handle| match h {
            Handle::Rgba { pixels, .. } => pixels.to_vec(),
            _ => unreachable!(),
        };
        let fila = media.sprite("fila").unwrap().as_ref().unwrap();
        assert_eq!((fila.handles.len(), fila.width, fila.height), (3, 2, 2));
        assert_eq!(&rgba(&fila.handles[1])[..4], &[0, 255, 0, 255], "segundo quadro: verde");
        assert_eq!(&rgba(&fila.handles[2])[12..16], &[0, 0, 255, 255]);
        assert!(media.checker(fit(2, 2, Some(20))).is_some());
        let grade = media.sprite("grade").unwrap().as_ref().unwrap();
        assert_eq!(grade.handles.len(), 6, "a grade 3×2 só tem 6 quadros");
        assert_eq!(&rgba(&grade.handles[3])[..4], &[255, 0, 0, 255], "quarto quadro: começo da segunda linha");
        assert!(media.sprite("grande").unwrap().is_err(), "quadro maior que a folha");
        assert!(media.sprite("falta").unwrap().is_err());
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn loads_sounds() {
        let root = tmp("sons");
        let samples: Vec<i16> = (0..800).map(|i| if i % 2 == 0 { 8000 } else { -8000 }).collect();
        std::fs::write(root.join("a.wav"), crate::wav::pcm16(8000, 1, &samples)).unwrap();
        std::fs::write(root.join("b.ogg"), "ogg").unwrap();
        std::fs::write(root.join("c.txt"), "texto").unwrap();
        std::fs::write(root.join("d.wav"), "não é wav").unwrap();
        let doc = parse(
            json!({
                "format": "idetea-ui", "version": 1,
                "components": [
                    { "id": "a", "component": "audio", "path": "a.wav", "loop": true },
                    { "id": "b", "component": "audio", "path": "b.ogg" },
                    { "id": "c", "component": "audio", "path": "c.txt" },
                    { "id": "d", "component": "audio", "path": "d.wav" },
                    { "id": "e", "component": "audio", "path": "falta.wav" }
                ]
            })
            .to_string()
            .as_bytes(),
            &Limits::default(),
        )
        .unwrap();
        let media = Media::load(&root, &doc, &MediaLimits::default());
        let a = media.sound("a.wav").unwrap().as_ref().unwrap();
        let info = a.info.unwrap();
        assert_eq!((info.frames, info.rate), (800, 8000));
        assert!(a.wave.is_some());
        let b = media.sound("b.ogg").unwrap().as_ref().unwrap();
        assert!(b.info.is_none() && b.note.is_none(), "toca, sem metadados");
        assert!(media.sound("c.txt").unwrap().is_err(), "extensão fora da lista");
        assert!(media.sound("d.wav").unwrap().as_ref().unwrap().note.is_some(), "WAV ilegível ainda toca");
        assert!(media.sound("falta.wav").unwrap().is_err());
        assert_eq!(stamps(&root, &doc).len(), 5, "os sons entram na assinatura");
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
