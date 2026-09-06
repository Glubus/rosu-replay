//! These exercise the Rust side of the WASM bindings, not a browser runtime.
#![cfg(feature = "wasm")]
mod support;
use rosu_replay::{
    wasm::{parse_replay_data_wasm, WasmGameMode, WasmReplay},
    Replay,
};
use support::*;

#[test]
fn bindings_preserve_stable_and_lazer_replays() {
    for bytes in [
        include_bytes!("../assets/test.osr").as_slice(),
        include_bytes!("../assets/test_lazer.osr").as_slice(),
    ] {
        let native = Replay::from_bytes(bytes).unwrap();
        let wasm = WasmReplay::from_bytes(bytes).unwrap();
        assert_eq!(wasm.username(), native.common().username);
        assert_eq!(wasm.score(), native.common().score);
        assert_eq!(wasm.event_count(), native.common().replay_data.len());
        assert_eq!(Replay::from_bytes(&wasm.pack().unwrap()).unwrap(), native);
    }
}

#[test]
fn bindings_surface_corrupt_lazer_metadata_as_an_error() {
    let bytes = fixture(0, 30000001, -1, "16|256|192|1", &[1, 2]);
    assert!(WasmReplay::from_bytes(&bytes).is_err());
}

#[test]
fn bindings_parse_each_ruleset() {
    for (mode, data) in [
        (WasmGameMode::Std, "16|256|192|1,20|128|256|2"),
        (WasmGameMode::Taiko, "16|0|0|1,20|0|0|4"),
        (WasmGameMode::Catch, "16|256|0|1,20|128|0|0"),
        (WasmGameMode::Mania, "16|5|0|0,20|8|0|0"),
    ] {
        assert_eq!(
            parse_replay_data_wasm(&compress(data.as_bytes()), true, false, mode).unwrap(),
            2
        );
    }
}
