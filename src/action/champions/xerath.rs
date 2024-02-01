use std::{
    sync::{atomic::AtomicBool, Arc},
    thread,
    time::{Duration, Instant},
};

use crate::{
    action::{calculate_ap_damage, just_move, predict_player_position},
    io::output::{self},
    point2,
    sdk::game::Game,
    sdk::{
        self,
        game::match_functions::{self, world_to_minimap},
        player_manager::{
            player_functions::{self, is_casting},
            Player,
        },
    },
    utils, GameTasks, PROC_HANDLE,
};

use super::Champion;
const Q_MAX_CHANNEL_TIME: f32 = 1.75;
const Q_MAX_RANGE: f32 = 1450.0;
const Q_DELAY: f32 = 0.5;
const Q_MANA_COST: f32 = 100.0;

const W_RANGE: f32 = 1000.0;
const W_CAST_TIME: f32 = 0.25;
const W_DELAY: f32 = 0.5;
const W_MANA_COST: f32 = 100.0;

const R_MANA_COST: f32 = 100.0;
const R_RANGE: f32 = 5000.0;
const R_CAST: f32 = 0.2;
const R_BASE_DMG: [f32; 3] = [180.0, 230.0, 280.0];
const R_AP_RATIO: f32 = 0.40;
pub struct Xerath {
    is_casting_r: Arc<AtomicBool>,
}

impl Xerath {
    pub fn new() -> Self {
        Self {
            is_casting_r: Arc::new(AtomicBool::new(false)),
        }
    }
    fn q_spell(&self, game: &Game, targets: &Vec<&Player>) -> Option<f32> {
        if game.local_player.get_mana(game.process_handle).unwrap() < Q_MANA_COST
            || game.local_player.ability_manager.q.level < 1
            || game.local_player.ability_manager.q.cooldown > game.get_time().unwrap()
            || game.local_player.health < 1.0
        {
            return None;
        }
        let local_player_address = game.local_player.base_address;
        for player in targets.into_iter() {
            let distance = nc::na::distance(
                &point2!(game.local_player.position.x, game.local_player.position.z),
                &point2!(player.position.x, player.position.z),
            );
            let channel_time = if distance < 735.0 {
                0.6
            } else {
                0.6 + (distance - 735.0) / 428.0
            };

            let target_ai_address = player.ai_address;
            let target_address = player.base_address;
            let screen_scaling = game.game_to_screen_scaling;
            let mod_address = game.module_address;
            let original_pos = output::get_cursor_pos();

            thread::spawn(move || {
                let proc_handle = unsafe { PROC_HANDLE.unwrap() };
                output::key_down('q');

                let start_channel = Instant::now();

                while start_channel.elapsed() < Duration::from_secs_f32(3.2) {
                    std::thread::sleep(Duration::from_millis(200));
                    let q_range = 735.0 + (start_channel.elapsed().as_secs_f32() * 428.0);
                    println!("getting position");
                    let distance = nc::na::distance(
                        &player_functions::get_position(target_address, proc_handle),
                        &player_functions::get_position(local_player_address, proc_handle),
                    );
                    println!("Done getting pos");
                    if distance <= q_range - 100.0 {
                        println!("predictiing pos");
                        let predicted_pos = predict_player_position(target_ai_address, Q_DELAY, proc_handle);

                        print!("moving to pos");
                        if let Some(screen_pos) = match_functions::world_to_screen(
                            mod_address,
                            proc_handle,
                            &nc::na::Point3::new(predicted_pos.x, predicted_pos.y, predicted_pos.z),
                        ) {
                            print!("moving to pos");
                            output::cursor_move(screen_pos.x * screen_scaling.x, screen_pos.y * screen_scaling.y);
                            output::key_up('q');
                            std::thread::sleep(Duration::from_secs_f32(0.01));
                            output::cursor_move(original_pos.x * screen_scaling.x, original_pos.y * screen_scaling.y);
                            break;
                        }
                    }
                    just_move();
                }
            });
            return Some(7.0);
        }
        return None;
    }

