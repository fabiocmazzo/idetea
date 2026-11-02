//! Leitor de WAV, só o necessário para o componente `audio`: formato, duração, pico e a forma de onda.
//!
//! PCM de 8, 16, 24 e 32 bits e float de 32 e 64 bits (também no `WAVE_FORMAT_EXTENSIBLE`). Outros formatos
//! (ADPCM, µ-law) dão erro; o som continua tocando pelo tocador do sistema.

/// O que se sabe de um WAV.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WavInfo {
    pub channels: u16,
    pub rate: u32,
    pub bits: u16,
    pub float: bool,
    /// Amostras por canal.
    pub frames: u64,
    /// Maior valor absoluto, de 0 a 1 (1 = 0 dBFS).
    pub peak: f32,
}

impl WavInfo {
    pub fn seconds(&self) -> f64 {
        if self.rate == 0 { 0.0 } else { self.frames as f64 / f64::from(self.rate) }
    }

    /// Pico em dBFS (0 é o máximo; `None`: silêncio).
    pub fn peak_db(&self) -> Option<f32> {
        (self.peak > 0.0).then(|| 20.0 * self.peak.log10())
    }

    /// Chega ao máximo: pode estar estourando.
    pub fn clips(&self) -> bool {
        self.peak >= 0.999
    }
}

/// É um WAV (pelo cabeçalho, não pela extensão).
pub fn is_wav(bytes: &[u8]) -> bool {
    bytes.len() >= 12 && &bytes[0..4] == b"RIFF" && &bytes[8..12] == b"WAVE"
}

/// Lê o WAV e devolve, para cada uma das `columns` colunas da forma de onda, o menor e o maior valor (-1 a 1).
pub fn read(bytes: &[u8], columns: usize) -> Result<(WavInfo, Vec<(f32, f32)>), String> {
    if !is_wav(bytes) {
        return Err("RIFF/WAVE".into());
    }
    let mut format = None;
    let mut data = None;
    let mut at = 12;
    while at + 8 <= bytes.len() {
        let id = &bytes[at..at + 4];
        let size = u32::from_le_bytes(bytes[at + 4..at + 8].try_into().unwrap()) as usize;
        let start = at + 8;
        // Um WAV gravado aos poucos pode dizer tamanho maior que o arquivo: usa o que existe.
        let end = start.saturating_add(size).min(bytes.len());
        match id {
            b"fmt " => format = Some(fmt(&bytes[start..end])?),
            b"data" => data = Some(&bytes[start..end]),
            _ => {}
        }
        at = start.saturating_add(size).saturating_add(size & 1);
    }
    let (channels, rate, bits, float) = format.ok_or("fmt")?;
    let data = data.ok_or("data")?;
    let width = usize::from(bits / 8);
    let frame = width * usize::from(channels);
    if frame == 0 {
        return Err("fmt".into());
    }
    let frames = data.len() / frame;
    let columns = columns.max(1);
    let mut wave = vec![(0.0f32, 0.0f32); columns];
    let mut peak = 0.0f32;
    for (i, chunk) in data.chunks_exact(frame).enumerate() {
        let col = &mut wave[i * columns / frames.max(1)];
        for s in chunk.chunks_exact(width) {
            let v = sample(s, float);
            peak = peak.max(v.abs());
            col.0 = col.0.min(v);
            col.1 = col.1.max(v);
        }
    }
    let info = WavInfo { channels, rate, bits, float, frames: frames as u64, peak: peak.min(1.0) };
    Ok((info, wave))
}

/// (canais, taxa, bits, float) do bloco `fmt `.
fn fmt(b: &[u8]) -> Result<(u16, u32, u16, bool), String> {
    if b.len() < 16 {
        return Err("fmt".into());
    }
    let u16_at = |i: usize| u16::from_le_bytes([b[i], b[i + 1]]);
    let mut tag = u16_at(0);
    if tag == 0xFFFE && b.len() >= 26 {
        tag = u16_at(24); // primeiros bytes do GUID do subformato
    }
    let channels = u16_at(2);
    let rate = u32::from_le_bytes(b[4..8].try_into().unwrap());
    let bits = u16_at(14);
    let float = match (tag, bits) {
        (1, 8 | 16 | 24 | 32) => false,
        (3, 32 | 64) => true,
        _ => return Err(format!("formato {tag}, {bits} bits")),
    };
    if channels == 0 {
        return Err("fmt".into());
    }
    Ok((channels, rate, bits, float))
}

