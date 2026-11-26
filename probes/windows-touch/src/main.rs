//! Touch probe for issue #277 of Exile Market: on a Windows touchscreen, a single tap outside
//! the window over the game doesn't close it, a double tap does.
//!
//! Exile Market reads the pointer every 20 ms and counts a click when a button is held at one
//! reading and wasn't at the one before (`GetAsyncKeyState`, its high bit). This probe reads the
//! same things every millisecond, and the low bit too ("pressed since the last reading"), and
//! writes each change to a log next to the executable, during timed phases announced by beeps:
//! single taps, long presses, and clicks of a real mouse for reference.
//!
//! Since 0.2.0, it also listens through a low-level mouse hook (`WH_MOUSE_LL`), the fix planned
//! for Exile Market: each press the hook is told of is written with a `hook` mark, and whether
//! Windows made it from a touch, a pen or a mouse (`dwExtraInfo`), so that the log tells whether
//! the hook sees the taps the readings miss.
//!
//! It only reads: it sends no input, reads no memory of another program, and uses no network.
//! Throwaway code, never merged.

#[cfg(windows)]
fn main() {
    windows::run();
}

#[cfg(not(windows))]
fn main() {
    eprintln!("This probe runs on Windows only.");
}

#[cfg(windows)]
mod windows {
    use std::fs::File;
    use std::io::{self, BufWriter, Write};
    use std::path::PathBuf;
    use std::thread;
    use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