    fn w_spell(&self, game: &Game, targets: &Vec<&Player>) -> Option<f32> {


        if game.local_player.get_mana(game.process_handle).unwrap() < W_MANA_COST
            || game.local_player.ability_manager.w.level < 1
            || game.local_player.ability_manager.w.cooldown > game.get_time().unwrap()
            || game.local_player.health < 1.0
        {
            return None;
        }

        for player in targets.into_iter() {
            let distance = nc::na::distance(
                &point2!(game.local_player.position.x, game.local_player.position.z),
                &point2!(player.position.x, player.position.z),
            );
            if distance > W_RANGE {
                continue;
            }

            let target_ai_address = player.ai_address;
            let screen_scaling = game.game_to_screen_scaling;
            let local_player_address = game.local_player.base_address;
            let original_pos = output::get_cursor_pos();
            let predicted_pos = predict_player_position(target_ai_address, W_CAST_TIME, game.process_handle);

            let mod_address = game.module_address;
            if let Some(screen_pos) = match_functions::world_to_screen(
                mod_address,
                unsafe { PROC_HANDLE.unwrap() },
                &nc::na::Point3::new(predicted_pos.x, predicted_pos.y, predicted_pos.z),
            ) {
                thread::spawn(move || {
                    output::cursor_move(screen_pos.x * screen_scaling.x, screen_pos.y * screen_scaling.y);
                    output::key_send('w');
                    std::thread::sleep(Duration::from_secs_f32(W_CAST_TIME));
                    if is_casting(local_player_address, unsafe { PROC_HANDLE.unwrap() }) {
                        output::cursor_move(original_pos.x * screen_scaling.x, original_pos.y * screen_scaling.y);
                    }
                });
                return Some(W_CAST_TIME);
            }
        }
        return None;
    }
    fn r_spell(&self, game: &Game, targets: &Vec<&Player>) -> bool {
        if game.local_player.get_mana(game.process_handle).unwrap() < R_MANA_COST
            || game.local_player.ability_manager.r.level < 1
            || game.local_player.ability_manager.r.cooldown > game.get_time().unwrap()
        {
            return false;
        }
        let (number_of_r_cast, base_dmg) = match game.local_player.ability_manager.r.level {
            1 => (4, R_BASE_DMG[0]),
            2 => (5, R_BASE_DMG[1]),
            3 => (6, R_BASE_DMG[2]),
            _ => return false,
        };

        for player in targets.into_iter() {
            let distance = nc::na::distance(
                &point2!(game.local_player.position.x, game.local_player.position.z),
                &point2!(player.position.x, player.position.z),
            );
            if distance > R_RANGE - 500.0 {
                continue;
            }
            let dmg = calculate_ap_damage(base_dmg, R_AP_RATIO, game, &player) * number_of_r_cast as f32;
            if 2 > 1 {
                //player.health - 100.0 {
                let minimap_pos = point2!(game.minimap_position.x, game.minimap_position.y);
                let minimap_size = game.minimap_size;
                let target_ai_address = player.ai_address;
                let target_base_address = player.base_address;
                let local_player_address = game.local_player.ai_address;
                let screen_scaling = game.game_to_screen_scaling;
                let lp = game.local_player.position;
                // if game.local_player.get_active_spell(game.process_handle).is_err() {
                //     return None;
                // };
                // let writer_flag = Arc::clone(&self.is_casting_r);
                // thread::spawn(move || {
                println!("XERATH_R: Starting");
                output::key_send('r');
                // writer_flag.store(true, std::sync::atomic::Ordering::Relaxed);
                std::thread::sleep(Duration::from_millis(500));
                let original_pos = output::get_cursor_pos();
                let proc_handle = unsafe { PROC_HANDLE.unwrap() };

                for i in 0..number_of_r_cast {
                    if !is_casting(local_player_address, proc_handle)
                        || player_functions::get_health(target_base_address, proc_handle) < 1.0
                    {
                        println!("dead {}", i);
                        println!("XERATH_R: Exiting target dead");
                        for _ in i..number_of_r_cast {
                            output::key_send('r');
                            std::thread::sleep(Duration::from_millis(200));
                        }
                        // writer_flag.store(false, std::sync::atomic::Ordering::Relaxed);
                        output::cursor_move(original_pos.x * screen_scaling.x, original_pos.y * screen_scaling.x);
                        break;
                    }
                    let predicted_pos = predict_player_position(target_ai_address, R_CAST, proc_handle);
                    let minimap_screen_pos = match_functions::world_to_minimap(
                        &nc::na::Point3::new(predicted_pos.x, predicted_pos.y, predicted_pos.z),
                        minimap_pos,
                        minimap_size,
                        screen_scaling,
                    );
                    output::cursor_move(minimap_screen_pos.x + 324.0, minimap_screen_pos.y + 164.0);

                    // let minimap_screen_pos =
                    //     utils::world_to_minimap(&nc::na::Point3::new(lp.x, lp.y, lp.z), minimap_pos, minimap_size);

                    // output::cursor_move(minimap_screen_pos.x + 320.0, minimap_screen_pos.y + 170.0);
                    std::thread::sleep(Duration::from_millis(200));
                    output::key_send('r');
                    std::thread::sleep(Duration::from_millis(500));
                    //std::thread::sleep(Duration::from_secs_f32(R_CAST));
                    // if utils::is_player_moving(local_player_ai_address)
                    //     || utils::is_player_dead(local_player_ai_address)
                    //     || utils::is_player_dead(target_ai_address)
                    // {
                    //     // Player is moving stop r cast or player is dead or target is dead
                    //     //output::key_send('r');
                    //     break;
                    // }
                }
                //writer_flag.store(false, std::sync::atomic::Ordering::Relaxed);
                output::cursor_move(original_pos.x * screen_scaling.x, original_pos.y * screen_scaling.x);
                // });
                return true;
            }
        }
        return false;
    }
}
impl Champion for Xerath {
    fn offense_tick(&self, game: &Game, game_tasks: &mut GameTasks) -> bool {
        //println!("starting offensive tick",);
        //std::thread::sleep(Duration::from_millis(50));
        // if self.is_casting_r.load(std::sync::atomic::Ordering::Relaxed) {
        //     //println!("Is casting xerath r",);
        //     return false;
        // }
        if game_tasks.casting_spell_cooldown > Instant::now() {
            if game.local_player.mana < 30.0 {
                game_tasks.casting_spell_cooldown += Duration::from_millis(100);
            }
            if game_tasks.auto_attack_cooldown < Instant::now() {
                self.auto_attack(game, game_tasks);
            }
            if game_tasks.move_cooldown < Instant::now() {
                just_move();
                game_tasks.move_cooldown = Instant::now() + Duration::from_secs_f32(0.05);
            }

            return false;
        }

        let targets = game.player_manager.get_sorted_and_filtered_players(
            &nc::na::Point3::new(
                game.local_player.position.x,
                game.local_player.position.y,
                game.local_player.position.z,
            ),
            &game_tasks.target_mode,
        );

        if let Some(spell_duration) = self.w_spell(game, &targets) {
            println!("Sending w");
            game_tasks.casting_spell_cooldown = Instant::now() + Duration::from_secs_f32(spell_duration + 0.5);
            return true;
        }
        if let Some(spell_duration) = self.q_spell(game, &targets) {
            println!("Sending q");
            game_tasks.casting_spell_cooldown = Instant::now() + Duration::from_secs_f32(spell_duration + 0.5);
            return true;
        }
        if self.r_spell(game, &targets) {
            println!("Sending r");
            game_tasks.casting_spell_cooldown = Instant::now() + Duration::from_secs_f32(1.5);
            return true;
        }
        if game_tasks.move_cooldown < Instant::now() {
            just_move();
            game_tasks.move_cooldown = Instant::now() + Duration::from_secs_f32(0.05);
        }
        return false;
    }
}

