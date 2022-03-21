# septadrop

![screenshot](screenshot.png)

A block game made in Rust and [SFML](https://www.sfml-dev.org/).

For packaging instructions, see the [build folder](build).

## Game

### Controls

- `🠔`/`🠖` arrow keys: horizontal movement
- `🠕` arrow key: rotate
- `🠗` arrow key: fast forward
- `Space`: hard drop
- `Esc`: pause (will also automatically pause on lost focus)

### Shortcuts

- `Ctrl`+`1`, `Ctrl`+`0`: 100% scale (default)
- `Ctrl`+`2`: 200% scale
- `Ctrl`+`4`: 400% scale
- `Ctrl`+`-`: Zoom out
- `Ctrl`+`+`: Zoom in
- `Ctrl`+`Q`: Quit

### Scoring and mechanics

You receive 40 points for a 1-clear, 100 points for a 2-clear, 300 points for a 3-clear, and 1200 points for a 4-clear, multiplied by your level plus one. Your level increases by one for every five lines cleared. If you would like to configure these values, change them in the [config file](src/config.rs#L19-L23) and [rebuild](build).

The next dropped block is picked purely randomly, there is no semi-random queue system. As a result of this, runs are heavily influenced by RNG. A queue system may be added in the future, but no promises.

## Acknowledgements

- Graphics are using the [Lost Century 24 Palette](https://lospec.com/palette-list/lost-century-24) by CalmRadish, and are made with the wonderful sprite and pixel art editor [Aseprite](https://github.com/aseprite/aseprite/).
- Text is using [Boxy Bold Font](https://opengameart.org/content/boxy-bold-font) by Clint Bellanger, CC0
- Sound effects are made using [jsfxr](https://github.com/chr15m/jsfxr) by Eric Fredricksen, a JavaScript port of DrPetter's [sfxr](http://www.drpetter.se/project_sfxr.html).
- and last but not least to my friend [CarlyRaeJepsenStan](https://github.com/CarlyRaeJepsenStan) for feedback and playtesting.