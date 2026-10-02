mod support;
use rosu_replay::{unpacker::Unpacker, Replay};
use serde_json::json;
use support::*;

#[test]
fn all_judgements_and_future_metadata_survive_reencoding() {
    let mut info = score();
    info["statistics"] = json!({"perfect":500,"good":20,"great":10,"small_tick_hit":12,
        "small_tick_miss":2,"large_tick_miss":3,"small_bonus":4,"large_bonus":5,"future_judgement":6});
    info["maximum_statistics"] = info["statistics"].clone();
    info["future_metadata"] = json!({"revision":3});
    let input = fixture(0, 30000001, -1, "16|256|192|1", &json_tail(&info));
    let output = Replay::from_bytes(&input).unwrap().pack().unwrap();
    let actual = read_json(&output);
    assert_eq!(actual["statistics"], info["statistics"]);
    assert_eq!(actual["maximum_statistics"], info["maximum_statistics"]);
    assert_eq!(actual["future_metadata"], info["future_metadata"]);
}

#[test]
fn unknown_mod_settings_survive_reencoding() {
    let mut info = score();
    info["mods"] = json!([{"acronym":"DT","settings":{"speed_change":1.2,"future_option":true}},
        {"acronym":"ZZ","settings":{"nested":{"value":42}}}]);
    let input = fixture(0, 30000001, -1, "16|256|192|1", &json_tail(&info));
    assert_eq!(
        read_json(&Replay::from_bytes(&input).unwrap().pack().unwrap())["mods"],
        info["mods"]
    );
}

#[test]
fn signed_offline_user_and_64_bit_score_are_accepted() {
    let mut info = score();
    info["user_id"] = json!(-1);
    info["rank"] = json!(null);
    info["total_score_without_mods"] = json!(4294967296_i64);
    let input = fixture(0, 30000001, -1, "16|256|192|1", &json_tail(&info));
    let actual = read_json(&Replay::from_bytes(&input).unwrap().pack().unwrap());
    assert_eq!(actual["user_id"], -1);
    assert_eq!(actual["total_score_without_mods"], 4294967296_i64);
}

#[test]
fn old_online_id_is_sign_extended() {
    assert_eq!(
        Unpacker::new(&[255u8; 4][..])
            .unpack_replay_id(20121008)
            .unwrap(),
        -1
    );
}

#[test]
fn version_boundaries_determine_written_id_width() {
    for (version, id, expected) in [
        (20121007, 0, vec![]),
        (20121008, -1, vec![255; 4]),
        (20140720, 123, vec![123, 0, 0, 0]),
        (20140721, -1, vec![255; 8]),
        (30000000, -1, vec![255; 8]),
    ] {
        let input = fixture(0, version, id, "16|256|192|1", &[]);
        let output = Replay::from_bytes(&input).unwrap().pack().unwrap();
        assert_eq!(suffix(&output), expected, "version {version}");
    }
}

#[test]
fn empty_lazer_block_is_valid() {
    let input = fixture(0, 30000001, -1, "16|256|192|1", &[0; 4]);
    let output = Replay::from_bytes(&input).unwrap().pack().unwrap();
    assert_eq!(&suffix(&output)[8..], &[0; 4]);
}

#[test]
fn truncated_lazer_length_is_an_error() {
    for tail in [&[][..], &[1][..], &[1, 2][..], &[1, 2, 3][..]] {
        let input = fixture(0, 30000001, -1, "16|256|192|1", tail);
        assert!(Replay::from_bytes(&input).is_err(), "accepted {tail:?}");
    }
}

#[test]
fn negative_rng_seed_survives_the_signed_frame_sentinel() {
    let input = fixture(
        0,
        30000001,
        -1,
        "16|256|-500|1,20|256|-500|0,-12345|0|0|-2147483648",
        &json_tail(&score()),
    );
    let replay = Replay::from_bytes(&input).unwrap();
    assert_eq!(replay.common().rng_seed, Some(i32::MIN));
    assert_eq!(replay.common().replay_data.len(), 2);
    assert_eq!(replay.common().replay_data[1].time_delta(), 20);
    assert_eq!(Replay::from_bytes(&replay.pack().unwrap()).unwrap(), replay);
}

