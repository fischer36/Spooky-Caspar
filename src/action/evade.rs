use crate::{
    action::champions::Champion,
    io,
    overlay::overlay::{RenderCircular, RenderMissile, RenderObject},
    point2,
    sdk::game::Game,
    sdk::player_manager::{ActiveSpell, Skillshot},
    sdk::walls,
    GameTasks, Visuals,
};
use std::time::{Duration, Instant};

pub enum Spell {
    RectangleProjectile {
        id: usize,
        expires_at: Instant,
        start_pos: nc::na::Point2<f32>,
        end_pos: nc::na::Point2<f32>,
        width: f32,
        evade_shape: Option<EvadeShape>,
    },
    Circular {
        id: usize,
        expires_at: Instant,
        center: nc::na::Point2<f32>,
        radius: f32,
        evade_shape: Option<EvadeShape>,
    },
    Other {
        id: usize,
        expires_at: Instant,
    },
}

impl Spell {
    pub fn new(active_spell: &ActiveSpell, game: &Game) -> Self {
        let local_player_position = point2!(game.local_player.position.x, game.local_player.position.z);

        let evade_shape = EvadeShape::new(&local_player_position, active_spell);
     
        match active_spell.ability.skillshot {
            Skillshot::Circular { radius, dodge } if dodge => Spell::Circular {
                id: active_spell.ability.base_address,
                expires_at: Instant::now() + Duration::from_secs_f32(0.8),
                center: point2!(active_spell.end_pos.x, active_spell.end_pos.z),
                radius,
                evade_shape,
            },
            Skillshot::Linear {
                speed,
                width,
                dodge,
                range,
            } if dodge => {
                let expiry = Instant::now() + Duration::from_secs_f32((range / speed) + 0.25);
                Spell::RectangleProjectile {
                    id: active_spell.ability.base_address,
                    expires_at: expiry,
                    start_pos: point2!(active_spell.start_pos.x, active_spell.start_pos.z),
                    end_pos: point2!(active_spell.end_pos.x, active_spell.end_pos.z),
                    width,
                    evade_shape,
                }
            }
            _ => Spell::Other {
                id: active_spell.ability.base_address,
                expires_at: Instant::now() + Duration::from_secs_f32(2.5),
            },
        }
    }

    pub fn generate_points(
        &self,
        player_position: &nc::na::Point2<f32>,
        player_radius: f32,
    ) -> Vec<nc::na::Point2<f32>> {
        let mut points: Vec<nc::na::Point2<f32>> = match self {
            Spell::Circular {
                center,
                radius,
                evade_shape,
                ..
            } => {
                let mut evasion_points = Vec::with_capacity(15);
                let evasion_radius = radius + player_radius + 50.0; // Distance needed to safely evade

                for i in (0..360).step_by(24) {
                    let angle_rad = i as f32 * std::f32::consts::PI / 180.0;
                    let rotated_x = evasion_radius * angle_rad.cos();
                    let rotated_y = evasion_radius * angle_rad.sin();
                    let evasion_point = point2!(center.x + rotated_x, center.y + rotated_y);

                    evasion_points.push(evasion_point);
                }
                evasion_points.sort_by(|a, b| {
                    let dist_squared_a = (a - player_position).norm_squared();
                    let dist_squared_b = (b - player_position).norm_squared();
                    dist_squared_a
                        .partial_cmp(&dist_squared_b)
                        .unwrap_or(std::cmp::Ordering::Equal)
                });
                evasion_points
            }
            Spell::RectangleProjectile {
                start_pos,
                end_pos,
                width,
                evade_shape,
                ..
            } => {
                // This will generate path vectors with an overestimated length to avoid the projectile, I fix this by adjusting the length in the "to_evade" function.
                let mut evasion_points = Vec::with_capacity(15);
                let evasion_radius = width / 2.0 + player_radius; // Distance needed to safely evade

                for i in (0..360).step_by(24) {
                    let angle_rad = i as f32 * std::f32::consts::PI / 180.0;
                    let rotated_x = evasion_radius * angle_rad.cos();
                    let rotated_y = evasion_radius * angle_rad.sin();
                    let evasion_point = point2!(player_position.x + rotated_x, player_position.y + rotated_y);

                    evasion_points.push(evasion_point);
                }

                evasion_points.sort_by(|a, b| {
                    let distance_a = nc::na::distance(a, &start_pos) + nc::na::distance(a, &end_pos);
                    let distance_b = nc::na::distance(b, &start_pos) + nc::na::distance(b, &end_pos);
                    distance_b.partial_cmp(&distance_a).unwrap_or(std::cmp::Ordering::Equal)
                });

                evasion_points
            }
            _ => Vec::new(),
        };

        points
    }

