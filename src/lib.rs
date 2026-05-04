use std::ffi::{c_char, c_int, c_void};

mod typing_indicator;

// ---------------------------------------------------------------------------
// SDL3 event types — mirrors SDL3's C structs (SDL3.Core.cs bindings).
// SDL_Event is a union padded to 128 bytes; all variants start at offset 0.
// ---------------------------------------------------------------------------
#[allow(non_camel_case_types)]
pub mod sdl3 {
    // Common header shared by every event variant.
    #[repr(C)]
    #[derive(Copy, Clone)]
    pub struct SDL_CommonEvent {
        pub event_type: u32,
        pub reserved:   u32,
        pub timestamp:  u64,
    }

    #[repr(C)]
    #[derive(Copy, Clone)]
    pub struct SDL_WindowEvent {
        pub event_type: u32,
        pub reserved:   u32,
        pub timestamp:  u64,
        pub window_id:  u32,
        pub data1:      i32,
        pub data2:      i32,
    }

    /// scancode = SDL_Scancode (C enum ≅ u32), mod_ = SDL_Keymod (Uint16).
    #[repr(C)]
    #[derive(Copy, Clone)]
    pub struct SDL_KeyboardEvent {
        pub event_type: u32,
        pub reserved:   u32,
        pub timestamp:  u64,
        pub window_id:  u32,
        pub which:      u32,
        pub scancode:   u32,
        pub key:        u32,
        pub mod_:       u16,
        pub raw:        u16,
        pub down:       u8,  // SDLBool
        pub repeat_:    u8,  // SDLBool
        // 2 bytes implicit trailing padding (repr(C) aligns to 8)
    }

    #[repr(C)]
    #[derive(Copy, Clone)]
    pub struct SDL_MouseMotionEvent {
        pub event_type: u32,
        pub reserved:   u32,
        pub timestamp:  u64,
        pub window_id:  u32,
        pub which:      u32,
        pub state:      u32,  // SDL_MouseButtonFlags
        pub x:          f32,
        pub y:          f32,
        pub xrel:       f32,
        pub yrel:       f32,
    }

    #[repr(C)]
    #[derive(Copy, Clone)]
    pub struct SDL_MouseButtonEvent {
        pub event_type: u32,
        pub reserved:   u32,
        pub timestamp:  u64,
        pub window_id:  u32,
        pub which:      u32,
        pub button:     u8,
        pub down:       u8,  // SDLBool
        pub clicks:     u8,
        pub padding:    u8,
        pub x:          f32,
        pub y:          f32,
    }

    #[repr(C)]
    #[derive(Copy, Clone)]
    pub struct SDL_MouseWheelEvent {
        pub event_type: u32,
        pub reserved:   u32,
        pub timestamp:  u64,
        pub window_id:  u32,
        pub which:      u32,
        pub x:          f32,
        pub y:          f32,
        pub direction:  u32,  // SDL_MouseWheelDirection
        pub mouse_x:    f32,
        pub mouse_y:    f32,
    }

    #[repr(C)]
    #[derive(Copy, Clone)]
    pub struct SDL_QuitEvent {
        pub event_type: u32,
        pub reserved:   u32,
        pub timestamp:  u64,
    }

    #[repr(C)]
    #[derive(Copy, Clone)]
    pub struct SDL_UserEvent {
        pub event_type: u32,
        pub reserved:   u32,
        pub timestamp:  u64,
        pub window_id:  u32,
        pub code:       i32,
        pub data1:      *mut std::ffi::c_void,
        pub data2:      *mut std::ffi::c_void,
    }

