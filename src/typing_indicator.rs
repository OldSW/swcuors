use std::time::{Duration, Instant};

const TYPING_DELAY: Duration = Duration::from_secs(5);
const TYPING_COUNT_THRESHOLD: u32 = 10;

pub struct TypingIndicator {
    last_packet: Option<Instant>,
    count: u32,
}

impl TypingIndicator {
    pub const fn new() -> Self {
        Self { last_packet: None, count: 0 }
    }

    pub fn reset(&mut self) {
        self.count = 0;
        self.last_packet = None;
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

    /// Packet 0xBF — Unicode Speech Request (client → server).
    /// SubCommand EF.
    fn send_typing_packet() {

        // headerSize = 4
        let header_size: usize = 5;
        let total_length = header_size;

        let mut packet = vec![0u8; total_length];
        let mut i = 0;

        packet[i] = 0xBF;                            i += 1; // packet id
        packet[i] = (total_length >> 8) as u8;       i += 1; // length high
        packet[i] = (total_length & 0xFF) as u8;     i += 1; // length low
        packet[i] = 0x00; packet[i + 1] = 0xEF;              // subcommand

        crate::inject_to_server(&mut packet);
    }
}
