//! Per-cell element fields that the shared `Cell` struct does not carry.
//!
//! `ctype`, `life`, and `tmp*` follow the reference particle fields closely
//! enough for electronics and specials. The grid index is the particle
//! identity: moving a cell swaps the aux record with it.

use realmatter_core::cell::{cell_flags, Cell};

use super::ids::*;

#[derive(Clone, Copy, Debug)]
pub struct ElemAux {
    pub ctype: u16,
    pub life: i16,
    pub tmp: i32,
    pub tmp2: i32,
    pub tmp3: i32,
}

impl Default for ElemAux {
    fn default() -> Self {
        Self {
            ctype: 0,
            life: 0,
            tmp: 0,
            tmp2: 0,
            tmp3: 0,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Player {
    pub present: bool,
    pub index: i32,
    pub legs: [f32; 16],
    pub accs: [f32; 8],
    pub comm: u8,
    pub pcomm: u8,
    pub elem: u16,
    pub frames: u16,
    pub life: i16,
    pub vx: f32,
    pub vy: f32,
    pub x: f32,
    pub y: f32,
}

impl Default for Player {
    fn default() -> Self {
        Self {
            present: false,
            index: -1,
            legs: [0.0; 16],
            accs: [0.0; 8],
            comm: 0,
            pcomm: 0,
            elem: DUST,
            frames: 0,
            life: 100,
            vx: 0.0,
            vy: 0.0,
            x: 0.0,
            y: 0.0,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct PortalCargo {
    pub material: u16,
    pub temp: f32,
    pub ctype: u16,
}

#[derive(Clone, Debug)]
pub struct ElementSim {
    pub aux: Vec<ElemAux>,
    pub players: [Player; 2],
    pub fighters: Vec<Player>,
    pub fighcount: u16,
    pub wireless: [[u8; CHANNELS]; 2],
    pub rng: u32,
    pub tick: u64,
    pub portals: Vec<Vec<PortalCargo>>,
}

impl Default for ElementSim {
    fn default() -> Self {
        Self::new()
    }
}

impl ElementSim {
    pub fn new() -> Self {
        Self {
            aux: Vec::new(),
            players: [Player::default(), Player::default()],
            fighters: (0..MAX_FIGHTERS).map(|_| Player::default()).collect(),
            fighcount: 0,
            wireless: [[0; CHANNELS]; 2],
            rng: 0xA341_316C,
            tick: 0,
            portals: (0..CHANNELS).map(|_| Vec::new()).collect(),
        }
    }

    pub fn ensure(&mut self, n: usize) {
        if self.aux.len() < n {
            self.aux.resize(n, ElemAux::default());
        }
    }

    pub fn clear(&mut self) {
        for a in &mut self.aux {
            *a = ElemAux::default();
        }
        self.players = [Player::default(), Player::default()];
        for f in &mut self.fighters {
            *f = Player::default();
        }
        self.fighcount = 0;
        self.wireless = [[0; CHANNELS]; 2];
        self.portals.iter_mut().for_each(|p| p.clear());
    }

    pub fn rng_u32(&mut self) -> u32 {
        let mut x = self.rng;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        if x == 0 {
            x = 0xA341_316C;
        }
        self.rng = x;
        x
    }

    pub fn rng_below(&mut self, n: u32) -> u32 {
        if n == 0 {
            return 0;
        }
        self.rng_u32() % n
    }

    pub fn set_input(&mut self, slot: usize, bits: u8) {
        if slot < 2 {
            self.players[slot].comm = bits;
        }
    }

    /// Prepare `cell` (already filled with the newly painted material) at `index`.
    /// `old` is the cell being replaced. Returns false when the paint is rejected.
    pub fn on_paint(&mut self, cells: &[Cell], index: usize, cell: &mut Cell, old: &Cell) -> bool {
        self.ensure(index + 1);
        let new_id = cell.material;
        if old.material == FIGH && new_id != FIGH {
            self.free_fighter_index(index as i32);
        }
        if old.material == STKM && new_id != STKM {
            self.players[0].present = false;
            self.players[0].index = -1;
        }
        if old.material == STK2 && new_id != STK2 {
            self.players[1].present = false;
            self.players[1].index = -1;
        }
        if old.material == SOAP && new_id != SOAP {
            super::soap::detach(cells, self, index);
        }

        if matches!(old.material, CLNE | PCLN | PBCN | FIGH)
            && new_id != EMPTY
            && new_id != old.material
            && new_id != SPRK
        {
            self.aux[index].ctype = new_id;
            if old.material == FIGH {
                let slot = self.aux[index].tmp;
                if slot >= 0 {
                    if let Some(f) = self.fighters.get_mut(slot as usize) {
                        f.elem = new_id;
                    }
                }
            }
            *cell = *old;
            return true;
        }

        if new_id == SPRK {
            if conducts(old.material) && old.material != SPRK && self.aux[index].life == 0 {
                let life = spark_birth_life(old.material);
                self.aux[index].ctype = old.material;
                self.aux[index].life = life;
                cell.material = SPRK;
                cell.temperature = old.temperature;
                warm_spark(cell);
                cell.vx = 0.0;
                cell.vy = 0.0;
                cell.set_flag(cell_flags::FIXED, true);
                return true;
            }
            if old.material == INST && self.aux[index].life == 0 {
                self.aux[index].ctype = INST;
                self.aux[index].life = 4;
                cell.temperature = old.temperature;
                cell.vx = 0.0;
                cell.vy = 0.0;
                cell.set_flag(cell_flags::FIXED, true);
                return true;
            }
            return false;
        }

        self.aux[index] = ElemAux::default();
        if new_id == SOAP {
            self.aux[index].tmp = -1;
            self.aux[index].tmp2 = -1;
        }
        if new_id == STKM {
            if self.players[0].present && self.players[0].index != index as i32 {
                return false;
            }
            self.bind_player(0, index, cell);
        }
        if new_id == STK2 {
            if self.players[1].present && self.players[1].index != index as i32 {
                return false;
            }
            self.bind_player(1, index, cell);
        }
        if new_id == FIGH {
            let slot = self.alloc_fighter(index);
            if slot < 0 {
                return false;
            }
            self.aux[index].tmp = slot;
            self.aux[index].ctype = DUST;
        }
        if new_id == NEUT {
            self.aux[index].tmp = 0;
        }
        true
    }

    fn bind_player(&mut self, slot: usize, index: usize, cell: &Cell) {
        let mut p = Player::default();
        p.present = true;
        p.index = index as i32;
        p.life = 100;
        p.x = 0.0;
        p.y = 0.0;
        // x/y filled by the caller via cell coordinates in init_legs.
        let _ = cell;
        self.players[slot] = p;
    }

    pub fn init_player_at(&mut self, slot: usize, index: usize, x: i32, y: i32) {
        if slot >= 2 {
            return;
        }
        let p = &mut self.players[slot];
        p.present = true;
        p.index = index as i32;
        p.life = 100;
        p.x = x as f32;
        p.y = y as f32;
        p.vx = 0.0;
        p.vy = 0.0;
        init_legs(p, x as f32, y as f32);
    }

    pub fn alloc_fighter(&mut self, index: usize) -> i32 {
        if self.fighcount as usize >= MAX_FIGHTERS {
            return -1;
        }
        let slot = self.fighters.iter().position(|f| !f.present);
        let Some(slot) = slot else {
            return -1;
        };
        let mut p = Player::default();
        p.present = true;
        p.index = index as i32;
        p.life = 100;
        p.elem = DUST;
        self.fighters[slot] = p;
        self.fighcount += 1;
        slot as i32
    }

    pub fn init_fighter_at(&mut self, slot: usize, index: usize, x: i32, y: i32) {
        if slot >= self.fighters.len() {
            return;
        }
        let p = &mut self.fighters[slot];
        p.x = x as f32;
        p.y = y as f32;
        p.index = index as i32;
        p.present = true;
        p.life = 100;
        init_legs(p, x as f32, y as f32);
    }

    pub fn free_fighter_index(&mut self, index: i32) {
        for f in &mut self.fighters {
            if f.present && f.index == index {
                *f = Player::default();
                self.fighcount = self.fighcount.saturating_sub(1);
            }
        }
    }

    pub fn free_fighter_slot(&mut self, slot: usize) {
        if slot < self.fighters.len() && self.fighters[slot].present {
            self.fighters[slot] = Player::default();
            self.fighcount = self.fighcount.saturating_sub(1);
        }
    }

    /// Swap two cells and the element records that belong to them.
    /// Soap link indices and entity anchors are rewritten so they follow the matter.
    pub fn exchange(&mut self, cells: &mut [Cell], a: usize, b: usize) {
        if a == b || a >= cells.len() || b >= cells.len() {
            return;
        }
        cells.swap(a, b);
        self.ensure(cells.len());
        self.aux.swap(a, b);
        let ai = a as i32;
        let bi = b as i32;
        for (n, aux) in self.aux.iter_mut().enumerate() {
            if n < cells.len() && cells[n].material == SOAP {
                remap(&mut aux.tmp, ai, bi);
                remap(&mut aux.tmp2, ai, bi);
            }
        }
        for p in &mut self.players {
            remap(&mut p.index, ai, bi);
        }
        for f in &mut self.fighters {
            remap(&mut f.index, ai, bi);
        }
    }

    pub fn export_trailer(&self) -> Vec<u8> {
        let mut w = Writer::default();
        w.u32(1);
        w.u32(self.aux.len() as u32);
        for a in &self.aux {
            w.u16(a.ctype);
            w.i16(a.life);
            w.i32(a.tmp);
            w.i32(a.tmp2);
            w.i32(a.tmp3);
        }
        for p in &self.players {
            write_player(&mut w, p);
        }
        w.u16(self.fighcount);
        for f in &self.fighters {
            write_player(&mut w, f);
        }
        w.u32(self.rng);
        w.u64(self.tick);
        for b in &self.wireless[0] {
            w.u8(*b);
        }
        let mut cargo = 0u32;
        for ch in &self.portals {
            cargo += ch.len() as u32;
        }
        w.u32(cargo);
        for (ch, items) in self.portals.iter().enumerate() {
            for item in items {
                w.u16(ch as u16);
                w.u16(item.material);
                w.f32(item.temp);
                w.u16(item.ctype);
            }
        }
        let mut payload = w.buf;
        let len = payload.len() as u32;
        payload.extend(len.to_le_bytes());
        payload.extend(b"K7AX");
        payload
    }

    pub fn import_payload(&mut self, bytes: &[u8]) -> bool {
        let mut r = Reader { buf: bytes, pos: 0 };
        let Some(ver) = r.u32() else { return false };
        if ver != 1 {
            return false;
        }
        let Some(n) = r.u32() else { return false };
        let n = n as usize;
        self.ensure(n);
        for i in 0..n {
            let Some(ctype) = r.u16() else { return false };
            let Some(life) = r.i16() else { return false };
            let Some(tmp) = r.i32() else { return false };
            let Some(tmp2) = r.i32() else { return false };
            let Some(tmp3) = r.i32() else { return false };
            self.aux[i] = ElemAux {
                ctype,
                life,
                tmp,
                tmp2,
                tmp3,
            };
        }
        for p in &mut self.players {
            if !read_player(&mut r, p) {
                return false;
            }
        }
        let Some(fc) = r.u16() else { return false };
        self.fighcount = fc;
        for f in &mut self.fighters {
            if !read_player(&mut r, f) {
                return false;
            }
        }
        let Some(rng) = r.u32() else { return false };
        let Some(tick) = r.u64() else { return false };
        self.rng = rng;
        self.tick = tick;
        for b in &mut self.wireless[0] {
            let Some(v) = r.u8() else { return false };
            *b = v;
        }
        self.wireless[1] = [0; CHANNELS];
        let Some(cargo) = r.u32() else { return false };
        for p in &mut self.portals {
            p.clear();
        }
        for _ in 0..cargo {
            let Some(ch) = r.u16() else { return false };
            let Some(material) = r.u16() else {
                return false;
            };
            let Some(temp) = r.f32() else { return false };
            let Some(ctype) = r.u16() else { return false };
            let ch = ch as usize;
            if ch < self.portals.len() && self.portals[ch].len() < 16 {
                self.portals[ch].push(PortalCargo {
                    material,
                    temp,
                    ctype,
                });
            }
        }
        true
    }
}

pub fn split_trailer(bytes: &[u8]) -> (&[u8], &[u8]) {
    if bytes.len() < 8 || &bytes[bytes.len() - 4..] != b"K7AX" {
        return (bytes, &[]);
    }
    let len =
        u32::from_le_bytes(bytes[bytes.len() - 8..bytes.len() - 4].try_into().unwrap()) as usize;
    if bytes.len() < 8 + len {
        return (bytes, &[]);
    }
    let start = bytes.len() - 8 - len;
    (&bytes[..start], &bytes[start..start + len])
}

pub fn spark_birth_life(id: u16) -> i16 {
    match id {
        WATR => 6,
        SLTW => 5,
        RSST => 5,
        _ => 4,
    }
}

pub fn warm_spark(cell: &mut Cell) {
    let id = cell.material;
    let _ = id;
}

pub fn warm_new_spark(cell: &mut Cell, underlying: u16) {
    if warms_when_sparked(underlying) && cell.temperature + 10.0 < 673.0 {
        cell.temperature += 10.0;
    }
}

pub fn init_legs(p: &mut Player, x: f32, y: f32) {
    p.legs = [
        x - 1.0,
        y + 6.0,
        x - 1.0,
        y + 6.0,
        x - 3.0,
        y + 12.0,
        x - 3.0,
        y + 12.0,
        x + 1.0,
        y + 6.0,
        x + 1.0,
        y + 6.0,
        x + 3.0,
        y + 12.0,
        x + 3.0,
        y + 12.0,
    ];
    p.accs = [0.0; 8];
    p.comm = 0;
    p.pcomm = 0;
    p.frames = 0;
}

fn remap(v: &mut i32, a: i32, b: i32) {
    if *v == a {
        *v = b;
    } else if *v == b {
        *v = a;
    }
}

pub fn at(
    w: usize,
    h: usize,
    x: i32,
    y: i32,
    dx: i32,
    dy: i32,
    wrap: bool,
) -> Option<(usize, i32, i32)> {
    if w == 0 || h == 0 {
        return None;
    }
    let mut nx = x + dx;
    let mut ny = y + dy;
    if wrap {
        nx = nx.rem_euclid(w as i32);
        ny = ny.rem_euclid(h as i32);
    } else if nx < 0 || ny < 0 || nx >= w as i32 || ny >= h as i32 {
        return None;
    }
    Some((ny as usize * w + nx as usize, nx, ny))
}

pub fn midpoint_blocks(
    cells: &[Cell],
    w: usize,
    h: usize,
    x: i32,
    y: i32,
    nx: i32,
    ny: i32,
) -> bool {
    let mx = (x + nx) / 2;
    let my = (y + ny) / 2;
    if mx < 0 || my < 0 || mx >= w as i32 || my >= h as i32 {
        return false;
    }
    cells[my as usize * w + mx as usize].material == INSL
}

#[derive(Default)]
struct Writer {
    buf: Vec<u8>,
}
impl Writer {
    fn u8(&mut self, v: u8) {
        self.buf.push(v);
    }
    fn u16(&mut self, v: u16) {
        self.buf.extend(v.to_le_bytes());
    }
    fn i16(&mut self, v: i16) {
        self.buf.extend(v.to_le_bytes());
    }
    fn u32(&mut self, v: u32) {
        self.buf.extend(v.to_le_bytes());
    }
    fn i32(&mut self, v: i32) {
        self.buf.extend(v.to_le_bytes());
    }
    fn u64(&mut self, v: u64) {
        self.buf.extend(v.to_le_bytes());
    }
    fn f32(&mut self, v: f32) {
        self.buf.extend(v.to_le_bytes());
    }
}

struct Reader<'a> {
    buf: &'a [u8],
    pos: usize,
}
impl<'a> Reader<'a> {
    fn take(&mut self, n: usize) -> Option<&'a [u8]> {
        if self.pos + n > self.buf.len() {
            return None;
        }
        let s = &self.buf[self.pos..self.pos + n];
        self.pos += n;
        Some(s)
    }
    fn u8(&mut self) -> Option<u8> {
        self.take(1).map(|b| b[0])
    }
    fn u16(&mut self) -> Option<u16> {
        self.take(2)
            .map(|b| u16::from_le_bytes(b.try_into().unwrap()))
    }
    fn i16(&mut self) -> Option<i16> {
        self.take(2)
            .map(|b| i16::from_le_bytes(b.try_into().unwrap()))
    }
    fn u32(&mut self) -> Option<u32> {
        self.take(4)
            .map(|b| u32::from_le_bytes(b.try_into().unwrap()))
    }
    fn i32(&mut self) -> Option<i32> {
        self.take(4)
            .map(|b| i32::from_le_bytes(b.try_into().unwrap()))
    }
    fn u64(&mut self) -> Option<u64> {
        self.take(8)
            .map(|b| u64::from_le_bytes(b.try_into().unwrap()))
    }
    fn f32(&mut self) -> Option<f32> {
        self.take(4)
            .map(|b| f32::from_le_bytes(b.try_into().unwrap()))
    }
}

fn write_player(w: &mut Writer, p: &Player) {
    w.u8(u8::from(p.present));
    w.i32(p.index);
    for v in p.legs {
        w.f32(v);
    }
    for v in p.accs {
        w.f32(v);
    }
    w.u8(p.comm);
    w.u8(p.pcomm);
    w.u16(p.elem);
    w.u16(p.frames);
    w.i16(p.life);
    w.f32(p.vx);
    w.f32(p.vy);
    w.f32(p.x);
    w.f32(p.y);
}

fn read_player(r: &mut Reader, p: &mut Player) -> bool {
    let Some(present) = r.u8() else { return false };
    let Some(index) = r.i32() else { return false };
    let mut legs = [0.0; 16];
    for v in &mut legs {
        let Some(x) = r.f32() else { return false };
        *v = x;
    }
    let mut accs = [0.0; 8];
    for v in &mut accs {
        let Some(x) = r.f32() else { return false };
        *v = x;
    }
    let Some(comm) = r.u8() else { return false };
    let Some(pcomm) = r.u8() else { return false };
    let Some(elem) = r.u16() else { return false };
    let Some(frames) = r.u16() else { return false };
    let Some(life) = r.i16() else { return false };
    let Some(vx) = r.f32() else { return false };
    let Some(vy) = r.f32() else { return false };
    let Some(x) = r.f32() else { return false };
    let Some(y) = r.f32() else { return false };
    *p = Player {
        present: present != 0,
        index,
        legs,
        accs,
        comm,
        pcomm,
        elem,
        frames,
        life,
        vx,
        vy,
        x,
        y,
    };
    true
}