    // SDL_EventType constants (decimal values from SDL3.Core.cs).
    pub const SDL_EVENT_QUIT:                    u32 = 256;  // 0x100
    pub const SDL_EVENT_WINDOW_SHOWN:            u32 = 514;  // 0x202
    pub const SDL_EVENT_WINDOW_HIDDEN:           u32 = 515;
    pub const SDL_EVENT_WINDOW_EXPOSED:          u32 = 516;
    pub const SDL_EVENT_WINDOW_MOVED:            u32 = 517;
    pub const SDL_EVENT_WINDOW_RESIZED:          u32 = 518;
    pub const SDL_EVENT_WINDOW_MINIMIZED:        u32 = 521;
    pub const SDL_EVENT_WINDOW_MAXIMIZED:        u32 = 522;
    pub const SDL_EVENT_WINDOW_RESTORED:         u32 = 523;
    pub const SDL_EVENT_WINDOW_MOUSE_ENTER:      u32 = 524;
    pub const SDL_EVENT_WINDOW_MOUSE_LEAVE:      u32 = 525;
    pub const SDL_EVENT_WINDOW_FOCUS_GAINED:     u32 = 526;  // 0x20E
    pub const SDL_EVENT_WINDOW_FOCUS_LOST:       u32 = 527;
    pub const SDL_EVENT_WINDOW_CLOSE_REQUESTED:  u32 = 528;
    pub const SDL_EVENT_KEY_DOWN:                u32 = 768;  // 0x300
    pub const SDL_EVENT_KEY_UP:                  u32 = 769;
    pub const SDL_EVENT_TEXT_INPUT:              u32 = 771;  // 0x303
    pub const SDL_EVENT_MOUSE_MOTION:            u32 = 1024; // 0x400
    pub const SDL_EVENT_MOUSE_BUTTON_DOWN:       u32 = 1025;
    pub const SDL_EVENT_MOUSE_BUTTON_UP:         u32 = 1026;
    pub const SDL_EVENT_MOUSE_WHEEL:             u32 = 1027;

    // SDL_Scancode range for letter keys A–Z.
    pub const SDL_SCANCODE_A: u32 = 4;
    pub const SDL_SCANCODE_Z: u32 = 29;
    // SDL_Scancode values for Enter keys (physical key position, layout-independent).
    pub const SDL_SCANCODE_RETURN:   u32 = 40;
    pub const SDL_SCANCODE_KP_ENTER: u32 = 88;

    /// SDL3 event union. Always 128 bytes; read `event_type` first to
    /// determine which variant is active, then access that field.
    #[repr(C)]
    #[derive(Copy, Clone)]
    pub union SDL_Event {
        pub event_type: u32,
        pub common:     SDL_CommonEvent,
        pub window:     SDL_WindowEvent,
        pub key:        SDL_KeyboardEvent,
        pub motion:     SDL_MouseMotionEvent,
        pub button:     SDL_MouseButtonEvent,
        pub wheel:      SDL_MouseWheelEvent,
        pub quit:       SDL_QuitEvent,
        pub user:       SDL_UserEvent,
        padding:        [u8; 128],
    }
}

// ---------------------------------------------------------------------------
// ArtInfo — matches CUO_API.ArtInfo (pack 1, three int64 fields).
// ---------------------------------------------------------------------------
#[repr(C, packed(1))]
pub struct ArtInfo {
    pub address:         i64,
    pub size:            i64,
    pub compressed_size: i64,
}

// ---------------------------------------------------------------------------
// Function pointer types for functions CUO provides to the plugin.
//
// Old-style (byte[]& / ref byte[]):
//   OnPacketSendRecv: bool(byte[]& data, int& length)
//   When called from native code this is bool(**u8, *int32) — the caller passes
//   the address of a byte* variable so CUO can read/replace the buffer.
//
// New-style (IntPtr / *mut u8):
//   OnPacketSendRecv_new_intptr: bool(IntPtr data, ref int length)
//   Simpler — just a raw pointer to the packet bytes.
// ---------------------------------------------------------------------------

/// Old-style: `bool Recv(ref byte[] data, ref int length)` — double pointer.
type FnRecvOld       = unsafe extern "C" fn(data: *mut *mut u8, length: *mut c_int) -> bool;
type FnSendOld       = unsafe extern "C" fn(data: *mut *mut u8, length: *mut c_int) -> bool;
/// New-style: `bool Recv_new(IntPtr data, ref int length)`.
type FnRecvNew       = unsafe extern "C" fn(data: *mut u8, length: *mut c_int) -> bool;
type FnSendNew       = unsafe extern "C" fn(data: *mut u8, length: *mut c_int) -> bool;

