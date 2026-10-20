# Speaker notes — Lesson 11: The record-player UI

**Branch** `ws/simulator/11-ui` · **Optional pool** · ~35 min

This lesson is a bit different from the others. Most of `ui.rs` is egui craft, and
there are not many *interesting* ways for it to go wrong. I kept it because two of
its ideas are genuinely worth knowing, and because the UI is the part you have been
looking at all day.

*(If the day slips, this is the first lesson to turn into a demo.)*

**Goal.** You overlay an egui interface on a Bevy 3D scene, and you drive the whole
visualization from a single `f32`.

---

## Removed on this branch

`sim/sim/src/ui.rs` — the body of `setup_egui`

`sim/sim/src/ui_runner.rs` — the `play_time_sec` advance and clamp, the fine-seek
key handling, and `RunnerGuiState::handle_new_bots`

I kept `viewport_ui`, every button and panel, the camera, the help and error modals,
the file dialog, and all the styling.

Two scoping choices. `viewport_ui` stays because without it *nothing* renders in any
mode, so you would have no baseline and no way to verify anything, and it is egui
version-churn plumbing, not an idea. The transport *buttons* stay and only the state
changes behind them are gone, so the exercise is "make the clock move" rather than
"rebuild an egui layout".

## Idea 1 — the overlay camera (8 min)

```rust
egui_global_settings.auto_create_primary_context = false;
commands.spawn_scene(bsn! {
    PrimaryEguiContext
    Camera2d
    template_value(RenderLayers::none())   // render nothing from the world
    Camera { order: 1 }                     // draw after the 3D camera
});
```

(A plain `commands.spawn((…, Camera { order: 1, ..default() }))` is an equally good
answer; `main` uses Bevy 0.19's `bsn!` scene macro.)

Four lines, and each one is load-bearing. We turn off the automatic egui context and
create our own, attached to a dedicated 2D camera that renders *no* render layers and
sorts *after* the scene camera. The result is a UI layer composited on top of the 3D
view, with no interference in either direction.

The general lesson: in an ECS renderer, "UI on top of the scene" is not a special
mode, it is **a second camera with a render order and an empty layer mask**. If you
come from retained-mode GUI toolkits, this way of seeing it may be clarifying.

`viewport_ui` is the small companion piece, and it is already written for you:
since egui 0.36, panels are shown inside a `Ui` rather than directly on the
`Context`, so every gui pass starts by building a root `Ui` covering the viewport at
`LayerId::background()`. It is version-churn plumbing, so I will not dwell on it.

Note that if you leave `setup_egui` empty, egui falls back to auto-creating its own
context, so the UI still appears; it is just not *ours*. So in case you are wondering
what 11.1 buys you: it is control over which camera the UI lives on.

## Idea 2 — the UI is a pure function of `play_time_sec` (10 min)

This is the real content, and it is a payoff from Lesson 06.

There is **one** piece of mutable playback state:

```rust
if gui_state.play_active { gui_state.play_time_sec += time.delta_secs(); }
gui_state.play_time_sec = gui_state.play_time_sec.min(play_max_sec).max(0.0);
```

Every control writes to that one `f32`, and everything else *reads* it:

- `Home` → 0, `End` → max
- `,` / `.` → ±1 s, rounded to a whole second
- `shift` / `ctrl` + those → ±1 ms
- `ctrl+shift` + those → ±1 tick
- the slider → direct assignment

Then `sync_bot_body`, `sync_bot_wheel` and the status panel all derive from it.

I want to make this point explicitly, because it is transferable: **there is no
playback state machine.** No "seeking" mode, no invalidation, no re-simulation, no
difference between playing forward, scrubbing backwards and stepping one 500 µs tick.
Every one of those is "assign a different float". What would the same feature cost if
the visualizer were driving the physics engine live? *(Ask the room.)* That contrast
is the lesson.

Order matters in one place: clamp, then apply user commands, then clamp again,
because the keyboard handlers can push the value out of range after the first clamp.
Small detail, visible bug (the scrub bar sticks at the end).

## Idea 3 — hot-loading a robot from another thread (8 min)

`handle_new_bots` is more interesting than it looks. Loading a robot means running a
whole 60-second simulation, which takes seconds: far too long to block a 60 fps UI.

So the flow is: the file dialog yields a path → `std::thread::spawn` runs
`run_bot_from_file` off-thread → the result arrives on an `mpsc::Sender` held in the
`RunnerGuiState` resource → each frame, `handle_new_bots` drains the receiver with
`try_recv` and spawns visualizations for whatever arrived.

Two things worth noticing:

- The channel endpoints are wrapped in `Mutex` purely to satisfy Bevy's `Resource`
  bound (`Sync`), not because there is contention. A reasonable "the type system
  asked for it" moment.
- The **exact same channel** is how `server.rs` works. In `Serve` mode an HTTP POST
  of a `.wasm` file gets simulated on a worker thread and its `Sender` is the same
  one the file dialog uses. That is how the competition mode runs: robots arrive over
  HTTP, race, and appear in the ranked stack. One channel, two producers, and the UI
  cannot tell the difference.

  *(Show it if the network cooperates.)*
  ```bash
  cargo run --release -p sim -- serve &
  curl -X POST --data-binary @bots/bot.wasm http://localhost:9999
  ```

`auto_run` then controls whether new arrivals replace a same-named robot and restart
the loop: the competition-display behaviour.

## Verification

```bash
cd sim && cargo run --release -p sim -- run -i bots/bot.wasm
```

Space plays, the scrub bar seeks, `ctrl+shift+.` advances a single tick, and `+` loads
another robot without the UI stuttering. Press `/` for the built-in help: it is the
`HELP_TEXT` markdown constant rendered by `egui_commonmark`, and it also documents
every binding you are implementing. In other words, **the help dialog is the spec for
this lesson.**

## Notes to self

- I keep this short and conceptual. Nobody needs to learn egui's layout API today,
  and if the conversation turns into egui Q&A the slot is gone.
- If I turn it into a demo, the three beats survive intact in about eight minutes:
  the overlay camera, the single `f32`, the channel shared with the HTTP server.
