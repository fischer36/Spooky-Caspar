use std::iter::zip;
use std::sync::mpsc::Sender;
use std::time::{Duration, Instant};

use egui_backend::egui::epaint::{QuadraticBezierShape, Shadow};
use egui_backend::egui::{
    Color32, CursorIcon, FontData, FontDefinitions, FontFamily, FontId, FullOutput, Response, RichText, Stroke,
    TextureHandle, Ui, Vec2, Window,
};
use egui_backend::sdl2::pixels::Color;
use egui_backend::{egui, sdl2};
use egui_backend::{sdl2::event::Event, DpiScaling, ShaderVersion};

use egui::widgets::Widget;
use egui_sdl2_gl as egui_backend;
use image::GenericImageView;
use sdl2::video::SwapInterval;

const SCREEN_SIZE: Pos2 = egui::pos2(2560.0, 1440.0);
const WINDOW_SIZE: Pos2 = egui::pos2(650.0, 650.0);

const WINDOW_POSITION: Pos2 = egui::pos2(
    (SCREEN_SIZE.x / 2.0) - (WINDOW_SIZE.x / 2.0),
    (SCREEN_SIZE.y / 2.0) - (WINDOW_SIZE.y / 2.0),
);

const CENTER: Pos2 = egui::pos2(WINDOW_SIZE.x / 2.0, WINDOW_SIZE.y / 2.0);

const OUTER_RADIUS: f32 = 290.0;
const INNER_RADIUS: f32 = OUTER_RADIUS / 1.8;
const TRANSPARENT_COLOR: (u8, u8, u8) = (8, 8, 8);

#[derive(Clone, PartialEq)]
enum MyWidget {
    Tab(MyTab),
    Checkbox(MyCheckbox),
    Back(MyTab),
    ColorPicker(MyColorPicker),
    OptionSelector(MyOptionSelector),
}
#[derive(Clone, PartialEq)]
struct MyColorPicker {
    name: String,
    description: String,
    is_open: bool,
    current_color: [f32; 4],
}

#[derive(Clone, PartialEq)]
struct MyCheckbox {
    name: String,
    description: String,
    enabled: bool,
}

#[derive(Clone, PartialEq)]
struct MyTab {
    widgets: Vec<MyWidget>,
    name: String,
    description: String,
    icon: Option<TextureHandle>,
}

#[derive(Clone, PartialEq)]
struct MyOptionSelector {
    name: String,
    options: Vec<String>,
    selected_index: usize,
    description: String,
}

pub struct ClientInfo {
    pub screen_width: f32,
    pub screen_height: f32,
    pub game_width: f32,
    pub game_height: f32,
    pub league_proccess_id: usize,
    pub enemy_count: usize,
}

pub enum GuiMessage {
    Hud(bool),
    Evade(bool),
    Offensive(bool),
    OverlayColor(Color32),
    OverlayEvadePath(bool),
    OverlayEnemy(bool),
    OverlayProjectiles(bool),
    OverlayAttackRange(bool),
    ChangeTargetMode(TargetMode),
}

