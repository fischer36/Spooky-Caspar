pub mod champions;
pub mod evade;
use std::{
    os::raw::c_void,
    sync::{atomic::AtomicBool, Arc},
    time::Instant,
};

use external::read;
use log::info;

pub struct Tasks {
    is_evading: Arc<AtomicBool>,
    is_casting_spell: Arc<AtomicBool>,
    is_auto_attacking: Arc<AtomicBool>,
}

use crate::{
    action::champions::{karthus::Karthus, xerath::Xerath, Generic},
    io::output::{self, *},
    point2,
    sdk::game::Game,
    sdk::offsets,
    sdk::player_manager::{
        player_functions::{self},
        Player,
    },
    utils::{
        extract_bool_from_buffer, extract_f32_from_buffer, extract_u32_from_buffer, extract_vector3_from_buffer,
        TargetMode,
    },
    GameTasks,
};

use self::champions::Champion;

#[inline(always)]
pub fn just_move() {
    output::send_mouse_click(MouseButton::Right);
}

pub fn calculate_ap_damage(spell_base_damage: f32, ap_ratio: f32, game: &Game, target: &Player) -> f32 {
    let ap = game.local_player.get_ability_power(game.process_handle);
    let (magic_pen_flat, magic_pen_percent) = game.local_player.get_magic_pen(game.process_handle);
    let raw_damage = spell_base_damage + (ap_ratio * ap);

    let effective_mr = (target.get_magic_resist(game.process_handle) * (1.0 - magic_pen_percent)) - magic_pen_flat;
    let damage_multiplier = if effective_mr >= 0.0 {
        100.0 / (100.0 + effective_mr)
    } else {
        2.0 - 100.0 / (100.0 - effective_mr)
    };

    raw_damage * damage_multiplier
}
pub fn calculate_ad_damage(attack_base_damage: f32, ad_ratio: f32, game: &Game, target: &Player) -> f32 {
    let ad = player_functions::get_ad(game.local_player.base_address, game.process_handle);
    let raw_lethality = player_functions::get_lethality(game.local_player.base_address, game.process_handle);
    let player_level = player_functions::get_level(game.local_player.base_address, game.process_handle);

    let armor_pen_percent =
        player_functions::get_armor_pen_percent(game.local_player.base_address, game.process_handle);

    let flat_armor_pen = raw_lethality * (player_level as f32 / 18.0);
    let raw_damage = attack_base_damage + (ad_ratio * ad);

    let target_armor = player_functions::get_armor(target.base_address, game.process_handle);

    let armor_after_percent_pen = target_armor * (1.0 - armor_pen_percent);

    let effective_armor = armor_after_percent_pen - flat_armor_pen;

    let damage_multiplier = if effective_armor >= 0.0 {
        100.0 / (100.0 + effective_armor)
    } else {
        2.0 - 100.0 / (100.0 - effective_armor)
    };

    raw_damage * damage_multiplier
}

pub fn get_champion_mod(champion_name: &str) -> Box<dyn Champion> {
    match champion_name {
        "Xerath" => {
            info!("Xerath module loaded");
            Box::new(Xerath::new())
        }
        "Karthus" => {
            info!("Karthus module loaded");
            Box::new(Karthus {
                q_cooldown: Instant::now(),
            })
        }
        _ => {
            println!("Unsupported champion name: {}", champion_name);
            Box::new(Generic)
        }
    }
}

pub fn predict_player_position(
    player_ai_address: usize,
    for_when_seconds: f32,
    proccess_handle: *mut c_void,
) -> nc::na::Point3<f32> {
    let buffer = read::<[u8; 0x420]>(proccess_handle, player_ai_address)
        .map_err(|e| format!("Error reading AI manager buffer: {:?}", e))
        .unwrap();

    // Extract values from buffer
    let is_moving = extract_bool_from_buffer(&buffer, offsets::OBJ_AI_MANAGER_IS_MOVING).unwrap();
    let current_position = extract_vector3_from_buffer(&buffer, offsets::OBJ_AI_MANAGER_CURRENT_POSITION).unwrap();
    if !is_moving {
        println!("PREDICTION: Not moving");
        return nc::na::Point3::new(current_position.x, current_position.y, current_position.z);
    }
    let start_path = extract_vector3_from_buffer(&buffer, offsets::OBJ_AI_MANAGER_START_PATH).unwrap();
    let end_path = extract_vector3_from_buffer(&buffer, offsets::OBJ_AI_MANAGER_END_PATH).unwrap();
    let target_position = extract_vector3_from_buffer(&buffer, offsets::OBJ_AI_MANAGER_TARGET_POSITION).unwrap();
    let direction = (end_path - current_position).normalize();
    println!("direction{:?}", direction);
    let is_dashing = extract_bool_from_buffer(&buffer, offsets::OBJ_AI_MANAGER_IS_DASHING).unwrap();
    if is_dashing {
        let dash_speed = extract_f32_from_buffer(&buffer, offsets::OBJ_AI_MANAGER_DASH_SPEED).unwrap();
        let dash_pos = current_position + direction * (dash_speed * for_when_seconds);
        println!("PREDICTION: Dashing");
        return nc::na::Point3::new(dash_pos.x, 0.0, dash_pos.z);
    }
    let movement_speed = extract_f32_from_buffer(&buffer, offsets::OBJ_AI_MANAGER_MOVEMENT_SPEED).unwrap();
    let path_segments_count = extract_u32_from_buffer(&buffer, offsets::OBJ_AI_MANAGER_PATH_SEGMENTS_COUNT).unwrap();
    if path_segments_count <= 2 {
        let predicted_pos = current_position + direction * (movement_speed * for_when_seconds);
        println!("Path less than two current position + dirrection");
        return nc::na::Point3::new(predicted_pos.x, 0.0, predicted_pos.z);
    }
    let paths = match get_paths_vector(path_segments_count, player_ai_address, proccess_handle) {
        Ok(paths) => paths,
        Err(e) => {
            println!("Paths error:{:?}", e);
            let predicted_pos = current_position + direction * (movement_speed * for_when_seconds);

            return nc::na::Point3::new(predicted_pos.x, 0.0, predicted_pos.z);
        }
    };
    let predicted_path_pos = predict_position_on_path(paths, movement_speed, for_when_seconds);
    println!("predicting on path");
    return predicted_path_pos;
}

