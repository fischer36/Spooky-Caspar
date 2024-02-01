extern crate external;

use crate::overlay::window_manipulation::is_window_borderless;
use crate::sdk::minion_manager::MinionManager;
use crate::sdk::offsets::{self, LOCAL_PLAYER};
use crate::sdk::player_manager::{Player, PlayerManager, Skillshot};

use crate::{point2, PROCESS_NAME};
use external::{read, Process};
use log::info;
use nc::na::{Matrix4, Point3, Vector2, Vector3, Vector4};

use std::os::raw::c_void;
use std::time::Instant;
#[derive(Debug, Clone)]
pub struct Missile {
    pub name: String,
    pub src_id: u32,
    pub speed: u32,
    pub current_pos: nc::na::Vector3<f32>,
    pub start_pos: nc::na::Vector3<f32>,
    pub end_pos: nc::na::Vector3<f32>,
}
pub struct Game {
    pub local_player: Player,
    pub player_manager: PlayerManager,
    pub minion_manager: MinionManager,
    //pub missile_manager_address: usize,
    //pub missile_manager_list_address: usize,
    pub process_handle: *mut c_void,
    pub module_address: usize,
    pub game_to_screen_scaling: nc::na::Vector2<f32>,
    pub minimap_size: f32,
    pub minimap_position: nc::na::Point2<f32>,
    pub extra_screen_units: nc::na::Vector2<f32>,
}

impl Game {
    pub fn new(screen_size: nc::na::Vector2<f32>) -> Result<Self, Box<dyn std::error::Error>> {
        let start_time = Instant::now();

        info!("Trying to create game instance");
        let process = Process::from_process_name(PROCESS_NAME)?;
        let process_handle = process.process_handle;
        let module_address = process.get_module_base(PROCESS_NAME)?;
        info!("Process instance initialized");

        let local_player = Player::new(process_handle, module_address, LOCAL_PLAYER)
            .map_err(|e| format!("Error initializing local player {}", e))?;
        info!("Local player initialized");

        let player_manager = PlayerManager::new(&local_player.team, process_handle, module_address)
            .map_err(|e| format!("Error initializing player manager {}", e))?;

        let enemy_count = player_manager.enemy_obj_list.len();
        info!(
            "Player manager initialized, enemy count: {}",
            player_manager.enemy_obj_list.len()
        );

        // let missile_manager_address = read::<usize>(process_handle, module_address + offsets::MINION_MANAGER)?;
        //let missile_manager_list_address =
        //    read::<usize>(process_handle, missile_manager_address + offsets::MISSILE_MANAGER_LIST)?;
        // info!("Missile lists initialized");

        let minion_manager = MinionManager::new(process_handle, module_address)
            .map_err(|e| format!("Error initializing minion manager {}", e))?;
        info!("Minion manager initialized");

        let renderer_address = read::<usize>(process_handle, module_address + offsets::RENDERER)?;

        let renderer_width = read::<u32>(process_handle, renderer_address + offsets::RENDERER_WIDTH)?;
        let renderer_height = read::<u32>(process_handle, renderer_address + offsets::RENDERER_HEIGHT)?;

        let game_to_screen_scaling = nc::na::Vector2::new(
            screen_size.x / renderer_width as f32,
            screen_size.y / renderer_height as f32,
        );
        let extra_screen_units = if is_window_borderless() {
            nc::na::Vector2::new(16.0, 36.0)
        } else {
            nc::na::Vector2::new(0.0, 0.0)
        };

        let minimap_object = read::<usize>(process_handle, module_address + offsets::MINIMAP_OBJECT)?;
        let minimap_object_hud = read::<usize>(process_handle, minimap_object + offsets::MINIMAP_OBJECT_HUD)?;
        let minimap_object_pos =
            read::<nc::na::Point2<f32>>(process_handle, minimap_object_hud + offsets::MINIMAP_OBJECT_HUD_POS)?;
        let minimap_object_size = read::<f32>(process_handle, minimap_object_hud + offsets::MINIMAP_OBJECT_HUD_SIZE)?;

        println!(
            "minimap pos:{} minimap size:{:?}",
            minimap_object_pos, minimap_object_size
        );
        let game = Game {
            local_player: local_player,
            player_manager: player_manager,
            minion_manager: minion_manager,
            //missile_manager_address: missile_manager_address,
            //missile_manager_list_address: missile_manager_list_address,
            process_handle: process_handle,
            module_address: module_address,
            game_to_screen_scaling: game_to_screen_scaling,
            minimap_size: minimap_object_size,
            minimap_position: minimap_object_pos,
            extra_screen_units: extra_screen_units,
        };
        info!("Sucessfully initialized game\nIt took {:?}", start_time.elapsed());
        return Ok(game);
    }

    // pub fn get_missile(&self) -> Option<Missile> {
    //     let missile_address = read::<usize>(
    //         self.process_handle,
    //         self.missile_manager_list_address + offsets::MISSILE_MANAGER_MISSILE_3,
    //     )
    //     .unwrap();
    //     if missile_address == self.missile_manager_list_address {
    //         return None;
    //     }
    //     let missile_info_address = read::<usize>(
    //         self.process_handle,
    //         missile_address + offsets::MISSILE_MANAGER_ENTRY_INFO,
    //     )
    //     .unwrap();

