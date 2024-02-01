const KEY_ARRAY: [char; 4] = ['q', 'w', 'e', 'r'];
use crate::sdk::game::match_functions::*;
use crate::sdk::game::{self, Game};
use crate::sdk::offsets::{self};
use crate::utils::{
    extract_bool_from_buffer, extract_f32_from_buffer, extract_u32_from_buffer, extract_u64_from_buffer,
    extract_vector3_from_buffer, get_distance, TargetMode,
};
use crate::{sdk, PROC_HANDLE};

use external::{read, read_buffer};
use log::{info, warn};
use nc::na::Vector3;
use std::ffi::CStr;

use std::fs;
use std::os::raw::{c_char, c_void};
use std::time::Instant;

#[derive(Debug, Clone)]
pub struct PlayerManager {
    pub enemy_obj_list: Vec<Player>,
}

impl PlayerManager {
    pub fn new(
        local_player_team_value: &u32,

        process_handle: *mut c_void,
        module_address: usize,
    ) -> Result<Self, String> {
        let mut enemy_obj_list = Vec::new();

        let player_manager = read::<usize>(process_handle, module_address + sdk::offsets::CHAMPION_MANAGER)
            .map_err(|e| format!("Error reading player manager address: {}", e))?;

        let player_manager_list = read::<usize>(process_handle, player_manager + sdk::offsets::CHAMPION_MANAGER_LIST)
            .map_err(|e| format!("Error reading player manager address: {}", e))?;

        for n in 1..10 {
            let champion_offset: usize = n as usize * 0x8;

            if let Ok(enemy_obj) = Player::new(process_handle, player_manager_list, champion_offset) {
                println!("{}", enemy_obj.champion_name);
                if enemy_obj.team != *local_player_team_value {
                    enemy_obj_list.push(enemy_obj);
                }
            } else {
                break;
            }
        }

        Ok(Self { enemy_obj_list })
    }