    pub fn try_render_and_get_screen_pos(&self, game: &Game, visuals: &Visuals) -> Option<RenderObject> {
        match self {
            Spell::RectangleProjectile {
                id,
                expires_at,
                start_pos,
                end_pos,
                width,
                evade_shape,
            } => {
                if let Some(screen_start_pos) =
                    game.world_to_screen(&nc::na::Point3::new(start_pos.x, 0.0, start_pos.y))
                {
                    if let Some(screen_end_pos) = game.world_to_screen(&nc::na::Point3::new(end_pos.x, 0.0, end_pos.y))
                    {
                        return Some(RenderObject::Missile(RenderMissile {
                            name: id.to_string(),
                            start_pos: screen_start_pos,
                            end_pos: screen_end_pos,
                            width: *width,
                            color: visuals.color,
                        }));
                    }
                }
            }
            Spell::Circular {
                id,
                expires_at,
                center,
                radius,
                evade_shape,
            } => {
                if let Some(screen_pos) = game.world_to_screen(&nc::na::Point3::new(center.x, 0.0, center.y)) {
                    return Some(RenderObject::Circular(RenderCircular {
                        name: id.to_string(),
                        pos: screen_pos,
                        radius: *radius,
                        color: visuals.color,
                    }));
                }
            }
            Spell::Other { id, expires_at } => return None,
        }
        return None;
    }
}
pub struct EvadeShape {
    shape: nc::shape::ShapeHandle<f32>,
    isometry: nc::na::Isometry2<f32>,
}

impl EvadeShape {
    pub fn new(
        local_player_position: &nc::na::Point2<f32>,
        active_spell: &crate::sdk::player_manager::ActiveSpell,
    ) -> Option<Self> {
        if nc::na::distance(
            local_player_position,
            &point2!(active_spell.start_pos.x, active_spell.start_pos.z),
        ) > 1500.0
            && nc::na::distance(
                local_player_position,
                &point2!(active_spell.end_pos.x, active_spell.end_pos.z),
            ) > 1500.0
            || active_spell.ability.skillshot == crate::sdk::player_manager::Skillshot::False
        {
            return None;
        }

        match active_spell.ability.skillshot {
            crate::sdk::player_manager::Skillshot::Circular { radius, dodge } if dodge => {
                let shape = nc::shape::ShapeHandle::new(nc::shape::Ball::new(radius));
                let isometry = ncollide2d::na::Isometry2::new(
                    nc::na::Vector2::new(active_spell.end_pos.x, active_spell.end_pos.z),
                    0.0,
                );
                return Some(EvadeShape { shape, isometry });
            }
            crate::sdk::player_manager::Skillshot::Linear {
                speed: _,
                width,
                dodge,
                range,
            } if dodge => {


                let direction = (active_spell.end_pos - active_spell.start_pos).normalize();
                let end_pos = active_spell.start_pos + (direction * (range + width));
                let length = nc::na::distance(
                    &point2!(active_spell.start_pos.x, active_spell.start_pos.z),
                    &point2!(end_pos.x, end_pos.z),
                );

                let shape = nc::shape::ShapeHandle::new(nc::shape::Cuboid::new(nc::na::Vector2::new(
                    (length) / 2.0,
                    (width) / 2.0,
                )));

                let angle: f32 = direction.z.atan2(direction.x);
                let midpoint = nc::na::Vector2::new(
                    (active_spell.start_pos.x + active_spell.end_pos.x) / 2.0,
                    (active_spell.start_pos.z + active_spell.end_pos.z) / 2.0,
                );
                // Create the isometry with the calculated rotation
                let isometry = ncollide2d::na::Isometry2::new(nc::na::Vector2::new(midpoint.x, midpoint.y), angle);

                return Some(EvadeShape { shape, isometry });
            }
            _ => {
                println!("None2");
                return None;
            }
        }
    }