type FnGetPacketLength  = unsafe extern "C" fn(id: c_int) -> i16;
type FnGetPlayerPos     = unsafe extern "C" fn(x: *mut c_int, y: *mut c_int, z: *mut c_int) -> bool;
type FnCastSpell        = unsafe extern "C" fn(idx: c_int);
type FnRequestMove      = unsafe extern "C" fn(dir: c_int, run: bool) -> bool;
/// `void SetTitle(string title)` — the title is ANSI/UTF-8 null-terminated.
type FnSetTitle         = unsafe extern "C" fn(title: *const c_char);
/// `void GetStaticImage(ushort g, ref ArtInfo art)`.
type FnGetStaticImage   = unsafe extern "C" fn(g: u16, art: *mut ArtInfo);
/// `string GetUOFilePath()` — returns a null-terminated ANSI string.
type FnGetUOFilePath    = unsafe extern "C" fn() -> *const c_char;
/// `bool GetStaticData(int index, ref ulong flags, ref byte weight, ref byte layer,
///                     ref int count, ref ushort animid, ref ushort lightidx,
///                     ref byte height, ref string name)`.
type FnGetStaticData    = unsafe extern "C" fn(
    index:    c_int,
    flags:    *mut u64,
    weight:   *mut u8,
    layer:    *mut u8,
    count:    *mut c_int,
    animid:   *mut u16,
    lightidx: *mut u16,
    height:   *mut u8,
    name:     *mut *mut c_char,
) -> bool;
/// `bool GetTileData(int index, ref ulong flags, ref ushort textid, ref string name)`.
type FnGetTileData      = unsafe extern "C" fn(
    index:  c_int,
    flags:  *mut u64,
    textid: *mut u16,
    name:   *mut *mut c_char,
) -> bool;
/// `bool GetCliloc(int cliloc, [LPStr] string args, bool capitalize,
///                 [Out, LPStr] out string buffer)`.
type FnGetCliloc        = unsafe extern "C" fn(
    cliloc:     c_int,
    args:       *const c_char,
    capitalize: bool,
    buffer:     *mut *mut c_char,
) -> bool;

// ---------------------------------------------------------------------------
// PluginHeader — must match ClassicUO's C# struct layout exactly.
//
// C# layout on x64 (LayoutKind.Sequential, natural alignment):
//   offset  0: int32  ClientVersion
//   offset  4: [4 bytes padding — repr(C) inserts this automatically]
//   offset  8: IntPtr HWND
//   offset 16: IntPtr OnRecv
//   …          (all remaining fields are IntPtr = 8 bytes each on x64)
// ---------------------------------------------------------------------------
#[repr(C)]
pub(crate) struct PluginHeader {
    client_version:             i32,
    // 4 bytes padding (repr(C) natural alignment, pointer follows i32)
    hwnd:                       *mut c_void,
    on_recv:                    *mut c_void,
    on_send:                    *mut c_void,
    on_hotkey_pressed:          *mut c_void,
    on_mouse:                   *mut c_void,
    on_player_position_changed: *mut c_void,
    on_client_closing:          *mut c_void,
    on_initialize:              *mut c_void,
    on_connected:               *mut c_void,
    on_disconnected:            *mut c_void,
    on_focus_gained:            *mut c_void,
    on_focus_lost:              *mut c_void,
    get_uo_file_path:           *mut c_void,
    recv:                       *mut c_void,
    send:                       *mut c_void,
    get_packet_length:          *mut c_void,
    get_player_position:        *mut c_void,
    cast_spell:                 *mut c_void,
    get_static_image:           *mut c_void,
    tick:                       *mut c_void,
    request_move:               *mut c_void,
    set_title:                  *mut c_void,
    on_recv_new:                *mut c_void,
    on_send_new:                *mut c_void,
    recv_new:                   *mut c_void,
    send_new:                   *mut c_void,
    on_draw_cmd_list:           *mut c_void,
    sdl_window:                 *mut c_void,
    on_wnd_proc:                *mut c_void,
    get_static_data:            *mut c_void,
    get_tile_data:              *mut c_void,
    get_cliloc:                 *mut c_void,
}

// ---------------------------------------------------------------------------
// Plugin state — holds every CUO-provided function pointer after Install().
// ClassicUO calls all plugin callbacks from its main thread, so static mut
// is safe here.
// ---------------------------------------------------------------------------
struct PluginState {
    client_version:      i32,
    // Old-style packet injection (byte** / double pointer).
    recv:                Option<FnRecvOld>,
    send:                Option<FnSendOld>,
    // New-style packet injection (byte* / single pointer).
    recv_new:            Option<FnRecvNew>,
    send_new:            Option<FnSendNew>,
    get_packet_length:   Option<FnGetPacketLength>,
    get_player_position: Option<FnGetPlayerPos>,
    cast_spell:          Option<FnCastSpell>,
    request_move:        Option<FnRequestMove>,
    set_title:           Option<FnSetTitle>,
    get_static_image:    Option<FnGetStaticImage>,
    get_uo_file_path:    Option<FnGetUOFilePath>,
    get_static_data:     Option<FnGetStaticData>,
    get_tile_data:       Option<FnGetTileData>,
    get_cliloc:          Option<FnGetCliloc>,
}

