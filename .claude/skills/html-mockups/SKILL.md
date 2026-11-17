---
name: html-mockups
description: Build the interactive HTML mock-up of a user interface from its approved wireframe, in docs/mockups/, with every window, scenario and state, and 3 or 4 styles compared with tabs. Use it after the wireframes are approved, when the owner asks for a mock-up, a style, a visual design, or a new section of an existing mock-up. Serving and checking the page is the mockup-screenshots skill; iterating on the chosen style, the mockup-decisions skill.
---

# HTML mock-ups

Goal: choose the style with real content, every scenario and every state, before writing the interface. The mock-up answers one question at a time: the layout is already settled by the wireframe (`excalidraw-wireframes` skill), so the discussion is only about the look.

## 1. Build the page

Before drawing the options of a design question:

- **Ask the owner for their own idea first**, or draw it alongside: theirs beat the three entries drawn for a bug report (an icon in the title bar rather than a link or a row in the settings).
- **Look at the real data** the window will show: the 771 items of a list had only 639 images (several shared one icon), which ruled out an icon alone before any option was drawn.

Start from `assets/mockup-template.html` (read its comments), copied to `docs/mockups/<name>.html`:

- **Small files**, so that a change reads only what it needs: the page `docs/mockups/<name>.html` only loads, in order, the files of `docs/mockups/<name>/`: `css/` (the fonts, the tokens, the page, then one sheet per block of the interface), `js/` (the shared helpers and icons, one script per block, the sample data, the rendering last) and `sections/` (one script per section, each adding its windows to `SECTIONS`). Plain `<link>` and `<script>` tags, no modules, so that the files share their constants. The template is one file: split it as soon as the page has more than a few blocks.
- **Every frame of the wireframe becomes a window**, at its real size in logical pixels and with its title bar, with sample data close to real ones (long names, large numbers, the other language). Add every scenario and state in `SECTIONS`: empty, loading, waiting, each error, each kind of content. **A new section goes at the top** of `SECTIONS`, so that the owner finds it at once; past ten sections, the page shows a table of contents under its header, one link per section.
- **3 or 4 styles on the same page**, switched with the tabs at the top; the address keeps the choice (`?theme=…`) to share it. Each style only sets **tokens** (CSS variables under `:root[data-theme="…"]`): fonts, colors, radius, spacing. The components use nothing else, so that a style is cheap to add or tweak.
- The styles explore **new directions** for this application, its users and its subject, from sober to spectacular: the owner decides by seeing, and a variant set aside is not wasted. If the project has a `design-journal.md`, it records past experiences: it may inspire one style, never all of them, and nothing in it is a default.
- **Only what egui draws well**: flat fills, strokes, rounded corners, shadows, the fonts and images the application embeds, simple motions (a hover, a fade, a size that changes over time). An effect of the browser that egui lacks (a blur behind a panel, a gradient on a text, a complex transition) would be approved and then never built.
- **Fonts the application can embed** (e.g. SIL Open Font License, from Google Fonts), preloaded with `font-display: block`, so that the page and its screenshots only finish loading once the fonts are there.
- **Icons as SVG lines** in the text color, not text symbols (⚙, ⌕, ✓), which render small and off-center.
- **Every notice is drawn with its way out**: how it closes, for how long, what brings it back. A notice the user can't dismiss stays for as long as they ignore it: a dot on an icon and a line in the settings, left until the user updates, gave way to a bubble that closes for good and a discreet link where the version is written.
- **A motion is chosen by clicking it**, not by reading about it: draw each variant as a window that plays it on a click (180 ms with an overshoot, 220 ms smooth, 140 ms quick settled a sliding pill in one look), then write the duration and its curve in `specs.md` and the tokens.

Pitfalls met before:

- A class shared by two components (`top`, `online`) silently broke the layout twice: prefix the classes of the application (`app-`) and of the states (`is-`).
- A JavaScript function named like a browser global (`top`, `name`, `status`) breaks the page: give the components specific names.
- A `url()` in a style sheet is relative to the sheet, not to the page: from `docs/mockups/<name>/css/`, `assets/` is `../../../../assets/`.

## 2. Hand it over

Serve the page and check each change with a headless screenshot before giving the owner its link (`mockup-screenshots` skill). Once the owner picks a style, iterate on it one decision at a time, then write the design tokens (`mockup-decisions` skill).
