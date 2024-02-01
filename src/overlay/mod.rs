pub mod overlay {
    // TODO Minimize and optimize. Perhaps make it look better.
    use std::path::Path;

    use crate::PROCESS_WINDOW_NAME;

    use super::window_manipulation::is_window_borderless;
    use super::window_manipulation::{self, get_hwnd, get_window_info};


    pub const SKEW_FACTOR: f32 = 0.82;
    use nc::na::Point2;
    use speedy2d::color::Color;
    use speedy2d::window::WindowHandler;
    use speedy2d::window::WindowHelper;
    use speedy2d::Graphics2D;
    use speedy2d::Window;
    use speedy2d::{
        dimen::Vector2,
        font::{Font, TextLayout, TextOptions},
        window::{UserEventSender, WindowCreationOptions, WindowPosition, WindowSize},
    };

    #[derive(Clone, Debug)]
    pub struct RenderEntity {
        pub name: String,
        pub pos: Point2<f32>,
        pub radius: f32,
        pub color: Color,
        pub target: bool,
        // pub d_summoner_name: String,
        // pub f_summoner_name: String,

        // pub d_summoner_cd: Option<f32>,
        // pub f_summoner_cd: Option<f32>,
    }

    #[derive(Clone, Debug)]
    pub struct RenderMissile {
        pub name: String,
        pub start_pos: Point2<f32>,
        pub end_pos: Point2<f32>,
        pub width: f32,
        pub color: Color,
    }
    #[derive(Clone, Debug)]
    pub struct RenderCircular {
        pub name: String,
        pub pos: Point2<f32>,
        pub radius: f32,
        pub color: Color,
    }
    #[derive(Clone, Debug)]
    pub struct AutoAttackRange {
        pub name: String,
        pub pos: Point2<f32>,
        pub radius: f32,

        pub color: Color,
    }

    #[derive(Clone, Debug)]
    pub struct RenderText {
        pub name: String,
        pub color: Color,
    }
    #[derive(Clone, Debug)]
    pub enum RenderObject {
        Entity(RenderEntity),
        Missile(RenderMissile),
        Circular(RenderCircular),
        Arrow {
            start_pos: Point2<f32>,
            end_pos: Point2<f32>,
            color: Color,
        },
        Text(RenderText),
        AttackRange(AutoAttackRange),
    }

    pub fn start_overlay() -> Window<Option<Vec<RenderObject>>> {
        let game_hwnd = get_hwnd(PROCESS_WINDOW_NAME.to_owned()).unwrap();
        let window_info = get_window_info(game_hwnd).unwrap();
        let (extra_x, extra_y) = if !is_window_borderless() { (16, 39) } else { (0, 0) };
        let (extra_x2, extra_y2) = if is_window_borderless() { (8, 32) } else { (0, 0) };
        println!("window_info: {:?}", window_info);
        let window_options = WindowCreationOptions::new_windowed(
            WindowSize::PhysicalPixels(Vector2 {
                x: (window_info.2 - extra_x) as u32,
                y: (window_info.3 - extra_y) as u32,
            }),
            Some(WindowPosition::PrimaryMonitorPixelsFromTopLeft(Vector2 {
                x: window_info.0 - extra_x2,
                y: window_info.1 - extra_y2,
            })),
        )
        .with_always_on_top(true)
        .with_transparent(true)
        .with_decorations(false)
        .with_resizable(false);

        let title = "OVERLAY".to_owned();
        let window: Window<Option<Vec<RenderObject>>> = Window::new_with_user_events(&title, window_options).unwrap();
        let hwnd = window_manipulation::get_hwnd(title).unwrap();
        window_manipulation::set_click_through(hwnd).unwrap();

        return window;
    }

    // #[derive(Debug)]
    // struct SummonerSpellImages {
    //     flash: ImageHandle,
    //     cleanse: ImageHandle,
    //     exhaust: ImageHandle,
    //     teleport: ImageHandle,
    //     smite: ImageHandle,
    //     heal: ImageHandle,
    //     ignite: ImageHandle,
    //     ghost: ImageHandle,
    //     snowball: ImageHandle,
    //     barrier: ImageHandle,
    // }
    pub struct MyWindowHandler {
        pub user_event_sender: UserEventSender<Option<Vec<RenderObject>>>,
        pub draw_list: Vec<RenderObject>,
        pub font: Font,
        //pub summoner_spell_images: Option<SummonerSpellImages>,
    }

    impl MyWindowHandler {
        pub fn new(sender: UserEventSender<Option<Vec<RenderObject>>>) -> Self {
            let font = Font::new(include_bytes!("../../BigBlueTerm437.TTF")).unwrap();
            Self {
                user_event_sender: sender,
                draw_list: Vec::new(),
                font: font,
                //summoner_spell_images: None,
            }
        }
    }

    impl WindowHandler<Option<Vec<RenderObject>>> for MyWindowHandler {
        fn on_draw(&mut self, helper: &mut WindowHelper<Option<Vec<RenderObject>>>, graphics: &mut Graphics2D) {
            graphics.clear_screen(speedy2d::color::Color::TRANSPARENT);
            let mut y_offset = 40.0;
            for render_object in self.draw_list.iter() {
                match render_object {
                    RenderObject::AttackRange(attack_range) => {
                        let octagon_size: f32 = attack_range.radius * 0.94;
                        let mut points = Vec::new();

    
                        let vertical_scale = 0.757; 
                        let horizontal_scale = 0.9; 

                        for i in 0..32 {
                            let angle = 2.0 * std::f32::consts::PI / 32.0 * i as f32;
                            let base_x = octagon_size * angle.cos();
                            let base_y = octagon_size * angle.sin();

                            let x = attack_range.pos.x + base_x * horizontal_scale;
                            let y = attack_range.pos.y + base_y * vertical_scale;

                            points.push((x + 7.0, y + 35.0));
                        }

                        let start_color = attack_range.color;
                        // let end_color = Color::from_rgba(
                        //     255.0 - start_color.r(),
                        //     255.0 - start_color.b(),
                        //     255.0 - start_color.g(),
                        //     255.0 - start_color.a(),
                        // );
                        for i in 0..32 {
                            let t = i as f32 / 31.0;
                            // let lerp_color = Color::from_rgba(
                            //     start_color.r() + ((end_color.r() - start_color.r()) * t),
                            //     start_color.g() + ((end_color.g() - start_color.b()) * t),
                            //     start_color.b() + ((end_color.b() - start_color.b()) * t),
                            //     start_color.a() + ((end_color.a() - start_color.a()) * t),
                            // );

                            let (start_x, start_y) = points[i];
                            let (end_x, end_y) = points[(i + 1) % 32];
                            graphics.draw_line(
                                Vector2::new(start_x, start_y),
                                Vector2::new(end_x, end_y),
                                4.0,
                                start_color,
                            );
                        }
                    }

                    RenderObject::Entity(entity) => {
                        // if self.summoner_spell_images.is_none() {
                        //     self.summoner_spell_images = Some(load_spell_images(graphics));
                        // }

                        // if let Some(images) = &self.summoner_spell_images {
                        //     let image_d = match entity.d_summoner_name.as_str() {
                        //         "Flash" => &images.flash,
                        //         "Cleanse" => &images.cleanse,
                        //         "Exhaust" => &images.exhaust,
                        //         "Teleport" => &images.teleport,
                        //         "Smite" => &images.smite,
                        //         "Heal" => &images.heal,
                        //         "Ignite" => &images.ignite,
                        //         "Ghost" => &images.ghost,
                        //         "Snowball" => &images.snowball,
                        //         "Barrier" => &images.barrier,
                        //         _ => &images.flash, // or handle unknown names appropriately
                        //     };
                        //     let image_f = match entity.f_summoner_name.as_str() {
                        //         "Flash" => &images.flash,
                        //         "Cleanse" => &images.cleanse,
                        //         "Exhaust" => &images.exhaust,
                        //         "Teleport" => &images.teleport,
                        //         "Smite" => &images.smite,
                        //         "Heal" => &images.heal,
                        //         "Ignite" => &images.ignite,
                        //         "Ghost" => &images.ghost,
                        //         "Snowball" => &images.snowball,
                        //         "Barrier" => &images.barrier,
                        //         _ => &images.flash, // or handle unknown names appropriately
                        //     };
                        //     // Define image position (adjust X and Y as needed)
                        //     let image_pos_d = Vector2::new(entity.pos.x + 83.0, entity.pos.y - 128.0);
                        //     let image_pos_f = Vector2::new(entity.pos.x + 83.0 + 23.0, entity.pos.y - 127.0);
                        //     // Render the image
                        //     graphics.draw_image(image_pos_d, image_d);
                        //     graphics.draw_image(image_pos_f, image_f);
                        //     let square_size = Vector2::new(23.0, 23.0); // Example size for cooldown squares
                        //     let square_offset = Vector2::new(5.0, 5.0); // Offset for text within the square

                        //     // Draw the semi-transparent black square for D summoner spell if cooldown is active
                        //     if let Some(seconds) = entity.d_summoner_cd {
                        //         let square_rect_d = Rectangle::new(image_pos_d, image_pos_d + square_size);
                        //         graphics.draw_rectangle(square_rect_d, Color::from_rgba(0.0, 0.0, 0.0, 0.5)); // Semi-transparent black

                        //         // Draw the cooldown text
                        //         let text_layout_d =
                        //             self.font
                        //                 .layout_text(&seconds.to_string(), 13.0, TextOptions::default());
                        //         let text_pos_d = image_pos_d + square_offset;
                        //         graphics.draw_text(text_pos_d, Color::WHITE, &text_layout_d);
                        //     }

                        //     // Draw the semi-transparent black square for F summoner spell if cooldown is active
                        //     if let Some(seconds) = entity.f_summoner_cd {
                        //         let square_rect_f = Rectangle::new(image_pos_f, image_pos_f + square_size);
                        //         graphics.draw_rectangle(square_rect_f, Color::from_rgba(0.0, 0.0, 0.0, 0.5)); // Semi-transparent black

                        //         // Draw the cooldown text
                        //         let text_layout_f =
                        //             self.font
                        //                 .layout_text(&seconds.to_string(), 13.0, TextOptions::default());
                        //         let text_pos_f = image_pos_f + square_offset;
                        //         graphics.draw_text(text_pos_f, Color::WHITE, &text_layout_f);
                        //     }
                        // }
                        if entity.target {
                            let start_x = 1200.0 / 1.3;
                            let start_y = 1200.0 / 1.3;
                            let end_x = entity.pos.x;
                            let end_y = entity.pos.y;

                            graphics.draw_line(
                                Vector2::new(start_x, start_y),
                                Vector2::new(end_x, end_y),
                                4.0, // Line width
                                entity.color,
                            );
                        }
        
                        let octagon_size: f32 = entity.radius * 0.74; 
                                                                  
                        let mut points = Vec::new();

                        for i in 0..12 {
                            let angle = 2.0 * std::f32::consts::PI / 12.0 * i as f32; 
                            let x = entity.pos.x + octagon_size * angle.cos();
                            let y = entity.pos.y + octagon_size * angle.sin() * SKEW_FACTOR;
                            points.push((x, y));
                        }

                        for i in 0..12 {
                            let (start_x, start_y) = points[i];
                            let (end_x, end_y) = points[(i + 1) % 12]; 
                            graphics.draw_line(
                                Vector2::new(start_x, start_y),
                                Vector2::new(end_x, end_y),
                                4.0, // Line width
                                entity.color,
                            );
                        }

                        // Draw the entity's name
                        // let text_layout = self.font.layout_text(&entity.name, 20.0, TextOptions::default());
                        // graphics.draw_text(
                        //     Vector2::new(entity.pos.x - 100.0, entity.pos.y),
                        //     entity.color,
                        //     &text_layout,
                        // );
                    }
                    RenderObject::Circular(circular) => {
  
                        let octagon_size: f32 = circular.radius * 0.74; 
                                                                     
                        let mut points = Vec::new();

                        for i in 0..8 {
                            let angle = 2.0 * std::f32::consts::PI / 8.0 * i as f32;
                            let x = circular.pos.x + octagon_size * angle.cos();
                            let y = circular.pos.y + octagon_size * angle.sin() * SKEW_FACTOR;
                            points.push((x, y));
                        }

            
                        for i in 0..8 {
                            let (start_x, start_y) = points[i];
                            let (end_x, end_y) = points[(i + 1) % 8]; 
                            graphics.draw_line(
                                Vector2::new(start_x, start_y),
                                Vector2::new(end_x, end_y),
                                4.0,
                                circular.color,
                            );
                        }
                    }
                    RenderObject::Missile(missile) => {
                        let direction = Vector2::new(
                            missile.end_pos.x - missile.start_pos.x,
                            missile.end_pos.y - missile.start_pos.y,
                        );

       
                        let norm_direction = direction.normalize().unwrap_or(Vector2::new(0.0, 0.0));

                        let perpendicular =
                            nc::na::Vector2::new(-norm_direction.y, norm_direction.x) * (missile.width * 0.5 / 2.0);

                        let skew_factor = 0.5;
                        let skew_direction = 1.0; 

                        let skew_adjustment = nc::na::Vector2::new(0.0, skew_factor * skew_direction * perpendicular.x);
                        let corner1 = missile.start_pos + perpendicular + skew_adjustment;
                        let corner2 = missile.start_pos + (-perpendicular) + skew_adjustment;
                        let corner3 = missile.end_pos + perpendicular + skew_adjustment;
                        let corner4 = missile.end_pos + (-perpendicular) + skew_adjustment;


                        let convert_to_speedy2d = |p: Point2<f32>| speedy2d::dimen::Vector2::new(p.x, p.y);

                        let [corner1_speedy, corner2_speedy, corner3_speedy, corner4_speedy] =
                            [corner1, corner2, corner3, corner4].map(convert_to_speedy2d);

                        // Draw lines between the corners
                        graphics.draw_line(corner1_speedy, corner3_speedy, 5.0, missile.color);
                        graphics.draw_line(corner3_speedy, corner4_speedy, 5.0, missile.color);
                        graphics.draw_line(corner4_speedy, corner2_speedy, 5.0, missile.color);
                        graphics.draw_line(corner2_speedy, corner1_speedy, 5.0, missile.color);
                    }
                    RenderObject::Arrow {
                        start_pos,
                        end_pos,
                        color,
                    } => {
                        graphics.draw_line(
                            Vector2::new(start_pos.x, start_pos.y),
                            Vector2::new(end_pos.x, end_pos.y),
                            5.0,
                            *color,
                        );

                        let arrowhead_size: f32 = 10.0; // Adjust the size of the arrowhead
                        let arrowhead_angle: f32 = 45.0_f32.to_radians(); // Adjust the angle of the arrowhead

                        let direction = Vector2::new(end_pos.x - start_pos.x, end_pos.y - start_pos.y)
                            .normalize()
                            .unwrap();
                        let left = Vector2::new(
                            direction.x * arrowhead_size * arrowhead_angle.cos()
                                - direction.y * arrowhead_size * arrowhead_angle.sin(),
                            direction.y * arrowhead_size * arrowhead_angle.cos()
                                + direction.x * arrowhead_size * arrowhead_angle.sin(),
                        );
                        let right = Vector2::new(
                            direction.x * arrowhead_size * arrowhead_angle.cos()
                                + direction.y * arrowhead_size * arrowhead_angle.sin(),
                            direction.y * arrowhead_size * arrowhead_angle.cos()
                                - direction.x * arrowhead_size * arrowhead_angle.sin(),
                        );

                        graphics.draw_line(
                            Vector2::new(end_pos.x, end_pos.y),
                            Vector2::new(end_pos.x - left.x, end_pos.y - left.y),
                            3.0,
                            *color,
                        );
                        graphics.draw_line(
                            Vector2::new(end_pos.x, end_pos.y),
                            Vector2::new(end_pos.x - right.x, end_pos.y - right.y),
                            3.0,
                            *color,
                        );
                    }
                    RenderObject::Text(text) => {
                        let text_layout = self.font.layout_text(&text.name, 16.0, TextOptions::default());
                        let text_width = text_layout.width();
                        let text_height = text_layout.height(); 

 
                        let padding = 6.0;
                        let background_height = text_height + padding * 2.0;
                        let background_width = text_width + padding * 2.0; 
            
                        let text_x = 870.0 + (background_width - text_width) / 2.0;
                        let text_y = y_offset + (background_height - text_height) / 2.0;

                        let background_color = Color::from_rgba(0.1, 0.1, 0.1, 0.7); 

             
                        let top_left = Vector2::new(870.0, y_offset);
                        let top_right = Vector2::new(870.0 + background_width, y_offset);
                        let bottom_right = Vector2::new(870.0 + background_width, y_offset + background_height);
                        let bottom_left = Vector2::new(870.0, y_offset + background_height);

                        graphics.draw_quad([top_left, top_right, bottom_right, bottom_left], background_color);

    
                        graphics.draw_text(Vector2::new(text_x, text_y), text.color, &text_layout);

                        y_offset += background_height + padding; 
                    }
                }
            }

            helper.request_redraw();
        }

        fn on_user_event(
            &mut self,
            helper: &mut WindowHelper<Option<Vec<RenderObject>>>,
            user_event: Option<Vec<RenderObject>>,
        ) {
            if let Some(updated_render_objects) = user_event {

                self.draw_list.retain(|ro| {
                    updated_render_objects.iter().any(|updated_ro| match (ro, updated_ro) {
                        (RenderObject::Entity(e1), RenderObject::Entity(e2)) => e1.name == e2.name,
                        (RenderObject::Missile(m1), RenderObject::Missile(m2)) => m1.name == m2.name,
                        (RenderObject::Arrow { .. }, RenderObject::Arrow { .. }) => false,
                        _ => false,
                    })
                });


                for updated_ro in updated_render_objects {
                    match updated_ro {
                        RenderObject::AttackRange(new_attack_range) => {

                            if let Some(render_object) = self.draw_list.iter_mut().find(|ro| match ro {
                                RenderObject::AttackRange(existing_attack_range) => {
                                    new_attack_range.name == existing_attack_range.name
                                }
                                _ => false,
                            }) {
                                *render_object = RenderObject::AttackRange(new_attack_range);
               
                            } else {
                                self.draw_list.push(RenderObject::AttackRange(new_attack_range));
                      
                            }
                        }
                        RenderObject::Entity(new_entity) => {
      
                            if let Some(render_object) = self.draw_list.iter_mut().find(|ro| match ro {
                                RenderObject::Entity(existing_entity) => existing_entity.name == new_entity.name,
                                _ => false,
                            }) {
                                *render_object = RenderObject::Entity(new_entity);
                        
                            } else {
                                self.draw_list.push(RenderObject::Entity(new_entity));
                     
                            }
                        }
                        RenderObject::Circular(new_circular) => {
              
                            if let Some(render_object) = self.draw_list.iter_mut().find(|ro| match ro {
                                RenderObject::Missile(existing_circular) => existing_circular.name == new_circular.name,
                                _ => false,
                            }) {
                                *render_object = RenderObject::Circular(new_circular);
                        
                            } else {
                                self.draw_list.push(RenderObject::Circular(new_circular));
                         
                            }
                        }
                        RenderObject::Missile(new_missile) => {
           
                            if let Some(render_object) = self.draw_list.iter_mut().find(|ro| match ro {
                                RenderObject::Missile(existing_missile) => existing_missile.name == new_missile.name,
                                _ => false,
                            }) {
                                *render_object = RenderObject::Missile(new_missile);
                           
                            } else {
                                self.draw_list.push(RenderObject::Missile(new_missile));
                           
                            }
                        }
                        RenderObject::Arrow {
                            color,
                            start_pos,
                            end_pos,
                        } => {
               
                            self.draw_list.retain(|ro| match ro {
                                RenderObject::Arrow {
                                    color,
                                    start_pos,
                                    end_pos,
                                } => false,
                                _ => true,
                            });

                            self.draw_list.push(RenderObject::Arrow {
                                color,
                                start_pos,
                                end_pos,
                            });
                        }

                        RenderObject::Text(new_text) => {
          
                            if let Some(render_object) = self.draw_list.iter_mut().find(|ro| match ro {
                                RenderObject::Missile(existing_text) => existing_text.name == new_text.name,
                                _ => false,
                            }) {
                                *render_object = RenderObject::Text(new_text);
                     
                            } else {
                                self.draw_list.push(RenderObject::Text(new_text));
                         
                            }
                        }
                    }
                }
            } else {
        
                self.draw_list.clear();
            }
            helper.request_redraw();
        }
    }

    // fn load_spell_images(graphics: &mut Graphics2D) -> SummonerSpellImages {
    //     let flash = graphics
    //         .create_image_from_file_path(None, ImageSmoothingMode::Linear, "assets/flash.png")
    //         .unwrap();
    //     let cleanse = graphics
    //         .create_image_from_file_path(None, ImageSmoothingMode::Linear, "assets/cleanse.png")
    //         .unwrap();
    //     let exhaust = graphics
    //         .create_image_from_file_path(None, ImageSmoothingMode::Linear, "assets/exhaust.png")
    //         .unwrap();
    //     let teleport = graphics
    //         .create_image_from_file_path(None, ImageSmoothingMode::Linear, "assets/teleport.png")
    //         .unwrap();
    //     let smite = graphics
    //         .create_image_from_file_path(None, ImageSmoothingMode::Linear, "assets/smite.png")
    //         .unwrap();
    //     let heal = graphics
    //         .create_image_from_file_path(
    //             Some(ImageFileFormat::PNG),
    //             ImageSmoothingMode::Linear,
    //             "assets/heal.png",
    //         )
    //         .unwrap();
    //     let ignite = graphics
    //         .create_image_from_file_path(None, ImageSmoothingMode::Linear, "assets/ignite.png")
    //         .unwrap();
    //     let ghost = graphics
    //         .create_image_from_file_path(None, ImageSmoothingMode::Linear, "assets/ghost.png")
    //         .unwrap();
    //     let snowball = graphics
    //         .create_image_from_file_path(None, ImageSmoothingMode::Linear, "assets/snowball.png")
    //         .unwrap();
    //     let barrier = graphics
    //         .create_image_from_file_path(None, ImageSmoothingMode::Linear, "assets/barrier.png")
    //         .unwrap();

    //     // Store the images
    //     return SummonerSpellImages {
    //         flash,
    //         cleanse,
    //         exhaust,
    //         teleport,
    //         smite,
    //         heal,
    //         ignite,
    //         ghost,
    //         snowball,
    //         barrier,
    //     };
    // }
}

