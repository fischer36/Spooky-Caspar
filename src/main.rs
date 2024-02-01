extern crate external;
//extern crate nalgebra as na;
extern crate ncollide2d as nc;
extern crate serde_derive;
extern crate speedy2d;
extern crate windows_sys;

mod action;
mod gui;
mod io;
mod memory;
mod overlay;
mod sdk;
mod tests;
mod utils;
use action::evade::Spell;
use std::fs;
use std::os::raw::c_void;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::{Duration, Instant};
use windows_sys::Win32::UI::Input::KeyboardAndMouse::VK_ESCAPE;

use external::close_proc_handle;
use log::info;

use action::champions::Champion;
use serde_derive::{
    Deserialize, Serialize,
};
use speedy2d::color::Color;
use speedy2d::window::UserEventSender;

use windows_sys::Win32::UI::WindowsAndMessaging::{GetSystemMetrics, SM_CXSCREEN, SM_CYSCREEN};

use crate::sdk::game::match_functions::get_view_proj_matrix;
use crate::sdk::game::Game;
use gui::menu::{
    start_menu, GuiMessage,
};
use io::input::{
    check_window, remove_keyboard_hook,
    KbHookMessage,
};
use overlay::{
    overlay::*, window_manipulation,
    window_manipulation::*,
};
use sdk::game::match_functions;
use sdk::minion_manager::*;
use sdk::offsets;
use sdk::player_manager::{
    ActiveSpell, Player,
};
use tests::*;
use utils::*;

pub const GAME_VERSION: f32 = 15.0;
pub const SPOOKY_CASPAR_BUILD: f32 =
    0.1;

// Structs, Enums, Types, Functions etc. (if any) go here
// ...

pub const PROCESS_NAME: &str =
    "League of Legends.exe";
pub const PROCESS_WINDOW_NAME: &str =
    "League Of Legends (TM) Client";
pub enum MainLoopThreadCommunication {
    KbHookThread(KbHookMessage),
    GuiThread(GuiMessage),
    Exit(Option<String>),
}

pub struct Flags {
    is_space_down: bool,
    is_q_down: bool,
    is_w_down: bool,
    is_e_down: bool,
    is_r_down: bool,
    show_gui: bool,
}

pub struct GameTasks {
    active_spell_list: Vec<Spell>,
    active_spell_cooldown_list:
        Vec<(usize, Instant)>,
    evading_too:
        (nc::na::Point2<f32>, Instant),
    auto_attack_cooldown: Instant,
    casting_spell_cooldown: Instant,

    move_cooldown: Instant,
    target_mode: TargetMode,
}

pub struct Visuals {
    render_list: Vec<RenderObject>,
    color: speedy2d::color::Color,
    draw_evade_path: bool,
    draw_user_attack_range: bool,
    draw_enemies: bool,
    draw_projectiles: bool,
}

pub struct Modules {
    main_cheat: bool,
    hud: bool,
    offense: bool,
    evade: bool,
    visuals: bool,
}

#[derive(
    Deserialize, Debug, Serialize,
)]
struct Keybinds {
    combo_key: String,
    test: String,
    test2: String,
}
#[derive(
    Deserialize, Debug, Serialize,
)]
struct Config {
    keybinds: Keybinds,
}

static mut GUI_HWND: isize = 00;
static mut OVERLAY_COLOR:
    speedy2d::color::Color =
    speedy2d::color::Color::from_rgba(
        0.67, 0.67, 0.67, 0.5,
    );
