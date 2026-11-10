//! Aba de PDF: as páginas uma embaixo da outra, desenhadas pelo `hayro` (puro Rust) numa thread.
//!
//! Só as páginas que aparecem (ou estão quase aparecendo) são desenhadas, e só algumas ficam na
//! memória. Zoom pelos botões; "Ajustar" acompanha a largura da aba.

use std::collections::{BTreeSet, HashMap};
use std::io;
use std::path::Path;
use std::sync::{Arc, Condvar, Mutex};

use hayro::hayro_interpret::InterpreterSettings;
use hayro::hayro_syntax::Pdf;
use hayro::vello_cpu::color::palette::css::WHITE;
use hayro::{PixmapSettings, RenderCache, RenderSettings};
use iced::widget::{Space, button, column, container, image, row, scrollable, sensor, text};
use iced::{Center, Element, Fill, Length, Size, Task, Theme};

pub fn is_pdf(path: &Path) -> bool {
    path.extension().is_some_and(|e| e.eq_ignore_ascii_case("pdf"))
}

/// Pontos do PDF (1/72 pol.) para pixels lógicos (1/96 pol.) com zoom 100%.
const PT_TO_PX: f32 = 96.0 / 72.0;
/// Desenha com mais pixels que o tamanho na tela: texto nítido em telas de alta densidade.
const OVERSAMPLE: f32 = 1.5;
/// Lado máximo da imagem de uma página (zoom alto não estoura a memória).
const MAX_SIDE: f32 = 6000.0;
/// Páginas desenhadas guardadas (as mais longe da última vista saem primeiro).
const KEEP_PAGES: usize = 12;
const GAP: f32 = 12.0;
const MIN_ZOOM: f32 = 0.25;
const MAX_ZOOM: f32 = 5.0;

#[derive(Debug, Clone)]
pub enum Msg {
    /// Página entrou na tela (ou está quase).
    Shown(usize),
    Hidden(usize),
    Rendered(Arc<Rendered>),
    ZoomIn,
    ZoomOut,
    Fit,
    /// Largura da área das páginas mudou.
    Resized(Size),
}

#[derive(Debug)]
pub struct Rendered {
    page: usize,
    generation: u64,
    handle: image::Handle,
}

/// Pedido para a thread que desenha.
struct Job {
    page: usize,
    scale: f32,
    generation: u64,
    reply: iced::futures::channel::oneshot::Sender<Arc<Rendered>>,
}

#[derive(Default)]
struct Queue {
    jobs: Vec<Job>,
    /// Pedidos de outra geração (zoom antigo) são jogados fora.
    generation: u64,
    closed: bool,
}

struct Worker {
    queue: Mutex<Queue>,
    wake: Condvar,
}

impl Worker {
    fn lock(&self) -> std::sync::MutexGuard<'_, Queue> {
        self.queue.lock().unwrap_or_else(|e| e.into_inner())
    }
}

pub struct PdfFile {
    /// Tamanho de cada página em pontos (já girada), ou o motivo de não abrir.
    pages: Result<Vec<(f32, f32)>, String>,
    worker: Option<Arc<Worker>>,
    zoom: f32,
    /// Acompanhando a largura da aba (até o usuário escolher um zoom).
    fit: bool,
    width: f32,
    /// Muda a cada zoom: desenhos e pedidos antigos deixam de valer.
    generation: u64,
    /// Página desenhada → (geração em que foi desenhada, imagem).
    images: HashMap<usize, (u64, image::Handle)>,
    /// Pedidas e ainda não chegaram (nesta geração).
    pending: BTreeSet<usize>,
    visible: BTreeSet<usize>,
    bytes: usize,
}

impl Drop for PdfFile {
    fn drop(&mut self) {
        if let Some(w) = &self.worker {
            w.lock().closed = true;
            w.wake.notify_all();
        }
    }
}

impl PdfFile {
    pub fn read(path: &Path) -> io::Result<Self> {
        let bytes = std::fs::read(path)?;
        let len = bytes.len();
        let pages = match Pdf::new(bytes.clone()) {
            Ok(pdf) => Ok(pdf.pages().iter().map(|p| p.render_dimensions()).collect::<Vec<_>>()),
            Err(e) => Err(describe(&e)),
        };
        let worker = pages.as_ref().ok().map(|_| spawn_worker(bytes));
        Ok(Self {
            pages,
            worker,
            zoom: 1.0,
            fit: true,
            width: 0.0,
            generation: 0,
            images: HashMap::new(),
            pending: BTreeSet::new(),
            visible: BTreeSet::new(),
            bytes: len,
        })
    }

