//! Host bindings for the `line-follower-robot` world.
//!
//! The `store` flag on the imports makes `bindgen!` hand every host function an
//! [`Access`](wasmtime::component::Access) to the store instead of just the
//! host data, which is what lets the implementation in
//! [`crate::wasm_host`] read the consumed fuel (and therefore the simulated
//! time) on every call.
wasmtime::component::bindgen!({
    path: "../../wit",
    world: "line-follower-robot",
    imports: { default: store | trappable },
});
