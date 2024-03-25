// TODO: Update offsets, remove not used ones.

// OFFSETS FOR PATCH 13.14
// Game State and Global Information
pub const RENDERER: usize = 0x228D1C0;
pub const RENDERER_WIDTH: usize = 0xC;
pub const RENDERER_HEIGHT: usize = 0x10;
pub const MINIMAP_OBJECT: usize = 0x2256E30;
pub const MINIMAP_OBJECT_HUD: usize = 0x290;
pub const MINIMAP_OBJECT_HUD_POS: usize = 0x60;
pub const MINIMAP_OBJECT_HUD_SIZE: usize = 0x68;
pub const GAME_TIME_SECONDS: usize = 0x2251518;

pub const LOCAL_PLAYER: usize = 0x2264528;
pub const VIEW_PROJECTION_MATRIX: usize = 0x22B0B90;

pub const TURRET_LIST: usize = 0x2250790;
pub const INHIB_LIST: usize = 0x2265740;
pub const MINION_MANAGER: usize = 0x2249D20;
pub const MINION_MANAGER_LIST: usize = 0x8; // Minions starts at 0x8

// Champion manager
pub const CHAMPION_MANAGER: usize = 0x2246C88;
pub const CHAMPION_MANAGER_LIST: usize = 0x8;

// These offsets are present in the following objects: players, minions, monsters.
pub const OBJ_IDX: usize = 0x10; // a unique ID
pub const OBJ_NAME: usize = 0x38A8; // c-string
pub const OBJ_POSITION: usize = 0x220; // 3 f32
pub const OBJ_IS_VISIBLE: usize = 0x340; // 1-byte bool
pub const OBJ_HEALTH: usize = 0x1088; // f32
pub const OBJ_MANA: usize = 0x370;
pub const OBJ_ATTACK_RANGE: usize = 0x16FC;
pub const OBJ_TEAM: usize = 0x3C; // u32 -> 100/200
pub const OBJ_MOVE_SPEED: usize = 0x16f4; // f32
pub const OBJ_BASE_ATTACK_DAMAGE: usize = 0x16B8;
pub const OBJ_BONUS_ATTACK_DAMAGE: usize = 0x1620;
pub const OBJ_LETHALITY: usize = 0x15E0;
pub const OBJ_PERCENT_ARMOR_PEN: usize = 0x1E28;

pub const OBJ_ARMOR: usize = 0x16DC;
pub const OBJ_LEVEL: usize = 0x40A0;
pub const OBJ_BONUS_ARMOR: usize = 0x16e0;
pub const OBJ_MAGIC_PEN: usize = 0x15C4;
pub const OBJ_MAGIC_PEN_PERCENT: usize = 0x15C8;

pub const OBJ_BONUS_MAGIC_RESISTANCE: usize = 0x16E8;
pub const OBJ_MAGIC_RESISTANCE: usize = 0x16E4;

pub const OBJ_ABILITY_POWER: usize = 0x1630;
pub const OBJ_DIRECTION_X: usize = 0x2190 + 0x88;
pub const OBJ_DIRECTION_Z: usize = 0x2190 + 0x48;
pub const OBJ_MAGIC_RESIST_BONUS: usize = 0x16E8;

pub const OBJ_BONUS_ATTACK_SPEED: usize = 0x1684;
pub const OBJ_BONUS_ATTACK_SPEED_RATIO: usize = 0x1668;

pub const OBJ_AI_MANAGER: usize = 0x3758;
pub const OBJ_AI_MANAGER_IS_MOVING: usize = 0x2BC;
pub const OBJ_AI_MANAGER_START_PATH: usize = 0x2D0;
pub const OBJ_AI_MANAGER_END_PATH: usize = 0x2DC;
pub const OBJ_AI_MANAGER_CURRENT_PATH_SEGMENT: usize = 0x2C0;
pub const OBJ_AI_MANAGER_PATH_SEGMENTS: usize = 0x2E8;
pub const OBJ_AI_MANAGER_PATH_SEGMENTS_COUNT: usize = 0x2F0;
pub const OBJ_AI_MANAGER_CURRENT_POSITION: usize = 0x414;
pub const OBJ_AI_MANAGER_IS_DASHING: usize = 0x324;
pub const OBJ_AI_MANAGER_DASH_SPEED: usize = 0x300;
pub const OBJ_AI_MANAGER_MOVEMENT_SPEED: usize = 0x2B8;
pub const OBJ_AI_MANAGER_TARGET_POSITION: usize = 0x14;