    pub fn update(&mut self, msg: Msg) -> Task<Msg> {
        match msg {
            Msg::Shown(page) => {
                self.visible.insert(page);
                return self.request(page);
            }
            Msg::Hidden(page) => {
                self.visible.remove(&page);
            }
            Msg::Rendered(r) => {
                if r.generation == self.generation {
                    self.pending.remove(&r.page);
                }
                // Desenho de um zoom antigo também serve até chegar o novo (só fica menos nítido).
                let newer = self.images.get(&r.page).is_some_and(|(g, _)| *g > r.generation);
                if !newer {
                    self.images.insert(r.page, (r.generation, r.handle.clone()));
                    self.evict(r.page);
                }
            }
            Msg::ZoomIn => return self.set_zoom(self.zoom * 1.25, false),
            Msg::ZoomOut => return self.set_zoom(self.zoom / 1.25, false),
            Msg::Fit => return self.set_zoom(self.fit_zoom(), true),
            Msg::Resized(size) => {
                self.width = size.width;
                if self.fit {
                    return self.set_zoom(self.fit_zoom(), true);
                }
            }
        }
        Task::none()
    }

    /// Zoom em que a página mais larga ocupa a largura da aba.
    fn fit_zoom(&self) -> f32 {
        let widest = self.pages.as_ref().ok().and_then(|p| p.iter().map(|(w, _)| *w).reduce(f32::max));
        match widest {
            Some(w) if self.width > 0.0 && w > 0.0 => (self.width - 2.0 * GAP - 16.0) / (w * PT_TO_PX),
            _ => 1.0,
        }
    }

    fn set_zoom(&mut self, zoom: f32, fit: bool) -> Task<Msg> {
        let zoom = zoom.clamp(MIN_ZOOM, MAX_ZOOM);
        self.fit = fit;
        if (zoom - self.zoom).abs() < 0.005 {
            return Task::none();
        }
        self.zoom = zoom;
        self.generation += 1;
        self.pending.clear();
        if let Some(w) = &self.worker {
            let mut q = w.lock();
            q.generation = self.generation;
            q.jobs.clear();
        }
        // Os sensores mudam de chave e avisam de novo quais páginas estão na tela: elas são pedidas aí.
        self.visible.clear();
        Task::none()
    }

    fn request(&mut self, page: usize) -> Task<Msg> {
        let current = self.images.get(&page).is_some_and(|(g, _)| *g == self.generation);
        let Some(worker) = &self.worker else { return Task::none() };
        if current || !self.pending.insert(page) {
            return Task::none();
        }
        let Some((w, h)) = self.pages.as_ref().ok().and_then(|p| p.get(page).copied()) else { return Task::none() };
        let mut scale = self.zoom * PT_TO_PX * OVERSAMPLE;
        scale = scale.min(MAX_SIDE / w.max(h).max(1.0));
        let (tx, rx) = iced::futures::channel::oneshot::channel();
        worker.lock().jobs.push(Job { page, scale, generation: self.generation, reply: tx });
        worker.wake.notify_one();
        Task::future(async move { rx.await.ok() }).and_then(|r| Task::done(Msg::Rendered(r)))
    }

    /// Guarda só as páginas mais perto da que acabou de chegar (e as que estão na tela).
    fn evict(&mut self, near: usize) {
        while self.images.len() > KEEP_PAGES {
            let far = self
                .images
                .keys()
                .copied()
                .filter(|p| !self.visible.contains(p))
                .max_by_key(|p| p.abs_diff(near));
            match far {
                Some(p) => {
                    self.images.remove(&p);
                }
                None => break,
            }
        }
    }