    pub fn is_point_colliding(&self, point: &nc::na::Point2<f32>, hitbox_radius: f32) -> Option<f32> {
        let point_shape = nc::shape::Ball::new(hitbox_radius); // Small radius for the point
        let point_isometry = nc::na::Isometry2::new(nc::na::Vector2::new(point.x, point.y), nc::na::zero());
        let shape_ref: &(dyn ncollide2d::shape::Shape<f32> + 'static) = &*self.shape;
        // Use contact query to check for collision within a margin
        if let Some(contact) = nc::query::contact(
            &self.isometry,
            shape_ref,
            &point_isometry,
            &*nc::shape::ShapeHandle::new(point_shape),
            10.0,
        ) {
            return Some(contact.depth);
        }
        None
    }

    pub fn distance_to_point(&self, point: &nc::na::Point2<f32>) -> f32 {
        let point_shape = nc::shape::Ball::new(0.0); // Radius 0, as we only need the point's position
        let point_isometry = nc::na::Isometry2::new(nc::na::Vector2::new(point.x, point.y), nc::na::zero()); // angle used to be nalgeba::zero()

      
        ncollide2d::query::distance(
            &self.isometry,
            &*self.shape,
            &point_isometry,
            &*nc::shape::ShapeHandle::new(point_shape),
        )
    }
}

pub fn to_evade_or_not_to_evade(
    game: &Game,
    game_tasks: &mut GameTasks,
    spell: &Spell,
    champion_module: &Box<dyn Champion>,
) {
    //println!("{:?},", spell);
    match spell {
        Spell::RectangleProjectile {
            evade_shape,
            start_pos: projectile_start,
            width: size,
            expires_at,
            ..
        }
        | Spell::Circular {
            evade_shape,
            center: projectile_start,
            radius: size,
            expires_at,
            ..
        } => {
            if let Some(evade_shape) = evade_shape {
                //println!("{:?}", evade_shape.s);
                let player_pos = &point2!(game.local_player.position.x, game.local_player.position.z);
                if let Some(collision_depth) =
                    evade_shape.is_point_colliding(player_pos, game.local_player.gameplay_radius)
                {
                    // if Instant::now() + Duration::from_secs_f32(evade_duration - 0.01) > *expires_at {
                    //     champion_module.spell_evade(&point);
                    //     println!("Cant dodge in time, trying to spell evade");
                    //     return;
                    // }
                    let mut evasion_points = spell.generate_points(player_pos, game.local_player.gameplay_radius);
                    evasion_points.retain(|point| {
                        !is_point_in_any_hazard_shape(game_tasks, point, game.local_player.gameplay_radius)
                    });
                    for point in evasion_points {
                        if walls::is_point_in_wall(&point) {
                            println!("wall");
                            continue;
                        }

                        let distance = nc::na::distance(&player_pos, &point);
                        let evade_duration = distance
                            / game
                                .local_player
                                .get_movement_speed(game.process_handle)
                                .expect("EVADE DURATION");

                        // Calculate the required distance to the point. Shorten the vector to only move the necessary ammount.
                        let required_distance =
                            evade_shape.distance_to_point(&point) + game.local_player.gameplay_radius;

                        // Calculate the direction vector and normalize it
                        let direction_vector: nc::na::Vector2<f32> = (point - player_pos).normalize();

                        // Scale the direction vector to the required distance
                        let adjusted_point = player_pos + direction_vector * required_distance;

                        game_tasks.evading_too = (
                            adjusted_point,
                            Instant::now() + Duration::from_secs_f32(evade_duration + 0.05),
                        );
                        execute_evasion(&game, game_tasks, adjusted_point);
                        return;
                    }
                } else {
                    println!("not gonna hit");
                }
            }
        }
        Spell::Other { .. } => {
            println!("other psell");
            return;
        }
    }
}
pub fn is_point_in_any_hazard_shape(game_tasks: &crate::GameTasks, point: &nc::na::Point2<f32>, pradius: f32) -> bool {
    for spell in game_tasks.active_spell_list.iter() {
        match spell {
            Spell::RectangleProjectile { evade_shape, .. } | Spell::Circular { evade_shape, .. } => {
                if let Some(evade_shape) = evade_shape {
                    if evade_shape.is_point_colliding(point, pradius).is_some() {
                        return true;
                    }
                }
            }
            _ => (),
        }
    }
    return false;
}

pub fn execute_evasion(game: &crate::sdk::game::Game, game_tasks: &mut crate::GameTasks, point: nc::na::Point2<f32>) {
    log::info!("Executing evasion");

    let original_pos = io::output::get_cursor_pos();
    let scale = game.game_to_screen_scaling;
    if let Some(screen_evade_point) = game.world_to_screen(&nc::na::Point3::new(point.x, 0.0, point.y)) {
        std::thread::spawn(move || {
            //input::block_input(true);

            //println!("{:?}", original_pos);

            io::output::key_send('t');
            io::output::cursor_move(screen_evade_point.x * scale.x, screen_evade_point.y * scale.y);

            io::output::send_mouse_click(io::output::MouseButton::Right);
            std::thread::sleep(std::time::Duration::from_secs_f32(0.10));
            //input::cursor_move(original_pos.x * scale.x, original_pos.y * scale.y);
            //input::block_input(false);
            std::thread::sleep(std::time::Duration::from_secs_f32(0.10));
            io::output::key_send('t');
            //std::thread::sleep(std::time::Duration::from_secs_f32(evade_duration / 1000.0));
            //std::thread::sleep(std::time::Duration::from_secs_f32(evade_duration));
        });
    }
}
