use std::ops::Add;
use temper_macros::Discriminant;
use temper_text::TextComponent;

#[derive(Clone)]
pub struct BossBarData {
    pub title: TextComponent,
    pub health: f32,
    pub color: BossbarColor,
    pub dividers: BossbarDividers,
    pub flags: BossbarFlags,
}

#[derive(Clone, Discriminant)]
pub enum BossbarColor {
    Pink,
    Blue,
    Red,
    Green,
    Yellow,
    Purple,
    White,
}

#[derive(Clone, Discriminant)]
pub enum BossbarDividers {
    None,
    SixNotches,
    TenNotches,
    TwelveNotches,
    TwentyNotches,
}

#[derive(Clone)]
pub struct BossbarFlags(u8);

impl BossbarFlags {
    const NONE: u8 = 0x0;
    const DARKSKY: u8 = 0x1;
    const DRAGON: u8 = 0x2;
    const FOG: u8 = 0x4;

    pub fn none() -> BossbarFlags {
        BossbarFlags(Self::NONE)
    }
    
    pub fn get(&self) -> u8 { 
        self.0
    }

    pub fn add_fog(&mut self) {
        let _ = self.0.add(Self::FOG);
    }

    pub fn add_dragon_bar(&mut self) {
        let _ = self.0.add(Self::DRAGON);
    }

    pub fn add_darkened_sky(&mut self) {
        let _ = self.0.add(Self::DARKSKY);
    }
}