    pub fn view(&self) -> Element<'_, Msg> {
        let pages = match &self.pages {
            Ok(p) => p,
            Err(e) => {
                return container(text(fl!("pdf-error", error = e.as_str())).size(13).style(text::secondary))
                    .padding(12)
                    .into();
            }
        };
        let current = self.visible.first().map_or(1, |p| p + 1);
        let small = |label: &'static str, msg: Msg| button(text(label).size(13)).on_press(msg).style(button::text).padding([0, 8]);
        let toolbar = row![
            text(fl!("pdf-page", current = current, total = pages.len())).size(12).style(text::secondary),
            Space::new().width(Fill),
            small("−", Msg::ZoomOut),
            text(format!("{:.0}%", self.zoom * 100.0)).size(12).width(44).align_x(Center),
            small("+", Msg::ZoomIn),
            button(text(fl!("pdf-fit")).size(12))
                .on_press(Msg::Fit)
                .padding([1, 8])
                .style(if self.fit { button::secondary } else { button::text }),
            text(format!("· {}", human_size(self.bytes))).size(12).style(text::secondary),
        ]
        .spacing(4)
        .align_y(Center)
        .padding([2, 6]);

        let mut list = column![].spacing(GAP).padding(GAP).align_x(Center).width(Fill);
        for (i, (w, h)) in pages.iter().enumerate() {
            let (w, h) = (w * PT_TO_PX * self.zoom, h * PT_TO_PX * self.zoom);
            let content: Element<'_, Msg> = match self.images.get(&i) {
                Some((_, handle)) => {
                    image(handle.clone()).width(w).height(h).filter_method(image::FilterMethod::Linear).into()
                }
                None => container(text("…").size(14).style(text::secondary))
                    .center(Fill)
                    .width(w)
                    .height(h)
                    .style(|_: &Theme| container::Style::default().background(iced::Color::WHITE))
                    .into(),
            };
            // A chave muda com o zoom: as páginas na tela avisam de novo e são redesenhadas.
            let page = sensor(content)
                .key(self.generation)
                .on_show(move |_| Msg::Shown(i))
                .on_hide(Msg::Hidden(i))
                .anticipate(400);
            list = list.push(page);
        }
        let pages = tea_core::style::scrollable(list).direction(scrollable::Direction::Both {
            vertical: tea_core::style::scrollbar(),
            horizontal: tea_core::style::scrollbar(),
        });
        let area = sensor(container(pages).width(Fill).height(Fill).style(|t: &Theme| {
            container::Style::default().background(t.extended_palette().background.weak.color)
        }))
        .on_resize(Msg::Resized);
        column![toolbar, area].spacing(2).width(Fill).height(Length::Fill).into()
    }
}

fn describe(e: &hayro::hayro_syntax::LoadPdfError) -> String {
    match e {
        hayro::hayro_syntax::LoadPdfError::Decryption(_) => fl!("pdf-encrypted"),
        _ => fl!("pdf-invalid"),
    }
}

/// Thread que desenha as páginas, sempre a pedida por último primeiro (a que o usuário está vendo).
fn spawn_worker(bytes: Vec<u8>) -> Arc<Worker> {
    let worker = Arc::new(Worker { queue: Mutex::new(Queue::default()), wake: Condvar::new() });
    let shared = worker.clone();
    let _ = std::thread::Builder::new().name("pdf".into()).spawn(move || {
        let Ok(pdf) = Pdf::new(bytes) else { return };
        let cache = RenderCache::new();
        let interpreter = InterpreterSettings::default();
        let render = RenderSettings::default();
        loop {
            let job = {
                let mut q = shared.lock();
                loop {
                    if q.closed {
                        return;
                    }
                    let generation = q.generation;
                    q.jobs.retain(|j| j.generation == generation && !j.reply.is_canceled());
                    if let Some(job) = q.jobs.pop() {
                        break job;
                    }
                    q = shared.wake.wait(q).unwrap_or_else(|e| e.into_inner());
                }
            };
            let Some(page) = pdf.pages().get(job.page) else { continue };
            let settings = PixmapSettings { x_scale: job.scale, y_scale: job.scale, bg_color: WHITE };
            let pixmap = hayro::render(page, &cache, &interpreter, &render, &settings);
            // Fundo branco opaco: o RGBA pré-multiplicado é igual ao normal.
            let handle = image::Handle::from_rgba(
                u32::from(pixmap.width()),
                u32::from(pixmap.height()),
                pixmap.data_as_u8_slice().to_vec(),
            );
            let _ = job.reply.send(Arc::new(Rendered { page: job.page, generation: job.generation, handle }));
        }
    });
    worker
}