#[test]
fn unknown_modes_and_broken_compressed_blocks_fail() {
    let mut input = fixture(4, 30000001, -1, "16|256|192|1", &json_tail(&score()));
    assert!(Replay::from_bytes(&input).is_err());
    input[0] = 0;
    input.pop();
    assert!(Replay::from_bytes(&input).is_err());
    let invalid = fixture(0, 30000001, -1, "16|256|192|1", &[3, 0, 0, 0, 1, 2, 3]);
    assert!(Replay::from_bytes(&invalid).is_err());
}

#[test]
fn declared_and_decompressed_sizes_obey_read_limits() {
    use rosu_replay::ReadLimits;
    let input = fixture(0, 30000001, -1, "16|256|192|1", &json_tail(&score()));
    for limits in [
        ReadLimits {
            max_compressed_bytes: 8,
            ..ReadLimits::default()
        },
        ReadLimits {
            max_decompressed_frames: 5,
            ..ReadLimits::default()
        },
        ReadLimits {
            max_decompressed_score: 5,
            ..ReadLimits::default()
        },
    ] {
        assert!(Replay::from_reader_with_limits(&input[..], limits).is_err());
    }
    let huge = fixture(0, 30000001, -1, "16|256|192|1", &i32::MAX.to_le_bytes());
    assert!(Replay::from_bytes(&huge).is_err());
    let null = fixture(0, 30000001, -1, "16|256|192|1", &(-1_i32).to_le_bytes());
    let replay = Replay::from_bytes(&null).unwrap();
    assert!(matches!(replay, Replay::Lazer(ref v) if v.score_info().is_none()));
}

#[test]
fn mod_variants_follow_each_replays_ruleset() {
    for (mode, frames, expected) in [
        (0, "16|256|192|1", rosu_mods::GameMode::Osu),
        (1, "16|0|0|1", rosu_mods::GameMode::Taiko),
        (2, "16|256|0|1", rosu_mods::GameMode::Catch),
        (3, "16|5|0|0", rosu_mods::GameMode::Mania),
    ] {
        let mut info = score();
        if mode == 3 {
            info["mods"]
                .as_array_mut()
                .unwrap()
                .push(json!({"acronym":"FI"}));
        }
        let replay =
            Replay::from_bytes(&fixture(mode, 30000001, -1, frames, &json_tail(&info))).unwrap();
        let Replay::Lazer(ref lazer) = replay else {
            panic!("wrong format");
        };
        let mods = lazer.mods().unwrap().unwrap();
        assert_eq!(mods.len(), if mode == 3 { 2 } else { 1 });
        for gamemod in &mods {
            assert_eq!(gamemod.mode(), expected);
        }
        assert_eq!(mods.clock_rate(), Some(1.2));
        assert_eq!(Replay::from_bytes(&replay.pack().unwrap()).unwrap(), replay);
    }
}

#[test]
fn modified_mode_cannot_write_incompatible_frames() {
    let mut replay = Replay::from_bytes(&fixture(0, 30000000, -1, "16|256|192|1", &[])).unwrap();
    replay.common_mut().mode = rosu_replay::GameMode::Mania;
    assert!(replay.pack().is_err());
}

