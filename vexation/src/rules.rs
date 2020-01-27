use super::{Table,Card,Rules, Well, GameObject};

pub trait TVVariant {
    fn size() -> usize;
}
pub struct TetraVex<V:TVVariant> { _dummy : V }
pub struct Two {}
impl TVVariant for Two {
    fn size() -> usize { 2 }
}
pub struct Three {}
impl TVVariant for Three {
    fn size() -> usize { 3 }
}
pub struct Four {}
impl TVVariant for Four {
    fn size() -> usize { 4 }
}
pub struct Five {}
impl TVVariant for Five {
    fn size() -> usize { 5 }
}
impl <V:TVVariant> TetraVex<V> {
    fn get_well(table : &Table, x : usize, y : usize) -> &Well {
        table.well((x * V::size() + y) * 2)
    }
}
impl <V:TVVariant> Rules for TetraVex<V> {
    fn table_size() -> (u32,u32) { (V::size() as u32 *54 * 2 + 32 , V::size() as u32*54 + 17 + 28) }
    fn new_game(tbl : &mut Table) {
        let mut cards = Card::deck(V::size());
        let size = V::size();
        for i in 0..size {
            for j in 0..size {
                tbl.add_well((i as i32 * 54 + 10,j  as i32 * 54 + 20), None);
                tbl.add_well((i as i32 * 54 + 10+ 10 + size as i32 * 54,j as i32 * 54 + 20 ), cards.pop());
            }
        }
    }
    fn can_place_well(table: &Table, well_id: usize, card: Card)  -> bool{
        if !(table.well(well_id).card.is_none()) { return false };
        if well_id % 2 == 0 {
            let x = well_id/2 / V::size();
            let y = well_id/2 % V::size();
            if x > 0 {
                if let Some(other) = Self::get_well(table, x - 1, y).card {
                    if other.values[3] != card.values[2] { return false; }
                }
            }
            if y > 0 {
                if let Some(other) = Self::get_well(table, x, y - 1).card {
                    if other.values[1] != card.values[0] { return false; }
                }
            }
            if x < V::size() - 1 {
                if let Some(other) = Self::get_well(table, x + 1, y).card {
                    if other.values[2] != card.values[3] { return false; }
                }
            }
            if y < V::size() - 1 {
                if let Some(other) = Self::get_well(table, x, y + 1).card {
                    if other.values[0] != card.values[1] { return false; }
                }
            }
            true
        } else { true }
    }  
    fn can_skim_well(well: &Well) -> bool {
        well.card.is_some()
    }
    fn well_clicked(tbl: &mut Table, well_id: usize) {
        if well_id % 2 == 0 && tbl.well(well_id).card.is_some() {
            for i in 0..V::size()*V::size() {
                if tbl.well(i *2 + 1).card.is_none() {
                    tbl.shift_then(GameObject::Well(well_id),GameObject::Well(i*2+1), Box::new(|tbl| {
                        tbl.end_move();
                    }));
                    return;
                }
            }
        }
    }
    fn game_won(table: &Table) -> bool {
        for i in 0..V::size() {
            for j in 0..V::size() {
                if Self::get_well(table, i, j).card.is_none() { return false; }
            }
        }
        true
    }
    fn placed_in_well(_table: &mut Table, _well_id: usize) {

    }

}