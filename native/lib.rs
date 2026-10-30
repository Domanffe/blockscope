//! The bundle's logic as a Cargo library, so `cargo test` runs its tests
//! without a Horizon build.
#[path = "../mixins/blockscope/runtime.rs"]
pub mod runtime;