pub mod window_manipulation {
    use std::ffi::CString;
    use std::os::raw::c_void;
    use std::ptr;
    use windows_sys::Win32::Foundation::{HWND, RECT};
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        GetWindowLongPtrA, GetWindowLongW, GetWindowRect, IsWindow, SetLayeredWindowAttributes, SetWindowLongPtrA,
        SetWindowLongW, SetWindowPos, ShowWindow, GWL_EXSTYLE, HWND_TOPMOST, LWA_COLORKEY, SWP_FRAMECHANGED,
        SWP_NOZORDER, SW_HIDE, SW_RESTORE, WS_EX_LAYERED, WS_EX_TRANSPARENT,
    };
    use windows_sys::Win32::UI::WindowsAndMessaging::{SWP_NOMOVE, SWP_NOSIZE};
    use windows_sys::Win32::UI::{
        Input::KeyboardAndMouse::{SetActiveWindow, SetFocus},
        WindowsAndMessaging::{FindWindowA, SetForegroundWindow},
    };

    pub fn get_hwnd(process: String) -> Result<isize, String> {
        let process_name = CString::new(process).unwrap();
        let hwnd = unsafe { FindWindowA(ptr::null(), process_name.as_ptr() as *const u8) };
        if hwnd == 0 {
            return Err("Unable to get HWND".to_owned());
        }
        return Ok(hwnd);
    }

    pub fn set_click_through(hwnd: isize) -> Result<String, String> {
        unsafe {
            let ex_style = GetWindowLongPtrA(hwnd, GWL_EXSTYLE) as u32;
            SetWindowLongPtrA(
                hwnd,
                GWL_EXSTYLE,
                (ex_style | WS_EX_LAYERED | WS_EX_TRANSPARENT) as isize,
            );
            SetWindowPos(
                hwnd,
                0,
                0,
                0,
                0,
                0,
                SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_FRAMECHANGED,
            );
        }
        return Ok("Click Through Set!".to_string());
    }

    pub fn unset_click_through(hwnd: isize) -> Result<String, String> {
        unsafe {
            let ex_style = GetWindowLongPtrA(hwnd, GWL_EXSTYLE) as u32;
   
            SetWindowLongPtrA(
                hwnd,
                GWL_EXSTYLE,
                (ex_style & !(WS_EX_LAYERED | WS_EX_TRANSPARENT)) as isize,
            );
            SetWindowPos(
                hwnd,
                0,
                0,
                0,
                0,
                0,
                SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_FRAMECHANGED,
            );
        }
        return Ok("Click Through Unset!".to_string());
    }

    pub fn set_transparent_color(hwnd: isize, color: u32) -> bool {
        unsafe {
    
            let ex_style = GetWindowLongW(hwnd, GWL_EXSTYLE);
            SetWindowLongW(hwnd, GWL_EXSTYLE, ex_style | WS_EX_LAYERED as i32);


            SetLayeredWindowAttributes(hwnd, color, 150, LWA_COLORKEY) != 0
        }
    }
    pub fn set_active(hwnd: isize) -> Result<String, String> {
        unsafe {
            SetForegroundWindow(hwnd);
            SetFocus(hwnd);
            SetActiveWindow(hwnd);
        }
        return Ok(format!(
            "Process ForegroundWindow, SetFocus, SetActiveWindow Set!{}",
            hwnd
        ));
    }

    pub fn is_window(hwnd: HWND) -> bool {
        unsafe { IsWindow(hwnd) != 0 }
    }

    pub fn hide_window(hwnd: *mut c_void) {
        unsafe {
            ShowWindow(hwnd as isize, SW_HIDE);
        }
    }

    pub fn show_window(hwnd: *mut c_void) {
        unsafe {
            ShowWindow(hwnd as isize, SW_RESTORE);
        }
    }

    pub fn get_window_info(hwnd: HWND) -> Result<(i32, i32, i32, i32), String> {
        let mut rect: RECT = RECT {
            left: 0,
            top: 0,
            right: 0,
            bottom: 0,
        };
        let success = unsafe { GetWindowRect(hwnd as HWND, &mut rect as *mut RECT) };
        if success != 0 {
            let x = rect.left;
            let y = rect.top;
            let width = rect.right - rect.left;
            let height = rect.bottom - rect.top;
            Ok((x, y, width, height))
        } else {

            Err("Could not get window information".to_owned())
        }
    }

    pub fn set_window_always_on_top(window: *mut c_void) {
        unsafe {
            SetWindowPos(window as isize, HWND_TOPMOST, 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE);
        }
    }
    pub fn set_window_always_on_top_isize(window: isize) {
        unsafe {
            SetWindowPos(window, HWND_TOPMOST, 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE);
        }
    }
    pub fn is_window_borderless() -> bool {
        let game_hwnd = get_hwnd(crate::PROCESS_WINDOW_NAME.to_owned()).unwrap();
        unsafe {
            let style = windows_sys::Win32::UI::WindowsAndMessaging::GetWindowLongPtrA(
                game_hwnd,
                windows_sys::Win32::UI::WindowsAndMessaging::GWL_STYLE,
            ) as u32;

            let is_borderless = (style
                & (windows_sys::Win32::UI::WindowsAndMessaging::WS_BORDER
                    | windows_sys::Win32::UI::WindowsAndMessaging::WS_CAPTION))
                == 0;

            return is_borderless;
        }
    }
}

mod tests {
    use crate::{overlay::window_manipulation::get_window_info, PROCESS_WINDOW_NAME};

    use super::window_manipulation::get_hwnd;

    #[test]
    fn window_size() {
        let game_hwnd = get_hwnd(PROCESS_WINDOW_NAME.to_owned()).unwrap();
        let window_info = get_window_info(game_hwnd).unwrap();
        println!("{:?}", window_info);
    }
    #[test]
    fn test_window() {
        let game_hwnd = get_hwnd(PROCESS_WINDOW_NAME.to_owned()).unwrap();
        unsafe {
            let style = windows_sys::Win32::UI::WindowsAndMessaging::GetWindowLongPtrA(
                game_hwnd,
                windows_sys::Win32::UI::WindowsAndMessaging::GWL_STYLE,
            ) as u32;

            let is_borderless = (style
                & (windows_sys::Win32::UI::WindowsAndMessaging::WS_BORDER
                    | windows_sys::Win32::UI::WindowsAndMessaging::WS_CAPTION))
                == 0;

            assert!(is_borderless, "Window should be borderless.");
        }
    }
}