    //     let missile_info_buffer = read::<[u8; 0x3A4]>(self.process_handle, missile_info_address).unwrap();
    //     let current_pos =
    //         extract_vector3_from_buffer(&missile_info_buffer, offsets::MISSILE_MANAGER_ENTRY_POSITION).unwrap();
    //     let start_pos =
    //         extract_vector3_from_buffer(&missile_info_buffer, offsets::MISSILE_MANAGER_ENTRY_START_POSITION).unwrap();
    //     let end_pos =
    //         extract_vector3_from_buffer(&missile_info_buffer, offsets::MISSILE_MANAGER_ENTRY_END_POSITION).unwrap();
    //     let speed = extract_u32_from_buffer(&missile_info_buffer, offsets::MISSILE_MANAGER_ENTRY_SPEED).unwrap();
    //     let src_id = extract_u32_from_buffer(&missile_info_buffer, offsets::MISSILE_MANAGER_ENTRY_SRC_IDX).unwrap();

    //     let raw_name = read::<[c_char; 20]>(
    //         self.process_handle,
    //         missile_info_address + offsets::MISSILE_MANAGER_ENTRY_NAME,
    //     )
    //     .unwrap();
    //     let c_str = unsafe { CStr::from_ptr(raw_name.as_ptr()) };
    //     let name = c_str.to_str().unwrap().to_string();

    //     let missile = Missile {
    //         name: name,
    //         src_id: src_id,
    //         speed: speed,
    //         current_pos: current_pos,
    //         start_pos: start_pos,
    //         end_pos: end_pos,
    //     };
    //     return Some(missile);
    // }

    pub fn get_time(&self) -> Result<f32, Box<dyn std::error::Error>> {
        let time = read::<f32>(self.process_handle, self.module_address + offsets::GAME_TIME_SECONDS)?;
        return Ok(time);
    }

    pub fn get_players_within_range(&self, range: f32) -> Option<Vec<&Player>> {
        let players_in_range: Vec<&Player> = self
            .player_manager
            .enemy_obj_list
            .iter()
            .filter(|&player| {
         

                let distance = nc::na::distance(
                    &point2!(self.local_player.position.x, self.local_player.position.z),
                    &point2!(player.position.x, player.position.z),
                );

                distance <= range && player.health > 1.0 && player.is_visible
            })
            .collect(); 

        if players_in_range.is_empty() {
            None
        } else {
            Some(players_in_range)
        }
    }

    // pub fn get_players_on_screen(&self) -> Vec<&Player> {
    //     self.player_manager
    //         .enemy_obj_list
    //         .iter()
    //         .filter(|player| {
    //             self.world_to_screen(&player.position).is_some() && player.is_visible && player.health > 1.0
    //         })
    //         .collect()
    // }

    pub fn get_relevant_players(&self) -> Vec<&Player> {
        let screen_width = 1920.0 / 0.5; // Convert screen width from pixels to game units
        let screen_height = 1080.0 / 0.5; // Convert screen height from pixels to game units
        let half_screen_width = screen_width / 2.0;
        let half_screen_height = screen_height / 2.0;

        self.player_manager
            .enemy_obj_list
            .iter() 
            .filter(|player| {

                let rel_x = player.position.x - self.local_player.position.x;
                let rel_y = player.position.y - self.local_player.position.y;

 
                rel_x.abs() <= half_screen_width
                    && rel_y.abs() <= half_screen_height
                    && player.is_visible
                    && player.health > 1.0
            })
            .collect()
    }
    pub fn get_view_proj_matrix(&self) -> Result<Matrix4<f32>, Box<dyn std::error::Error>> {
        let view_proj_matrix_values = read::<[f32; 32]>(
            self.process_handle,
            self.module_address + offsets::VIEW_PROJECTION_MATRIX,
        )?;

  
        let (view_values, proj_values) = view_proj_matrix_values.split_at(16);

        let view_matrix = Matrix4::from_column_slice(view_values);
        let proj_matrix = Matrix4::from_column_slice(proj_values);


        return Ok(proj_matrix * view_matrix);
    }

    pub fn world_to_screen(&self, pos: &nc::na::Point3<f32>) -> Option<nc::na::Point2<f32>> {
        let view_proj_matrix = self.get_view_proj_matrix().unwrap();


        let clip_coords = view_proj_matrix * Vector4::new(pos.x, pos.y, pos.z, 1.0);

        
        if clip_coords.w < 0.1 {
            return None;
        }

   
        let ndc = Vector3::new(
            clip_coords.x / clip_coords.w,
            clip_coords.y / clip_coords.w,
            clip_coords.z / clip_coords.w,
        );

        let screen_x = (1920.0 / 2.0) * (ndc.x + 1.0);
        let screen_y = (1080.0 / 2.0) * (1.0 - ndc.y);

      
        Some(point2!(screen_x, screen_y)) 
    }

