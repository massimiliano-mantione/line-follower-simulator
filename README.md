# Line Follower Simulator

A deterministic simulator for line-follower robots, built with [Bevy](https://bevyengine.org)
and [Rapier](https://rapier.rs). Robot logic is compiled to a WASM component and runs
sandboxed inside the simulator, and **the fuel that component consumes is the
simulation clock** — so a robot's control-loop latency is part of its simulated
behaviour, reproducibly and independently of the machine it runs on.

| | |
|---|---|
| `sim/` | the simulator: WASM host, physics, sensors, track, recording and playback |
| `bot/` | robot code: the device API, a small `no_std`-style async runtime, examples |
| `wit/` | the WASM component contract between the two |
| `workshops/` | workshop material — see below |

## Running it

```bash
cd sim
cargo run --release -p sim -- test                   # drive a robot yourself
cargo run --release -p sim -- run -i bots/bot.wasm   # simulate one, then watch it
cargo run --release -p sim -- serve                  # accept robots over HTTP
cargo test
```

Press `/` or `F1` in the app for the command help.

## Workshops

Two workshops are built on this repository, and they are independent of each other.

**[`workshops/line-follower`](workshops/line-follower)** — *programming* a line
follower: PID control, tuning, telemetry, async robot logic. Slides only; the code
you write lives in `bot/`.

**[`workshops/simulator`](workshops/simulator)** — *building* the simulator, in
twelve lessons. Each lesson is a branch holding this application with one feature
removed. Start with
[`workshops/simulator/README.md`](workshops/simulator/README.md).

## Licence

See [LICENSE](LICENSE).
