---
name: excalidraw-structures
description: Sketch a structure as an editable Excalidraw drawing of cards (the windows of an application and how one leads to another, its panels, the groups of its settings, the commands and their options, the tree of the contents of a site), with the current state and two leads side by side, in docs/wireframes/, which the owner rearranges by hand. Use it during the specifications, before any wireframe, when a question of structure or organization is open.
---

# Excalidraw structures

Goal: make an abstract structure concrete before it is written in `specs.md`, so that the owner sorts it with their hands rather than by reading a list.

## 1. Draw

One drawing per question of structure: the windows of an application and how one leads to the other, its panels, the groups of its settings, the commands of a command line and their options, the tree of the contents of a site. Moving cards by hand makes an abstract structure concrete; the owner loved it on sylent.art.

- One card per item (one group: its box and its label), the current state on the left, **two leads** side by side, a parking area for the items not placed yet, and the open questions in a corner.
- Draw it during the specifications, **before the wireframes**: on sylent.art, the tree of the contents came after the build, and the addresses changed twice.
- Once the owner has moved the cards and chosen a lead, write the structure in `specs.md`.
- Write the drawing with a Python generator kept in the scratchpad, from `scripts/excalidraw.py` (read its docstring), to `docs/wireframes/<name>.excalidraw`, then check it with `scripts/preview.py` (an SVG, then a PNG to look at): no card that overlaps another, every card grouped.

## 2. Hand it over

- Tell the owner how to open it: in VS Code with the **Excalidraw** extension (`pomdtr.excalidraw-editor`), or on excalidraw.com (**Open**, then **Save to…**). The VS Code extension neither autosaves nor reloads a file changed on disk: **Ctrl+S** before asking Claude Code to read it; before Claude Code rewrites it, save and close the tab.
- Read the owner's version (`Drawing.load`), list what moved before touching anything, and commit it first.
