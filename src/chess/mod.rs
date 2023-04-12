pub mod hub;
pub mod mainline;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Default)]
enum ColorPreference {
    #[default]
    Any,
    White,
    Black,
}

#[derive(Serialize, Deserialize, Debug, Default)]
enum OpponentPreference {
    #[default]
    Human,
    Lc0,
    Sockfish,
}

#[derive(Serialize, Deserialize, Debug, Default)]
struct TimeControl {
    main: u32, // main game time in seconds
    incr: u32,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct GamePreference {
    color: ColorPreference,
    tc: TimeControl,
    opponent: OpponentPreference,
}
