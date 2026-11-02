//! Aba de imagem: mostra o arquivo com zoom (roda do mouse) e arrastar, em vez do texto.

use std::io;
use std::path::Path;

use iced::widget::{column, container, image, row, text};
use iced::{Element, Fill, Theme};

/// Tipos que abrem como imagem (SVG é texto e tem a prévia).
const EXTENSIONS: &[&str] = &["png", "jpg", "jpeg", "gif", "webp", "bmp", "ico", "tif", "tiff", "tga", "qoi"];

pub fn is_image(path: &Path) -> bool {
    let ext = path.extension().map(|e| e.to_string_lossy().to_lowercase()).unwrap_or_default();
    EXTENSIONS.contains(&ext.as_str())
}

pub struct ImageFile {
    handle: image::Handle,
    /// Largura e altura, ou o motivo de não ter conseguido ler.
    size: Result<(u32, u32), String>,
    bytes: usize,
}

impl ImageFile {
    pub fn read(path: &Path) -> io::Result<Self> {
        // Os bytes, não o caminho: se o arquivo mudar no disco, a aba recarregada mostra a versão nova
        // (pelo caminho, o iced reaproveitaria a imagem antiga do cache).
        let bytes = std::fs::read(path)?;
        let size = ::image::ImageReader::new(io::Cursor::new(&bytes))
            .with_guessed_format()
            .map_err(|e| e.to_string())
            .and_then(|r| r.into_dimensions().map_err(|e| e.to_string()));
        Ok(Self { bytes: bytes.len(), handle: image::Handle::from_bytes(bytes), size })
    }

    pub fn view<'a, M: 'a>(&'a self) -> Element<'a, M> {
        let info = match &self.size {
            Ok((w, h)) => fl!("image-info", width = w, height = h, size = human_size(self.bytes)),
            Err(e) => fl!("image-error", error = e.as_str()),
        };
        // Imagem pequena (sprite, ícone): vizinho mais próximo, para a pixel art não borrar no zoom.
        let small = self.size.as_ref().is_ok_and(|(w, h)| *w <= SMALL && *h <= SMALL);
        let filter = if small { image::FilterMethod::Nearest } else { image::FilterMethod::Linear };
        let viewer = image::viewer(self.handle.clone())
            .width(Fill)
            .height(Fill)
            .min_scale(0.05)
            .max_scale(40.0)
            .filter_method(filter);
        let canvas = container(viewer).width(Fill).height(Fill).style(|t: &Theme| {
            container::Style::default().background(t.extended_palette().background.weak.color)
        });
        column![row![text(info).size(12).style(text::secondary)].padding([2, 6]), canvas].spacing(2).into()
    }
}

/// Até este lado (em pixels), a imagem é desenhada com o vizinho mais próximo.
const SMALL: u32 = 256;

fn human_size(bytes: usize) -> String {
    let b = bytes as f64;
    if b < 1024.0 {
        format!("{bytes} B")
    } else if b < 1024.0 * 1024.0 {
        format!("{:.1} KB", b / 1024.0)
    } else {
        format!("{:.1} MB", b / (1024.0 * 1024.0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn knows_image_files_and_reads_the_size() {
        assert!(is_image(Path::new("/p/Logo.PNG")));
        assert!(!is_image(Path::new("/p/icone.svg")));
        assert!(!is_image(Path::new("/p/main.rs")));

        // PNG 2×3 gerado na hora.
        let dir = std::env::temp_dir().join(format!("idetea-img-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("a.png");
        ::image::RgbaImage::new(2, 3).save(&path).unwrap();
        let img = ImageFile::read(&path).unwrap();
        assert_eq!(img.size, Ok((2, 3)));

        std::fs::write(&path, b"isto nao e png").unwrap();
        assert!(ImageFile::read(&path).unwrap().size.is_err());
        let _ = std::fs::remove_dir_all(&dir);
        assert_eq!(human_size(2048), "2.0 KB");
    }
}
