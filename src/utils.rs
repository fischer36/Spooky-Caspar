use crate::{
    sdk::game::{self, Game},
    sdk::minion_manager::MinionManager,
    sdk::offsets,
    sdk::player_manager::{get_predict_info_from_ai, Player},
    PROC_HANDLE,
};
pub const MINION_SIZE_RADIUS: f32 = 48.0;
use external::read;
use log::info;
use nc::na::{Matrix4, Vector4};
use nc::na::{Vector2, Vector3};
use std::os::raw::c_void;
#[derive(Debug, Clone, PartialEq)]
pub enum TargetMode {
    LowestHp,
    LowestDistance,
}

// pub fn predict_position(
//     ai_address: usize,
//     process_handle: *mut c_void,
//     time_seconds: f32,
// ) -> Result<nc::na::Point3<f32>, String> {
//     //Vector3<f32> {
//     println!("ok2");
//     println!("Predicting position");
//     let (current_pos, target_pos, start_path, end_path, path_count, movemvent_speed, is_moving, is_dashing, dash_speed) =
//         get_predict_info_from_ai(ai_address);

//     let paths = match get_paths_vector(ai_address, process_handle) {
//         Ok(paths) => paths,
//         Err(e) => return Err(e.to_string()),
//     };

//     println!("ok");
//     if paths.is_empty() {
//         return Err("Empty paths".to_string());
//     }
//     let predicted_pos = predict_position_on_path(paths, movemvent_speed, time_seconds);
//     println!("predict_position: GOT PREDICTED POSITION");
//     return Ok(predicted_pos);

//return current_pos;
// if !is_moving {
//     info!("Target is not moving");
//     return current_pos;
// }
// let mut direction = current_pos;
// //println!("start path{}", start_path);
// //println!("end path{}", end_path);
// //println!("target pos{}", target_pos);
// if path_count > 2 {
//     println!("Path count more than 2");
//     direction = (end_path - current_pos).normalize();
// } else {
//     println!("Path count less than 2");
//     return current_pos;
//     direction = (target_pos - current_pos).normalize();
// }
// //println!("Direction: {:?}", direction);
// //direction.y = 0.0;
// if is_dashing {
//     info!("Target is dashing");
//     return current_pos + direction * (dash_speed * time_seconds);
// }
// //println!("Current position{}", current_pos);
// //info!("Direction{}", direction);
// //println!("Start path{}", start_path);
// info!("Movement speed{}", movemvent_speed);
// return current_pos + direction * (movemvent_speed * time_seconds);
// }

pub fn is_player_dead(player_address: usize) -> bool {
    return read::<f32>(unsafe { PROC_HANDLE.unwrap() }, player_address + offsets::OBJ_HEALTH).unwrap() <= 1.0;
}

pub fn is_player_moving(ai_address: usize) -> bool {
    return read::<bool>(
        unsafe { PROC_HANDLE.unwrap() },
        ai_address + offsets::OBJ_AI_MANAGER_IS_MOVING,
    )
    .unwrap();
}
pub fn is_vector_interrupted_by_minion(
    line_start_pos: &Vector3<f32>,
    line_end_pos: &Vector3<f32>,
    line_width: &f32,
    minion_manager: &MinionManager,
) -> bool {
    for minion in minion_manager.list.iter() {
        if !minion.is_visible {
            continue;
        }
        if get_distance(line_start_pos, &minion.position) > 1000.0 {
            continue;
        }

        // Calculate the vector from the local player to the player.
        let line_vector = line_end_pos - line_start_pos;

        // Calculate the vector from the local player to the minion.
        let to_minion_vector = minion.position - line_start_pos;

        // Calculate the projection of the to_minion_vector onto the line_vector.
        // This finds the closest point on the line to the center of the minion.
        let projection_length = to_minion_vector.dot(&line_vector) / line_vector.magnitude_squared();
        let closest_point = line_start_pos + line_vector * projection_length.clamp(0.0, 1.0);

        // Calculate the vector from the closest point on the line to the minion's center.
        let closest_to_minion_vector = minion.position - closest_point;

        // Check if the distance from the closest point on the line to the minion's center
        // is less than the sum of the radii (minion's radius + half the line's width).

        if closest_to_minion_vector.magnitude() < MINION_SIZE_RADIUS + line_width / 2.0 {
            // The line is interrupted by the minion.
            return true;
        }
    }
    return false;
}
pub fn get_distance(pos1: &Vector3<f32>, pos2: &Vector3<f32>) -> f32 {
    return (pos1 - pos2).magnitude();
}

pub fn extract_vector3_from_buffer(buffer: &[u8], offset: usize) -> Result<Vector3<f32>, &'static str> {
    let size = 12; // Size needed for Vector3<f32> (3 * 4 bytes)

    if offset + size <= buffer.len() {
        let x = buffer[offset..offset + 4]
            .try_into()
            .map(f32::from_ne_bytes)
            .map_err(|_| "Conversion error")?;
        let y = buffer[offset + 4..offset + 8]
            .try_into()
            .map(f32::from_ne_bytes)
            .map_err(|_| "Conversion error")?;
        let z = buffer[offset + 8..offset + 12]
            .try_into()
            .map(f32::from_ne_bytes)
            .map_err(|_| "Conversion error")?;
        Ok(Vector3::new(x, y, z))
    } else {
        Err("Buffer overflow")
    }
}

pub fn extract_u64_from_buffer(buffer: &[u8], offset: usize) -> Result<u64, &'static str> {
    if offset + 8 <= buffer.len() {
        buffer[offset..offset + 8]
            .try_into()
            .map(u64::from_ne_bytes)
            .map_err(|_| "Conversion error")
    } else {
        Err("Buffer overflow")
    }
}
pub fn extract_u32_from_buffer(buffer: &[u8], offset: usize) -> Result<u32, &'static str> {
    if offset + 4 <= buffer.len() {
        buffer[offset..offset + 4]
            .try_into()
            .map(u32::from_ne_bytes)
            .map_err(|_| "Conversion error")
    } else {
        Err("Buffer overflow")
    }
}

pub fn extract_f32_from_buffer(buffer: &[u8], offset: usize) -> Result<f32, &'static str> {
    if offset + 4 <= buffer.len() {
        buffer[offset..offset + 4]
            .try_into()
            .map(f32::from_ne_bytes)
            .map_err(|_| "Conversion error")
    } else {
        Err("Buffer overflow")
    }
}
pub fn extract_bool_from_buffer(buffer: &[u8], offset: usize) -> Result<bool, &'static str> {
    if offset < buffer.len() {
        Ok(buffer[offset] != 0)
    } else {
        Err("Buffer overflow")
    }
}
