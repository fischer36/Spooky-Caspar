
// UNUSED FOR NOW
struct Player {
    id: usize,
    address: usize,
    ai_address: usize,
    q: Option<Skillshot>,
    w: Option<Skillshot>,
    e: Option<Skillshot>,
    r: Option<Skillshot>,
}

impl Player {
    fn new(address: usize, offset: usize, process_handle: *mut c_void) -> Result<Self, String> {
        let attack_speed_windup_percent = champ_table.get("attack_speed_windup_percent")?;
        let id = memory::read<usize>(process_handle, champion_address, [offses::OBJ_IDX]);
        let q_spell = Skillshot::new(spell_address, &champion_name, &champion_database_parsed, 'Q');
        let w_spell = Skillshot::new(spell_address, &champion_name, &champion_database_parsed, 'W');
        let e_spell = Skillshot::new(spell_address, &champion_name, &champion_database_parsed, 'E');
        let r_spell = Skillshot::new(spell_address, &champion_name, &champion_database_parsed, 'R');

        Self {
            id: id,
            address: address,
            ai_address: get_ai_address(process_handle, address),
            q: q_spell,
            w: w_spell,
            e: e_spell,
            r: r_spell,
        }
    }
}


struct LocalPlayer {
    address: usize,
    ai_address: usize,
    hitbox_radius: f32,
    as_base: f32,
    as_bonus_ratio: f32,
    as_windup_percent: f32,
}

impl LocalPlayer {
    fn new(module_address: usize, process_handle: *mut c_void) -> Result<Self, String> {
        let address = memory::read::<usize>(process_handle, module_address, [offsets::LOCAL_PLAYER]);
        
        let champion_database_contents = fs::read_to_string("champion_database.toml")?;
        let champion_database_parsed = toml::from_str(&champion_database_contents)?;
        let champ_table = champion_database_parsed.get(champion_name)?;
        let radius = champ_table.get("radius")?.as_float()?;
        let attack_speed_base = champ_table.get("attack_speed_base")?;
        let attack_speed_ratio = champ_table.get("attack_speed_ratio")?;
        let attack_speed_windup_percent = champ_table.get("attack_speed_windup_percent")?;

        Self {
            address: address,
            ai_address: get_ai_address(process_handle, address),
            hitbox_radius: radius.as_float().unwrap(),
            as_base: attack_speed_base,
            as_bonus_ratio: attack_speed_ratio,
            as_windup_percent: attack_speed_windup_percent,
        }
    }
}

trait PlayerObject {
    fn champion_name(&self) -> String {

    }
}

// Spells in .toml are: missile, circular_aoe and boomerang.
// Known spells that implement flags:
// spells that come back to the player like a boomerang: ahri q, ekko q, akshan q, sivir q
// thresh q: end position/direction is not correct in active_spell but can be retrived through the missile manager.
// champions with form specific spells:
// nidalee q: "Javelin Toss" in human form, elise "Cocoon" in spider from,
// Also no current implemantation for charged spells and stacked spells.

enum Skillshot {
    // Linear missile starting from player position towards target position.
    Missile {
        address: u32,
        speed: f32,
        width: f32,
        length: f32,
        cast_time: f32,
    },
    // Spawned circular AOE, for example brand W.
    CircularAOE {
        address: u32,
        radius: f32,
        cast_time: f32,
    },
}

impl Skillshot {
    fn new(spell_address: usize, champion_name: &str, champion_data: &toml::Value, key: char) -> Option<Skillshot> {
        let spell_book = champion_data.get("spells")?;
        let spell_key = spell_book.get(&key.to_string())?;
        let skillshot_type = spell_key.get("skillshot_type")?;

        match skillshot_type {
            "missile" => {
                let skillshot_speed = spell_key.get("skillshot_speed").as_float()?.unwrap_or(1600.0);
                let skillshot_width = spell_key.get("skillshot_width").as_float()?.unwrap_or(200.0);
                let skillshot_length = spell_key.get("skillshot_range").as_float()?.unwrap_or(1300.0);
                let skillshot_cast_time = spell_key.get("skillshot_cast_time").as_float()?.unwrap_or(0.25);
                return Some(Skillshot::Missile {
                    address: spell_address,
                    speed: skillshot_speed,
                    width: skillshot_width,
                    length: skillshot_length,
                    cast_time: skillshot_cast_time,
                });
            }
            "circular_aoe" => {
                let skillshot_radius = spell_key.get("skillshot_radius").as_float()?.unwrap_or(250.0);
                let skillshot_cast_time = spell_key.get("skillshot_cast_time").as_float()?.unwrap_or(0.25);
                return Some(Skillshot::CircularAOE {
                    address: spell_address,
                    radius: skillshot_radius,
                    cast_time: skillshot_cast_time,
                });
            }
            _ => None,
        }
    }
}

pub fn get_ai_address(process_handle: *mut c_void, address: usize) -> usize {
    // https://www.unknowncheats.me/forum/league-of-legends/579576-ai-manager-decryption-x64.html

    let v1: *const u8 = ((address as usize) + offsets::OBJ_AI_MANAGER) as *const u8;
    let v3b = read::<u8>(process_handle, (v1 as usize) + 16).unwrap();
    let v7 = read::<u64>(process_handle, (v1 as usize) + (8 * v3b as u64 + 24) as usize).unwrap();
    let v5 = read::<u64>(process_handle, (v1 as usize) + 8).unwrap();
    let v7 = v7 ^ !v5;
    let return_val = read::<usize>(process_handle, v7 as usize + 16).unwrap();
    return return_val;
}

pub fn get_ai_address_2(process_handle: *mut c_void, address: usize) -> usize {
    // https://www.unknowncheats.me/forum/league-of-legends/579576-ai-manager-decryption-x64.html

    let v1: *const u8 = ((address as usize) + offsets::OBJ_AI_MANAGER) as *const u8;
    let v3b = read_memory::<u8>(process_handle, v1 as usize, vec![16]);
    let v7 = read_memory::<u64>(process_handle, v1 as usize, vec![8 * v3b as u64 + 24 as usize]);
    let v5 = read_memory::<u64>(process_handle, v1 as usize, vec![8]);
    let v7 = v7 ^ !v5;
    let ai_address = read_memory::<usize>(process_handle, v7 as usize, vec![16]);
    return ai_address;
}


