use realmatter_tpt::*;

fn cnt(sim: &Sim, t: u16) -> usize {
    sim.parts.iter().filter(|p| p.t == t).count()
}

fn main() {
    // --- Portal round trip: PRTI absorbs SAND, PRTO emits it. -------------
    let mut sim = Sim::new(64, 64);
    sim.paint(10, 10, elements_generated::pt::PRTI, 0);
    sim.paint(40, 40, elements_generated::pt::PRTO, 0);
    sim.paint(11, 10, elements_generated::pt::SAND, 0);
    let before = cnt(&sim, elements_generated::pt::SAND);
    for _ in 0..4 {
        sim.step();
    }
    let stored = sim.portal.len();
    let mut emitted = 0;
    for _ in 0..10 {
        sim.step();
        emitted = cnt(&sim, elements_generated::pt::SAND);
        if emitted > 0 {
            break;
        }
    }
    println!("portal: sand_before={before} stored={stored} emitted={emitted}");

    // --- Wireless: WIFI channel lights a conductor ring. ------------------
    let mut sim = Sim::new(48, 48);
    sim.paint(20, 20, elements_generated::pt::WIFI, 0);
    sim.paint(24, 20, elements_generated::pt::PSCN, 0);
    sim.paint(23, 20, elements_generated::pt::SPRK, 0);
    let mut wireless_seen = false;
    for _ in 0..20 {
        sim.step();
        if sim.wireless.active_count() > 0 {
            wireless_seen = true;
            break;
        }
    }
    println!("wireless: active_seen={wireless_seen} count={}", sim.wireless.active_count());

    // --- Stickman spawn + walk. ------------------------------------------
    let mut sim = Sim::new(64, 64);
    for x in 0..64 {
        sim.paint(x, 60, elements_generated::pt::BRCK, 0);
    }
    let head = sim.spawn_entity(entity::Entity::Player1, 32, 20);
    let start_x = head.map(|h| (sim.parts[h].x, sim.parts[h].y));
    sim.set_entity_input(entity::Entity::Player1, entity::input::RIGHT);
    for _ in 0..10 {
        sim.step();
    }
    let end_x = head.map(|h| (sim.parts[h].x, sim.parts[h].y));
    println!("stickman: head={head:?} start={start_x:?} end={end_x:?} alive={}", head.is_some());

    // --- Piston extension. ----------------------------------------------
    let mut sim = Sim::new(64, 64);
    sim.paint(20, 30, elements_generated::pt::PSTN, 0);
    sim.paint(18, 30, elements_generated::pt::PSCN, 0);
    sim.paint(19, 30, elements_generated::pt::SPRK, 0);
    let pstn_before = cnt(&sim, elements_generated::pt::PSTN);
    for _ in 0..20 {
        sim.step();
    }
    let pstn_after = cnt(&sim, elements_generated::pt::PSTN);
    println!("piston: before={pstn_before} after={pstn_after}");

    // --- SOAP links persist. --------------------------------------------
    let mut sim = Sim::new(64, 64);
    for x in 30..40 {
        sim.paint(x, 30, elements_generated::pt::SOAP, 0);
    }
    for _ in 0..5 {
        sim.step();
    }
    let linked = sim
        .parts
        .iter()
        .filter(|p| p.t == elements_generated::pt::SOAP && (p.ctype & 0x02) != 0)
        .count();
    println!("soap: linked_front={linked}");

    println!("coverage={:?}", coverage());
}