    pub fn update_list_enemy_info(&mut self, process_handle: *mut c_void) -> Result<String, String> {
        for enemy in self.enemy_obj_list.iter_mut() {
            match enemy.update_info(process_handle) {
                Ok(_) => continue,
                Err(error) => return Err(error),
            }
        }
        return Ok("Sucessfully updated every enemy".to_owned());
    }
    pub fn get_sorted_and_filtered_players(
        &self,
        local_player_position: &nc::na::Point3<f32>,
        target_mode: &TargetMode,
    ) -> Vec<&Player> {
        let mut filtered_players: Vec<&Player> = self
            .enemy_obj_list
            .iter()
            .filter(|player| player.health >= 1.0 && player.is_visible)
            .collect();

        match target_mode {
            TargetMode::LowestHp => {
                filtered_players.sort_by(|a, b| a.health.partial_cmp(&b.health).unwrap_or(std::cmp::Ordering::Equal));
            }
            TargetMode::LowestDistance => {
                filtered_players.sort_by(|a, b| {
                    let distance_a = nc::na::distance(
                        local_player_position,
                        &nc::na::Point3::new(a.position.x, a.position.y, a.position.z),
                    );
                    let distance_b = nc::na::distance(
                        local_player_position,
                        &nc::na::Point3::new(b.position.x, b.position.y, b.position.z),
                    );
                    distance_a.partial_cmp(&distance_b).unwrap_or(std::cmp::Ordering::Equal)
                });
            }
        }

        filtered_players
    }
    pub fn get_players_within_distance(&self, local_player_position: &Vector3<f32>) -> Vec<&Player> {
        let max_distance = 1100.0;

        self.enemy_obj_list
            .iter()
            .filter(|player| {
                get_distance(&local_player_position, &player.position) <= max_distance
                    && player.health >= 1.0
                    && player.is_visible
            })
            .collect()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Ability {
    pub key: char,
    pub base_address: usize,
    //pub name: String,
    pub skillshot: Skillshot,
    pub cooldown: f32,
    pub level: u32,
}

impl Ability {
    fn new(
        key: char,
        champion_db: &toml::Value,
        champion_name: &String,
        player_base_address: usize,
        process_handle: *mut c_void,
    ) -> Result<Self, String> {
        let key_index = KEY_ARRAY.iter().position(|&x| x == key).unwrap();
        let spell_slot_offset = key_index * 0x8;

        let spell_slot_address = read::<usize>(
            process_handle,
            player_base_address + sdk::offsets::OBJ_SPELL_BOOK + spell_slot_offset,
        )
        .unwrap();
        let spell_level = read::<u32>(process_handle, spell_slot_address + sdk::offsets::OBJ_SPELL_BOOK_LEVEL).unwrap();
        let spell_cd = read::<f32>(
            process_handle,
            spell_slot_address + sdk::offsets::OBJ_SPELL_BOOK_COOLDOWN,
        )
        .unwrap();

        let mut skillshot = Skillshot::False;
        champion_db
            .get(champion_name)
            .and_then(|champion| champion.get("spells"))
            .and_then(|spells| spells.get(&key.to_string()))
            .and_then(|key_spell| {
                if key_spell.get("is_skillshot").and_then(|v| v.as_bool()).unwrap_or(false) {
                    let skillshot_type = key_spell
                        .get("skillshot_type")
                        .and_then(|v| v.as_str())
                        .unwrap_or_default();
                    info!(
                        "Skillshot type found: {} for champion: {} key: {}",
                        skillshot_type, champion_name, key
                    );
                    match skillshot_type {
                        "linear" => {
                            let skillshot_width = key_spell
                                .get("skillshot_width")
                                .and_then(|v| v.as_integer())
                                .unwrap_or(200);
                            let skillshot_speed = key_spell
                                .get("skillshot_speed")
                                .and_then(|v| v.as_integer())
                                .unwrap_or(2000);
                            let skillshot_range = key_spell
                                .get("skillshot_range")
                                .and_then(|v| v.as_integer())
                                .unwrap_or(1050);
                            skillshot = Skillshot::Linear {
                                speed: skillshot_speed as f32,
                                width: skillshot_width as f32,
                                range: skillshot_range as f32,
                                dodge: true,
                            };
                        }
                        "circular" => {
                            let skillshot_radius = key_spell
                                .get("skillshot_radius")
                                .and_then(|v| v.as_integer())
                                .unwrap_or(200);
                            skillshot = Skillshot::Circular {
                                radius: skillshot_radius as f32,
                                dodge: true,
                            };
                        }
                        "trap" => {
                            let trap_radius = key_spell.get("trap_radius").and_then(|v| v.as_integer()).unwrap_or(70);

                            skillshot = Skillshot::Trap {
                                radius: trap_radius as f32,
                                duration: 30.0,
                                dodge: true,
                            };
                        }
                        "stationaryrectangle" => {
                            let length = key_spell
                                .get("skillshot_length")
                                .and_then(|v| v.as_integer())
                                .unwrap_or(300);
                            let width = key_spell
                                .get("skillshot_width")
                                .and_then(|v| v.as_integer())
                                .unwrap_or(300);
                            let duration = key_spell
                                .get("skillshot_duration")
                                .and_then(|v| v.as_integer())
                                .unwrap_or(300);
                            skillshot = Skillshot::StationaryRectangle {
                                length: length as f32,
                                width: width as f32,
                                duration: duration as f32,
                                dodge: true,
                            }
                        }
                        "directionbasedlinear" => {
                            println!("direction based");
                            let skillshot_width = key_spell
                                .get("skillshot_width")
                                .and_then(|v| v.as_integer())
                                .unwrap_or(200);
                            let skillshot_speed = key_spell
                                .get("skillshot_speed")
                                .and_then(|v| v.as_integer())
                                .unwrap_or(2000);
                            skillshot = Skillshot::DirectionBasedLinear {
                                parent_player_address: player_base_address.clone(),
                                length: 1000.0,
                                speed: skillshot_speed as f32,
                                width: skillshot_width as f32,
                                dodge: true,
                            };
                        }
                        _ => {
                            skillshot = Skillshot::False;
                            info!("Failed to get skillshot data for skillshot {}", key_spell);
                        }
                    }
                }
                Some(()) // Return Some to continue the chain; the value is not used
            });

        Ok(Self {
            key: key,
            base_address: spell_slot_address,
            cooldown: spell_cd,
            level: spell_level,
            skillshot: skillshot,
        })
    }

    pub fn update_info(&mut self, process_handle: *mut c_void) -> Result<String, String> {
        let spell_level = read::<u32>(process_handle, self.base_address + sdk::offsets::OBJ_SPELL_BOOK_LEVEL).unwrap();
        let spell_cd = read::<f32>(
            process_handle,
            self.base_address + sdk::offsets::OBJ_SPELL_BOOK_COOLDOWN,
        )
        .unwrap();
        self.cooldown = spell_cd;
        self.level = spell_level;
        return Ok("Successfully updated ability info".to_string());
    }

    pub fn is_spell_ready(&self, game: &Game) -> bool {
        let game_time = game.get_time().unwrap();

        let spell_cd = read::<f32>(
            game.process_handle,
            self.base_address + sdk::offsets::OBJ_SPELL_BOOK_COOLDOWN,
        )
        .unwrap();
        return spell_cd < game_time;
    }
}

#[derive(Debug, Clone)]
pub struct Player {
    pub base_address: usize,
    pub ai_address: usize,
    pub idx: usize,
    pub champion_name: String,
    pub team: u32,
    pub health: f32,
    pub mana: f32,
    pub position: Vector3<f32>,
    pub is_visible: bool,
    pub ability_manager: AbilityManager,
    pub gameplay_radius: f32,
    pub base_attack_speed: f32,
    pub bonus_attack_speed_multiplier: f32,
    pub windup_percent: f32,
    pub target: bool,
}

impl Player {
    pub fn new(process_handle: *mut c_void, address: usize, offset: usize) -> Result<Self, String> {
        let champion_address = read::<usize>(process_handle, address + offset)
            .map_err(|e| format!("Error champion address {:?}: {:?}", offset, e))?;

        // Check if champion_address is a null pointer
        if champion_address == 0 {
            return Err("Null pointer".to_string());
        }

        let team = read::<u32>(process_handle, champion_address + sdk::offsets::OBJ_TEAM)
            .map_err(|e| format!("Error reading team for address {:?}: {:?}", champion_address, e))?;

        // Check if team is not equal to 200 or 100
        if team != 200 && team != 100 {
            return Err("Invalid team value".to_string());
        }
        let ai_address = get_ai_address(process_handle, champion_address);
        let idx = read::<usize>(process_handle, champion_address + sdk::offsets::OBJ_IDX)
            .map_err(|e| format!("Error reading entity idx for {:?}: {:?}", offset, e))?;

        let raw_champion_name = read::<[c_char; 12]>(process_handle, champion_address + sdk::offsets::OBJ_NAME)
            .map_err(|e| format!("Error reading raw entity name for {:?}: {:?}", offset, e))?;

        let c_str = unsafe { CStr::from_ptr(raw_champion_name.as_ptr()) };
        let champion_name = c_str
            .to_str()
            .map_err(|err| format!("Error converting CStr to String: {:?}", err))?
            .to_string();
        let toml_contents =
            fs::read_to_string("champion_db.toml").map_err(|e| format!("Failed to read champion_db.toml: {:?}", e))?;
        info!("AI address for {} is: {:x}", champion_name, ai_address);
        // Parse the TOML file
        let toml_data: toml::Value =
            toml::from_str(&toml_contents).map_err(|e| format!("Failed to parse champion_db.toml: {:?}", e))?;

        // Adjust for potential case sensitivity issues
        let champion_key = champion_name.to_lowercase();

        // Extract the gameplay_radius for the champion
        let gameplay_radius = toml_data
            .get(&champion_key)
            .and_then(|champ_data| champ_data.get("gameplay_radius"))
            .and_then(|radius| radius.as_float().or_else(|| radius.as_integer().map(|i| i as f64)))
            .unwrap_or_else(|| {
                info!("Failed to find gameplay radius for {}", champion_key);
                // Use a default value (e.g., 100.0) if retrieval fails
                100.0
            });
        let base_attack_speed = toml_data
            .get(&champion_key)
            .and_then(|champ_data| champ_data.get("base_attack_speed"))
            .and_then(|attack_speed| {
                attack_speed
                    .as_float()
                    .or_else(|| attack_speed.as_integer().map(|i| i as f64))
            })
            .unwrap_or_else(|| {
                info!("Failed to find base attack speed for {}", champion_key);
                // Use a default value (e.g., 100.0) if retrieval fails
                0.6
            });
        let bonus_attack_speed_ratio = toml_data
            .get(&champion_key)
            .and_then(|champ_data| champ_data.get("attack_speed_ratio"))
            .and_then(|bonus_attack_speed_ratio| {
                bonus_attack_speed_ratio
                    .as_float()
                    .or_else(|| bonus_attack_speed_ratio.as_integer().map(|i| i as f64))
            })
            .unwrap_or_else(|| {
                info!("Failed to find base attack speed ratio for {}", champion_key);
                0.625
            });
        let windup_percent = toml_data
            .get(&champion_key)
            .and_then(|champ_data| champ_data.get("windup_percent"))
            .and_then(|bonus_attack_speed_ratio| {
                bonus_attack_speed_ratio
                    .as_float()
                    .or_else(|| bonus_attack_speed_ratio.as_integer().map(|i| i as f64))
            })
            .unwrap_or_else(|| {
                info!("Failed to find windup_percent for {}", champion_key);
                0.3
            });
        let position = read::<Vector3<f32>>(process_handle, champion_address + sdk::offsets::OBJ_POSITION).unwrap();
        let ability_manager = AbilityManager::new(process_handle, champion_address, toml_data, champion_key).unwrap();

        Ok(Self {
            base_address: champion_address,
            ai_address: ai_address,
            idx: idx,
            champion_name: champion_name,
            team: team,
            health: 0.0, // place holder
            mana: 0.0,   // place holder
            position: position,
            is_visible: false, // place holder
            ability_manager: ability_manager,
            gameplay_radius: gameplay_radius as f32,
            base_attack_speed: base_attack_speed as f32,
            bonus_attack_speed_multiplier: bonus_attack_speed_ratio as f32,
            windup_percent: windup_percent as f32,
            target: false,
        })
    }

    pub fn update_info(&mut self, process_handle: *mut c_void) -> Result<String, String> {
        let buffer = read::<[u8; 0x1088 + 0x4]>(process_handle, self.base_address)
            .map_err(|e| format!("Error reading player data buffer for {}: {:?}", self.champion_name, e))?;

        let health = extract_f32_from_buffer(&buffer, sdk::offsets::OBJ_HEALTH)
            .map_err(|e| format!("Error extracting is_dead_byte for {}: {}", self.champion_name, e))?;
        let mana = extract_f32_from_buffer(&buffer, sdk::offsets::OBJ_MANA)
            .map_err(|e| format!("Error extracting is_dead_byte for {}: {}", self.champion_name, e))?;
        let is_visible = extract_bool_from_buffer(&buffer, sdk::offsets::OBJ_IS_VISIBLE)
            .map_err(|e| format!("Error extracting is_visible_byte for {}: {}", self.champion_name, e))?;
        let position = extract_vector3_from_buffer(&buffer, sdk::offsets::OBJ_POSITION)
            .map_err(|e| format!("Error extracting position for {}: {}", self.champion_name, e))?;

        self.position = position;
        self.health = health;
        self.is_visible = is_visible;
        self.mana = mana;

        Ok(format!("Successfully updated info for {}", self.champion_name))
    }
    pub fn get_magic_pen(&self, process_handle: *mut c_void) -> (f32, f32) {
        return (
            read::<f32>(process_handle, self.base_address + sdk::offsets::OBJ_MAGIC_PEN).unwrap(),
            read::<f32>(process_handle, self.base_address + sdk::offsets::OBJ_MAGIC_PEN_PERCENT).unwrap(),
        );
    }
    pub fn get_direction(&self, process_handle: *mut c_void) -> Vector3<f32> {
        return Vector3::new(
            read::<f32>(process_handle, self.base_address + sdk::offsets::OBJ_DIRECTION_X).unwrap(),
            0.0,
            read::<f32>(process_handle, self.base_address + sdk::offsets::OBJ_DIRECTION_Z).unwrap(),
        );
    }
    pub fn get_ability_power(&self, process_handle: *mut c_void) -> f32 {
        return read::<f32>(process_handle, self.base_address + sdk::offsets::OBJ_ABILITY_POWER).unwrap();
    }

    pub fn get_magic_resist(&self, process_handle: *mut c_void) -> f32 {
        let bonus_magic_resist = read::<f32>(
            process_handle,
            self.base_address + sdk::offsets::OBJ_BONUS_MAGIC_RESISTANCE,
        )
        .unwrap();
        return 40.0 + bonus_magic_resist;
    }
    pub fn get_attack_speed(&self, process_handle: *mut c_void) -> Result<f32, String> {
        let bonus_attack_speed = read::<f32>(process_handle, self.base_address + sdk::offsets::OBJ_BONUS_ATTACK_SPEED)
            .map_err(|e| format!("Error reading bonus attack speed {:?}: {:?}", self.champion_name, e))?;
        let attack_speed = self.base_attack_speed + (bonus_attack_speed * self.bonus_attack_speed_multiplier);
        return Ok(attack_speed);
    }

    pub fn get_current_path(&self, process_handle: *mut c_void) -> Result<f32, String> {
        return Ok(read::<f32>(
            process_handle,
            self.ai_address + sdk::offsets::OBJ_AI_MANAGER_PATH_SEGMENTS_COUNT,
        )
        .map_err(|e| format!("Error reading current path for {:?}: {:?}", self.champion_name, e))?);
    }

    pub fn get_attack_range(&self, process_handle: *mut c_void) -> Result<f32, String> {
        return Ok(
            read::<f32>(process_handle, self.base_address + sdk::offsets::OBJ_ATTACK_RANGE)
                .map_err(|e| format!("Error reading attack range for {:?}: {:?}", self.champion_name, e))?,
        );
    }

    pub fn get_ai_manager_details(
        &self,
        handle: *mut c_void,
    ) -> Result<
        (
            Vector3<f32>,
            Vector3<f32>,
            Vector3<f32>,
            Vector3<f32>,
            u32,
            f32,
            bool,
            bool,
            f32,
            //Vector3<f32>,
            //Vector3<f32>,
            //bool,
            //Vector3<f32>,
            //u32,
        ),
        String,
    > {
        let buffer = read::<[u8; 0x420]>(handle, self.ai_address)
            .map_err(|e| format!("Error reading AI manager buffer: {:?}", e))?;

        // Extract values from buffer
        let is_moving = extract_bool_from_buffer(&buffer, sdk::offsets::OBJ_AI_MANAGER_IS_MOVING)
            .map_err(|_| "Failed to extract 'is_moving'")?;
        let current_position = extract_vector3_from_buffer(&buffer, sdk::offsets::OBJ_AI_MANAGER_CURRENT_POSITION)?;
        let is_dashing = extract_bool_from_buffer(&buffer, sdk::offsets::OBJ_AI_MANAGER_IS_DASHING)
            .map_err(|_| "Failed to extract 'is_dashing'")?;
        let start_path = extract_vector3_from_buffer(&buffer, sdk::offsets::OBJ_AI_MANAGER_START_PATH)?;
        let end_path = extract_vector3_from_buffer(&buffer, sdk::offsets::OBJ_AI_MANAGER_END_PATH)?;
        let target_position = extract_vector3_from_buffer(&buffer, sdk::offsets::OBJ_AI_MANAGER_TARGET_POSITION)?;
        let dash_speed = extract_f32_from_buffer(&buffer, sdk::offsets::OBJ_AI_MANAGER_DASH_SPEED)
            .map_err(|_| "Failed to extract 'dash_speed'")?;
        let movement_speed = extract_f32_from_buffer(&buffer, sdk::offsets::OBJ_AI_MANAGER_MOVEMENT_SPEED)?;
        //let current_path_segment = extract_vector3_from_buffer(&buffer, sdk::offsets::OBJ_AI_MANAGER_CURRENT_PATH_SEGMENT)?;
        let path_segments_count = extract_u32_from_buffer(&buffer, sdk::offsets::OBJ_AI_MANAGER_PATH_SEGMENTS_COUNT)?;

        Ok((
            current_position,
            target_position,
            start_path,
            end_path,
            path_segments_count,
            movement_speed,
            is_moving,
            is_dashing,
            dash_speed,
        ))
    }
    pub fn get_end_path(&self, process_handle: *mut c_void) -> Vector3<f32> {
        return read::<Vector3<f32>>(process_handle, self.ai_address + sdk::offsets::OBJ_AI_MANAGER_END_PATH).unwrap();
    }

    pub fn get_active_spell(&self, handle: *mut c_void) -> Result<ActiveSpell, String> {
        // Assuming the total size of all data we need to read

        let active_spell_address = read::<usize>(handle, self.base_address + sdk::offsets::OBJ_ACTIVE_SPELL)
            .map_err(|e| format!("Error reading active spell address for {}: {:?}", self.champion_name, e))?;

        if active_spell_address == 0 {
            return Err("No active spell".to_string());
        }

        let buffer = read::<[u8; 0xEC]>(handle, active_spell_address)
            .map_err(|e| format!("Error reading active spell data for {}: {:?}", self.champion_name, e))?;

        let spell_slot = extract_u32_from_buffer(&buffer, sdk::offsets::OBJ_ACTIVE_SPELL_SLOT)
            .map_err(|e| format!("Error extracting spell slot for {}: {}", self.champion_name, e))?;

        let spell_key = match spell_slot {
            0 => 'q',
            1 => 'w',
            2 => 'e',
            3 => 'r',
            _ => return Err("Invalid spell slot".to_string()),
        };

        let src_id = extract_u32_from_buffer(&buffer, sdk::offsets::OBJ_ACTIVE_SPELL_SRC_ID)
            .map_err(|e| format!("Error extracting source ID for {}: {}", self.champion_name, e))?;
        let target_id = extract_u32_from_buffer(&buffer, sdk::offsets::OBJ_ACTIVE_SPELL_TARGET_ID)
            .map_err(|e| format!("Error extracting target ID for {}: {}", self.champion_name, e))?;
        let start_pos = extract_vector3_from_buffer(&buffer, sdk::offsets::OBJ_ACTIVE_SPELL_START_POSITION)
            .map_err(|e| format!("Error extracting start position for {}: {}", self.champion_name, e))?;
        let end_pos = extract_vector3_from_buffer(&buffer, sdk::offsets::OBJ_ACTIVE_SPELL_END_POSITION)
            .map_err(|e| format!("Error extracting end position for {}: {}", self.champion_name, e))?;

        let ability = match spell_key {
            'q' => &self.ability_manager.q,
            'w' => &self.ability_manager.w,
            'e' => &self.ability_manager.e,
            'r' => &self.ability_manager.r,
            _ => return Err(format!("Ability not found for key {}", spell_key)),
        };

        Ok(ActiveSpell {
            ability,
            key: spell_key,
            src_id,
            target_id,
            start_pos,
            end_pos,
        })
    }

    pub fn get_mana(&self, process_handle: *mut c_void) -> Result<f32, String> {
        let obj_mana = read::<f32>(process_handle, self.base_address + sdk::offsets::OBJ_MANA).map_err(|e| {
            format!(
                "Failed to read mana for champion: {:?} error: {:?}",
                self.champion_name, e
            )
        })?;
        return Ok(obj_mana);
    }

    pub fn get_movement_speed(&self, process_handle: *mut c_void) -> Result<f32, String> {
        let movement_speed =
            read::<f32>(process_handle, self.base_address + sdk::offsets::OBJ_MOVE_SPEED).map_err(|e| {
                format!(
                    "Failed reading movement speed for champion: {:?} error: {:?}",
                    self.champion_name, e
                )
            })?;
        return Ok(movement_speed);
    }

    pub fn get_buffs(&mut self, process_handle: *mut c_void) -> Result<Vec<Buff>, String> {
        let start_time = Instant::now();
        let entry_array_begin_address: usize = read(process_handle, self.base_address + 0x27F8 + 0x18)
            .map_err(|_| "Failed to read entry_array_begin_address")?;
        let entry_array_end_address: usize = read(process_handle, self.base_address + 0x27F8 + 0x20)
            .map_err(|_| "Failed to read entry_array_end_address")?;

        let buffer_size = entry_array_end_address
            .checked_sub(entry_array_begin_address)
            .ok_or("Invalid buffer size")?;

        if buffer_size % 0x10 != 0 {
            return Err("Invalid buffer size: not a multiple of 0x10".to_string());
        }

        let buff_buffer = read_buffer(process_handle, entry_array_begin_address, buffer_size).unwrap();
        let mut buff_list = Vec::new();

        for i in 0..buffer_size / 0x10 {
            let buff_offset = 0x10 * i;

            let buff_address = extract_u64_from_buffer(&buff_buffer, buff_offset).unwrap() as usize;

            let info_address = read::<usize>(process_handle, buff_address + 0x10)
                .map_err(|_| format!("Failed to read info address at index {}", i))?;
            //let buff_duration = read::<u32>(process_handle, info_address + 0x1C).map_err(|_| "Failed to read buff duration")?;
            let name_address = match read::<usize>(process_handle, info_address + 0x8) {
                Ok(name_address) => name_address,
                Err(_) => continue,
            };

            let string: [c_char; 30] = read(process_handle, name_address).map_err(|_| "Failed to read string")?;

            // Ensure safe string conversion
            if let Ok(c_str) = unsafe { CStr::from_ptr(string.as_ptr()) }.to_str() {
                let buff = Buff {
                    name: c_str.to_string(),
                    duration: 0.0,
                };

                buff_list.push(buff);
            }
        }
        info!("Successfully read buffs\ntime it took:{:?}", start_time.elapsed());
        Ok(buff_list)
    }
}

#[derive(Debug, Clone)]
pub struct SummonerSpell {
    pub name: String,
    spell_address: usize,
}

impl SummonerSpell {
    pub fn new(
        spell_offset: usize,
        process_handle: *mut c_void,
        player_address: usize,
    ) -> Result<SummonerSpell, String> {
        let spell_slot_address = read::<usize>(
            process_handle,
            player_address + sdk::offsets::OBJ_SPELL_BOOK + spell_offset,
        )
        .unwrap();
        let spell_slot_info = read::<usize>(process_handle, spell_slot_address + 0x130).unwrap();
        let info_data = read::<usize>(process_handle, spell_slot_info + 0x60).unwrap();
        let name_pointer = read::<usize>(process_handle, info_data + 0x80).unwrap();
        let raw_spell_name = read::<[c_char; 16]>(process_handle, name_pointer)
            .map_err(|e| format!("Error reading raw spellname for {:?}: {:?}", spell_offset, e))?;

        let c_str = unsafe { CStr::from_ptr(raw_spell_name.as_ptr()) };
        let spell_name = c_str
            .to_str()
            .map_err(|err| format!("Error converting CStr to String: {:?}", err))?
            .to_string();
        // Haste, heal, exhaust smite, cleanse = boost
        let mut formatted_spell_name = spell_name[8..].to_string();
        match spell_name.as_str() {
            "S12_SummonerTele" => formatted_spell_name = "Teleport".to_string(),
            "SummonerHaste" => formatted_spell_name = "Ghost".to_string(),
            "SummonerBoost" => formatted_spell_name = "Cleanse".to_string(),
            _ => (),
        }

        //println!("name{}", formatted_spell_name);
        Ok(Self {
            name: formatted_spell_name,
            spell_address: spell_slot_address,
        })
    }
    pub fn get_cd(&self, process_handle: *mut c_void) -> Result<f32, String> {
        let spell_cd = read::<f32>(
            process_handle,
            self.spell_address + sdk::offsets::OBJ_SPELL_BOOK_COOLDOWN,
        )
        .map_err(|e| format!("Error reading summ spell cd for {:?}: {:?}", self.name, e))?;
        return Ok(spell_cd);
    }
}

#[derive(Debug, Clone, Copy)]
pub struct AbilityManager {
    pub q: Ability,
    pub w: Ability,
    pub e: Ability,
    pub r: Ability,
    //pub d: SummonerSpell,
    //pub f: SummonerSpell,
}

impl AbilityManager {
    pub fn new(
        process_handle: *mut c_void,
        player_base_address: usize,
        champion_db: toml::Value,
        champion_name: String,
    ) -> Result<Self, String> {
        let formatted_champion_name = champion_name.to_lowercase().replace(" ", "").replace("'", "");

        let q = Ability::new(
            'q',
            &champion_db,
            &formatted_champion_name,
            player_base_address,
            process_handle,
        )?;
        let w = Ability::new(
            'w',
            &champion_db,
            &formatted_champion_name,
            player_base_address,
            process_handle,
        )?;
        let e = Ability::new(
            'e',
            &champion_db,
            &formatted_champion_name,
            player_base_address,
            process_handle,
        )?;
        let r = Ability::new(
            'r',
            &champion_db,
            &formatted_champion_name,
            player_base_address,
            process_handle,
        )?;
        // let d_spell_slot_offset = 0x20;
        //let f_spell_slot_offset = 0x28;

        //let d = SummonerSpell::new(d_spell_slot_offset, process_handle, player_base_address)?;
        //let f = SummonerSpell::new(f_spell_slot_offset, process_handle, player_base_address)?;
        Ok(Self { q, w, e, r }) //d, f })
    }

    pub fn update(&mut self, process_handle: *mut c_void) {
        self.q.update_info(process_handle).unwrap();
        self.w.update_info(process_handle).unwrap();
        self.e.update_info(process_handle).unwrap();
        self.r.update_info(process_handle).unwrap();
    }
    pub fn abilities_mut(&mut self) -> [&mut Ability; 4] {
        [&mut self.q, &mut self.w, &mut self.e, &mut self.r]
    }

    pub fn abilities(&self) -> [&Ability; 4] {
        [&self.q, &self.w, &self.e, &self.r]
    }
}

#[derive(Debug, Clone)]
pub struct Buff {
    pub name: String,
    pub duration: f32,
}
#[derive(Debug, Clone)]
pub struct ActiveSpell<'a> {
    pub ability: &'a Ability,
    pub key: char,
    pub src_id: u32,
    pub target_id: u32,
    pub start_pos: Vector3<f32>, // Player object + Active spell offset. If no active spell this is 0. If active then this is a pointer.
    pub end_pos: Vector3<f32>,
}

#[derive(Debug, Clone, PartialEq, Copy)]
pub enum Skillshot {
    False,
    Linear {
        speed: f32,
        width: f32,
        range: f32,
        dodge: bool,
    },
    Circular {
        radius: f32,
        dodge: bool,
    },
    Trap {
        radius: f32,
        duration: f32,
        dodge: bool,
    },
    StationaryRectangle {
        length: f32,
        width: f32,
        duration: f32,
        dodge: bool,
    },
    DirectionBasedLinear {
        parent_player_address: usize,
        length: f32,
        speed: f32,
        width: f32,
        dodge: bool,
    },
}

pub fn get_predict_info_from_ai(
    ai_address: usize,
) -> (
    Vector3<f32>,
    Vector3<f32>,
    Vector3<f32>,
    Vector3<f32>,
    u32,
    f32,
    bool,
    bool,
    f32,
    //Vector3<f32>,
    //Vector3<f32>,
    //bool,
    //Vector3<f32>,
    //u32,
) {
    let buffer = read::<[u8; 0x420]>(unsafe { PROC_HANDLE.unwrap() }, ai_address).unwrap();

    // Extract values from buffer
    let is_moving = extract_bool_from_buffer(&buffer, sdk::offsets::OBJ_AI_MANAGER_IS_MOVING).unwrap();

    let current_position = extract_vector3_from_buffer(&buffer, sdk::offsets::OBJ_AI_MANAGER_CURRENT_POSITION).unwrap();
    let is_dashing = extract_bool_from_buffer(&buffer, sdk::offsets::OBJ_AI_MANAGER_IS_DASHING).unwrap();

    let start_path = extract_vector3_from_buffer(&buffer, sdk::offsets::OBJ_AI_MANAGER_START_PATH).unwrap();
    let end_path = extract_vector3_from_buffer(&buffer, sdk::offsets::OBJ_AI_MANAGER_END_PATH).unwrap();
    let target_position = extract_vector3_from_buffer(&buffer, sdk::offsets::OBJ_AI_MANAGER_TARGET_POSITION).unwrap();
    let dash_speed = extract_f32_from_buffer(&buffer, sdk::offsets::OBJ_AI_MANAGER_DASH_SPEED).unwrap();

    let movement_speed = extract_f32_from_buffer(&buffer, sdk::offsets::OBJ_AI_MANAGER_MOVEMENT_SPEED).unwrap();
    //let current_path_segment = extract_vector3_from_buffer(&buffer, sdk::offsets::OBJ_AI_MANAGER_CURRENT_PATH_SEGMENT).unwrap();;
    let path_segments_count =
        extract_u32_from_buffer(&buffer, sdk::offsets::OBJ_AI_MANAGER_PATH_SEGMENTS_COUNT).unwrap();

    (
        current_position,
        target_position,
        start_path,
        end_path,
        path_segments_count,
        movement_speed,
        is_moving,
        is_dashing,
        dash_speed,
    )
}

pub mod player_functions {
    use external::read;
    use std::os::raw::c_void;