pub const OBJ_BUFF_MANAGER: usize = 0x27F8; // POINTER TO AN ARRAY OF POINTERS TO BUFFS
pub const OBJ_SPELL_BOOK: usize = 0x2A50; // spells are separated by 4 bytes starting here
pub const OBJ_SPELL_BOOK_SLOT: usize = 0x8; // 8 bytes * 6 = q,w,e,r,f,d
pub const OBJ_SPELL_BOOK_INFO: usize = 0x130; // IDK SUPPOSODELY A POINTER TO MORE INFO
pub const OBJ_SPELL_BOOK_NAME: usize = 0x80; // c string
pub const OBJ_SPELL_BOOK_LEVEL: usize = 0x28; // u32
pub const OBJ_SPELL_BOOK_COOLDOWN: usize = 0x30; // Gives seconds f32 where spell is available again. subtract game time from spell time to get remaining cooldown for spell.

pub const OBJ_ACTIVE_SPELL: usize = 0x11B8; // Player object + Active spell offset. If active then this is a pointer to the spell containing info otherwise this is 0.
pub const OBJ_ACTIVE_SPELL_SLOT: usize = 0x10; // 0=Q, 1=W, 2=E, 3=R
pub const OBJ_ACTIVE_SPELL_SRC_ID: usize = 0x90;
pub const OBJ_ACTIVE_SPELL_AUTO_ATTACK: usize = 0x10; // Will be 0 unless its an auto attack
pub const OBJ_ACTIVE_SPELL_TARGET_ID: usize = 0xE8;
pub const OBJ_ACTIVE_SPELL_START_POSITION: usize = 0xAC; // [f32, 3];
pub const OBJ_ACTIVE_SPELL_END_POSITION: usize = 0xB8;
pub const OBJ_ACTIVE_SPELL_START_TIME: usize = 0x188;
pub const OBJ_ACTIVE_SPELL_END_TIME: usize = 0x170;
pub const OBJ_ACTIVE_SPELL_INFO: usize = 0x8;
pub const OBJ_ACTIVE_SPELL_INFO_NAME: usize = 0x18;

// Offset for missile manager for game
pub const MISSILE_MANAGER: usize = 0x22654F8; // Get pointer from this
pub const MISSILE_MANAGER_LIST: usize = 0x8; // Add 0x8
pub const MISSILE_MANAGER_COUNT: usize = 0x10;

// Offsets for every entry in the missile manager
pub const MISSILE_MANAGER_MISSILE_1: usize = 0x0;
pub const MISSILE_MANAGER_MISSILE_2: usize = 0x8;
pub const MISSILE_MANAGER_MISSILE_3: usize = 0x10;

pub const MISSILE_MANAGER_ENTRY_INFO: usize = 0x2E8;
pub const MISSILE_MANAGER_ENTRY_NAME: usize = 0x60;
pub const MISSILE_MANAGER_ENTRY_SPEED: usize = 0x88;
pub const MISSILE_MANAGER_ENTRY_POSITION: usize = 0x104;
pub const MISSILE_MANAGER_ENTRY_SRC_IDX: usize = 0x370; // 4 bytes
pub const MISSILE_MANAGER_ENTRY_DEST_IDX: usize = 0x3E8; // 4 bytes
pub const MISSILE_MANAGER_ENTRY_START_POSITION: usize = 0x3A0; // [f32; 3]
pub const MISSILE_MANAGER_ENTRY_END_POSITION: usize = 0x3AC; // [f32; 3]
                                                             //pub const SPELL_INFO: usize = 0x2E8;
