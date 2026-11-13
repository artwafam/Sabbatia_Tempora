# Sabbatia Tempora

<img width="1920" height="880" alt="banner" src="https://github.com/user-attachments/assets/65ec3953-d2a4-4cfc-878b-039b93edc352" />

A small companion that lives on your Linux desktop, among the icons and
folders. It is a figure from the 1909 Rider–Waite–Smith tarot: **the
Hermit** with his lantern, or **the Fool** with his little dog. He keeps
track of the season, the moon, the Wheel of the Year and the next Sabbat.

Most of the time only the figure is there, beneath your windows. Click him
and he talks in a speech bubble; click again for the next thing he has to
say. From the bubble, **Almanac ›** opens four tiles:

| Tile | Shows | Click to expand |
|---|---|---|
| **Season** | Astronomical season, start–end dates, progress | Start and end instants, description |
| **Moon** | Phase and illumination | Phase disk, lunar age, next four principal phases |
| **Wheel** | Current period between two festivals | Description of the period, the Sun's ecliptic longitude |
| **Next Sabbat** | Name, days remaining, date | How the date is set, history, significance, symbolism, sources |

It works fully offline, with no background services and no network access.

## The companions

<img width="400" height="836" alt="almanac" src="https://github.com/user-attachments/assets/8891c295-bcaa-4533-8572-e448a3aaa9ef" />

Each is cut out of a public-domain scan of the original card and jointed
like a paper puppet.

- **The Hermit:** head, lantern arm and staff, with the lantern on a pin so it
  swings.
- **The Fool:** head, the arm carrying his staff and bag (the bag swings),
  the arm holding the rose, the stepping leg, and the dog.
- **Moods:** they react to the almanac. The Hermit raises his lantern when a
  Sabbat is near or the moon is full, and bows his head on a new moon or late
  at night. The Fool lifts his rose and his dog perks up, or he hangs his
  head.
- **Movement:** now and then they move: a swing, a nod, a step, a hop. Between
  these gestures nothing is redrawn.
- **Choosing:** switch companions in the settings or from the right-click
  menu. Animations can be turned off.

## How they talk

**Clicking the figure** gives the informative lines, and each further click
gives the next:
- the next Sabbat and how far away it is;
- a full or new moon, or tonight's phase and illumination;
- a season about to turn;
- the current Wheel period.

**Talk ›** in the bubble (or "Talk" in the right-click menu) starts a
conversation in the companion's own voice. Click the bubble or Talk › again
to continue it. Conversations never replace the informative lines; they
happen only when you ask, or when the companion speaks up on his own.

The companions are characters, not assistants. They each have their own
voice:
- **The Hermit:** short, calm, dryly funny.
- **The Fool:** curious, easily distracted, always ready to go, with a dog
  who has opinions.

About 50 entries each, in English and Brazilian Portuguese:
- **Contents:** greetings, seasons, moon phases, each Sabbat, time of day,
  humour, and short conversations that continue when you click again (the
  bubble shows "…" when there is more).
- **What they react to:** what the almanac says today. A festival, a full or
  new moon, or a season turning gets mentioned first, and then not again for
  a few hours. Recent lines aren't repeated.
- **Gestures:** each line comes with a gesture. The Hermit raises his
  lantern, bows his head, or stays perfectly still. The Fool lifts his rose,
  swings his bag, steps or ponders, and the dog hops.
- **Speaking unprompted:** rarely, and never at night. The Fool might
  comment now and then, or call out after a long quiet. The Hermit mostly
  keeps his silence, and speaks up only for something worth noticing. Such a
  bubble closes by itself after 12 seconds.

## On the desktop

- **Placement:** he sits on the desktop layer, beneath all windows, on every
  workspace, and out of the taskbar and window switcher (on X11). Set
  `always_on_top = true` to have him float above windows instead.
- **Moving him:** drag him; his spot is remembered by where his feet stand.
  The bubble opens upwards, so he never moves when it opens.
- **Menu:** right-click him or his bubble for Talk, Almanac, Settings, Switch
  companion, Switch hemisphere and Quit.
- **The talk bubble:** it closes by itself after 20 seconds if left alone.

The transparent background needs a compositing window manager. Cinnamon
always composites. On Wayland the compositor decides where windows go.

## Languages

English and Português (Brasil). The language follows the system locale by
default and can be changed in the settings. Both the interface and the
season and Sabbat texts are translated.

## Install

Requires a Rust toolchain (1.95+). On Linux Mint, also install `build-essential`.

```sh
./packaging/install.sh            # builds and installs into ~/.local
./packaging/install.sh uninstall
```

For a Debian package, run `cargo install cargo-deb && cargo deb`.

```sh
sabbatia            # open the widget
sabbatia --print    # print today's almanac in the terminal
```

## Configuration

`~/.config/sabbatia/config.toml` is created on first run and documents itself.
Most options are also in the settings panel.

```toml
hemisphere = "southern"          # or "northern"; default southern
cross_quarters = "traditional"   # or "astronomical"
latitude = -23.55                # optional; sets the hemisphere if none is given
timezone = "America/Sao_Paulo"   # IANA name; empty = system zone
language = "auto"                # "en" or "pt-BR"
character = "hermit"             # or "fool"
figure_size = 150.0              # figure height in pixels (90–280)
animations = true
always_on_top = false            # false: on the desktop, beneath windows
autostart = false
width = 300.0                    # almanac bubble width
```

## How dates are determined

**Seasons** are astronomical. They begin at the instants of the equinoxes and
solstices, in your local time zone.

**Sabbats.** The solstice and equinox festivals are astronomical. Samhain,
Imbolc, Beltane and Lughnasadh fall on fixed traditional days (the 1st of
Feb, May, Aug and Nov, celebrated from the eve). Alternatively, with
`cross_quarters = "astronomical"`, they use the moments when the Sun reaches
45°, 135°, 225° or 315° of ecliptic longitude. The expanded view shows both
dates. The Southern Hemisphere is the default, with the wheel turned by six
months.

| Quantity | Method | Accuracy |
|---|---|---|
| Equinoxes, solstices | Meeus, *Astronomical Algorithms* (2nd ed.), ch. 27 | < 1 min (verified against published 2000/2024/2026 times) |
| Cross-quarter points | Meeus ch. 25 solar longitude, iterated | ≈ 15 min |
| Principal lunar phases | Meeus ch. 49 | < 1 min (verified against Meeus examples and the 2024–25 eclipses) |
| Illuminated fraction | Meeus ch. 48 | < 1 % |

## Development

```sh
cargo test                       # 65 tests
cargo run
SABBATIA_PREVIEW=talk cargo run  # open with today's informative line showing
```

`SABBATIA_PREVIEW` also accepts `chat` (a conversation opener), `almanac`,
`season`, `moon`, `wheel`, `sabbat`, `settings`, `happy` and `sleepy`.
`talk:<id>` or `talk:<id>:<step>`
shows one particular line, for example `talk:h.waxing:1` (ids are in
`src/dialogue/lines.rs`). The art is rebuilt from the
original scans with `tools/prepare_puppet.py hermit|fool` and
`tools/prepare_icons.py`.

See [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) and [CREDITS.md](CREDITS.md).

## License

MIT, see [LICENSE](LICENSE). The tarot artwork is public domain and the
Nunito font is under the SIL Open Font License; see [CREDITS.md](CREDITS.md).