fn human_size(bytes: usize) -> String {
    let b = bytes as f64;
    if b < 1024.0 * 1024.0 { format!("{:.0} KB", b / 1024.0) } else { format!("{:.1} MB", b / (1024.0 * 1024.0)) }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// PDF mínimo com `n` páginas A4 (595 × 842 pt), cada uma com um texto.
    fn sample_pdf(n: usize) -> Vec<u8> {
        let mut objs: Vec<String> = Vec::new();
        let kids: Vec<String> = (0..n).map(|i| format!("{} 0 R", 4 + 2 * i)).collect();
        objs.push("<< /Type /Catalog /Pages 2 0 R >>".into());
        objs.push(format!("<< /Type /Pages /Kids [{}] /Count {n} >>", kids.join(" ")));
        objs.push("<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>".into());
        for i in 0..n {
            objs.push(format!(
                "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 595 842] /Resources << /Font << /F1 3 0 R >> >> /Contents {} 0 R >>",
                5 + 2 * i
            ));
            let stream = format!("BT /F1 24 Tf 72 720 Td (Pagina {}) Tj ET", i + 1);
            objs.push(format!("<< /Length {} >>\nstream\n{stream}\nendstream", stream.len()));
        }
        let mut out = String::from("%PDF-1.4\n");
        let mut offsets = Vec::new();
        for (i, o) in objs.iter().enumerate() {
            offsets.push(out.len());
            out.push_str(&format!("{} 0 obj\n{o}\nendobj\n", i + 1));
        }
        let xref = out.len();
        out.push_str(&format!("xref\n0 {}\n0000000000 65535 f \n", objs.len() + 1));
        for off in offsets {
            out.push_str(&format!("{off:010} 00000 n \n"));
        }
        out.push_str(&format!("trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n", objs.len() + 1));
        out.into_bytes()
    }

    #[test]
    fn reads_pages_and_renders_in_the_worker() {
        let dir = std::env::temp_dir().join(format!("idetea-pdf-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("a.pdf");
        std::fs::write(&path, sample_pdf(3)).unwrap();
        let pdf = PdfFile::read(&path).unwrap();
        assert_eq!(pdf.pages.as_ref().unwrap(), &vec![(595.0, 842.0); 3]);

        let worker = pdf.worker.as_ref().unwrap();
        let (tx, rx) = iced::futures::channel::oneshot::channel();
        worker.lock().jobs.push(Job { page: 1, scale: 1.0, generation: 0, reply: tx });
        worker.wake.notify_one();
        let start = std::time::Instant::now();
        let r = iced::futures::executor::block_on(rx).unwrap();
        eprintln!("página desenhada em {:?}", start.elapsed());
        assert_eq!(r.page, 1);
        let image::Handle::Rgba { width, height, pixels, .. } = &r.handle else { panic!("não é RGBA") };
        assert_eq!((*width, *height), (595, 842));
        // Tem texto: algum pixel não é branco.
        assert!(pixels.chunks(4).any(|p| p[0] < 128), "página em branco");

        std::fs::write(&path, b"%PDF-1.4 lixo").unwrap();
        assert!(PdfFile::read(&path).unwrap().pages.is_err());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn zoom_drops_old_requests_and_fit_follows_the_width() {
        let dir = std::env::temp_dir().join(format!("idetea-pdf-zoom-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("a.pdf");
        std::fs::write(&path, sample_pdf(2)).unwrap();
        let mut pdf = PdfFile::read(&path).unwrap();
        let _ = pdf.update(Msg::Resized(Size::new(595.0 * PT_TO_PX + 2.0 * GAP + 16.0, 500.0)));
        assert!((pdf.zoom - 1.0).abs() < 0.01 && pdf.fit, "ajusta à largura: {}", pdf.zoom);
        let _ = pdf.update(Msg::ZoomIn);
        assert!(!pdf.fit && (pdf.zoom - 1.25).abs() < 0.01);
        let generation = pdf.generation;
        let _ = pdf.update(Msg::Resized(Size::new(300.0, 500.0)));
        assert_eq!(pdf.generation, generation, "zoom escolhido: mudar a largura não muda o zoom");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