fn sample(s: &[u8], float: bool) -> f32 {
    match (s.len(), float) {
        (1, _) => (f32::from(s[0]) - 128.0) / 128.0,
        (2, _) => f32::from(i16::from_le_bytes([s[0], s[1]])) / 32768.0,
        (3, _) => (i32::from_le_bytes([0, s[0], s[1], s[2]]) >> 8) as f32 / 8_388_608.0,
        (4, false) => i32::from_le_bytes([s[0], s[1], s[2], s[3]]) as f32 / 2_147_483_648.0,
        (4, true) => f32::from_le_bytes([s[0], s[1], s[2], s[3]]),
        (8, _) => f64::from_le_bytes(s.try_into().unwrap()) as f32,
        _ => 0.0,
    }
    .clamp(-1.0, 1.0)
}

#[cfg(test)]
pub(crate) fn pcm16(rate: u32, channels: u16, samples: &[i16]) -> Vec<u8> {
    let data: Vec<u8> = samples.iter().flat_map(|s| s.to_le_bytes()).collect();
    let mut out = Vec::new();
    out.extend_from_slice(b"RIFF");
    out.extend_from_slice(&(36 + data.len() as u32).to_le_bytes());
    out.extend_from_slice(b"WAVEfmt ");
    out.extend_from_slice(&16u32.to_le_bytes());
    out.extend_from_slice(&1u16.to_le_bytes());
    out.extend_from_slice(&channels.to_le_bytes());
    out.extend_from_slice(&rate.to_le_bytes());
    out.extend_from_slice(&(rate * u32::from(channels) * 2).to_le_bytes());
    out.extend_from_slice(&(channels * 2).to_le_bytes());
    out.extend_from_slice(&16u16.to_le_bytes());
    out.extend_from_slice(b"data");
    out.extend_from_slice(&(data.len() as u32).to_le_bytes());
    out.extend_from_slice(&data);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_pcm16() {
        // 1 s de estéreo a 8 kHz: metade em silêncio, metade no máximo positivo num canal.
        let mut samples = Vec::new();
        for i in 0..8000 {
            samples.push(if i >= 4000 { i16::MAX } else { 0 });
            samples.push(if i >= 4000 { -16384 } else { 0 });
        }
        let (info, wave) = read(&pcm16(8000, 2, &samples), 4).unwrap();
        assert_eq!((info.channels, info.rate, info.bits, info.float, info.frames), (2, 8000, 16, false, 8000));
        assert!((info.seconds() - 1.0).abs() < 1e-9);
        assert!(info.clips(), "chega a 0 dBFS");
        assert!(info.peak_db().unwrap().abs() < 0.01);
        assert_eq!(wave[0], (0.0, 0.0), "começo em silêncio");
        assert!(wave[3].0 <= -0.49 && wave[3].1 >= 0.99);
    }

    #[test]
    fn other_widths_and_errors() {
        assert_eq!(sample(&[0xFF, 0xFF, 0x7F], false), 8_388_607.0 / 8_388_608.0);
        assert_eq!(sample(&[0x00, 0x00, 0x80], false), -1.0);
        assert_eq!(sample(&[128], false), 0.0);
        assert_eq!(sample(&0.5f32.to_le_bytes(), true), 0.5);
        assert!(read(b"nada", 4).is_err());
        let mut adpcm = pcm16(8000, 1, &[0; 10]);
        adpcm[20] = 2; // formato 2: ADPCM
        assert!(read(&adpcm, 4).unwrap_err().contains("formato 2"));
        // Tamanho do `data` maior que o arquivo: lê o que existe.
        let mut cut = pcm16(8000, 1, &[100; 10]);
        let n = cut.len();
        cut[n - 24..n - 20].copy_from_slice(&1000u32.to_le_bytes());
        assert_eq!(read(&cut, 4).unwrap().0.frames, 10);
        let silent = read(&pcm16(8000, 1, &[0; 10]), 4).unwrap().0;
        assert_eq!(silent.peak_db(), None);
    }
}