fn get_paths_vector(
    path_count: u32,
    ai_address: usize,
    process_handle: *mut c_void,
) -> Result<Vec<nc::na::Point3<f32>>, String> {
    let paths_pointer = match read::<usize>(process_handle, ai_address + offsets::OBJ_AI_MANAGER_PATH_SEGMENTS) {
        Ok(pc) => {
            if pc == 0 {
                return Err("NULL PTR".to_string());
            }
            pc
        }
        Err(e) => return Err(format!("Path ptr error: {}", e).to_string()),
    };

    println!("path ptr {}", paths_pointer);

    let mut paths = Vec::new();
    for i in 0..path_count - 1 {
        if i > 4 {
            break;
        }
        let path_offset = i * 0xC;
        let path = read::<nc::na::Point3<f32>>(process_handle, paths_pointer + path_offset as usize).unwrap();
        paths.push(path);
    }
    return Ok(paths);
}
fn predict_position_on_path(
    path: Vec<nc::na::Point3<f32>>,
    target_movement_speed: f32,
    in_time_seconds: f32,
) -> nc::na::Point3<f32> {
    println!("predict_position_on_path called",);
    let travel_distance = target_movement_speed * in_time_seconds;

    let mut cumulative_distance = 0.0;
    for i in 0..path.len() - 1 {
        let point = path[i];
        let next_point = path[i + 1];
        let segment_distance = nc::na::distance(&point, &next_point);

        if cumulative_distance + segment_distance > travel_distance {
            let remaining_distance = travel_distance - cumulative_distance;
            let fraction = remaining_distance / segment_distance;

            return point + (next_point - point) * fraction;
        }

        cumulative_distance += segment_distance;
    }
    println!("predict_position_on_path finished",);
    *path.last().unwrap_or(&nc::na::Point3::new(0.0, 0.0, 0.0))
}

pub fn sort_player_list(game: &Game, target_mode: &TargetMode, player_list: &mut Vec<&Player>) {
    match target_mode {
        TargetMode::LowestHp => {
            player_list.sort_by_key(|&player| player.health as i32);
        }
        TargetMode::LowestDistance => {
            player_list.sort_by(|&a, &b| {
                let dist_a = nc::na::distance(
                    &point2!(a.position.x, game.local_player.position.x),
                    &point2!(a.position.z, game.local_player.position.z),
                );

                let dist_b = nc::na::distance(
                    &point2!(b.position.x, game.local_player.position.x),
                    &point2!(b.position.z, game.local_player.position.z),
                );
                dist_a.partial_cmp(&dist_b).unwrap_or(std::cmp::Ordering::Equal)
            });
        }
    }
}

pub fn target_selector<'a>(
    local_player: &Player,
    enemy_list: &Vec<&'a Player>,
    target_mode: &TargetMode,
) -> Option<&'a Player> {
    match target_mode {
        TargetMode::LowestHp => enemy_list.iter().min_by_key(|e| e.health as u32).copied(),
        TargetMode::LowestDistance => enemy_list
            .iter()
            .min_by(|a, b| {
                let dist_a = nc::na::distance(
                    &point2!(a.position.x, local_player.position.x),
                    &point2!(a.position.z, local_player.position.z),
                );

                let dist_b = nc::na::distance(
                    &point2!(b.position.x, local_player.position.x),
                    &point2!(b.position.z, local_player.position.z),
                );

                dist_a.partial_cmp(&dist_b).unwrap_or(std::cmp::Ordering::Equal)
            })
            .copied(),
    }
}
