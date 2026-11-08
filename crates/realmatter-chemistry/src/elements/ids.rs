//! Material ids owned by the element update pass.
//!
//! Ids 0–25 match the powder-shell palette. Later ids are electrical and
//! special elements. They are stable so saved worlds keep their meaning.

pub const EMPTY: u16 = 0;
pub const STNE: u16 = 1;
pub const SAND: u16 = 2;
pub const WATR: u16 = 3;
pub const OIL: u16 = 4;
pub const STM: u16 = 5;
pub const FIRE: u16 = 6;
pub const WOOD: u16 = 7;
pub const IRON: u16 = 8;
pub const LAVA: u16 = 9;
pub const ACID: u16 = 10;
pub const SALT: u16 = 11;
pub const SMKE: u16 = 12;
pub const ICE: u16 = 13;
pub const GLAS: u16 = 14;
pub const GUN: u16 = 15;
pub const SPRK: u16 = 16;
pub const SNOW: u16 = 17;
pub const COPR: u16 = 18;
pub const NITR: u16 = 19;
pub const HYGN: u16 = 20;
pub const CO2: u16 = 21;
pub const PLNT: u16 = 22;
pub const GOLD: u16 = 23;
pub const DUST: u16 = 24;
pub const BOMB: u16 = 25;
pub const METL: u16 = 26;
pub const BTRY: u16 = 27;
pub const PSCN: u16 = 28;
pub const NSCN: u16 = 29;
pub const INST: u16 = 30;
pub const INWR: u16 = 31;
pub const WIFI: u16 = 32;
pub const SWCH: u16 = 33;
pub const PUMP: u16 = 34;
pub const GPMP: u16 = 35;
pub const HSWC: u16 = 36;
pub const PBCN: u16 = 37;
pub const LCRY: u16 = 38;
pub const PPIP: u16 = 39;
pub const PIPE: u16 = 40;
pub const EMP: u16 = 41;
pub const SLTW: u16 = 42;
pub const RSST: u16 = 43;
pub const NTCT: u16 = 44;
pub const PTCT: u16 = 45;
pub const INSL: u16 = 46;
pub const PSTN: u16 = 47;
pub const FRME: u16 = 48;
pub const STKM: u16 = 49;
pub const STK2: u16 = 50;
pub const FIGH: u16 = 51;
pub const SOAP: u16 = 52;
pub const CLNE: u16 = 53;
pub const PCLN: u16 = 54;
pub const PRTI: u16 = 55;
pub const PRTO: u16 = 56;
pub const PLUT: u16 = 57;
pub const NEUT: u16 = 58;
pub const PHOT: u16 = 59;
pub const ELEC: u16 = 60;
pub const BRMT: u16 = 61;

pub const CHANNELS: usize = 100;
pub const MAX_FIGHTERS: usize = 100;
pub const ATM: f32 = 101_325.0;

/// Conducts under the reference `PROP_CONDUCTS` rule.
/// `INST` is intentionally absent: only a PSCN flood charges it.
pub fn conducts(id: u16) -> bool {
    matches!(
        id,
        METL | IRON | COPR | PSCN | NSCN | INWR | NTCT | PTCT | WATR | SLTW | RSST
    )
}

pub fn life_decays(id: u16) -> bool {
    matches!(
        id,
        SPRK | WATR
            | SLTW
            | PSCN
            | NSCN
            | INST
            | INWR
            | NTCT
            | PTCT
            | RSST
            | METL
            | IRON
            | COPR
            | EMP
    )
}

pub fn is_solid_obstacle(id: u16) -> bool {
    !matches!(
        id,
        EMPTY
            | SAND
            | WATR
            | OIL
            | STM
            | FIRE
            | LAVA
            | ACID
            | SALT
            | SMKE
            | GUN
            | SNOW
            | NITR
            | HYGN
            | CO2
            | GOLD
            | DUST
            | SLTW
            | RSST
            | SOAP
            | NEUT
            | PHOT
            | ELEC
    )
}

