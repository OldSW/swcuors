use std::time::{Duration, Instant};

const TYPING_DELAY: Duration = Duration::from_secs(5);
const TYPING_COUNT_THRESHOLD: u32 = 10;
const TYPING_TEXT: &str = "[typing]\0";

pub struct TypingIndicator {
    last_packet: Option<Instant>,
    count: u32,
}

impl TypingIndicator {
    pub const fn new() -> Self {
        Self { last_packet: None, count: 0 }
    }

    /// Call on every SDL_EVENT_KEY_UP.
    pub fn update(&mut self) {
        let elapsed_past_delay = match self.last_packet {
            None => true, // never sent — treat as "infinitely overdue"
            Some(t) => t.elapsed() > TYPING_DELAY,
        };

        if elapsed_past_delay && self.count > TYPING_COUNT_THRESHOLD {
            self.last_packet = Some(Instant::now());
            self.count = 0;
            Self::send_typing_packet();
        }

        self.count += 1;
    }

    /// Packet 0xAD — Unicode Speech Request (client → server).
    /// Type 0 = regular, font 3, colour 0x0026, text "[typing]" in UTF-16 BE.
    fn send_typing_packet() {
        let text_bytes = TYPING_TEXT.encode_utf16().flat_map(|c| c.to_be_bytes()).collect::<Vec<_>>();

        // headerSize = 13 matches the original; text is placed at byte 12 and
        // the extra zero at the end comes from the vec initialisation.
        let header_size: usize = 13;
        let total_length = header_size + text_bytes.len();

        let mut packet = vec![0u8; total_length];
        let mut i = 0;

        packet[i] = 0xAD;                            i += 1; // packet id
        packet[i] = (total_length >> 8) as u8;       i += 1; // length high
        packet[i] = (total_length & 0xFF) as u8;     i += 1; // length low
        packet[i] = 0x00;                            i += 1; // type: regular
        packet[i] = 0x00; packet[i + 1] = 0x03;     i += 2; // font: 3
        packet[i] = 0x00; packet[i + 1] = 0x26;     i += 2; // colour: 0x0026
        packet[i] = 0x00;                            i += 1; // language[0]
        packet[i] = 0x00; packet[i + 1] = 0x00;     i += 2; // language[1,2]
        packet[i] = 0x00;                            i += 1; // language[3]
        // byte at i (=12) stays 0x00; text begins at i via Array.Copy equivalent
        packet[i..i + text_bytes.len()].copy_from_slice(&text_bytes);

        crate::inject_to_server(&mut packet);
    }
}
