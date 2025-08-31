use rosu_replay::{Replay, ReplayEvent};
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let osr_path = Path::new("assets/test.osr");

    // Check if the file exists
    if !osr_path.exists() {
        eprintln!("Error: File 'assets/test.osr' not found!");
        eprintln!("Please place a valid .osr file at 'assets/test.osr' to run this example.");
        return Ok(());
    }

    println!("Reading replay from: {}", osr_path.display());

    // Parse the replay file
    match Replay::from_path(osr_path) {
        Ok(replay) => {
            println!("\n=== Replay Information ===");
            println!("Username: {}", replay.username);
            println!("Game Mode: {:?}", replay.mode);
            println!("Game Version: {}", replay.game_version);
            println!("Beatmap Hash: {}", replay.beatmap_hash);
            println!("Score: {}", replay.score);
            println!("Max Combo: {}", replay.max_combo);
            println!("Perfect: {}", replay.perfect);
            println!("Mods: {:?} (value: {})", replay.mods, replay.mods.value());
            println!("Timestamp: {}", replay.timestamp);
            println!("Replay ID: {}", replay.replay_id);

            // Hit counts
            println!("\n=== Hit Counts ===");
            println!("300s: {}", replay.count_300);
            println!("100s: {}", replay.count_100);
            println!("50s: {}", replay.count_50);
            println!("Gekis: {}", replay.count_geki);
            println!("Katus: {}", replay.count_katu);
            println!("Misses: {}", replay.count_miss);

            // RNG seed
            if let Some(seed) = replay.rng_seed {
                println!("\n=== RNG Seed ===");
                println!("Seed: {}", seed);
            }

            // Life bar information
            if let Some(ref life_bar) = replay.life_bar_graph {
                println!("\n=== Life Bar ===");
                println!("Life bar states: {}", life_bar.len());
                if !life_bar.is_empty() {
                    println!(
                        "First state: time={}ms, life={}",
                        life_bar[0].time, life_bar[0].life
                    );
                    println!(
                        "Last state: time={}ms, life={}",
                        life_bar[life_bar.len() - 1].time,
                        life_bar[life_bar.len() - 1].life
                    );
                }
            } else {
                println!("\n=== Life Bar ===");
                println!("No life bar data available");
            }

            // Replay data information
            println!("\n=== Replay Data ===");
            println!("Total events: {}", replay.replay_data.len());

            if !replay.replay_data.is_empty() {
                println!("\nFirst 5 events:");
                for (i, event) in replay.replay_data.iter().take(5).enumerate() {
                    match event {
                        ReplayEvent::Osu(e) => {
                            println!(
                                "  {}: Osu - time_delta={}ms, x={}, y={}, keys={}",
                                i + 1,
                                e.time_delta,
                                e.x,
                                e.y,
                                e.keys.value()
                            );
                        }
                        ReplayEvent::Taiko(e) => {
                            println!(
                                "  {}: Taiko - time_delta={}ms, x={}, keys={}",
                                i + 1,
                                e.time_delta,
                                e.x,
                                e.keys.value()
                            );
                        }
                        ReplayEvent::Catch(e) => {
                            println!(
                                "  {}: Catch - time_delta={}ms, x={}, dashing={}",
                                i + 1,
                                e.time_delta,
                                e.x,
                                e.dashing
                            );
                        }
                        ReplayEvent::Mania(e) => {
                            println!(
                                "  {}: Mania - time_delta={}ms, keys={}",
                                i + 1,
                                e.time_delta,
                                e.keys.value()
                            );
                        }
                    }
                }

                if replay.replay_data.len() > 5 {
                    println!("  ... and {} more events", replay.replay_data.len() - 5);
                }
            }

            // Calculate total replay duration
            let total_time: i32 = replay
                .replay_data
                .iter()
                .map(|event| event.time_delta())
                .sum();

            if total_time > 0 {
                let minutes = total_time / 60000;
                let seconds = (total_time % 60000) / 1000;
                let milliseconds = total_time % 1000;
                println!(
                    "\nTotal replay duration: {}:{:02}.{:03}",
                    minutes, seconds, milliseconds
                );
            }

            // Try to write the replay back to verify our packer works
            println!("\n=== Testing Write Functionality ===");
            let output_path = "assets/test_output.osr";
            match replay.write_path(output_path) {
                Ok(()) => {
                    println!("Successfully wrote replay to: {}", output_path);

                    // Verify by reading it back
                    match Replay::from_path(output_path) {
                        Ok(replay_copy) => {
                            println!("Successfully verified written replay!");
                            println!("Original username: {}", replay.username);
                            println!("Copy username: {}", replay_copy.username);
                            println!("Scores match: {}", replay.score == replay_copy.score);
                        }
                        Err(e) => {
                            eprintln!("Error reading back written replay: {}", e);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Error writing replay: {}", e);
                }
            }
        }
        Err(e) => {
            eprintln!("Error reading replay: {}", e);
            eprintln!("Make sure the file is a valid .osr replay file.");
        }
    }

    Ok(())
}