    pub fn screen_to_world(
        &self,
        screen_pos: Vector2<f32>,
        view_proj_matrix: &Matrix4<f32>,
        screen_width: f32,
        screen_height: f32,
        camera_y: f32, 
    ) -> Option<Point3<f32>> {

        let ndc_x = (2.0 * screen_pos.x) / screen_width - 1.0;
        let ndc_y = 1.0 - (2.0 * screen_pos.y) / screen_height;
        let ndc_z = 1.0; 

        let ndc = Vector4::new(ndc_x, ndc_y, ndc_z, 1.0);

        let inv_view_proj = view_proj_matrix.try_inverse().expect("Matrix inversion failed");
        let world_coords = inv_view_proj * ndc;

        if world_coords.w.abs() < f32::EPSILON {
            return None;
        }

        let mut world_pos = Point3::from_homogeneous(world_coords).expect("Homogeneous division failed");


        world_pos.y = camera_y;

        Some(world_pos)
    }

    pub fn game_skillshots(&self) -> Vec<(String, Skillshot, char)> {
        let mut game_skillshots = Vec::new();
        for player in self.player_manager.enemy_obj_list.iter() {
            for spell in player.ability_manager.abilities() {
                if spell.skillshot != Skillshot::False {
                    game_skillshots.push((player.champion_name.clone(), spell.skillshot.clone(), spell.key.clone()));
                }
            }
        }
        return game_skillshots;
    }
}

pub mod match_functions {
    use external::read;
    use nc::na::{Matrix4, Vector3, Vector4};
    use std::os::raw::c_void;

    use crate::{point2, sdk::offsets};

    pub fn world_to_minimap(
        &pos: &nc::na::Point3<f32>,
        minimap_position: nc::na::Point2<f32>,
        minimap_size: f32,
        screen_scaing: nc::na::Vector2<f32>,
    ) -> nc::na::Point2<f32> {
        let scaled_pos = point2!(pos.x / 15000.0, pos.z / 15000.0);
        let screen_pos = point2!(
            minimap_position.x + scaled_pos.x * minimap_size,
            minimap_position.y + minimap_size - (scaled_pos.y * minimap_size)
        );
        screen_pos
    }
    pub fn world_to_screen(
        module_address: usize,
        process_handle: *mut c_void,
        pos: &nc::na::Point3<f32>,
    ) -> Option<nc::na::Point2<f32>> {
        let view_proj_matrix = get_view_proj_matrix(module_address, process_handle).unwrap();


        let clip_coords = view_proj_matrix * Vector4::new(pos.x, pos.y, pos.z, 1.0);

     
        if clip_coords.w < 0.1 {
            return None;
        }


        let ndc = Vector3::new(
            clip_coords.x / clip_coords.w,
            clip_coords.y / clip_coords.w,
            clip_coords.z / clip_coords.w,
        );


        let screen_x = (1920.0 / 2.0) * (ndc.x + 1.0);
        let screen_y = (1080.0 / 2.0) * (1.0 - ndc.y);


        Some(point2!(screen_x, screen_y))
    }

    pub fn get_view_proj_matrix(
        module_address: usize,
        process_handle: *mut c_void,
    ) -> Result<Matrix4<f32>, Box<dyn std::error::Error>> {
        let view_proj_matrix_values =
            read::<[f32; 32]>(process_handle, module_address + offsets::VIEW_PROJECTION_MATRIX)?;

    
        let (view_values, proj_values) = view_proj_matrix_values.split_at(16);


        let view_matrix = Matrix4::from_column_slice(view_values);
        let proj_matrix = Matrix4::from_column_slice(proj_values);


        return Ok(proj_matrix * view_matrix);
    }

    pub fn get_ai_address(process_handle: *mut c_void, entity_base: usize) -> usize {
        // Credits: https://www.unknowncheats.me/forum/league-of-legends/579576-ai-manager-decryption-x64.html

        let v1: *const u8 = ((entity_base as usize) + offsets::OBJ_AI_MANAGER) as *const u8;

        let v3b = read::<u8>(process_handle, (v1 as usize) + 16).unwrap();
        let v7 = read::<u64>(process_handle, (v1 as usize) + (8 * v3b as u64 + 24) as usize).unwrap();
        let v5 = read::<u64>(process_handle, (v1 as usize) + 8).unwrap();
        let v7 = v7 ^ !v5;
        let return_val = read::<usize>(process_handle, v7 as usize + 16).unwrap();
        return return_val;
    }

    pub fn get_direction_any_player(player_address: usize, process_handle: *mut c_void) -> Vector3<f32> {
        let direction_x = read::<f32>(process_handle, player_address + offsets::OBJ_DIRECTION_X).unwrap();
        let direction_z = read::<f32>(process_handle, player_address + offsets::OBJ_DIRECTION_Z).unwrap();
        println!("direction_x: {} direction_y:{}", direction_x, direction_z);
        return Vector3::new(direction_x, 0.0, direction_z);
    }
}
