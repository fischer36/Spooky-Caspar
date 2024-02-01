pub mod game;
pub mod minion_manager;
pub mod offsets;
pub mod player_manager;
pub mod walls;

// Custom point2 macro because I am unable to use nc::nalgebra point macro for some reason.
#[macro_export]
macro_rules! point2 {
    ($x:expr, $y:expr) => {
        nc::na::Point2::new($x, $y)
    };
}
