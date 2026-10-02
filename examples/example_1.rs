use rosu_replay::Replay;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "assets/test_lazer.osr".into());
    let replay = Replay::from_path(path)?;
    println!("Player: {}", replay.common().username);
    println!(
        "Mode: {:?}, frames: {}",
        replay.common().mode,
        replay.common().replay_data.len()
    );
    match &replay {
        Replay::Stable(stable) => {
            println!(
                "Stable version: {}, mods: {:?}, online ID: {:?}",
                stable.version().get(),
                stable.mods(),
                stable.online_id()
            );
        }
        Replay::Lazer(lazer) => {
            println!("Lazer format version: {}", lazer.version().get());
            if let Some(info) = lazer.score_info() {
                println!(
                    "Client: {}, score ID: {}",
                    info.client_version, info.online_id
                );
                println!("Statistics: {:?}", info.statistics);
                println!("Mods for this ruleset: {:?}", lazer.mods()?);
            }
        }
    }
    let encoded = replay.pack()?;
    assert_eq!(Replay::from_bytes(&encoded)?, replay);
    Ok(())
}
