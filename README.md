# Hollowdeep

A grimdark terminal roguelike RPG built in Rust.

*Descend into the cursed depths, face eldritch horrors, and forge your path through corruption and darkness.*

## Screenshots

```
┌─ Sunken Catacombs - Floor 1 [Unicode] ─────────────────────┐┌─ Status ─────────────┐
│                                                            ││ Hero                 │
│     ████████████████                                       ││                      │
│     █··············█     ████████████                      ││ HP: 100/100          │
│     █··☺···········█     █··········█                      ││ MP: 50/50            │
│     █··············███████··········█                      ││ SP: 50/50            │
│     █··············▼·····█··········█                      ││                      │
│     █··············███████··········█                      ││ Level 1              │
│     ████████████████     ████████████                      ││ XP: 0/100            │
│                                                            ││                      │
└────────────────────────────────────────────────────────────┘└──────────────────────┘
┌─ Messages ─────────────────────────────────────────────────────────────────────────┐
│ You descend into the Hollowdeep...                                                 │
└────────────────────────────────────────────────────────────────────────────────────┘
```

## Features

### Implemented (Phase 1)

- **Multiple Render Modes**
  - ASCII: Classic roguelike (`@`, `#`, `.`)
  - Unicode: Rich symbols (`☺`, `█`, `▼`)
  - Nerd Font: Icon support
  - Kitty Graphics: Full PNG sprite rendering (Ghostty/Kitty/WezTerm/iTerm2)

- **Procedural Generation**
  - Room + corridor dungeons (Sunken Catacombs, Hollow Cathedral)
  - Cellular automata caves (Bleeding Crypts, The Abyss)
  - 4 distinct biomes with unique generation

- **Core Systems**
  - Field of view with shadowcasting
  - True color rendering (24-bit RGB)
  - Adaptive UI layout
  - Message log system
  - Game state machine
  - Floor descent

### Planned (Phase 2+)

- [ ] ECS-based entities with real stats
- [ ] Turn-based combat system
- [ ] Enemy AI (melee, ranged, caster archetypes)
- [ ] Item system with affixes and synergies
- [ ] Grid-based inventory (RE4 style)
- [ ] Skill shrines and progression
- [ ] Boss fights with multiple phases
- [ ] Save/load system
- [ ] Lua mod support
- [ ] Audio (kira)

## Installation

### Requirements

- Rust 1.70+ (install via [rustup](https://rustup.rs))
- A terminal with true color support
- **Recommended:** Ghostty, Kitty, WezTerm, or iTerm2 for sprite rendering

### Build & Run

```bash
git clone https://github.com/yourusername/hollowdeep.git
cd hollowdeep
cargo run --release
```

## Controls

| Key | Action |
|-----|--------|
| Arrow Keys / HJKL | Move |
| Y U B N | Diagonal movement |
| Space / . | Wait |
| > | Descend stairs |
| I | Inventory |
| C | Character sheet |
| M | Map view |
| R | Cycle render mode |
| Esc | Pause menu |
| Ctrl+Q | Quit |

## Project Structure

```
hollowdeep/
├── Cargo.toml
├── README.md
├── src/
│   ├── main.rs              # Entry point
│   ├── lib.rs               # Library root
│   ├── game/                # Game state, turns, time
│   ├── ecs/                 # Entity Component System
│   ├── world/               # Map, tiles, generation
│   ├── render/              # Multi-mode rendering
│   │   ├── mode.rs          # Render mode detection
│   │   ├── kitty.rs         # Kitty graphics protocol
│   │   ├── sprites.rs       # Sprite sheet loading
│   │   └── tilemap.rs       # Tile rendering
│   ├── entities/            # Player, enemies, NPCs
│   ├── combat/              # Stats, damage, abilities
│   ├── items/               # Items, inventory, loot
│   ├── progression/         # XP, skills, difficulty
│   ├── ui/                  # Terminal UI (ratatui)
│   ├── audio/               # Sound (kira)
│   ├── save/                # Save/load
│   ├── mods/                # Lua modding
│   └── data/                # RON data loading
└── assets/
    ├── sprites/             # PNG sprite sheets
    ├── data/                # Game data (RON files)
    │   ├── items/
    │   ├── enemies/
    │   ├── biomes/
    │   └── skills/
    └── audio/
        ├── sfx/
        └── ambient/
```

## Adding Custom Sprites

Place sprite sheets in `assets/sprites/`:

```
assets/sprites/
├── tileset.png      # Terrain tiles (floor, wall, stairs, etc.)
├── entities.png     # Player, enemies, bosses
├── items.png        # Weapons, armor, potions
└── ui.png           # UI elements
```

### Sprite Sheet Format

- Use a grid layout (e.g., 16x16 pixels per sprite)
- Sprites are numbered left-to-right, top-to-bottom
- Transparent backgrounds supported (PNG alpha)

### Predefined Sprite IDs

| ID Range | Category |
|----------|----------|
| 0-99 | Terrain (floor, wall, doors, stairs, decorations) |
| 100-199 | Entities (player, enemies, bosses, NPCs) |
| 200-299 | Items (weapons, armor, consumables) |
| 300-399 | UI elements |
| 400-499 | Effects |

## Technical Stack

| Component | Library |
|-----------|---------|
| Terminal UI | ratatui + crossterm |
| ECS | hecs |
| Data Format | RON (Rust Object Notation) |
| Audio | kira |
| Scripting | mlua (Lua 5.4) |
| RNG | rand + noise |

## Design Philosophy

- **Grimdark theme**: Body horror, corruption, moral ambiguity
- **Environmental storytelling**: Lore through items and environment, not cutscenes
- **Risk/reward**: Multiple paths with varying difficulty and loot
- **Classless progression**: Build emerges from items and shrine choices
- **Deep item synergies**: Specific combinations create powerful effects
- **Adjustable difficulty**: Player chooses challenge level per run

## Biomes

| Zone | Floors | Theme | Generation |
|------|--------|-------|------------|
| Sunken Catacombs | 1-5 | Undead, tutorial | Rooms + corridors |
| Bleeding Crypts | 6-10 | Blood cultists, corruption | Cellular automata |
| Hollow Cathedral | 11-15 | Fallen angels, vertical | Large open areas |
| The Abyss | 16-20 | Eldritch horrors, final boss | Mixed |

## License

MIT License - See LICENSE file for details.

## Contributing

Contributions welcome! Please read CONTRIBUTING.md before submitting PRs.

## Acknowledgments

- Inspired by classic roguelikes: NetHack, DCSS, Angband
- Modern influences: Hades, Slay the Spire, Dead Cells
- Built with the incredible Rust gamedev ecosystem