impl PluginState {
    const fn new() -> Self {
        Self {
            client_version:      0,
            recv:                None,
            send:                None,
            recv_new:            None,
            send_new:            None,
            get_packet_length:   None,
            get_player_position: None,
            cast_spell:          None,
            request_move:        None,
            set_title:           None,
            get_static_image:    None,
            get_uo_file_path:    None,
            get_static_data:     None,
            get_tile_data:       None,
            get_cliloc:          None,
        }
    }
}

static mut STATE: PluginState = PluginState::new();
static TYPING_INDICATOR: std::sync::Mutex<typing_indicator::TypingIndicator> =
    std::sync::Mutex::new(typing_indicator::TypingIndicator::new());

/// Reinterpret a raw `*mut c_void` as a typed function pointer.
/// Returns `None` for null pointers.
unsafe fn load_fn<T: Copy>(ptr: *mut c_void) -> Option<T> {
    if ptr.is_null() {
        return None;
    }
    debug_assert_eq!(std::mem::size_of::<*mut c_void>(), std::mem::size_of::<T>());
    Some(unsafe { std::mem::transmute_copy(&ptr) })
}

// ---------------------------------------------------------------------------
// Public helpers — safe wrappers around the CUO-provided functions.
// ---------------------------------------------------------------------------

pub fn get_player_position() -> Option<(i32, i32, i32)> {
    let (mut x, mut y, mut z) = (0i32, 0i32, 0i32);
    let ok = unsafe {
        let f = STATE.get_player_position?;
        f(&mut x, &mut y, &mut z)
    };
    ok.then_some((x, y, z))
}

pub fn cast_spell(idx: i32) {
    unsafe {
        if let Some(f) = STATE.cast_spell {
            f(idx);
        }
    }
}

pub fn request_move(dir: i32, run: bool) -> bool {
    unsafe { STATE.request_move.map(|f| f(dir, run)).unwrap_or(false) }
}

pub fn get_packet_length(id: i32) -> Option<i16> {
    unsafe { STATE.get_packet_length.map(|f| f(id)) }
}

pub fn get_uo_file_path() -> Option<std::ffi::CString> {
    unsafe {
        let ptr = STATE.get_uo_file_path.map(|f| f())?;
        if ptr.is_null() {
            return None;
        }
        Some(std::ffi::CStr::from_ptr(ptr).to_owned())
    }
}

/// Inject `data` as a packet received from the server (new-style, preferred).
pub fn inject_to_client(data: &mut [u8]) -> bool {
    unsafe {
        if let Some(f) = STATE.recv_new {
            let mut len = data.len() as c_int;
            return f(data.as_mut_ptr(), &mut len);
        }
        // Fall back to old-style if new-style is unavailable.
        if let Some(f) = STATE.recv {
            let mut ptr = data.as_mut_ptr();
            let mut len = data.len() as c_int;
            return f(&mut ptr, &mut len);
        }
        false
    }
}

/// Inject `data` as a packet sent from the client to the server (new-style, preferred).
pub fn inject_to_server(data: &mut [u8]) -> bool {
    unsafe {
        if let Some(f) = STATE.send_new {
            let mut len = data.len() as c_int;
            return f(data.as_mut_ptr(), &mut len);
        }
        if let Some(f) = STATE.send {
            let mut ptr = data.as_mut_ptr();
            let mut len = data.len() as c_int;
            return f(&mut ptr, &mut len);
        }
        false
    }
}

// ---------------------------------------------------------------------------
// Plugin callbacks — CUO calls these via the pointers written in Install().
//
// All use `extern "C"` (cdecl) to match what CUO expects.
// Bool params/returns are safe on x64: the value always lives in the low byte
// of the register regardless of whether C# marshals it as 1-byte or 4-byte.
// ---------------------------------------------------------------------------

unsafe extern "C" fn on_initialize() {
    let ver = unsafe { STATE.client_version };
    eprintln!("[swcuors] initialized (client_version={ver})");
}

unsafe extern "C" fn on_connected() {
    eprintln!("[swcuors] connected");
}

unsafe extern "C" fn on_disconnected() {
    eprintln!("[swcuors] disconnected");
}

unsafe extern "C" fn on_client_closing() {
    eprintln!("[swcuors] client closing");
}

unsafe extern "C" fn on_focus_gained() {}

unsafe extern "C" fn on_focus_lost() {}

/// Called every game frame — keep this fast.
unsafe extern "C" fn on_tick() {}

/// Return `true` to let CUO handle the hotkey, `false` to consume it.
unsafe extern "C" fn on_hotkey_pressed(key: c_int, mod_: c_int, pressed: bool) -> bool {
    let _ = (key, mod_, pressed);
    true
}

