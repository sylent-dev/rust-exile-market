---
name: mockup-decisions
description: Iterate with the owner on the chosen style of the HTML mock-up, one decision at a time, each with its own control on the page (tabs, a list per role, a slider for a value tuned by eye), write each decision in specs.md, then finish with the design tokens. Use it once the owner has picked a style of the mock-up, when a design decision is open, or to tune a detail of the interface.
---

# Decisions on the mock-up

Goal: settle the look one small decision at a time, on screen, rather than by imagining; then hand the code one set of tokens. The page is built by the `html-mockups` skill and checked by the `mockup-screenshots` skill after each change.

## 1. Iterate with the owner

1. The owner picks a style, or mixes two. Iterate on it alone, **one change at a time** (the main color, then the logo, then a font…).
2. **Give each open decision its own control** in `VARIANTS`, so that the owner compares on screen instead of imagining; once decided, remove it, keep the choice in the tokens, and write the decision in the design section of `specs.md` in the same commit, with the options set aside. About a hundred small decisions went this way on the first project, each in a minute. The control depends on the decision:
   - **A few distinct variants** (two button shapes, three header variants, five offsets…): a group of tabs, with the variants also side by side when the difference is fine.
   - **Values that combine** (a font per role, colors, sizes): one list per role (`select: true`, with the CSS `variable` it sets), which changes the page at once. The owner then tries every combination instead of waiting for a new board; a list "for everything" that the others follow until they are changed saves many clicks.
   - **Values tuned by eye** (a width, an angle, a darkness): a slider (`range`, with the CSS `variable` it sets), whose value shows beside it. The owner tunes it on their own screen, then sends back the address, which holds the values.
   - **The address keeps every choice**, so that the owner can reload the page or send the exact combination.
   - **An important element is shown alone and in real conditions**, with what surrounds it behind, both following the controls.
   - **Number the proposals once**, from 1 upward across every series: letters, or a numbering that starts again in each series, get mixed up ("the B" of which series?).
3. **Pixel alignments are tuned by eye on the owner's real screen**, with a tab of offsets (0, 0.5, 1, 1.5 px): the display scale of headless screenshots differs, so they only give the direction. Align icons on the lowercase letters next to them, not on the box of the text.
4. When the owner points at something, change only that, and say what changed. **When their words leave a visible detail open** (« the same color code as the others », while the new element has no rank to pick a color from), ask for that detail by name instead of picking one and taking a short « yes » for it: a red picked that way had to be undone.
5. From then on, **the mock-up is the single source of truth** of the interface: say so in `specs.md`; the wireframe only fixed the first layout and isn't kept up to date.

## 2. Finish: the design tokens

- Write the chosen style in `specs.md` (visual style section) as named tokens: colors (bands, text, weak text, accent, warning, error, the colors of the domain), sizes (spacing, radius, heights), typography (fonts, size and weight per role, number formatting), icons and images (source, license, what is bundled in `assets/` with an `assets/NOTICE.md`), shapes and motion.
- Remove the styles that were not chosen from the page, or keep them in a section that says they were rejected.
- If the project has a `design-journal.md` (or the template it came from), add an entry: the context, the choices kept, the ones rejected, and why.
- The application then implements exactly these tokens, in one module, so that changing a color or a size happens in one place.
