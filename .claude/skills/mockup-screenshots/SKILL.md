---
name: mockup-screenshots
description: Serve the HTML mock-up of docs/mockups/ without cache on the port of the project, give the owner its link, and check each change with a headless screenshot (Playwright, or Firefox) of the window concerned, which lists the console errors and the texts cut or wrapped. Use it each time the mock-up changes, before giving its link or asking for a review. For the window of the application itself, use egui-screenshots.
---

# Serving and checking the mock-up

Goal: the owner opens the latest page at once, and Claude Code sees what it changed before asking for a review. The screenshots are Claude Code's own check, kept in the scratchpad, looked at, then deleted.

## 1. Serve it

Copy `assets/serve.py` to `docs/mockups/serve.py`, and serve the repository root with it, so that the page reaches `assets/` even in a sandboxed browser (Flatpak, Snap) that only receives the HTML file, and a reload never mixes old and new files (it sends `Cache-Control: no-store`). Use the port of the project, written in `docs/technical-choices.md`: two projects on the same port took each other's server over. Keep the server running in the background:

```sh
python3 docs/mockups/serve.py <port>
```

Each time the mock-up changes, check that the page answers with the change, then give the owner `http://127.0.0.1:<port>/docs/mockups/<name>.html` with the section to look at; they reload after each change. If the page answers 404, check which project answers on the port before restarting.

## 2. Check each change

Before asking for a review, check each change with a headless screenshot (`scripts/screenshot.sh <url> <output.png> [width] [height | full] [selector]`), kept in the scratchpad, and look at it, then delete it: the screenshots are for this check only, the owner looks at the mock-up itself. Capture only the window concerned, with a selector (`'#section-1 .page-slot:nth-child(4)'`), rather than the whole page. **Look at a resizable window at its default width and at the owner's** (ask it once, and note it in `docs/technical-choices.md`): a sentence on three lines in the owner's wider window took four at the default width. To compare the styles at a glance, export one large PNG per style to `outputs/`, deleted once the style is chosen.

The script uses **Playwright** and its own Chromium when installed, otherwise Firefox (normal or Flatpak). Prefer Playwright: its Chromium is closer to the one of VS Code where the owner looks at the mock-up (Firefox draws the fonts a few pixels wider or narrower); it waits for the scripts to settle and prints the errors of the browser console (scripts, missing files) and fails on them; with a selector it captures that window only and lists its texts cut (wider than their box) or wrapped (on more lines than written), with their widths, the measures a screenshot doesn't give; `full` captures the whole page; `SCALE=1.5` takes the scale of a screen other than 100 %. It is also the only choice when Claude Code runs in a sandbox that doesn't see the browsers of the system, such as VS Code installed as a Flatpak. It checks the mock-up page only, whatever the kind of application. Install it once, outside the projects (about 150 MB):

```sh
mkdir -p ~/.local/share/mockup-tools && cd ~/.local/share/mockup-tools
npm init -y && npm install playwright && npx playwright install chromium
```

- A test that needs extra code (a delay, a simulated click) runs on a copy of the page (`docs/mockups/test-*.html`, deleted after), never on the file the owner has open: they reload it at any time. Stop the helper servers by their PID, never with `pkill -f` and a pattern that also matches the shell's own command.
- A headless browser renders without the GPU: a color or WebGL bug the owner reports doesn't show in a screenshot. Build a small test page of variants and ask the owner to name the wrong ones.
- `firefox --screenshot` shoots before the scripts settle: an animation is measured or shot with Playwright, driven by a small script of its own (see `scripts/screenshot.cjs`).