    use windows_sys::Win32::Foundation::POINT;
    use windows_sys::Win32::Media::{timeBeginPeriod, timeEndPeriod};
    use windows_sys::Win32::System::Diagnostics::Debug::Beep;
    use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
        GetAsyncKeyState, VK_LBUTTON, VK_MBUTTON, VK_RBUTTON,
    };
    use windows_sys::Win32::UI::WindowsAndMessaging::GetCursorPos;

    use crate::hook;

    /// The time to get ready before the first phase: switch to the game.
    const READY: Duration = Duration::from_secs(10);
    /// The length of each phase.
    const PHASE: Duration = Duration::from_secs(20);
    /// The wait between two readings, made precise to 1 ms by `timeBeginPeriod`.
    const POLL: Duration = Duration::from_millis(1);
    /// A move of the pointer shorter than this, with no button change, isn't written: it would
    /// fill the log with the moves of a mouse.
    const MOVE_LOGGED: i32 = 25;

    const BUTTONS: [(u16, &str); 3] = [
        (VK_LBUTTON, "left"),
        (VK_RBUTTON, "right"),
        (VK_MBUTTON, "middle"),
    ];

    /// Each phase: what to do, announced by as many beeps as its number.
    const PHASES: [&str; 3] = [
        "SINGLE TAPS: about 5, one per second, each at a new place",
        "LONG PRESSES: 2, each finger held about 1 second",
        "REFERENCE: about 5 clicks with a mouse or the mouse mode of the device (nothing if none)",
    ];

    pub fn run() {
        let path = log_path();
        let mut log = match File::create(&path) {
            Ok(file) => BufWriter::new(file),
            Err(error) => {
                println!("Can't write the log {}: {error}", path.display());
                wait_for_close();
                return;
            }
        };
        // SAFETY: a plain call without pointers; undone by `timeEndPeriod` below.
        unsafe { timeBeginPeriod(1) };
        let start = Instant::now();
        let hooked = hook::install(start);

        println!("Touch probe for Exile Market (issue #277). It only reads the pointer.");
        println!("Don't tap this window: tap the desktop, or the game.");
        println!("Starting in {} seconds.", READY.as_secs());
        let _ = writeln!(log, "touch probe {}", env!("CARGO_PKG_VERSION"));
        let _ = writeln!(
            log,
            "# low-level mouse hook: {}",
            if hooked { "installed" } else { "NOT installed" }
        );
        let mut reader = Reader::new(start);
        reader.watch(READY, &mut log);
        let _ = writeln!(log, "# ready time over");
        for (index, task) in PHASES.iter().enumerate() {
            let number = index + 1;
            println!();
            println!("Phase {number} of {}: {task}", PHASES.len());
            let _ = writeln!(log, "# phase {number}: {task}");
            beeps(number);
            let longest_gap = reader.watch(PHASE, &mut log);
            let _ = writeln!(
                log,
                "# end of phase {number}: longest gap between two readings {} ms",
                longest_gap.as_millis()
            );
            let _ = log.flush();
        }

        hook::remove();
        // SAFETY: undoes the `timeBeginPeriod` above.
        unsafe { timeEndPeriod(1) };
        // SAFETY: plain values; a failure only means no sound.
        unsafe { Beep(440, 800) };
        let _ = log.flush();
        println!();
        println!("Done. The log is: {}", path.display());
        println!("Attach it to the issue. You can close this window.");
        wait_for_close();
    }

    /// `touch-probe-<seconds>.log` next to the executable, or in the current folder.
    fn log_path() -> PathBuf {
        let seconds = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |since| since.as_secs());
        let name = format!("touch-probe-{seconds}.log");
        std::env::current_exe()
            .ok()
            .and_then(|exe| exe.parent().map(|folder| folder.join(&name)))
            .unwrap_or_else(|| PathBuf::from(name))
    }

    fn beeps(count: usize) {
        for _ in 0..count {
            // SAFETY: plain values; a failure only means no sound.
            unsafe { Beep(880, 150) };
            thread::sleep(Duration::from_millis(150));
        }
    }

    fn wait_for_close() {
        let _ = io::stdin().read_line(&mut String::new());
    }

    /// A button read once: held now (high bit), pressed since the last reading (low bit).
    #[derive(Clone, Copy, PartialEq, Eq)]
    struct Button {
        held: bool,
        pressed_since: bool,
    }

    fn read_button(key: u16) -> Button {
        // SAFETY: a plain call without pointers.
        let state = unsafe { GetAsyncKeyState(i32::from(key)) };
        #[expect(
            clippy::cast_sign_loss,
            reason = "the bits of the state, read as they are"
        )]
        let bits = state as u16;
        Button {
            held: bits & 0x8000 != 0,
            pressed_since: bits & 1 != 0,
        }
    }

    fn read_pointer() -> Option<(i32, i32)> {
        let mut point = POINT { x: 0, y: 0 };
        // SAFETY: `point` is a valid POINT that lives for the call.
        let read = unsafe { GetCursorPos(&raw mut point) };
        (read != 0).then_some((point.x, point.y))
    }

    struct Reader {
        start: Instant,
        held: [bool; 3],
        logged_at: Option<(i32, i32)>,
    }

    impl Reader {
        fn new(start: Instant) -> Self {
            Self {
                start,
                held: BUTTONS.map(|(key, _)| read_button(key).held),
                logged_at: read_pointer(),
            }
        }

        /// Reads the pointer and the buttons every millisecond for `length`, and writes each
        /// change; gives the longest gap between two readings.
        fn watch(&mut self, length: Duration, log: &mut impl Write) -> Duration {
            let end = Instant::now() + length;
            let mut last = Instant::now();
            let mut longest_gap = Duration::ZERO;
            while Instant::now() < end {
                thread::sleep(POLL);
                // The hook is called from here, within `PeekMessageW`.
                hook::pump();
                for line in hook::take_lines() {
                    let _ = writeln!(log, "{line}");
                }
                let now = Instant::now();
                longest_gap = longest_gap.max(now - last);
                last = now;
                self.read_once(now, log);
            }
            longest_gap
        }

        fn read_once(&mut self, now: Instant, log: &mut impl Write) {
            let pointer = read_pointer();
            let at = pointer.map_or_else(
                || "pointer unreadable".to_owned(),
                |(x, y)| format!("at {x},{y}"),
            );
            let ms = (now - self.start).as_millis();
            let mut button_event = false;
            for (index, (key, name)) in BUTTONS.iter().enumerate() {
                let button = read_button(*key);
                if button.held != self.held[index] {
                    let change = if button.held { "down" } else { "up" };
                    let _ = writeln!(log, "{ms:>7} ms  {name} {change}  {at}");
                    self.held[index] = button.held;
                    button_event = true;
                }
                if button.pressed_since && !button.held {
                    let _ = writeln!(
                        log,
                        "{ms:>7} ms  {name} pressed and released between two readings  {at}"
                    );
                    button_event = true;
                }
            }
            let moved = match (pointer, self.logged_at) {
                (Some((x, y)), Some((was_x, was_y))) => {
                    (x - was_x).abs().max((y - was_y).abs()) >= MOVE_LOGGED
                }
                (Some(_), None) => true,
                _ => false,
            };
            if moved && !button_event {
                let _ = writeln!(log, "{ms:>7} ms  pointer moved  {at}");
            }
            if moved || button_event {
                self.logged_at = pointer;
            }
        }
    }
}

/// The low-level mouse hook: installed on the main thread, whose loop pumps its messages every
/// millisecond, and called by Windows from within that pumping.
#[cfg(windows)]
mod hook {
    use std::sync::{Mutex, OnceLock, PoisonError};
    use std::time::Instant;

