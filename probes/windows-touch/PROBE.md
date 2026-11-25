# Touch probe (issue #277)

A tiny program that logs what Windows reports of the pointer and its buttons while you tap the screen, so that we can see why a single tap outside the window of Exile Market doesn't close it while a double tap does.

**What it does:** for about 1 minute 30 seconds, it reads the position of the pointer and whether a mouse button is held, every millisecond, and writes each change to a text file next to it. It sends no input, reads no memory of the game or of another program, and uses no network. Its whole code is [`src/main.rs`](src/main.rs).

## Get it

Either:

- **Download it**: the `touch-probe` artifact of the latest run of the [Touch probe workflow](https://github.com/sylent-dev/rust-exile-market/actions/workflows/touch-probe.yml) (you need to be signed in to GitHub), then unzip it. It isn't signed, so Windows may show an alert: "More info", then "Run anyway".
- **Or build it yourself**, from this folder of the branch `explore/windows-touch-probe`: `cargo build --release`, then `target\release\touch-probe.exe`.

## Run it twice

The probe announces each phase with beeps, so that you don't need to see its window: **1 beep, 2 beeps, 3 beeps, 4 beeps**, each phase lasting 20 seconds, then one long beep at the end. Keep the sound on.

| Beeps | What to do during the 20 seconds |
|---|---|
| 1 | **Single taps**: about 5, one per second, each at a new place |
| 2 | **Long presses**: 2, each finger held about 1 second |
| 3 | **Double taps**: 2 or 3, each at a new place |
| 4 | **Reference**: about 5 clicks with a mouse, or with the controller in its mouse mode; nothing if you have neither |

**Run 1, on the desktop, without the game.** Start `touch-probe.exe`. You have 10 seconds before the first beep. Tap the empty desktop, never the window of the probe itself (a tap in a console window can pause it).

**Run 2, over the game.** Start Path of Exile 2 in windowed fullscreen, in your hideout or a town (your taps will move the character, and a long press may cast the skill of the right button). Start `touch-probe.exe`, then switch to the game within the 10 seconds (tap it in the taskbar). Tap the ground of the game. Exile Market can stay closed.

## Send the result

Each run writes `touch-probe-<number>.log` next to the executable; its window shows the path at the end. Attach both files to [issue #277](https://github.com/sylent-dev/rust-exile-market/issues/277) (drag them into a comment), and say which one is the desktop and which one the game.

Thanks a lot!
