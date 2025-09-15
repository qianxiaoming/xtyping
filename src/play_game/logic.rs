use bevy::prelude::*;
use crate::play_game::marker::*;

pub fn update_game_time(
    time: Res<Time>,
    mut query: Query<(&mut Text, &mut GameTime)>,
) {
    if let Ok((mut text, mut clock)) = query.single_mut() {
        let elapsed = time.elapsed_secs_f64() - clock.start_time;
        let total_seconds = elapsed as u64;

        if total_seconds != clock.last_second {
            clock.last_second = total_seconds;

            let h = total_seconds / 3600;
            let m = (total_seconds % 3600) / 60;
            let s = total_seconds % 60;

            *text = Text::new(format!("{:02}:{:02}:{:02}", h, m, s));
        }
    }
}