# Architecture

```
src/
├── lib.rs                 library crate: everything except the GUI
├── astro/                 pure astronomy (no calendars, hemispheres or config)
│   ├── time.rs            Julian Day ⇄ UTC, ΔT
│   ├── sun.rs             equinoxes/solstices (Meeus 27), solar longitude (Meeus 25)
│   └── moon.rs            principal phases (Meeus 49), illumination (Meeus 48)
├── calendar/              calendar meaning of astronomical instants
│   ├── mod.rs             Hemisphere, Season, Sabbat, DateBasis, MoonPhase
│   ├── almanac.rs         Almanac::compute(now, tz, settings) → snapshot
│   └── lore.rs            descriptive texts per language, with sources
├── i18n.rs                Lang, UI strings, names, date and number formatting
├── companion.rs           Character (Hermit, Fool), mood and lines, from the almanac
├── config.rs              TOML config, window state, XDG autostart entry
├── main.rs                CLI, TZ override, transparent desktop-layer window
└── ui/                    egui front end (binary crate only)
    ├── app.rs             figure + speech bubble (talk, almanac, settings), placement, menu
    ├── puppet.rs          generic rig, per-character choreography, textured-mesh rendering
    ├── desktop.rs         X11 EWMH hints: below windows, skip taskbar/pager, all workspaces
    ├── assets.rs          embedded PNGs → mipmapped textures
    ├── text.rs            display strings, built once per recompute
    ├── glyphs.rs          moon-phase disk, label and progress helpers
    └── theme.rs           palette from the card art, Nunito fonts, style

assets/
├── hermit/                5 puppet parts + rig.toml
├── fool/                  7 puppet parts (incl. the dog) + rig.toml
├── icons/                 4 circular crops from the cards
└── fonts/                 Nunito SemiBold/ExtraBold (OFL)

tools/
├── prepare_puppet.py      scan → cut-out → puppet parts, per character (Pillow only)
└── prepare_icons.py       scans → circular icons
```

## Rules between the layers

- `astro` works in Julian Ephemeris Days and degrees. It has no notion of local time.
- `calendar` is generic over `chrono::TimeZone`, so tests use `FixedOffset`
  and the app uses `Local`. The fixed-versus-astronomical distinction is
  explicit in `DateBasis`.
- `companion` is pure: almanac in, mood and lines out. It's unit-tested.
- The UI only reads `Almanac`, `Texts` and `companion`. It performs no astronomy.

## The puppets

Each companion is cut from its 1910 scan, then split by polygons into
parts.
- **The Hermit:** his flat teal sky is colour-keyed out.
- **The Fool:** he stands before yellow sky, snow and rock, so the
  background is grown inwards from outside his outline through sky-, snow-
  and rock-coloured pixels only. Smith's ink outlines stop the growth, and
  they are thickened first so small gaps can't leak.

`rig.toml` lists parts in drawing order, each with an offset, an optional
pivot, an optional `parent` and `hang = true` for pendulums.
- **Transforms:** the renderer composes affine transforms parent-first. A
  hanging part (the lantern, the bag) keeps its own world angle, so it swings
  with gravity, not with the arm carrying it.
- **Joints:** each moving part keeps an overlap with the body around its
  pivot. Parts drawn in front of the body are also backed near the joint with
  flat cloth colour, so a rotation never opens a hole.

Choreography lives in `puppet.rs`, one entry per character.
- **Mood poses:** a pose per mood, eased over 0.9 s.
- **Gestures:**
  - Hermit: swing, nod, lift.
  - Fool: step, look up, offer the rose, dog hop, bag swing.
  Each fades fully to rest over its last 15%.

## The window

The window is transparent and frameless, and has **one fixed size**: the
figure at its bottom-left corner, with room above for the largest bubble.
- **Never resized while interacting:** it is resized only when the figure
  size or bubble width settings change. Resizing a transparent GL window
  mid-interaction produced visible glitches.
- **Layout:** everything is laid out from the window's actual edges. When a
  bubble's height changes, the frame is laid out again (`request_discard`)
  before it is shown.
- **Click-through:** on X11 an *input shape* limits the clickable area to the
  figure and the open bubble, so the empty parts let clicks through to the
  desktop icons. It covers the whole window while a menu is open or a drag is
  in progress.
- **Dragging:** the window follows the pointer's real screen position, read
  from X11 (`query_pointer`), relative to where the button went down. It
  keeps redrawing while dragging and settles on the release position, so it
  can't lag or overshoot. Window managers don't move dock windows, which is
  why the app moves itself.
- **Desktop layer:** on X11 the window is a dock-type window kept *below* all
  others, out of the taskbar and pager, on all workspaces. Cinnamon ignores
  these requests while the window is still being mapped, so they are sent
  again at 0.5 s and 2 s.
- **Saved position:** where the figure's feet stand.

## Performance

- **Rendering:** the window draws only on input, at local midnight or every
  10 minutes for the data, and during a gesture at 20 fps for about 2–3 s
  every 20–45 s (35–70 s when sleepy). Measured idle (figure only, animations on):
  about 220 ms of CPU per minute (0.4%) and 28 MB private memory.
- **Work per recompute:** a few dozen trigonometric series, and all strings
  are formatted then, not per frame.
- **Binary and assets:** the renderer is glow (OpenGL); wgpu and accesskit
  are disabled. The release binary is 7.2 MB including about 500 KB of art and
  fonts.
- **Memory:** about 28.5 MB private. The rest of RSS is shared Mesa libraries.

## Desktop integration notes

- **Menu:** quit and settings live in the right-click menu and the bubble,
  since there are no window buttons.
- **Autostart:** writes or removes `~/.config/autostart/sabbatia.desktop`.
- **Show Desktop (Super+D):** not tested. Dock-type windows are usually left
  visible.
