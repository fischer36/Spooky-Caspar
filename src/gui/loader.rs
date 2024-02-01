use egui_backend::egui::FullOutput;

use egui_backend::{egui, gl, sdl2};
use egui_backend::{sdl2::event::Event, DpiScaling, ShaderVersion};
use std::time::Instant;
// Alias the backend to something less mouthful
use egui_sdl2_gl as egui_backend;
use sdl2::video::SwapInterval;

use crate::{GAME_VERSION, SPOOKY_CASPAR_BUILD};
// Make the l0ader check for active match through: https://127.0.0.1:2999/liveclientdata/allgamedata
fn start_loader() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("Spooky Caspar", 400, 250)
        .opengl()
        .build()
        .unwrap();

    // Create a window context
    let _ctx = window.gl_create_context().unwrap();
    let shader_ver = ShaderVersion::Default;

    let (mut painter, mut egui_state) = egui_backend::with_sdl2(&window, shader_ver, DpiScaling::Default);
    let egui_ctx = egui::Context::default();
    let mut event_pump = sdl_context.event_pump().unwrap();

    let mut quit = false;
    let start_time = Instant::now();

    'running: loop {
        window.subsystem().gl_set_swap_interval(SwapInterval::VSync).unwrap();

        egui_state.input.time = Some(start_time.elapsed().as_secs_f64());
        egui_ctx.begin_frame(egui_state.input.take());

        egui::CentralPanel::default().show(&egui_ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading(&format!("Spooky Caspar Version: {}", SPOOKY_CASPAR_BUILD));

                ui.label(&format!("Game Version: {}", GAME_VERSION));
            });
            if ui.button("Check for match").clicked() {
                if quit {
                    ui.label(&format!("Active match not found: {}", GAME_VERSION));
                } else {
                    ui.label(&format!("Active match found: {}", GAME_VERSION));
                }
            }
        });

        let FullOutput {
            platform_output,
            repaint_after,
            textures_delta,
            shapes,
        } = egui_ctx.end_frame();

        // Process ouput
        egui_state.process_output(&window, &platform_output);

        let paint_jobs = egui_ctx.tessellate(shapes);
        painter.paint_jobs(None, textures_delta, paint_jobs);
        window.gl_swap_window();

        if !repaint_after.is_zero() {
            if let Some(event) = event_pump.wait_event_timeout(5) {
                match event {
                    Event::Quit { .. } => break 'running,
                    _ => {
                        // Process input event
                        egui_state.process_input(&window, event, &mut painter);
                    }
                }
            }
        } else {
            for event in event_pump.poll_iter() {
                match event {
                    Event::Quit { .. } => break 'running,
                    _ => {
                        // Process input event
                        egui_state.process_input(&window, event, &mut painter);
                    }
                }
            }
        }

        if quit {
            break;
        }
    }
}

fn check_for_active_match() -> bool {
    true
}

fn get_hwid() -> String {
    let id = hardware_id::get_id().unwrap();
    return id;
}