    use windows_sys::Win32::Foundation::{LPARAM, LRESULT, WPARAM};
    use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        CallNextHookEx, HC_ACTION, LLMHF_INJECTED, MSG, MSLLHOOKSTRUCT, PM_REMOVE, PeekMessageW,
        SetWindowsHookExW, UnhookWindowsHookEx, WH_MOUSE_LL, WM_LBUTTONDOWN, WM_LBUTTONUP,
        WM_MBUTTONDOWN, WM_MBUTTONUP, WM_RBUTTONDOWN, WM_RBUTTONUP,
    };

    /// The mark Windows puts in `dwExtraInfo` of a mouse event it made from a pen or a touch.
    const PEN_OR_TOUCH: usize = 0xFF51_5700;
    const PEN_OR_TOUCH_MASK: usize = 0xFFFF_FF00;
    /// Set with that mark when it came from a touch rather than a pen.
    const TOUCH: usize = 0x80;

    static START: OnceLock<Instant> = OnceLock::new();
    /// The hook, as its address, so that it can be kept in a static.
    static HOOK: Mutex<Option<usize>> = Mutex::new(None);
    static LINES: Mutex<Vec<String>> = Mutex::new(Vec::new());

    /// Installs the hook; whether Windows took it.
    pub fn install(start: Instant) -> bool {
        let _ = START.set(start);
        // SAFETY: a null name asks for the module of this executable.
        let module = unsafe { GetModuleHandleW(std::ptr::null()) };
        // SAFETY: `on_mouse` matches HOOKPROC and lives as long as the program.
        let handle = unsafe { SetWindowsHookExW(WH_MOUSE_LL, Some(on_mouse), module, 0) };
        if handle.is_null() {
            return false;
        }
        *HOOK.lock().unwrap_or_else(PoisonError::into_inner) = Some(handle as usize);
        true
    }

    pub fn remove() {
        if let Some(handle) = HOOK.lock().unwrap_or_else(PoisonError::into_inner).take() {
            // SAFETY: the handle SetWindowsHookExW gave, removed once.
            unsafe { UnhookWindowsHookEx(handle as _) };
        }
    }

    /// Lets Windows call the hook for the events of the mouse since the last call.
    pub fn pump() {
        // SAFETY: MSG is plain data, which PeekMessageW fills.
        let mut message: MSG = unsafe { std::mem::zeroed() };
        // SAFETY: `message` is a valid MSG that lives for the call.
        while unsafe { PeekMessageW(&raw mut message, std::ptr::null_mut(), 0, 0, PM_REMOVE) } != 0
        {
        }
    }

    pub fn take_lines() -> Vec<String> {
        std::mem::take(&mut *LINES.lock().unwrap_or_else(PoisonError::into_inner))
    }

    unsafe extern "system" fn on_mouse(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        #[expect(clippy::cast_sign_loss, reason = "HC_ACTION is 0")]
        let action = code >= 0 && code as u32 == HC_ACTION;
        #[expect(
            clippy::cast_possible_truncation,
            reason = "the messages fit in 32 bits"
        )]
        let button = match wparam as u32 {
            WM_LBUTTONDOWN => Some("left down"),
            WM_LBUTTONUP => Some("left up"),
            WM_RBUTTONDOWN => Some("right down"),
            WM_RBUTTONUP => Some("right up"),
            WM_MBUTTONDOWN => Some("middle down"),
            WM_MBUTTONUP => Some("middle up"),
            _ => None,
        };
        if let (true, Some(button)) = (action, button) {
            // SAFETY: for WH_MOUSE_LL with HC_ACTION, lparam points to a MSLLHOOKSTRUCT that
            // lives for the call.
            let event = unsafe { &*(lparam as *const MSLLHOOKSTRUCT) };
            let source = if event.dwExtraInfo & PEN_OR_TOUCH_MASK == PEN_OR_TOUCH {
                if event.dwExtraInfo & TOUCH == 0 {
                    "pen"
                } else {
                    "touch"
                }
            } else {
                "mouse"
            };
            let injected = if event.flags & LLMHF_INJECTED == 0 {
                ""
            } else {
                ", injected"
            };
            let ms = START.get().map_or(0, |start| start.elapsed().as_millis());
            LINES
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .push(format!(
                    "{ms:>7} ms  hook: {button}  at {},{}  ({source}{injected})",
                    event.pt.x, event.pt.y
                ));
        }
        // SAFETY: the arguments Windows gave, handed to the next hook.
        unsafe { CallNextHookEx(std::ptr::null_mut(), code, wparam, lparam) }
    }
}