fn main() {
    // TODO: OFFENSE: WALK WHEN CANT OFFENSE, ADD DO NOTHING FIELD FOR XERATH R STANDS TILL, MUTUAL INTERFACE FOR CHECKING WHAT SPELL IS READY.
    // TODO: GUI: ICON, SETTINGS, OFFENSIVE OPTIONS,

    // Logging for debug
    env_logger::init();

    // User config
    //let config = load_config("./config.toml");

    // Thread communication to main loop thread
    let (
        main_loop_sender,
        main_loop_receiver,
    ) = mpsc::channel();

    // Key listener
    start_keyboard_hook_thread(
        main_loop_sender.clone(),
    );

    // Get overlay window handler so we can get and utilize the sender
    let overlay_window =
        start_overlay();
    let overlay_event_sender =
        overlay_window
            .create_user_event_sender();

    // Main game loop
    game_loop(
        main_loop_receiver,
        main_loop_sender,
        overlay_event_sender.clone(),
    );

    // Start overlay
    overlay_window.run_loop(
        MyWindowHandler::new(
            overlay_event_sender,
        ),
    );
}

fn game_loop(
    main_loop_reciver: Receiver<
        MainLoopThreadCommunication,
    >,
    main_loop_sender: Sender<
        MainLoopThreadCommunication,
    >,
    overlay_sender: UserEventSender<
        Option<Vec<RenderObject>>,
    >,
) {
    let screen_width = unsafe {
        GetSystemMetrics(SM_CXSCREEN)
    };
    let screen_height = unsafe {
        GetSystemMetrics(SM_CYSCREEN)
    };
    std::thread::spawn(move || {
        let mut game = Game::new(
            nc::na::Vector2::new(
                screen_width as f32,
                screen_height as f32,
            ),
        )
        .unwrap();
        unsafe {
            PROC_HANDLE = Some(
                game.process_handle,
            );
        }
        let radius = game
            .local_player
            .get_attack_range(
                game.process_handle,
            )
            .unwrap();
        let view_proj =
            get_view_proj_matrix(
                game.module_address,
                game.process_handle,
            );
        let mut points = Vec::new();
        let center = game
            .world_to_screen(
                &nc::na::Point3::new(
                    game.local_player
                        .position
                        .x,
                    game.local_player
                        .position
                        .y,
                    game.local_player
                        .position
                        .z,
                ),
            );
        println!("Center{:?}", center);
        // Calculate the positions of the octagon's vertices in world space
        for i in 0..8 {
            let angle = 2.0
                * std::f32::consts::PI
                / 8.0
                * i as f32; // 16 sides for an octagon
            let x = game
                .local_player
                .position
                .x
                + radius * angle.cos();
            let y = game
                .local_player
                .position
                .z
                + radius * angle.sin();
            let world_point =
                nc::na::Point3::new(
                    x, 1.0, y,
                ); // Assuming the center is at (0,0) in world space

            // Transform the world space point to screen space
            let screen_point = game
                .world_to_screen(
                    &world_point,
                );
            println!(
                "{:?}",
                screen_point
            );
            // Store the screen space point
            points.push(screen_point);
        }
        // println!("{:?}", points);
        let mut champion_module =
            action::get_champion_mod(
                &game
                    .local_player
                    .champion_name,
            );

        let mut flags = Flags {
            is_space_down: false,
            is_q_down: false,
            is_w_down: false,
            is_e_down: false,
            is_r_down: false,
            show_gui: true,
        };
        let mut game_tasks = GameTasks {
            active_spell_list: Vec::new(),
            active_spell_cooldown_list: Vec::new(),
            evading_too: (point2!(0.0, 0.0), Instant::now()),
            auto_attack_cooldown: Instant::now(),
            casting_spell_cooldown: Instant::now(),
            move_cooldown: Instant::now(),

            target_mode: TargetMode::LowestDistance,
        };
        let mut visuals = Visuals {
            render_list: Vec::new(),
            color: unsafe {
                OVERLAY_COLOR
            },
            draw_enemies: true,
            draw_evade_path: true,
            draw_projectiles: true,
            draw_user_attack_range:
                true,
        };
        let mut modules = Modules {
            main_cheat: true,
            hud: true,
            offense: true,
            evade: true,
            visuals: true,
        };
        std::thread::spawn(move || {
            start_menu(
                main_loop_sender,
            );
        });
        std::thread::sleep(std::time::Duration::from_secs(1)); // For the gui to catch up
        unsafe {
            GUI_HWND = get_hwnd(
                "Spooky Caspar"
                    .to_string(),
            )
            .unwrap()
        };
        window_manipulation::set_window_always_on_top(unsafe { GUI_HWND } as *mut c_void);

        info!("Entering main loop");
        loop {
            process_thread_receive(
                &main_loop_reciver,
                &mut game,
                &mut modules,
                &mut flags,
                &mut game_tasks,
                &mut visuals,
            );
            if !modules.main_cheat
                || !unsafe {
                    check_window()
                }
            {
                overlay_sender.send_event(None).expect("error sending event to ovelray");
                std::thread::sleep(std::time::Duration::from_secs(1));
                continue;
            }

            update_cheat_instance(&mut game, &mut modules, &mut flags, &mut game_tasks, &mut visuals)
                .expect("error updating cheat instance");
            iterate_players(
                &mut game,
                &mut modules,
                &mut flags,
                &mut game_tasks,
                &mut visuals,
                &mut champion_module,
            );

            // If any visual module is enabled send to overlay thread in form of a vec containing RenderObjects
            if modules.visuals {
                if visuals
                    .render_list
                    .is_empty()
                {
                    overlay_sender.send_event(None).expect("error sending None to overlay");
                } else {
                    overlay_sender
                        .send_event(Some(visuals.render_list.clone()))
                        .expect("error sending render list to overlay");
                }
            }

            std::thread::sleep(std::time::Duration::from_millis(15));
        }
    });
}

