use std::time::Instant;

use crate::{
    offsets,
    sdk::player_manager::Player,
    utils::{
        extract_bool_from_buffer, extract_f32_from_buffer, extract_u32_from_buffer, extract_u64_from_buffer,
        extract_vector3_from_buffer, get_distance,
    },
};
use external::{read, read_buffer};
use log::info;
use nc::na::Vector3;
use std::os::raw::c_void;
#[derive(Debug, Clone)]
pub struct Minion {
    pub address: usize,
    pub id: u32,
    pub health: f32,
    pub position: Vector3<f32>,
    pub team: u16,
    pub is_visible: bool,
}

#[derive(Debug, Clone)]
pub struct MinionManager {
    pub manager_address: usize,
    pub list: Vec<Minion>,
    pub count: usize,
}

impl MinionManager {
    pub fn new(process_handle: *mut c_void, module_address: usize) -> Result<Self, String> {
        let start_time = Instant::now();
        let mut minion_list = Vec::new();

        let manager_address = read::<usize>(process_handle, module_address + offsets::MINION_MANAGER)
            .map_err(|e| format!("Err reading minion-manager address: {}", e))?;

        let list_address = read::<usize>(process_handle, manager_address + offsets::MINION_MANAGER_LIST)
            .map_err(|e| format!("Err reading list ptr: {}", e))?;

        let minion_count = read::<u32>(process_handle, manager_address + 0x10)
            .map_err(|e| format!("Err reading minion count: {}", e))?;

        if minion_count == 0 {
            return Ok(Self {
                manager_address: manager_address,
                count: minion_list.len(),
                list: minion_list,
            });
        }
        let buffer_size = 0x8 * minion_count as usize;

        let minions_buffer = read_buffer(process_handle, list_address, buffer_size)
            .map_err(|e| format!("Error reading minion data buffer: {}", e))?;

        for i in 0..minion_count {
            let minion_offset = 0x8 * i;

            let minion_address = extract_u64_from_buffer(&minions_buffer, minion_offset as usize).map_err(|e| {
                format!(
                    "Error extracting minion pointer: {} index: {} offset: {}",
                    e, i, minion_offset
                )
            })? as usize;

            let minion_team = match read::<u16>(process_handle, minion_address + offsets::OBJ_TEAM) {
                Ok(minion_team) => minion_team,
                Err(e) => {
                    continue;
                }
            };

            if minion_team != 200 {
                continue;
            }
            let minion_data_buffer = read::<[u8; 0x1088 + 0x4]>(process_handle, minion_address)
                .map_err(|e| format!("Error reading minion data buffer: {}", e))?;

            let minion_position = extract_vector3_from_buffer(&minion_data_buffer, offsets::OBJ_POSITION)
                .map_err(|e| format!("Error reading minion position: {}", e))?;

            let minion_team = extract_u32_from_buffer(&minion_data_buffer, offsets::OBJ_TEAM)
                .map_err(|e| format!("Error reading minion team: {}", e))?;

            let minion_is_visible = extract_bool_from_buffer(&minion_data_buffer, offsets::OBJ_IS_VISIBLE)
                .map_err(|e| format!("Error reading minion is_visible: {}", e))?;

            let minion_id = extract_u32_from_buffer(&minion_data_buffer, offsets::OBJ_IDX)
                .map_err(|e| format!("Error reading minion id: {}", e))?;

            let minion_health = extract_f32_from_buffer(&minion_data_buffer, offsets::OBJ_HEALTH)
                .map_err(|e| format!("Error reading minion health: {}", e))?;

            let minion = Minion {
                address: minion_address,
                id: minion_id,
                health: minion_health,
                position: minion_position,
                team: minion_team as u16,
                is_visible: minion_is_visible,
            };
            minion_list.push(minion);
        }

        info!(
            "Sucessfully initialized minions\nminions count: {}\ntime it took: {:?}",
            minion_list.len(),
            start_time.elapsed()
        );
        Ok(Self {
            manager_address: manager_address,
            count: minion_list.len(),
            list: minion_list,
        })
    }

    pub fn update_list(&mut self, process_handle: *mut c_void) -> Result<String, String> {
        let start_time = Instant::now();
        info!("Trying to update minion list");
        let mut i = 0;
        while i != self.list.len() {
            let minion_data_buffer = read::<[u8; 0x1088 + 0x4]>(process_handle, self.list[i].address)
                .map_err(|e| format!("Error reading minion data buffer: {}", e))?;
            let minion_health = extract_f32_from_buffer(&minion_data_buffer, offsets::OBJ_HEALTH)
                .map_err(|e| format!("Error reading minion data buffer: {}", e))?;
            if minion_health < 1.0 {
                self.list.remove(i);
            } else {
                self.list[i].position = extract_vector3_from_buffer(&minion_data_buffer, offsets::OBJ_POSITION)
                    .map_err(|e| format!("Error reading minion data buffer: {}", e))?;

                self.list[i].team = extract_u32_from_buffer(&minion_data_buffer, offsets::OBJ_TEAM)
                    .map_err(|e| format!("Error reading minion team: {}", e))?
                    as u16;

                self.list[i].is_visible = extract_bool_from_buffer(&minion_data_buffer, offsets::OBJ_IS_VISIBLE)
                    .map_err(|e| format!("Error reading minion is_visible: {}", e))?;

                self.list[i].id = extract_u32_from_buffer(&minion_data_buffer, offsets::OBJ_IDX)
                    .map_err(|e| format!("Error reading minion id: {}", e))?;
                i += 1;
            }
        }
        if self.list.len() == 0 {
            return Err("Minion list is empty".to_string());
        }
        info!(
            "Sucessfully updated minion list\ntime it took: {:?}",
            start_time.elapsed()
        );
        return Ok("Sucessfully updated minions".to_string());
    }

    pub fn refresh_list(&mut self, process_handle: *mut c_void, module_address: usize) -> Result<(), String> {
        // Reinitialize the minion list to reflect the current state of the game
        let new_minion_manager = MinionManager::new(process_handle, module_address)?;
        self.list = new_minion_manager.list;
        self.count = new_minion_manager.count;

        Ok(())
    }
}
