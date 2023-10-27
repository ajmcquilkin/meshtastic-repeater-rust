use std::time::Instant;

use super::types::CurrentTime;

pub fn get_current_time() -> CurrentTime {
    Instant::now()
}