pub fn start_menu(main_loop_sender: Sender<MainLoopThreadCommunication>) {
    let back_tab = MyWidget::Back(MyTab {
        widgets: Vec::new(),
        name: "[X] Back".to_string(),
        description: "Go back to main menu".to_string(),
        icon: None,
    });
    let mut tab_path = "".to_string();

    let mut offense_tab = MyTab {
        widgets: vec![
            MyWidget::Tab(MyTab {
                name: "Orbwalker".to_string(),
                description: "Orbwalker options".to_string(),
                widgets: vec![MyWidget::Checkbox(MyCheckbox {
                    name: "Enable".to_string(),
                    description: "Orbwalk when holding space".to_string(),
                    enabled: true,
                })],
                icon: None,
            }),
            MyWidget::Tab(MyTab {
                name: "Champ module".to_string(),
                description: "Champion module options".to_string(),
                widgets: Vec::new(),
                icon: None,
            }),
            MyWidget::OptionSelector(MyOptionSelector {
                name: "Target mode".to_string(),
                description: "Change focus priority".to_string(),
                options: vec!["Lowest Health".to_string(), "Lowest distance".to_string()],
                selected_index: 0,
            }),
        ],
        name: "Offense".to_string(),
        description: "Offensive options".to_string(),
        icon: None,
    };
    let mut evade_tab = MyTab {
        widgets: vec![MyWidget::Checkbox(MyCheckbox {
            name: "Enable".to_string(),
            description: "Dodge skillshots automaticlly".to_string(),
            enabled: true,
        })],
        name: "Evade".to_string(),
        description: "Evade options".to_string(),
        icon: None,
    };
    let mut visuals_tab = MyTab {
        icon: None,
        name: "Visuals".to_string(),
        description: "Visuals options".to_string(),
        widgets: vec![
            MyWidget::Checkbox(MyCheckbox {
                name: "Hud".to_string(),
                description: "Shows enabled modules".to_string(),
                enabled: true,
            }),
            MyWidget::Checkbox(MyCheckbox {
                name: "Evade path".to_string(),
                description: "Draws evasion path when evading".to_string(),
                enabled: true,
            }),
            MyWidget::Checkbox(MyCheckbox {
                name: "Projectiles".to_string(),
                description: "Draws projectiles".to_string(),
                enabled: true,
            }),
            MyWidget::Checkbox(MyCheckbox {
                name: "Attack range".to_string(),
                description: "Draws player attack range".to_string(),
                enabled: true,
            }),
            MyWidget::ColorPicker(MyColorPicker {
                name: "Visuals color".to_string(),
                description: "Change color for visuals".to_string(),
                current_color: [0.67, 0.67, 0.67, 0.5],
                is_open: false,
            }),
        ],
    };
    let mut misc_tab = MyTab {
        icon: None,
        name: "Misc".to_string(),
        description: "Misc modules".to_string(),
        widgets: vec![
            MyWidget::Checkbox(MyCheckbox {
                name: "Follow teamate".to_string(),
                description: "Follows chosen teamate".to_string(),
                enabled: true,
            }),
            MyWidget::OptionSelector(MyOptionSelector {
                name: "Teamate".to_string(),
                description: "Set which teamate to follow".to_string(),
                options: vec!["thejoker70".to_string(), "test123".to_string()],
                selected_index: 0,
            }),
        ],
    };
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem
        .window("SPOOKY CASPAR", WINDOW_SIZE.x as u32, WINDOW_SIZE.y as u32)
        .position(WINDOW_POSITION.x as i32, WINDOW_POSITION.y as i32)
        .opengl()
        .borderless()
        .build()
        .unwrap();

    let _ctx = window.gl_create_context().unwrap();
    let shader_ver = ShaderVersion::Default;
    let (mut painter, mut egui_state) = egui_backend::with_sdl2(&window, shader_ver, DpiScaling::Default);
    let egui_ctx = egui::Context::default();
    let mut event_pump = sdl_context.event_pump().unwrap();
    let image = image::open("../spooky-caspar/assets/evade.png").unwrap();
    let size = [image.width() as _, image.height() as _];
    let image_buffer = image.to_rgba8();
    let color_image = egui::ColorImage::from_rgba_unmultiplied(size, &image_buffer);
    let evade_icon = egui_ctx.load_texture("evade_image", color_image, egui::TextureOptions::default());

    let image = image::open("../spooky-caspar/assets/visuals.png").unwrap();
    let size = [image.width() as _, image.height() as _];
    let image_buffer = image.to_rgba8();
    let color_image = egui::ColorImage::from_rgba_unmultiplied(size, &image_buffer);
    let visuals_icon = egui_ctx.load_texture("evade_image", color_image, egui::TextureOptions::default());

    let image = image::open("../spooky-caspar/assets/offense.png").unwrap();
    let size = [image.width() as _, image.height() as _];
    let image_buffer = image.to_rgba8();
    let color_image = egui::ColorImage::from_rgba_unmultiplied(size, &image_buffer);
    let offense_icon = egui_ctx.load_texture("offense_image", color_image, egui::TextureOptions::default());
    let mut fonts = FontDefinitions::default();
    //window.set_opacity(0.2);

    // offense_tab.icon = Some(offense_icon);
    // visuals_tab.icon = Some(visuals_icon);
    // evade_tab.icon = Some(evade_icon);
    let main_menu: MyTab = MyTab {
        name: "Spooky Caspar".to_string(),
        description: "test".to_string(),
        widgets: vec![
            MyWidget::Tab(offense_tab),
            MyWidget::Tab(evade_tab),
            MyWidget::Tab(visuals_tab),
            MyWidget::Tab(misc_tab),
        ],
        icon: None,
    };
    let mut current_tab = main_menu.clone();
    fonts.font_data.insert(
        "my_font".to_owned(),
        FontData::from_static(include_bytes!("../../BigBlueTerm437.ttf")),
    );

    fonts
        .families
        .get_mut(&FontFamily::Proportional)
        .unwrap()
        .insert(0, "my_font".to_owned());

    egui_ctx.set_fonts(fonts);
    //catppuccin_egui::set_theme(&egui_ctx, catppuccin_egui::MACCHIATO);
    let hwnd = window_manipulation::get_hwnd("SPOOKY CASPAR".to_string()).unwrap();
    window_manipulation::set_transparent_color(hwnd, 0x080808);
    window_manipulation::set_window_always_on_top_isize(hwnd);
    let mut click_cd = Instant::now();
    let mut quit = false;
    let mut color_picker_window: Option<MyColorPicker> = None;
    let mut option_selector_window: Option<MyOptionSelector> = None;
    let mut color = [1.0, 1.0, 1.0, 0.5];
    'running: loop {
        window.subsystem().gl_set_swap_interval(SwapInterval::VSync).unwrap();

        // Begin the egui frame
        egui_ctx.begin_frame(egui_state.input.take());

        // Process SDL2 events
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'running,

                _ => {
                    egui_state.process_input(&window, event, &mut painter);
                }
            }
        }

        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(egui::Color32::from_rgb(
                TRANSPARENT_COLOR.0,
                TRANSPARENT_COLOR.1,
                TRANSPARENT_COLOR.2,
            )))
            .show(&egui_ctx, |ui| {
                if let Some(pos) = ui
                    .add(CircleMenu::new(
                        &mut current_tab,
                        &tab_path,
                        CENTER,
                        OUTER_RADIUS,
                        INNER_RADIUS,
                    ))
                    .interact_pointer_pos()
                {
                    if Instant::now() > click_cd {
                        let click_offset: Pos2 = Pos2::new(pos.x - CENTER.x, pos.y - CENTER.y);
                        if click_offset.distance(egui::Pos2::new(0.0, 0.0)) > INNER_RADIUS
                            && click_offset.distance(egui::Pos2::new(0.0, 0.0)) < OUTER_RADIUS
                        {
                            if let Some(widget_index) = what_widget(current_tab.widgets.len(), click_offset) {
                                let clicked_widget = &mut current_tab.widgets[widget_index];

                                click_cd = Instant::now() + Duration::from_millis(200);

                                match clicked_widget {
                                    MyWidget::Tab(tab) => {
                                        let mut new_tab = tab.clone();
                                        new_tab.widgets.insert(0, back_tab.clone());

                                        let mut tab_path_string: String = tab_path.chars().collect();
                                        tab_path_string.push('/');
                                        tab_path_string.extend(new_tab.name.chars());
                                        tab_path = tab_path_string;
                                        current_tab = new_tab;
                                    }
                                    MyWidget::Checkbox(ref mut checkbox) => {
                                        checkbox.enabled = !checkbox.enabled;
                                        match checkbox.description.as_str() {
                                            "Orbwalk when holding space" => {
                                                main_loop_sender
                                                    .send(MainLoopThreadCommunication::GuiThread(
                                                        GuiMessage::Offensive(checkbox.enabled),
                                                    ))
                                                    .unwrap();
                                            }
                                            "Dodge skillshots automaticlly" => {
                                                main_loop_sender
                                                    .send(MainLoopThreadCommunication::GuiThread(GuiMessage::Evade(
                                                        checkbox.enabled,
                                                    )))
                                                    .unwrap();
                                            }
                                            "Shows enabled modules" => {
                                                main_loop_sender
                                                    .send(MainLoopThreadCommunication::GuiThread(GuiMessage::Hud(
                                                        checkbox.enabled,
                                                    )))
                                                    .unwrap();
                                            }
                                            "Draws evasion path when evading" => {
                                                main_loop_sender
                                                    .send(MainLoopThreadCommunication::GuiThread(
                                                        GuiMessage::OverlayEvadePath(checkbox.enabled),
                                                    ))
                                                    .unwrap();
                                            }
                                            "Draws projectiles" => {
                                                main_loop_sender
                                                    .send(MainLoopThreadCommunication::GuiThread(
                                                        GuiMessage::OverlayProjectiles(checkbox.enabled),
                                                    ))
                                                    .unwrap();
                                            }
                                            "Draws player attack range" => {
                                                main_loop_sender
                                                    .send(MainLoopThreadCommunication::GuiThread(
                                                        GuiMessage::OverlayAttackRange(checkbox.enabled),
                                                    ))
                                                    .unwrap();
                                            }
                                            _ => println!("Unknown checkbox description: {}", checkbox.description),
                                        }
                                    }
                                    MyWidget::Back(tab) => {
                                        option_selector_window = None;
                                        color_picker_window = None;
                                        current_tab = main_menu.clone();
                                        tab_path = "".to_string();
                                    }
                                    MyWidget::ColorPicker(color_picker) => {
                                        color_picker_window = Some(color_picker.clone());
                                    }
                                    MyWidget::OptionSelector(option_selector) => {
                                        option_selector_window = Some(option_selector.clone());
                                    }
                                }
                            }
                        }
                    }
                }
            });

        if let Some(color_picker_clone) = color_picker_window.clone() {
            option_selector_window = None;
            egui::Window::new("")
                .title_bar(false)
                .fixed_pos(Pos2::new(220.0, 260.0))
                .frame(
                    egui::Frame::none()
                        .stroke(Stroke::NONE)
                        .shadow(Shadow::NONE)
                        .inner_margin(egui::Margin::symmetric(40.0, 40.0))
                        .outer_margin(egui::Margin::symmetric(0.0, 0.0))
                        .fill(Color32::from_rgba_premultiplied(
                            color[0] as u8,
                            color[1] as u8,
                            color[2] as u8,
                            255,
                        )),
                )
                .show(&egui_ctx, |ui| {
                    CENTER.x;
                    ui.color_edit_button_rgba_premultiplied(&mut color);
                    ui.horizontal(|ui| {
                        if ui
                            .add(
                                egui::Button::new(egui::RichText::new("Select").color(Color32::BLACK))
                                    .fill(Color32::WHITE),
                            )
                            .clicked()
                        {
                            if let Some(index) = current_tab.widgets.iter().position(|widget| {
                                if let MyWidget::ColorPicker(cp) = widget {
                                    cp.name == color_picker_clone.name
                                } else {
                                    false
                                }
                            }) {
                                if let MyWidget::ColorPicker(color_picker) = &mut current_tab.widgets[index] {
                                    color_picker.current_color = color_picker_clone.current_color;
                                    color_picker_window = None; // Close the color picker window
                                }
                            }
                        }
                        if ui
                            .add(
                                egui::Button::new(egui::RichText::new("Cancel").color(Color32::BLACK))
                                    .fill(Color32::WHITE),
                            )
                            .clicked()
                        {
                            main_loop_sender
                                .send(MainLoopThreadCommunication::GuiThread(GuiMessage::OverlayColor(
                                    Color32::from_rgba_premultiplied(
                                        color[0] as u8,
                                        color[1] as u8,
                                        color[2] as u8,
                                        color[3] as u8,
                                    ),
                                )))
                                .unwrap();
                            color_picker_window = None; 
                        }
                    });
                });
        }
        if let Some(mut op_clone) = option_selector_window.clone() {
            color_picker_window = None;
            egui::Window::new("")
                .title_bar(false)
                .fixed_pos(Pos2::new(220.0, 260.0))
                .frame(
                    egui::Frame::none()
                        .stroke(Stroke::NONE)
                        .shadow(Shadow::NONE)
                        .inner_margin(egui::Margin::symmetric(40.0, 40.0))
                        .outer_margin(egui::Margin::symmetric(0.0, 0.0))
                        .fill(Color32::from_rgba_premultiplied(
                            color[0] as u8,
                            color[1] as u8,
                            color[2] as u8,
                            255,
                        )),
                )
                .show(&egui_ctx, |ui| {
                    ui.horizontal(|ui| {
                        // Left arrow button
                        if ui.button("<-").clicked() {
                            op_clone.selected_index = op_clone
                                .selected_index
                                .checked_sub(1)
                                .unwrap_or_else(|| op_clone.options.len() - 1);
                            option_selector_window = Some(op_clone.clone()); // Reflect the updated state
                        }

                        // Display the currently selected option
                        ui.label(&op_clone.options[op_clone.selected_index]);

                        // Right arrow button
                        if ui.button("->").clicked() {
                            op_clone.selected_index = (op_clone.selected_index + 1) % op_clone.options.len();
                            option_selector_window = Some(op_clone.clone()); // Reflect the updated state
                        }
                    });

                    ui.horizontal(|ui| {
                        if ui.button("Select").clicked() {
                            // Update the main data structure with the selected index
                            if let Some(index) = current_tab.widgets.iter().position(|widget| {
                                if let MyWidget::OptionSelector(os) = widget {
                                    os.name == op_clone.name
                                } else {
                                    false
                                }
                            }) {
                                if let MyWidget::OptionSelector(option_selector) = &mut current_tab.widgets[index] {
                                    option_selector.selected_index = op_clone.selected_index;
                                    match option_selector.options[op_clone.selected_index].as_str() {
                                        "Lowest Health" => {
                                            main_loop_sender
                                                .send(MainLoopThreadCommunication::GuiThread(
                                                    GuiMessage::ChangeTargetMode(TargetMode::LowestHp),
                                                ))
                                                .unwrap();
                                        }
                                        "Lowest Distance" => {
                                            main_loop_sender
                                                .send(MainLoopThreadCommunication::GuiThread(
                                                    GuiMessage::ChangeTargetMode(TargetMode::LowestDistance),
                                                ))
                                                .unwrap();
                                        }
                                        _ => println!("Unknown os description: {}", option_selector.description),
                                    }
                                }
                            }
                            option_selector_window = None; // Close the option selector window
                        }

                        if ui.button("Cancel").clicked() {
                            option_selector_window = None; // Close the option selector window without saving the selection
                        }
                    });
                });
        }
        // End the egui frame and render
        let FullOutput {
            platform_output,
            repaint_after,
            textures_delta,
            shapes,
        } = egui_ctx.end_frame();
        egui_state.process_output(&window, &platform_output);
        let paint_jobs = egui_ctx.tessellate(shapes);
        painter.paint_jobs(None, textures_delta, paint_jobs);
        window.gl_swap_window();

        // Check if we need to schedule a repaint
        if !repaint_after.is_zero() {
            egui_ctx.request_repaint();
        }

        if quit {
            std::process::exit(0);
        }
    }
}

