// Game State and Global Information
// Updated for 14.6
pub const RENDERER: usize = 0x23AD258;
pub const RENDERER_WIDTH: usize = 0xC;
pub const RENDERER_HEIGHT: usize = 0x10;
pub const MINIMAP_OBJECT: usize = 0x2343E40;
pub const MINIMAP_OBJECT_HUD: usize = 0x240;
pub const MINIMAP_OBJECT_HUD_POS: usize = 0x60;
pub const MINIMAP_OBJECT_HUD_SIZE: usize = 0x68;
pub const GAME_TIME_SECONDS: usize = 0x233E7E8;

pub const LOCAL_PLAYER: usize = 0x23524A8;  // Updated from third list
pub const VIEW_PROJECTION_MATRIX: usize = 0x23A4170;

pub const TURRET_LIST: usize = 0x2338490;  // Updated from third list
pub const INHIB_LIST: usize = 0x2342068;  // Not in the second or third lists, unchanged
pub const MINION_MANAGER: usize = 0x2249D20;  // Not in the second or third lists, unchanged
pub const MINION_MANAGER_LIST: usize = 0x8;  // Not in the second or third lists, unchanged

// Champion manager
pub const CHAMPION_MANAGER: usize = 0x232fac0;  // Not in the second or third lists, unchanged
pub const CHAMPION_MANAGER_LIST: usize = 0x8;  // Not in the second or third lists, unchanged

// These offsets are present in the following objects: players, minions, monsters.
pub const OBJ_IDX: usize = 0x10;  // Not in the second or third lists, unchanged
pub const OBJ_NAME: usize = 0x43D8;  // Updated from third list
pub const OBJ_POSITION: usize = 0x220;  // Updated from third list
pub const OBJ_IS_VISIBLE: usize = 0x348;  // Updated from third list
pub const OBJ_HEALTH: usize = 0x1088; // f32
pub const OBJ_MANA: usize = 0x370;
pub const OBJ_ATTACK_RANGE: usize = 0x16FC;
pub const OBJ_TEAM: usize = 0x3C; // u32 -> 100/200
pub const OBJ_MOVE_SPEED: usize = 0x16F4; // f32
pub const OBJ_BASE_ATTACK_DAMAGE: usize = 0x16B8;
pub const OBJ_BONUS_ATTACK_DAMAGE: usize = 0x1620;
pub const OBJ_LETHALITY: usize = 0x15E0;
pub const OBJ_PERCENT_ARMOR_PEN: usize = 0x1E28;

pub const OBJ_ARMOR: usize = 0x1954;
pub const OBJ_LEVEL: usize = 0x4E00;
pub const OBJ_BONUS_ARMOR: usize = 0x1958;
pub const OBJ_MAGIC_PEN: usize = 0x1834;
pub const OBJ_MAGIC_PEN_PERCENT: usize = 0x11C8;

pub const OBJ_BONUS_MAGIC_RESISTANCE: usize = 0x1960;
pub const OBJ_MAGIC_RESISTANCE: usize = 0x195C;

pub const OBJ_ABILITY_POWER: usize = 0x1898;
pub const OBJ_DIRECTION_X: usize = 0x28B;
pub const OBJ_DIRECTION_Z: usize = 0x28F;
pub const OBJ_MAGIC_RESIST_BONUS: usize = 0x1960;

pub const OBJ_BONUS_ATTACK_SPEED: usize = 0x1684;
pub const OBJ_BONUS_ATTACK_SPEED_RATIO: usize = 0x1668;

pub const OBJ_AI_MANAGER: usize = 0x4290;
pub const OBJ_AI_MANAGER_IS_MOVING: usize = 0x2DC;
pub const OBJ_AI_MANAGER_START_PATH: usize = 0x2F0;
pub const OBJ_AI_MANAGER_END_PATH: usize = 0x2FC;
pub const OBJ_AI_MANAGER_CURRENT_PATH_SEGMENT: usize = 0x2E0;
pub const OBJ_AI_MANAGER_PATH_SEGMENTS: usize = 0x370;
pub const OBJ_AI_MANAGER_PATH_SEGMENTS_COUNT: usize = 0x368;
pub const OBJ_AI_MANAGER_CURRENT_POSITION: usize = 0x308;
pub const OBJ_AI_MANAGER_IS_DASHING: usize = 0x324;
pub const OBJ_AI_MANAGER_DASH_SPEED: usize = 0x300;
pub const OBJ_AI_MANAGER_MOVEMENT_SPEED: usize = 0x2DC;
pub const OBJ_AI_MANAGER_TARGET_POSITION: usize = 0x308;


pub const OBJ_BUFF_MANAGER: usize = 0x27F8;
pub const OBJ_SPELL_BOOK: usize = 0x3C58;
pub const OBJ_SPELL_BOOK_SLOT: usize = 0x28;
pub const OBJ_SPELL_BOOK_INFO: usize = 0x130;
pub const OBJ_SPELL_BOOK_NAME: usize = 0x28;
pub const OBJ_SPELL_BOOK_LEVEL: usize = 0x28;
pub const OBJ_SPELL_BOOK_COOLDOWN: usize = 0x30;

pub const OBJ_ACTIVE_SPELL: usize = 0x3190;
pub const OBJ_ACTIVE_SPELL_SLOT: usize = 0x10;
pub const OBJ_ACTIVE_SPELL_SRC_ID: usize = 0x68;
pub const OBJ_ACTIVE_SPELL_AUTO_ATTACK: usize = 0x10;
pub const OBJ_ACTIVE_SPELL_TARGET_ID: usize = 0xC0;
pub const OBJ_ACTIVE_SPELL_START_POSITION: usize = 0xC0;
pub const OBJ_ACTIVE_SPELL_END_POSITION: usize = 0xCC;
pub const OBJ_ACTIVE_SPELL_START_TIME: usize = 0x15C;
pub const OBJ_ACTIVE_SPELL_END_TIME: usize = 0x4C0;
pub const OBJ_ACTIVE_SPELL_INFO: usize = 0x8;
pub const OBJ_ACTIVE_SPELL_INFO_NAME: usize = 0x28;


// Offset for missile manager for game
pub const MISSILE_MANAGER: usize = 0x2334EF0;
pub const MISSILE_MANAGER_LIST: usize = 0x8;
pub const MISSILE_MANAGER_COUNT: usize = 0x10;

// Offsets for every entry in the missile manager
pub const MISSILE_MANAGER_MISSILE_1: usize = 0x0;
pub const MISSILE_MANAGER_MISSILE_2: usize = 0x8;
pub const MISSILE_MANAGER_MISSILE_3: usize = 0x10;

pub const MISSILE_MANAGER_ENTRY_INFO: usize = 0x2E8;
pub const MISSILE_MANAGER_ENTRY_NAME: usize = 0xB0;
pub const MISSILE_MANAGER_ENTRY_SPEED: usize = 0x78;
pub const MISSILE_MANAGER_ENTRY_POSITION: usize = 0x38C;
pub const MISSILE_MANAGER_ENTRY_SRC_IDX: usize = 0x370;
pub const MISSILE_MANAGER_ENTRY_DEST_IDX: usize = 0x3C8;
pub const MISSILE_MANAGER_ENTRY_START_POSITION: usize = 0x38C;
pub const MISSILE_MANAGER_ENTRY_END_POSITION: usize = 0x398;

                                                             //pub const SPELL_INFO: usize = 0x2E8;