mod tests {
    use std::time::Duration;

    use nc::na::Vector2;
    use windows_sys::Win32::UI::WindowsAndMessaging::{GetSystemMetrics, SM_CXSCREEN, SM_CYSCREEN};

    use crate::match_functions::world_to_minimap;
    use crate::sdk::{game::Game, walls::MAP_CHUNKS};
    #[test]
    fn test_minimap_pos() {
        std::thread::sleep(Duration::from_secs_f32(2.0));
        let screen_width = unsafe { GetSystemMetrics(SM_CXSCREEN) };
        let screen_height = unsafe { GetSystemMetrics(SM_CYSCREEN) };

        let mut game = Game::new(nc::na::Vector2::new(screen_width as f32, screen_height as f32)).unwrap();

        let row_position = game.local_player.position.z / 15000.0 * 5.0;
        let col_position = game.local_player.position.x / 15000.0 * 5.0;

        let row_index = (5.0 - row_position).floor();
        let col_index = col_position.floor();
        println!("row: {} col:{}", row_index, col_index);

        let row_fraction = row_position % 1.0;
        let col_fraction = col_position % 1.0;
        println!("{}, {}", row_fraction, col_fraction);


        let tile_row = (20.0 - (row_fraction * 20.0)).floor() as usize;
        let tile_col = (col_fraction * 20.0).floor() as usize;

        println!("tile row: {} tile col:{}", tile_row, tile_col);
        let st = std::time::Instant::now();
        println!("{}", game.minimap_size);
        if row_index < 5.0 && col_index < 5.0 {
            let map_chunk = &MAP_CHUNKS[row_index as usize][col_index as usize];
            if tile_row < 20 && tile_col < 20 {
                let tile_value = map_chunk.rows[tile_row][tile_col];
                if tile_value == 0 {
                    println!("The tile value is 0.");
                } else {
                    println!("The tile value is not 0, it is: {}", tile_value);
                }
            } else {
                println!("Tile position is out of bounds.");
            }
        } else {
            println!("Chunk position is out of bounds.");
        }
        println!("{:?}", st.elapsed());
    }
}
