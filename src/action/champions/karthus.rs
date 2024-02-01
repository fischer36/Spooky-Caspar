use std::time::Instant;

use super::Champion;

pub struct Karthus {
    pub q_cooldown: Instant,
}

impl Champion for Karthus {}
