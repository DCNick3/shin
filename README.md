
This is an attempt at re-implementation of game engine used by most [visual novels](https://en.wikipedia.org/wiki/Visual_novel) released by [Entergram](http://www.entergram.co.jp/).

## Status

The initial implementation is focused on running switch version of [Umineko When They Cry Saku: Nekobako to Musou no Koukyoukyoku](https://tinfoil.io/Title/01006A300BA2C000), with the intention to support other games in the future.

As of writing, most of the basic game functionality works:
- Character sprites
- Backgrounds
- BGM & SFX
- Text

However, there are still a lot of missing advanced features and shortcuts in the above. It's not playable yet.

The intent is to follow the engine as closely as possible, so that it can be used to run the original games.

![screenshot.png](screenshot.png)


## Try it

To try it out you need to have [Rust](https://www.rust-lang.org/) installed.

You would also need to have the game files. Extract the romfs from the game dump, which would give you a `data.rom` file. It's sha256sum should be `6d90eb0bacacf769a7e4634407622b047acd711c47debb28136d7bab3fd0e591`.

Then run the following commands in the `shin` directory:

```bash
cargo run --release
```

If you encounter any issues, please open an issue on GitHub.

## What else is in the box

Aside from the game engine, this repo also includes `shin-core` - a library for working with the game data.

There is also `sdu` - a CLI interface for the `shin-core` library. It can be used to extract game data, and to convert it to other more conventional formats.

You can install it with `cargo install --path sdu`.

For now, we have support for extracting the following data:
- `.rom` - Game data archive
- `.bup` - Character sprites
- `.pic` - Backgrounds & CGs
- `.nxa` - Game audio
- `.snr` - Limited support for game scenario (no decompilation, only tracing the execution)
- `.fnt` - Font data
- `.txa` - Texture archives (used mostly for UI)

Support for other formats used is planned.
