use realmatter_tpt::*;
fn main() {
    let mut n = 0;
    let mut names = Vec::new();
    for t in 1..PT_NUM as u16 {
        let def = element(t);
        if !def.enabled { continue; }
        if def.has_update && updates::update_fn(t).is_none() {
            n += 1;
            names.push(def.abbr);
        }
    }
    println!("unimplemented update count: {}", n);
    println!("{}", names.join(" "));
}