#[test]
fn malformed_version_combinations_cannot_be_deserialized() {
    use rosu_replay::{LazerVersion, StableVersion};
    assert!(StableVersion::new(30000000).is_err());
    assert!(LazerVersion::new(29999999).is_err());
    let replay = Replay::from_bytes(&fixture(0, 20121008, -1, "16|256|192|1", &[])).unwrap();
    let mut value = serde_json::to_value(&replay).unwrap();
    value["Stable"]["online_id"] = json!(2147483648_i64);
    assert!(serde_json::from_value::<Replay>(value).is_err());
    let old = Replay::from_bytes(&fixture(0, 30000000, -1, "16|256|192|1", &[])).unwrap();
    let mut value = serde_json::to_value(old).unwrap();
    value["Lazer"]["score_info"] = score();
    assert!(serde_json::from_value::<Replay>(value).is_err());
}

#[test]
fn original_zero_online_id_is_preserved() {
    for version in [20121008, 20140721] {
        let replay = Replay::from_bytes(&fixture(0, version, 0, "16|256|192|1", &[])).unwrap();
        assert_eq!(replay.legacy_online_id(), Some(0));
        assert_eq!(
            suffix(&replay.pack().unwrap()),
            vec![0; if version == 20121008 { 4 } else { 8 }]
        );
    }
}

#[test]
fn diagnostic_output_preserves_lazer_suffix_but_is_not_an_osr() {
    let input = fixture(0, 30000001, -1, "16|256|192|1", &json_tail(&score()));
    let replay = Replay::from_bytes(&input).unwrap();
    let diagnostic = replay.pack_uncompressed().unwrap();
    assert_eq!(read_json(&diagnostic)["statistics"], score()["statistics"]);
    assert!(Replay::from_bytes(&diagnostic).is_err());
}

#[test]
fn invalid_compression_preset_returns_an_error() {
    let replay = Replay::from_bytes(&fixture(0, 20140721, -1, "16|256|192|1", &[])).unwrap();
    assert!(replay
        .pack_with(&rosu_replay::Packer::new().with_preset(10))
        .is_err());
}

#[test]
fn overflowing_string_length_is_rejected_without_truncation() {
    let mut input = vec![0x0b];
    input.extend([0x80; 9]);
    input.push(2);
    assert!(Unpacker::new(&input[..]).unpack_string().is_err());
}

#[test]
fn pre_unix_fractional_timestamps_keep_their_ticks() {
    use chrono::TimeZone;
    let bytes = 621355967999999999_i64.to_le_bytes();
    let timestamp = Unpacker::new(&bytes[..]).unpack_timestamp().unwrap();
    assert_eq!(timestamp, chrono::Utc.timestamp_opt(-1, 999999900).unwrap());
}

#[test]
fn invalid_dotnet_ticks_are_errors_instead_of_panics_or_current_time() {
    for ticks in [i64::MIN, -1, 3155378976000000000, i64::MAX] {
        assert!(Unpacker::new(&ticks.to_le_bytes()[..])
            .unpack_timestamp()
            .is_err());
    }
}

#[test]
fn unlimited_output_limit_does_not_overflow() {
    let bytes = fixture(0, 20140721, -1, "16|256|192|1", &[]);
    let limits = rosu_replay::ReadLimits {
        max_decompressed_frames: usize::MAX,
        ..Default::default()
    };
    assert!(Replay::from_reader_with_limits(&bytes[..], limits).is_ok());
}

#[test]
fn writing_out_of_range_timestamps_is_an_error() {
    let mut replay = Replay::from_bytes(&fixture(0, 20140721, -1, "16|256|192|1", &[])).unwrap();
    replay.common_mut().timestamp = chrono::DateTime::<chrono::Utc>::MAX_UTC;
    assert!(replay.pack().is_err());
}

#[test]
fn compressed_score_block_cannot_hide_trailing_bytes() {
    let mut tail = json_tail(&score());
    let len = i32::from_le_bytes(tail[..4].try_into().unwrap());
    tail[..4].copy_from_slice(&(len + 3).to_le_bytes());
    tail.extend([1, 2, 3]);
    let bytes = fixture(0, 30000001, -1, "16|256|192|1", &tail);
    assert!(Replay::from_bytes(&bytes).is_err());
}