fn iterate_players(
    game: &mut Game,
    modules: &mut Modules,
    flags: &mut Flags,
    game_tasks: &mut GameTasks,
    visuals: &mut Visuals,
    champion_module: &Box<dyn Champion>,
) {
    for player in game
        .player_manager
        .enemy_obj_list
        .iter()
    {
        if player.health < 1.0
            || !player.is_visible
        {
            continue;
        }
        if visuals.draw_enemies {
            render_player(
                game, player, visuals,
            );
        }
        if let Ok(active_spell) = player
            .get_active_spell(
                game.process_handle,
            )
        {
            handle_active_spell(
                &active_spell,
                game,
                modules,
                flags,
                game_tasks,
                visuals,
                champion_module,
            );
        }
    }
    if visuals.draw_user_attack_range {
        render_attack_range(
            game, visuals,
        )
    };

    if modules.offense
        && game.local_player.health
            > 1.0
        && game_tasks.evading_too.1
            < Instant::now()
    {
        let mut color =
            Color::from_rgba(
                0.67, 0.67, 0.67, 1.0,
            );

        if flags.is_space_down {
            color = Color::from_rgba(
                1.0, 1.0, 1.0, 1.0,
            );
            champion_module
                .offense_tick(
                    game, game_tasks,
                );
            //combo_protocol(game, cheat_flags);
        } else {
            for player in game
                .player_manager
                .enemy_obj_list
                .iter_mut()
            {
                if player.target {
                    player.target =
                        false;
                    break; // Exit the loop after modifying the first player found with target == true
                }
            }
        }
        if modules.hud {
            visuals.render_list.push(RenderObject::Text(RenderText {
                color: color,
                name: format!("Orbwalker [SPACE]"),
            }));
        }
    }
    if modules.evade && modules.hud {
        let mut color =
            Color::from_rgba(
                0.67, 0.67, 0.67, 1.0,
            );
        if game_tasks.evading_too.1
            > Instant::now()
        {
            color = Color::from_rgba(
                1.0, 1.0, 1.0, 1.0,
            );
        }
        visuals.render_list.push(
            RenderObject::Text(
                RenderText {
                    color: color,
                    name: format!(
                        "Evade"
                    ),
                },
            ),
        );
    }
    if modules.visuals && modules.hud {
        visuals.render_list.push(RenderObject::Text(RenderText {
            color: Color::from_rgba(0.67, 0.67, 0.67, 1.0),
            name: format!("Visuals"),
        }));
    }
}

