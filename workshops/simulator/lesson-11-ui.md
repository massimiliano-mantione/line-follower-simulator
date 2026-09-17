# Lesson 11 — The record-player UI

> **Optional, and the smallest of the set.** Most of the UI code is egui craft that
> is hard to get wrong interestingly. Three ideas in it are genuinely worth having,
> and those are the three you implement.

> **Goal.** Overlay an egui interface on a Bevy 3D scene, and drive the entire
> visualization from a single `f32`.

## The three ideas

**1. "UI on top of the scene" is just a second camera.** Not a special mode — an
entity with `Camera2d`, a render order *after* the scene camera, and an empty render
layer mask so it draws nothing from the world. If you come from a retained-mode GUI
toolkit this is worth sitting with for a minute.

**2. The whole UI is a pure function of `play_time_sec`.** There is one piece of
mutable playback state, a single `f32`. Every control writes to it; the
visualization systems read it. That is *why* there is no seeking mode, no
invalidation, no re-simulation, and no difference between playing forward, scrubbing
backwards, and stepping one 500 µs tick. Each is "assign a different float".

**3. Loading a robot happens on another thread.** Simulating a 60-second race takes
seconds — far too long to block a 60 fps UI. So it runs off-thread and the result
arrives over an `mpsc` channel. The same channel is what `server.rs` sends down when
a robot arrives by HTTP POST: one channel, two producers, and the UI cannot tell.

## Your task

| File | | What |
|---|---|---|
| `ui.rs` | 11.1 | `setup_egui` — the overlay camera |
| `ui_runner.rs` | 11.2 | advance the playback clock |
| `ui_runner.rs` | 11.3 | the fine-seek controls |
| `ui_runner.rs` | 11.4 | `handle_new_bots` — drain the channel, spawn visualizations |

**The help dialog is the spec for 11.3.** Press `/` or `F1` in the running app: the
`HELP_TEXT` markdown constant documents every binding you are supposed to wire up.

`viewport_ui`, the camera buttons, the help and error modals, the file dialog and all
the styling are left alone.

## How to verify

```bash
cd sim
cargo run --release -p sim -- run -i bots/bot.wasm
```

- Before 11.4, loading a robot appears to do nothing at all.
- Before 11.2, the play button toggles but time never moves.
- After both: space plays, the scrub slider seeks, `Home`/`End` jump to the ends,
  `,`/`.` move by a second, `ctrl+shift+.` advances a single tick, and `+` loads
  another robot without the UI stuttering.

Then the server path, which exercises the same channel from the other end:

```bash
cargo run --release -p sim -- serve &
curl -X POST --data-binary @bots/bot.wasm http://localhost:9999
```

## Hints

<details>
<summary>Hint 1 — 11.2, playback runs away past the end of the recording</summary>

Clamp after advancing. And note the clamp has to happen *again* after the keyboard
handlers in 11.3 run, since they can push the value back out of range — look for the
"Clamp time again" comment.
</details>

<details>
<summary>Hint 2 — 11.3, holding the key jumps a second per frame</summary>

`just_pressed` versus `pressed`. Coarse seeking should fire once per key press; fine
seeking is the one that should repeat while held.
</details>

<details>
<summary>Hint 3 — 11.4, the UI freezes for several seconds when I load a robot</summary>

You are blocking on the channel. `try_recv` in a `while let`, never `recv`.
</details>

<details>
<summary>Hint 4 — in serve mode, re-submitting a robot stacks a second copy</summary>

That is the `auto_run` rule: match on `config.name` against the existing
`BotVisualization` entities and despawn the old one first.
</details>

## Solution

```bash
git diff ws/simulator/11-ui..main -- sim/sim/src/ui.rs sim/sim/src/ui_runner.rs
```