pub fn is_mobile_matter(id: u16) -> bool {
    matches!(
        id,
        SAND | WATR
            | OIL
            | STM
            | FIRE
            | LAVA
            | ACID
            | SALT
            | SMKE
            | GUN
            | SNOW
            | NITR
            | HYGN
            | CO2
            | GOLD
            | DUST
            | SLTW
            | RSST
            | SOAP
            | NEUT
            | PHOT
            | ELEC
    )
}

/// Room-temperature metals that warm slightly when a spark is born.
pub fn warms_when_sparked(id: u16) -> bool {
    matches!(id, METL | PSCN | NSCN | IRON)
}

pub fn channel_from_temp(temp: f32) -> usize {
    let mut c = ((temp - 73.15) / 100.0 + 1.0) as i32;
    if c < 0 {
        c = 0;
    }
    if c >= CHANNELS as i32 {
        c = CHANNELS as i32 - 1;
    }
    c as usize
}

pub fn base_rgba(id: u16) -> (u8, u8, u8, u8) {
    match id {
        EMPTY => (0, 0, 0, 0),
        STNE => (128, 128, 128, 255),
        SAND => (217, 194, 122, 255),
        WATR => (79, 195, 247, 220),
        OIL => (141, 110, 99, 230),
        STM => (176, 190, 197, 140),
        FIRE => (255, 111, 0, 255),
        WOOD => (109, 76, 65, 255),
        IRON => (144, 164, 174, 255),
        LAVA => (255, 61, 0, 255),
        ACID => (118, 255, 3, 220),
        SALT => (238, 238, 238, 255),
        SMKE => (97, 97, 97, 120),
        ICE => (179, 229, 252, 230),
        GLAS => (128, 222, 234, 180),
        GUN => (66, 66, 66, 255),
        SPRK => (255, 240, 80, 255),
        SNOW => (230, 240, 255, 255),
        COPR => (184, 115, 51, 255),
        NITR => (230, 220, 80, 220),
        HYGN => (210, 230, 255, 90),
        CO2 => (180, 190, 180, 140),
        PLNT => (46, 140, 60, 255),
        GOLD => (212, 175, 55, 255),
        DUST => (160, 140, 110, 255),
        BOMB => (90, 40, 40, 255),
        METL => (64, 64, 96, 255),
        BTRY => (133, 133, 5, 255),
        PSCN => (128, 80, 80, 255),
        NSCN => (80, 80, 128, 255),
        INST => (64, 64, 57, 255),
        INWR => (84, 65, 65, 255),
        WIFI => (64, 160, 96, 255),
        SWCH => (16, 59, 17, 255),
        PUMP => (10, 10, 59, 255),
        GPMP => (10, 59, 59, 255),
        HSWC => (59, 10, 10, 255),
        PBCN => (59, 29, 10, 255),
        LCRY => (80, 80, 80, 255),
        PPIP => (68, 68, 102, 255),
        PIPE => (60, 60, 60, 255),
        EMP => (102, 170, 255, 255),
        SLTW => (61, 92, 255, 220),
        RSST => (249, 91, 73, 230),
        NTCT => (80, 80, 64, 255),
        PTCT => (64, 80, 80, 255),
        INSL => (107, 83, 68, 255),
        PSTN => (170, 170, 85, 255),
        FRME => (136, 136, 136, 255),
        STKM => (240, 208, 160, 255),
        STK2 => (160, 192, 240, 255),
        FIGH => (255, 224, 160, 255),
        SOAP => (245, 245, 220, 230),
        CLNE => (200, 224, 0, 255),
        PCLN => (0, 224, 200, 255),
        PRTI => (144, 0, 144, 255),
        PRTO => (112, 0, 112, 255),
        PLUT => (58, 122, 58, 255),
        NEUT => (128, 192, 255, 255),
        PHOT => (255, 255, 128, 255),
        ELEC => (255, 255, 32, 255),
        BRMT => (112, 80, 64, 255),
        _ => (255, 0, 255, 255),
    }
}
