# Version 0.7.0

This is a release that contains big rewrites upder the hood. They might not improve compatibility that much initially (I
would expect the compat to be worse, actually), but will make implementing some engine features possible at all.

- Rewrite of the rendering engine to actually use the shaders/render passes in an equivalent to the way original game
  does it.
- Addition of `shin-window`: a common framework for starting a winit window, initializing wgpu and handling input. This
  makes test apps (like `shin-video`'s play example) easier to maintain. This is also where most of the web support will
  be contained.
- Rewrite of the input handling to somewhat resemble what the original game does.
- Rewrite of the `MessageLayer` to include all the features the original game has. Now we have messagebox sliding
  animations, overflow handling, keywait animations and voice support (though this needs still needs implementation from
  the audio engine side).
- Rewrite of the `LayerGroup`, `PageLayer` and `ScreenLayer` classes, which are much closer to what the game does.
- Stub implementation of `NewDrawableLayer` framework, paving the way for implementing various funny effects the scrip
  uses sometimes.
- Rewrite of `PictureLayer` and `BustupLayer` to no longer use stitched together textures (RIP), but do it the same way
  game does, by rendering in blocks. This indirectly fixes the ugly face seams we were getting due to some rounding
  bugs.

# Version 0.6.1

This release adds support for getting raw opus audio from nxa files to `sdu`. This allows preserving audio data without
additional re-encoding losses.

It also updates various dependencies of the engine, but this shouldn't change the visible functionality.

# Version 0.6.0

This is just an update of the changes accumulated over they year. This doesn't improve compat, mostly small internal
changes

- Add a very basic linear scenario disassembler. The output format is not stable yet, probably should work on making it
  compatible with what `shin-asm` expects.
- Implement & expose in sdu `shin-asm`: a way-too-much over-engineered assembler for SNR files. It is still largely
  unfinished and can assembly only very basic files.
- Add an optional gstreamer backend to `shin-video`, allowing for hardware-accelerated decoding. Not build by default,
  unsure on how to distribute on windows yet.
- Use typed `NumberSpec` values, which will get lowered to a more concrete type than `i32` when computed.
- Various dependency updates.

# Before that

Before this, I did not keep any changelog, sorry. Looking at the commits is the best you will get
