# shin_tasks

This crate provides an async and compute task executor for `shin`.

For compute tasks, it wraps rayon and provides a way to `.await` spawned tasks.

For async tasks, it wraps `smol`'s `async-executor`.
