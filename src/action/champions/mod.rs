use std::time::{Duration, Instant};

use crate::{
    io::output::{self, *},
    sdk::game::Game,
    GameTasks,
};

pub mod karthus;
pub mod xerath;
pub struct Generic;
impl Champion for Generic {}
pub trait Champion {
    fn offense_tick(&self, game: &Game, game_tasks: &mut GameTasks) -> bool {
        return false;
    }
    fn move_to(&self, screen_point: nc::na::Point2<f32>) {
        let original_pos = output::get_cursor_pos();
        output::cursor_move(screen_point.x, screen_point.y);
        std::thread::sleep(Duration::from_millis(10));
    }
    fn auto_attack(&self, game: &Game, game_tasks: &mut GameTasks) -> bool {
        let local_player_position = nc::na::Point3::new(
            game.local_player.position.x,
            game.local_player.position.y,
            game.local_player.position.z,
        );

        let attack_speed = game.local_player.get_attack_speed(game.process_handle).unwrap();
        let attack_delay = 1.0 / attack_speed;
        let attack_windup_time = attack_delay * game.local_player.windup_percent + 0.05;
        for target in game
            .player_manager
            .get_sorted_and_filtered_players(&local_player_position, &game_tasks.target_mode)
        {
            let target_position = nc::na::Point3::new(target.position.x, target.position.y, target.position.z);
            if nc::na::distance(&local_player_position, &target_position)
                < game.local_player.get_attack_range(game.process_handle).unwrap() - 80.0
            {
                let screen_scaling = game.game_to_screen_scaling;

                if let Some(screen_pos) = game.world_to_screen(&target_position) {
                    game_tasks.auto_attack_cooldown = Instant::now() + Duration::from_secs_f32(attack_delay);
                    game_tasks.casting_spell_cooldown = Instant::now() + Duration::from_secs_f32(attack_windup_time);
                    std::thread::spawn(move || {
                        let original_pos = output::get_cursor_pos();
                        output::cursor_move(screen_pos.x * screen_scaling.x, screen_pos.y * screen_scaling.y);

                        output::key_send('a');
                        output::send_mouse_click(MouseButton::Left);
                        std::thread::sleep(std::time::Duration::from_secs_f32(attack_windup_time));
                        output::cursor_move(original_pos.x * screen_scaling.x, original_pos.y * screen_scaling.x);
                        output::send_mouse_click(MouseButton::Right);
                    });
                    return true;
                }
            }
        }
        return false;
    }

    fn spell_evade(&self, point: &nc::na::Point2<f32>) -> bool {
        return false;
    }
}
