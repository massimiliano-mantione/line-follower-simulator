# Lesson 06 — Record and replay

> **Goal.** Record a race into a flat array, then render it as a pure function of
> playback time — and stack several robots on the same track, ranked.

## The setup

The simulation is a batch job: it runs a 60-second race in a couple of seconds (and
at a 100 µs period, *slower* than real time). The visualizer is a 60 fps interactive
app with a scrub bar. How do you connect them?

Not by simulating live. You cannot scrub backwards through a physics engine, you
cannot show five robots that raced at different moments side by side, and — the
clincher — rendering would have to happen inside the robot's own clock, so **drawing
a frame would cost the robot simulated time**.

So: **record, then replay.** The simulator writes one body `Transform` and two wheel
angles per tick into `ExecutionData`, and the visualizer never touches physics again.

And the architectural punchline:

> **The same entity tree is used twice, with two disjoint component sets.**

During simulation the bot entities carry `Collider`, `RigidBody`, `ImpulseJoint`,
`Velocity`. During visualization the *same* spawn functions build the same tree, but
what is attached is `BodyExecutionData` and `WheelExecutionData`. One
`EntityFeatures` enum — `Physics`, `Visualization`, `PhysicsAndVisualization` —
decides which systems and components exist. (`test` mode asks for **both**, which is
why you can drive a physically simulated robot with the arrow keys.)

## Your task

| File | | What |
|---|---|---|
| `execution-data/src/lib.rs` | 6.1 / 6.2 | `at_time_secs` for the body and for a wheel |
| | 6.3 | `status_at_time` — Waiting / Racing / EndedAt / OutAt |
| | 6.4 | `final_status` — the ranking key |
| | 6.5 | `Ord for BotFinalStatus` — rank two results |
| `sim/src/data.rs` | 6.6 | `store_data` — record one sample per tick |
| `sim/src/visualizer.rs` | 6.7 | `build_transform` — the stacked layer offset |
| | 6.8 | `spawn_bot_visualization` — build one robot's world |
| | 6.9 | `sync_bot_layers` — re-sort the stack by rank |
| | 6.10 / 6.11 | `sync_bot_body` / `sync_bot_wheel` — apply playback time |

Suggested order: 6.1, 6.6, 6.10 (now something moves), 6.2, 6.11, then 6.8 to get a
robot on screen at all, then 6.3–6.5 and 6.7/6.9 for the ranking and stacking.

All ~400 lines of mesh assembly in `bot/vis.rs` are left alone — that is craft, not
concept.

## How to verify

```bash
cd sim
cargo test -p execution-data            # nine tests, eight currently red
cargo run --release -p sim -- run -i bots/bot.wasm
```

Press space: the robot moves and its wheels turn at a rate matching its speed. `Home`
and `End` jump to the ends, `,` and `.` scrub by a second, `ctrl+shift+.` advances a
single 500 µs tick. Then use the `+` button to load a second robot and watch the two
track copies stack and re-sort by result.

Before 6.6 works, every run reports `data has 0 frames`.

## Hints

<details>
<summary>Hint 1 — 6.1, is there really no trick?</summary>

No. Divide the time by the period to get a sample index, clamp it into range, and
return that element. That is what makes scrubbing and backwards playback free.

Do handle the empty recording: `steps.len() - 1` on an empty vector underflows, and a
robot that traps during `setup` produces exactly that.
</details>

<details>
<summary>Hint 2 — 6.3, everybody's lap time is one second too long</summary>

You are reporting absolute playback time. Subtract the robot's own start time — the
countdown is not part of anyone's lap.
</details>

<details>
<summary>Hint 3 — 6.5, how do I compare four different kinds of outcome?</summary>

`kind_rank` and `kind_value` are already written. Compare the ranks first; only when
they are equal does the value decide. And since the values are `f32`, `Ord` needs a
total comparison — look at `f32::total_cmp`.
</details>

<details>
<summary>Hint 4 — the wheels teleport to the middle of the robot</summary>

You replaced the wheel's whole `Transform`. It is already positioned relative to the
body; only its `rotation` should change.
</details>

<details>
<summary>Hint 5 — the winner ends up hidden at the bottom of the stack</summary>

Ascending rank order puts the best result first, and layer 0 is the *bottom* of the
stack. One of those needs reversing.
</details>

<details>
<summary>Hint 6 — playback lags the recording by one tick</summary>

Check which schedule `store_data` runs in and what it is ordered after. Sampling
before this tick's sensors and transforms have been computed shifts the entire
recording.
</details>

## Solution

```bash
git diff ws/simulator/06-replay..main -- sim/execution-data/src/lib.rs sim/sim/src/data.rs sim/sim/src/visualizer.rs
```

## Where this goes next

That is the spine of the workshop complete: fuel became a clock, the clock drove the
physics, the physics moved a body, the body carried sensors, and the run became a
recording you can inspect.

If you have time left, the optional pool: `07-async-host`, `08-bot-async`,
`09-telemetry`, `10-track`, `11-ui`, `12-new-device`.
