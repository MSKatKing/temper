use std::ops::RangeInclusive;
use bevy_ecs::prelude::Resource;
use rand::RngExt;

#[derive(Resource)]
pub struct WeatherStatus {
    is_raining: bool,
    is_thundering: bool,

    raining_toggle: u32,
    thundering_toggle: u32,
    clear_cooldown: u32,
}

impl WeatherStatus {
    const RAIN_ON_TICKS: RangeInclusive<u32> = 12000..=24000;
    const RAIN_OFF_TICKS: RangeInclusive<u32> = 12000..=180000;
    
    const THUNDER_ON_TICKS: RangeInclusive<u32> = 3600..=15600;
    const THUNDER_OFF_TICKS: RangeInclusive<u32> = 12000..=180000;
    
    pub fn init() -> Self {
        let mut rng = rand::rng();
        
        Self {
            is_raining: false,
            is_thundering: false,
            
            raining_toggle: rng.random_range(Self::RAIN_OFF_TICKS),
            thundering_toggle: rng.random_range(Self::THUNDER_OFF_TICKS),
            clear_cooldown: 0,
        }
    }
    
    /// Ticks the weather system. Returns whether the weather flags were updated.
    pub fn tick(&mut self) -> bool {
        match self.clear_cooldown.checked_sub(1) {
            Some(cooldown) => {
                self.clear_cooldown = cooldown;
                return false;
            }
            None => {}
        }

        let rain_changed = match self.raining_toggle.checked_sub(1) {
            Some(toggle) => {
                self.raining_toggle = toggle;
                false
            }
            None => {
                self.is_raining = !self.is_raining;
                
                self.raining_toggle = if self.is_raining {
                    rand::rng().random_range(Self::RAIN_ON_TICKS)
                } else {
                    rand::rng().random_range(Self::RAIN_OFF_TICKS)
                };
                
                true
            }
        };

        let thunder_changed = match self.thundering_toggle.checked_sub(1) {
            Some(toggle) => {
                self.thundering_toggle = toggle;
                false
            }
            None => {
                self.is_thundering = !self.is_thundering;
                
                self.thundering_toggle = if self.is_thundering {
                    rand::rng().random_range(Self::THUNDER_ON_TICKS)
                } else {
                    rand::rng().random_range(Self::THUNDER_OFF_TICKS)
                };
                
                true
            }
        };

        rain_changed || thunder_changed
    }

    pub fn rain(&mut self) {
        self.clear_cooldown = 0;
        self.is_raining = true;
        self.raining_toggle = rand::rng().random_range(Self::RAIN_ON_TICKS);
    }

    pub fn thunder(&mut self) {
        self.clear_cooldown = 0;
        self.is_thundering = true;
        self.thundering_toggle = rand::rng().random_range(Self::THUNDER_ON_TICKS);
    }

    pub fn clear(&mut self) {
        self.is_raining = false;
        self.is_thundering = false;
        self.raining_toggle = 0;
        self.thundering_toggle = 0;

        self.clear_cooldown = rand::rng().random_range(Self::RAIN_OFF_TICKS);
    }
    
    pub fn rain_amount(&self) -> f32 {
        if self.is_raining {
            1.0
        } else {
            0.0
        }
    }
    
    pub fn thunder_amount(&self) -> f32 {
        if self.is_thundering {
            1.0
        } else {
            0.0
        }
    }
}