    use crate::sdk::{self, offsets};

    pub fn is_casting(player_address: usize, process_handle: *mut c_void) -> bool {
        let active_spell = read::<usize>(process_handle, player_address + sdk::offsets::OBJ_ACTIVE_SPELL).unwrap();
        return active_spell != 0;
    }
    pub fn get_position(player_address: usize, process_handle: *mut c_void) -> nc::na::Point3<f32> {
        let position =
            read::<nc::na::Point3<f32>>(process_handle, player_address + sdk::offsets::OBJ_POSITION).unwrap();
        return position;
    }
    pub fn is_autoattacking(player_address: usize, process_handle: *mut c_void) -> bool {
        let active_spell = read::<usize>(process_handle, player_address + sdk::offsets::OBJ_ACTIVE_SPELL).unwrap();
        let active_spell_s = read::<usize>(process_handle, player_address + sdk::offsets::OBJ_ACTIVE_SPELL).unwrap();
        return active_spell != 0;
    }

    pub fn get_level(player_address: usize, process_handle: *mut c_void) -> f32 {
        let level = read::<f32>(process_handle, player_address + sdk::offsets::OBJ_LEVEL).unwrap();
        return level;
    }
    pub fn get_health(player_address: usize, process_handle: *mut c_void) -> f32 {
        let heath = read::<f32>(process_handle, player_address + sdk::offsets::OBJ_HEALTH).unwrap();
        return heath;
    }
    pub fn get_ad(player_address: usize, process_handle: *mut c_void) -> f32 {
        let base_ad = read::<f32>(process_handle, player_address + sdk::offsets::OBJ_BASE_ATTACK_DAMAGE).unwrap();
        let bonus_ad = read::<f32>(process_handle, player_address + sdk::offsets::OBJ_BONUS_ATTACK_DAMAGE).unwrap();
        return base_ad + bonus_ad;
    }
    pub fn get_armor_pen_percent(player_address: usize, process_handle: *mut c_void) -> f32 {
        let armor_pen = read::<f32>(process_handle, player_address + sdk::offsets::OBJ_PERCENT_ARMOR_PEN).unwrap();
        return armor_pen;
    }
    pub fn get_lethality(player_address: usize, process_handle: *mut c_void) -> f32 {
        let lethality = read::<f32>(process_handle, player_address + sdk::offsets::OBJ_LETHALITY).unwrap();
        return lethality;
    }
    pub fn get_armor(player_address: usize, process_handle: *mut c_void) -> f32 {
        let armor = read::<f32>(process_handle, player_address + sdk::offsets::OBJ_ARMOR).unwrap();
        return armor;
    }
    pub fn get_bonus_armor(player_address: usize, process_handle: *mut c_void) -> f32 {
        let bonus_armor = read::<f32>(process_handle, player_address + sdk::offsets::OBJ_BONUS_ARMOR).unwrap();
        return bonus_armor;
    }
}