fn load_config(path: &str) -> Config {
    let config_text =
        fs::read_to_string(path)
            .expect(
            "Error reading config file",
        );
    toml::from_str(&config_text).expect(
        "Error parsing config file",
    )
}

fn render_evade_path(
    game: &Game,
    visuals: &mut Visuals,
    point: &nc::na::Point2<f32>,
) {
    if let Some(screen_player_pos) =
        game.world_to_screen(
            &nc::na::Point3::new(
                game.local_player
                    .position
                    .x,
                game.local_player
                    .position
                    .y,
                game.local_player
                    .position
                    .z,
            ),
        )
    {
        if let Some(screen_evade_pos) =
            game.world_to_screen(
                &nc::na::Point3::new(
                    point.x, 0.0,
                    point.y,
                ),
            )
        {
            visuals.render_list.push(RenderObject::Arrow {
                start_pos: screen_player_pos,
                end_pos: screen_evade_pos,
                color: visuals.color,
            });
        }
    }
}
fn render_attack_range(
    game: &Game,
    visuals: &mut Visuals,
) {
    if let Some(screen_player) = game
        .world_to_screen(
            &nc::na::Point3::new(
                game.local_player
                    .position
                    .x,
                game.local_player
                    .position
                    .y,
                game.local_player
                    .position
                    .z,
            ),
        )
    {
        let render_attack_range = RenderObject::AttackRange(AutoAttackRange {
            name: "local player".to_string(),
            radius: game
                .local_player
                .get_attack_range(game.process_handle)
                .expect("Error getting attack range"),
            pos: point2!(screen_player.x, screen_player.y),
            color: visuals.color.clone(),
        });
        visuals
            .render_list
            .push(render_attack_range);
    }
}

fn render_player(
    game: &Game,
    player: &Player,
    visuals: &mut Visuals,
) {
    if let Some(screen_player) = game
        .world_to_screen(
            &nc::na::Point3::new(
                player.position.x,
                player.position.y,
                player.position.z,
            ),
        )
    {
        let render_player = RenderObject::Entity(RenderEntity {
            name: player.champion_name.clone(),
            pos: point2!(screen_player.x, screen_player.y),
            radius: player.gameplay_radius,
            color: visuals.color.clone(),
            target: player.target,
        });
        visuals
            .render_list
            .push(render_player);
    }
}
static mut PROC_HANDLE: Option<
    *mut c_void,
> = None;
fn update_cheat_instance(
    game: &mut Game,
    modules: &mut Modules,
    flags: &mut Flags,
    game_tasks: &mut GameTasks,
    visuals: &mut Visuals,
) -> Result<String, String> {
    let current_time = Instant::now();

    // Update local player struct
    game.local_player.update_info(
        game.process_handle,
    )?;

    // Update local player abilities not included in update player struct as I dont need to update enemies abilities for now
    game.local_player
        .ability_manager
        .update(game.process_handle);

    // Update all enemy players
    game.player_manager
        .update_list_enemy_info(
            game.process_handle,
        )?;

    // Update current minion list
    // game.minion_manager.update_list(game.process_handle)?;

    // // Refresh minion list every 25 secs
    // if current_time.duration_since(game.last_minion_refresh).as_secs() >= 25 {
    //     // It's been more than 25 seconds since the last full update
    //     game.last_minion_refresh = current_time;
    //     game.minion_manager.refresh_list(game.process_handle, game.module_address);
    // }
    // Set current time

    game_tasks
        .active_spell_cooldown_list
        .retain(|spell_cd| {
            spell_cd.1.elapsed()
                < Duration::from_millis(
                    600,
                )
        });
    //println!("{:?}", game_tasks.active_spell_list.iter());
    // Remove expired projectiles from projectile list

    game_tasks.active_spell_list.retain(|spell| match spell {
        Spell::RectangleProjectile { expires_at, .. }
        | Spell::Circular { expires_at, .. }
        | Spell::Other { expires_at, .. } => *expires_at > current_time,
    });

    // Clear rendering list as this updates every tick
    visuals.render_list.clear();
    for spell in game_tasks
        .active_spell_list
        .iter()
    {
        if let Some(ob) = spell.try_render_and_get_screen_pos(game, visuals) {
            visuals.render_list.push(ob)
        }
    }

    if visuals.draw_evade_path
        && game_tasks.evading_too.1
            > current_time
    {
        render_evade_path(
            game,
            visuals,
            &game_tasks.evading_too.0,
        )
    }
    return Ok(
        "updated instance".to_string()
    );
}