use egui::{vec2, Context, Pos2, Rect, Sense, Shape};

use crate::overlay::{self, window_manipulation};
use crate::sdk::player_manager::Skillshot;
use crate::utils::TargetMode;
use crate::MainLoopThreadCommunication;

pub struct CircleMenu<'a> {
    current_tab: &'a mut MyTab,
    tab_path: &'a String,
    center: Pos2,
    outer_radius: f32,
    inner_radius: f32,
}

impl<'a> CircleMenu<'a> {
    fn new(
        current_tab: &'a mut MyTab,
        tab_path: &'a String,
        center: Pos2,
        outer_radius: f32,
        inner_radius: f32,
    ) -> Self {
        Self {
            current_tab,
            tab_path,
            center,
            outer_radius,
            inner_radius,
        }
    }
}

impl<'a> Widget for CircleMenu<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        let mut text_pos = Vec::new();
        let response = ui.allocate_response(ui.available_size(), Sense::click());
        let mut angle_step = 2.0 * std::f32::consts::PI / self.current_tab.widgets.len() as f32;
        if self.current_tab.widgets.len() == 1 {
            angle_step = 2.0 * std::f32::consts::PI
        }

        let mut description = None;
        let painter = ui.painter();
        let mut segments = Vec::new();

        for (i, widget) in self.current_tab.widgets.iter().enumerate() {
            let mut name = "het".to_string();
            let start_angle = i as f32 * angle_step;
            let end_angle = (i + 1) as f32 * angle_step;
            let mut color = Color32::WHITE;
            let mut text_color = Color32::BLACK;
            match widget {
                MyWidget::Tab(tab) => {
                    name = tab.name.clone();
                }
                MyWidget::Checkbox(checkbox) => {
                    name = checkbox.name.clone();
                    if checkbox.enabled {
                        let arc_thickness = 12.0; 
                        let segment_count = 30;

                        // Calculate the points of the arc
                        let mut points = Vec::new();
                        for j in 0..=segment_count {


                            let angle = lerp(start_angle, end_angle, j as f32 / segment_count as f32);

                            // Calculate the x and y coordinates based on the center and radius
                            let x = self.center.x + (self.outer_radius + 15.0) * angle.cos();
                            let y = self.center.y + (self.outer_radius + 15.0) * angle.sin();
                            // Add the point to the list
                            points.push(egui::Pos2::new(x, y));
                        }


                        for window in points.windows(2) {
                            if let [start, end] = *window {
                                painter.line_segment([start, end], (arc_thickness, Color32::BLACK));
                            }
                        }
                    }
                }

                MyWidget::Back(back) => name = back.name.clone(),
                MyWidget::ColorPicker(color_picker) => {
                    name = color_picker.name.clone();
                }
                MyWidget::OptionSelector(optionselector) => {
                    name = optionselector.name.clone();
                }
            }
            if let Some(pos) = ui.ctx().pointer_hover_pos() {
                let click_offset = pos - CENTER;
                let click_dist = click_offset.length();
                if click_dist > INNER_RADIUS && click_dist < OUTER_RADIUS {
                    if let Some(widget_index) = what_widget(
                        self.current_tab.widgets.len(),
                        Pos2::new(click_offset.x, click_offset.y),
                    ) {
                        let hovered_widget = &self.current_tab.widgets[widget_index];

                        match hovered_widget {
                            MyWidget::Tab(tab) => {
                                if tab.name == name {
                                    color = Color32::BLACK;
                                    text_color = Color32::WHITE;
                                    description = Some(tab.description.clone())
                                }
                            }
                            MyWidget::OptionSelector(optionselector) => {
                                if optionselector.name == name {
                                    color = Color32::BLACK;
                                    text_color = Color32::WHITE;
                                    description = Some(optionselector.description.clone())
                                }
                            }
                            MyWidget::ColorPicker(backtab) => {
                                if backtab.name == name {
                                    color = Color32::BLACK;
                                    text_color = Color32::WHITE;
                                    description = Some(backtab.description.clone())
                                }
                            }
                            MyWidget::Checkbox(checkbox) => {
                                if name.starts_with(&checkbox.name) {
                                    color = Color32::BLACK;
                                    text_color = Color32::WHITE;
                                    description = Some(checkbox.description.clone())
                                }
                            }
                            MyWidget::Back(backtab) => {
                                if backtab.name == name {
                                    color = Color32::BLACK;
                                    text_color = Color32::WHITE;
                                    description = Some(backtab.description.clone())
                                }
                            }
                        }
                    }
                }
            }

            let mut points = Vec::new();

            // Generate points for the outer arc
            for angle in (0..=20).map(|x| lerp(start_angle, end_angle, x as f32 / 20.0)) {
                points.push(self.center + Vec2::new(angle.cos(), angle.sin()) * self.outer_radius);
            }

            // Generate points for the inner arc
            for angle in (0..=20).map(|x| lerp(start_angle, end_angle, x as f32 / 20.0)).rev() {
                points.push(self.center + Vec2::new(angle.cos(), angle.sin()) * self.inner_radius);
            }

            // Convert Vec<Vec2> to Vec<Pos2>
            let points_pos2: Vec<Pos2> = points.iter().map(|&v| Pos2::new(v.x, v.y)).collect();

            // Draw the segment
            let segment = painter.add(Shape::convex_polygon(
                points_pos2,
                color,
                egui::Stroke::new(5.0, Color32::WHITE),
            ));
            segments.push(segment);

            let text_angle = (start_angle + end_angle) / 2.0;
            let label_pos = self.center
                + Vec2::new(text_angle.cos(), text_angle.sin()) * ((self.inner_radius + self.outer_radius) / 2.0);
            text_pos.push((label_pos, text_color));
            //painter.text(label_pos, Align2::CENTER_CENTER, name, FontId::default(), text_color);
        }
        ui.painter()
            .circle_stroke(self.center, self.outer_radius, egui::Stroke::new(5.0, Color32::WHITE));
        ui.painter().circle_filled(
            self.center,
            self.inner_radius,
            Color32::from_rgb(TRANSPARENT_COLOR.0, TRANSPARENT_COLOR.1, TRANSPARENT_COLOR.2),
        );
        ui.put(
            Rect {
                min: egui::Pos2::new(CENTER.x - 150.0, 0.0),
                max: egui::Pos2::new(CENTER.x + 150.0, 0.0),
            },
            egui::Label::new(
                egui::RichText::new(self.tab_path.to_lowercase())
                    .color(Color32::WHITE)
                    .background_color(Color32::BLACK),
            ),
        );

        if let Some(desc) = description {
            ui.put(
                Rect {
                    min: egui::Pos2::new(CENTER.x - 100.0, CENTER.y),
                    max: egui::Pos2::new(CENTER.x + 100.0, CENTER.y),
                },
                egui::Label::new(
                    egui::RichText::new(desc)
                        //.extra_letter_spacing(1.0)
                        .background_color(Color32::BLACK)
                        .color(Color32::WHITE),
                ),
            );
        }
        let mut icon: Option<TextureHandle> = None;
        for widgets in self.current_tab.widgets.iter() {
            match widgets {
                MyWidget::Tab(tab) => {
                    if let Some(tab_icon) = &tab.icon {
                        icon = Some(tab_icon.clone());
                    }
                }
                _ => (),
            }
        }

        let mut buffer = String::new();
        for ((pos, text_color), mut widget) in text_pos.iter_mut().zip(&mut self.current_tab.widgets) {
            let name = match widget {
                MyWidget::Tab(tab) => &tab.name,
                MyWidget::OptionSelector(os) => {
                    buffer.clear();
                    buffer = format!("[{}]{}", os.options[os.selected_index], os.name);
                    &buffer
                }
                MyWidget::Checkbox(checkbox) => {
                    buffer.clear();
                    if checkbox.enabled {
                        buffer = format!("[ON] {}", checkbox.name);
                    } else {
                        buffer = format!("[OFF] {}", checkbox.name);
                    }
                    &buffer
                }
                MyWidget::Back(back) => &back.name,

                MyWidget::ColorPicker(color_picker) => &color_picker.name,
            };
            let label_rect = Rect {
                min: egui::Pos2::new(pos.x - 100.0, pos.y - 100.0),
                max: egui::Pos2::new(pos.x + 100.0, pos.y + 100.0),
            };
            ui.put(
                label_rect,
                egui::Label::new(egui::RichText::new(name).color(*text_color)),
            );
            let icon_rect = Rect {
                min: egui::Pos2::new(pos.x - 100.0, pos.y + 80.0),
                max: egui::Pos2::new(pos.x + 100.0, pos.y + 80.0),
            };
            match widget {
                MyWidget::Tab(tab) => {
                    if let Some(tab_icon) = &tab.icon {
                        ui.put(
                            icon_rect,
                            egui::Image::new(tab_icon).max_size(egui::Vec2::new(32.0, 32.0)),
                        );
                    }
                }
                _ => (),
            }
        }

        response
    }
}

fn lerp(start: f32, end: f32, percent: f32) -> f32 {
    start + (end - start) * percent
}

fn calculate_angle(x: f32, y: f32) -> f32 {
    let angle_radians = f32::atan2(y, x);
    if angle_radians < 0.0 {
        angle_radians + (2.0 * std::f32::consts::PI)
    } else {
        angle_radians
    }
}

pub fn what_widget(widgets_count: usize, click_pos: Pos2) -> Option<usize> {
    let angle_step = 2.0 * std::f32::consts::PI / widgets_count as f32;
    let click_angle = calculate_angle(click_pos.x, click_pos.y);
    for i in 0..widgets_count {
        let start_angle = i as f32 * angle_step;
        let end_angle = (i + 1) as f32 * angle_step;
        if click_angle >= start_angle && click_angle < end_angle {
            return Some(i);
        }
    }

    return None;
}
