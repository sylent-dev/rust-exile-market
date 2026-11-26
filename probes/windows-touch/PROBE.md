# Touch probe (issue #277)

A tiny program that logs what Windows reports of the pointer and its buttons while you tap the screen.

- **Version 0.1** (done, thanks!) showed that a tap holds the button for only 1 to 3 ms, too short for the readings of Exile Market, every 20 ms.
- **Version 0.2** also listens through a **low-level mouse hook**, the fix planned for Exile Market, which is told of each press as it happens. Its log tells whether that hook sees the taps, and whether Windows marks them as touch.

**What it does:** for about 1 minute 10 seconds, it reads the pointer and its buttons every millisecond, listens to the hook, and writes each change to a text file next to it. It sends no input, reads no memory of the game or of another program, and uses no network. Its whole code is [`src/main.rs`](src/main.rs).

## Get it

Either:

- **Download it**: the `touch-probe` artifact of the latest run of the [Touch probe workflow](https://github.com/sylent-dev/rust-exile-market/actions/workflows/touch-probe.yml) (you need to be signed in to GitHub), then unzip it. It isn't signed, so Windows may show an alert: "More info", then "Run anyway".
- **Or build it yourself**, from this folder of the branch `explore/windows-touch-probe`: `cargo build --release`, then `target\release\touch-probe.exe`.

## Run it once, over the game

1. Start Path of Exile 2 in windowed fullscreen, in your hideout or a town (your taps will move the character, and a long press may cast the skill of the right button). Exile Market can stay closed.
2. Start `touch-probe.exe`, then switch to the game within the 10 seconds (tap it in the taskbar).
3. Follow the beeps, each phase lasting 20 seconds, then one long beep at the end. Keep the sound on.

| Beeps | What to do during the 20 seconds |
|---|---|
| 1 | **Single taps**: about 5, one per second, each at a new place |
| 2 | **Long presses**: 2, each finger held about 1 second |
| 3 | **Reference**: about 5 clicks with a mouse, or with the controller in its mouse mode; nothing if you have neither |

## Send the result

The run writes `touch-probe-<number>.log` next to the executable; its window shows the path at the end. Attach it to [issue #277](https://github.com/sylent-dev/rust-exile-market/issues/277) (drag it into a comment).

Thanks a lot!
