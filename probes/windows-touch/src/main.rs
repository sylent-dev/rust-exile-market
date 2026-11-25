//! Touch probe for issue #277 of Exile Market: on a Windows touchscreen, a single tap outside
//! the window over the game doesn't close it, a double tap does.
//!
//! Exile Market reads the pointer every 20 ms and counts a click when a button is held at one
//! reading and wasn't at the one before (`GetAsyncKeyState`, its high bit). This probe reads the
//! same things every millisecond, and the low bit too ("pressed since the last reading"), and
//! writes each change to a log next to the executable, during four timed phases announced by
//! beeps: single taps, long presses, double taps, and clicks of a real mouse for reference.
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

    /// The time to get ready before the first phase: switch to the game, for the second run.
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
    const PHASES: [&str; 4] = [
        "SINGLE TAPS: about 5, one per second, each at a new place",
        "LONG PRESSES: 2, each finger held about 1 second",
        "DOUBLE TAPS: 2 or 3, each at a new place",
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

        println!("Touch probe for Exile Market (issue #277). It only reads the pointer.");
        println!("Don't tap this window: tap the desktop, or the game.");
        println!("Starting in {} seconds.", READY.as_secs());
        let _ = writeln!(log, "touch probe {}", env!("CARGO_PKG_VERSION"));
        thread::sleep(READY);

        let start = Instant::now();
        let mut reader = Reader::new(start);
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
