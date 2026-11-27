# setup prompt

Paste this into Claude Code, Cursor, or any agent that can edit the repo.

> Add `dither-kit-rune` to this Rust project. It is a Rune component crate: ordered-dither
> charts, avatars, buttons and gradients, with the deterministic engine in `dither-core`
> and the Rune integration in `dither-kit-rune`.
>
> 1. Read `registry.json` for the item list and `registry/<item>.rs` for starter code.
> 2. Add the two crates as path or git dependencies. Rune must be pinned to
>    `dc64314b816afdd77689209f14647407034940cb`, and the workspace needs Rune's
>    `[patch.crates-io]` block copied from the root `Cargo.toml`.
> 3. Build the chart once in your state, then forward pointer input from the view:
>    `on_pointer_move_local` to `hover_at`, `on_hover_change` to `leave`, and
>    `on_key` for arrow-key inspection.
> 4. Run `cargo run -p gallery -- --scene <scene>` to compare against the reference scene.
>
> Do not vendor the engine. Keep `dither-core` free of UI dependencies.