fn start_keyboard_hook_thread(
    sender: Sender<
        MainLoopThreadCommunication,
    >,
) -> JoinHandle<()> {
    let hook_set_flag = Arc::new(
        AtomicBool::new(false),
    ); // So that it does not try to unhook a non exsisting hook/ hook an already existing hook.-
    let hook_set_flag_for_panic =
        Arc::clone(&hook_set_flag); // Clone for panic hook

    // Custom panic hook that deallocates resources if a panic occurs
    std::panic::set_hook(Box::new(
        move |_info| {
            println!(
                "Panic hook called"
            );
            if let Some(proc_handle) =
                unsafe { PROC_HANDLE }
            {
                close_proc_handle(
                    proc_handle,
                );
                println!("Process handle dropped");
            }
            if hook_set_flag_for_panic
                .load(Ordering::SeqCst)
            {
                io::input::remove_keyboard_hook();
                println!("Keyboard hook unhooked");
            }
            std::process::exit(0);
        },
    ));

    // Now spawn the thread.
    let handle = std::thread::spawn(
        move || {
            io::input::set_keyboard_hook_and_run_message_loop(hook_set_flag, sender);
        },
    );
    handle
}

fn process_thread_receive(
    keyboard_receiver: &Receiver<
        MainLoopThreadCommunication,
    >,
    game: &mut Game,
    modules: &mut Modules,
    flags: &mut Flags,
    game_tasks: &mut GameTasks,
    visuals: &mut Visuals,
) {
    if let Ok(recived_msg) =
        keyboard_receiver.try_recv()
    {
        match recived_msg {
            MainLoopThreadCommunication::Exit(optional_msg) => {
                if let Some(msg) = optional_msg {
                    println!("{}", msg);
                }
                exit(game.process_handle);
            }
            MainLoopThreadCommunication::KbHookThread(kbhook_msg) => match kbhook_msg {
                KbHookMessage::VKPress(vkkey) => match vkkey {
                    _ => (),
                },
                KbHookMessage::Keypress(key) => match key {
                    'u' => {
                        modules.main_cheat = !modules.main_cheat;
                    }
                    'm' => {
                        flags.show_gui = !flags.show_gui;
                        if flags.show_gui {
                            show_window(unsafe { GUI_HWND } as *mut c_void)
                        } else {
                            hide_window(unsafe { GUI_HWND } as *mut c_void)
                        }
                    }

                    _ => println!("Unknown key: {}", key),
                },
                KbHookMessage::SpaceIsDown(is_down) => flags.is_space_down = is_down,
                KbHookMessage::QIsDown(is_down) => flags.is_q_down = is_down,
                KbHookMessage::WIsDown(is_down) => flags.is_w_down = is_down,
                KbHookMessage::EIsDown(is_down) => flags.is_e_down = is_down,
                KbHookMessage::RIsDown(is_down) => flags.is_r_down = is_down,
            },
            MainLoopThreadCommunication::GuiThread(gui_msg) => {
                match gui_msg {
                    GuiMessage::Hud(is_enabled) => modules.hud = is_enabled,
                    GuiMessage::Evade(is_enabled) => modules.evade = is_enabled,
                    GuiMessage::Offensive(is_enabled) => modules.offense = is_enabled,
                    GuiMessage::OverlayColor(color32) => {
                        visuals.color =
                            speedy2d::color::Color::from_int_rgba(color32[0], color32[1], color32[2], color32[3]);
                    }
                    GuiMessage::OverlayEvadePath(is_enabled) => visuals.draw_evade_path = is_enabled,
                    GuiMessage::OverlayEnemy(is_enabled) => visuals.draw_enemies = is_enabled,
                    GuiMessage::OverlayProjectiles(is_enabled) => visuals.draw_projectiles = is_enabled,
                    GuiMessage::OverlayAttackRange(is_enabled) => visuals.draw_user_attack_range = is_enabled,
                    GuiMessage::ChangeTargetMode(target_mode) => game_tasks.target_mode = target_mode,
                };
                if visuals.draw_projectiles
                    || visuals.draw_evade_path
                    || visuals.draw_enemies
                    || visuals.draw_user_attack_range
                {
                    modules.visuals = true
                } else {
                    modules.visuals = false
                }
            }
        }
    }
}