unsafe extern "C" fn on_mouse(button: c_int, wheel: c_int) {
    let _ = (button, wheel);
}

unsafe extern "C" fn on_player_position_changed(x: c_int, y: c_int, z: c_int) {
    let _ = (x, y, z);
}

/// Packet FROM server → client. Return `false` to drop the packet.
unsafe extern "C" fn on_recv_new(data: *mut u8, length: *mut c_int) -> bool {
    if data.is_null() || length.is_null() {
        return true;
    }
    let _packet = unsafe { std::slice::from_raw_parts(data, *length as usize) };
    // inspect or modify _packet here; packet[0] is the UO packet ID
    true
}

/// Packet FROM client → server. Return `false` to drop the packet.
unsafe extern "C" fn on_send_new(data: *mut u8, length: *mut c_int) -> bool {
    if data.is_null() || length.is_null() {
        return true;
    }
    let _packet = unsafe { std::slice::from_raw_parts(data, *length as usize) };
    true
}

/// Called for each SDL event before CUO processes it.
/// Return 0 to let CUO handle the event, non-zero to consume it.
unsafe extern "C" fn on_wnd_proc(ev: *mut sdl3::SDL_Event) -> c_int {
    if ev.is_null() {
        return 0;
    }
    let event_type = unsafe { (*ev).event_type };
    match event_type {
        sdl3::SDL_EVENT_KEY_UP => {
            let scancode = unsafe { (*ev).key.scancode };
            if scancode >= sdl3::SDL_SCANCODE_A && scancode <= sdl3::SDL_SCANCODE_Z {
                TYPING_INDICATOR.lock().unwrap().update();
            } else if scancode == sdl3::SDL_SCANCODE_RETURN || scancode == sdl3::SDL_SCANCODE_KP_ENTER {
                TYPING_INDICATOR.lock().unwrap().reset();
            }
        }
        _ => {}
    }
    0 // pass event through to CUO
}

// ---------------------------------------------------------------------------
// Install — the single exported entry point ClassicUO looks for.
//
// CUO calls Install(&mut header) after loading the DLL. The header is already
// filled with client-side function pointers. The plugin reads those, then
// writes its own callback pointers back into the header before returning.
// CUO reads the callbacks after Install() returns.
// ---------------------------------------------------------------------------
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
unsafe extern "C" fn Install(header: *mut PluginHeader) {
    if header.is_null() {
        return;
    }
    let h = unsafe { &mut *header };

    // Snapshot every function pointer CUO provided.
    unsafe {
        STATE.client_version    = h.client_version;
        STATE.recv              = load_fn(h.recv);
        STATE.send              = load_fn(h.send);
        STATE.recv_new          = load_fn(h.recv_new);
        STATE.send_new          = load_fn(h.send_new);
        STATE.get_packet_length = load_fn(h.get_packet_length);
        STATE.get_player_position = load_fn(h.get_player_position);
        STATE.cast_spell        = load_fn(h.cast_spell);
        STATE.request_move      = load_fn(h.request_move);
        STATE.set_title         = load_fn(h.set_title);
        STATE.get_static_image  = load_fn(h.get_static_image);
        STATE.get_uo_file_path  = load_fn(h.get_uo_file_path);
        STATE.get_static_data   = load_fn(h.get_static_data);
        STATE.get_tile_data     = load_fn(h.get_tile_data);
        STATE.get_cliloc        = load_fn(h.get_cliloc);
    }

    // Register plugin callbacks.
    h.on_initialize              = on_initialize              as *mut c_void;
    // h.on_connected               = on_connected               as *mut c_void;
    // h.on_disconnected            = on_disconnected            as *mut c_void;
    // h.on_client_closing          = on_client_closing          as *mut c_void;
    // h.on_focus_gained            = on_focus_gained            as *mut c_void;
    // h.on_focus_lost              = on_focus_lost              as *mut c_void;
    // h.tick                       = on_tick                    as *mut c_void;
    // h.on_hotkey_pressed          = on_hotkey_pressed          as *mut c_void;
    // h.on_mouse                   = on_mouse                   as *mut c_void;
    // h.on_player_position_changed = on_player_position_changed as *mut c_void;
    // h.on_recv_new                = on_recv_new                as *mut c_void;
    // h.on_send_new                = on_send_new                as *mut c_void;
    h.on_wnd_proc                = on_wnd_proc                as *mut c_void;

    eprintln!("[swcuors] Install ok (client_version={})", h.client_version);
}