fn exit(proccess_handle: *mut c_void) {
    remove_keyboard_hook();
    external::close_proc_handle(
        proccess_handle,
    );
    std::process::exit(0);
}

static mut spellctr: i32 = 0;
fn handle_active_spell(
    active_spell: &ActiveSpell,
    game: &Game,
    modules: &mut Modules,
    flags: &mut Flags,
    game_tasks: &mut GameTasks,
    visuals: &mut Visuals,
    champion_module: &Box<dyn Champion>,
) {
    let start_time = Instant::now();

    if game_tasks
        .active_spell_cooldown_list
        .iter()
        .any(|spell| {
            spell.0
                == active_spell
                    .ability
                    .base_address
        })
    {
        //println!("already dealt wht");
        // Spell is already dealt with
        return;
    }

    unsafe { spellctr += 1 };
    println!("{}", unsafe { spellctr });
    game_tasks
        .active_spell_cooldown_list
        .push((
            active_spell
                .ability
                .base_address,
            Instant::now(),
        ));
    let spell =
        Spell::new(active_spell, game);

    //if game_tasks.evading_too.1 < Instant::now() {
    if modules.evade {
        action::evade::to_evade_or_not_to_evade(game, game_tasks, &spell, &champion_module);
    }

    // }
    game_tasks
        .active_spell_list
        .push(spell);
    info!("Active spell handling time:{:?}", start_time.elapsed());
}
// enum RenderObject {
//     Player {
//         id: usize,
//         pos: Point2<f32>,
//         hitbox_radius: Option<f32>,
//         attack_range_radius: Option<f32>,
//         is_target: bool,
//     },
//     SpellProjectile {
//         id: usize,
//         start: Point2<f32>,
//         end: Point2<f32>,
//         width: f32,
//     },
//     SpellCircular {
//         id: usize,
//         pos: Point2<f32>,
//         radius: f32,
//     },
//     EvasionArrow {
//         start: Point2<f32>,
//         end: Point2<f32>,
//     },
//     Text {
//         text: String,
//     },
// }
// impl RenderObject {
//     fn new(game: &Game, active_spell: &ActiveSpell) -> Option<RenderObject> {
//         match active_spell.ability.skillshot {
//             Skillshot::Linear { speed, width, dodge } => {
//                 if let Some(screen_start) = game.world_to_screen(&active_spell.start_pos) {
//                     if let Some(screen_end) = game.world_to_screen(&active_spell.end_pos) {
//                         return Some(RenderObject::SpellProjectile {
//                             id: active_spell.ability.base_address,
//                             start: Point2::new(screen_start.0, screen_start.1),
//                             end: Point2::new(screen_end.0, screen_end.1),
//                             width: width,
//                         });
//                     }
//                 }
//             }
//             Skillshot::Circular { radius, dodge } => {
//                 if let Some(screen_end) = game.world_to_screen(&active_spell.end_pos) {
//                     return Some(RenderObject::SpellCircular {
//                         id: active_spell.ability.base_address,
//                         pos: Point2::new(screen_end.0, screen_end.1),
//                         radius: radius,
//                     });
//                 }
//             }
//             _ => return None,
//         }
//         return None;
//     }
// }
