use super::{Table,Card,Suit,Rules, Stack, Well, GameObject};
use rand::thread_rng;
use rand::seq::SliceRandom;
use std::cmp::Ordering;


fn refresh_stacks(table: &mut Table) {
    for i in 0..table.stacks().len() {
        if table.stack(i).hidden_point >= table.stack(i).cards.len() {
            table.reveal(i, (table.stack(i).hidden_point + 1 - table.stack(i).cards.len()) as i32);
        }
    }
}
pub struct BakersDozen {}
impl BakersDozen {
    fn badness(cards : &[Card]) -> i32 {
        let mut badness = 0;
        for i in 0..cards.len()-1 {
            let v = cards[i].value;
            badness += cards.iter().skip(i).filter(|x| x.value > v).count() as i32;
        }
        badness
    }
    fn badness_of(table:&Table, obj: GameObject) -> i32 {
        match obj {
            GameObject::Stack(i) => Self::badness(&table.stack(i).cards),
            _ => 0,
        }
    }
    fn badness_add_of(table:&Table, obj: GameObject, card: Card) -> i32 {
        match obj {
            GameObject::Stack(i) => Self::badness_add(&table.stack(i).cards, card),
            _ => 0,
        }
    }
    fn badness_sub_of(table:&Table, obj: GameObject) -> i32 {
        match obj {
            GameObject::Stack(i) => Self::badness_sub(&table.stack(i).cards),
            _ => 0,
        }
    }
    fn badness_delta(table:&Table, source : GameObject, target: GameObject, card : Card ) -> i32  {
        let src_badness_before = Self::badness_of(table, source);
        let target_badness_before = Self::badness_of(table, target);
        let src_badness_after = Self::badness_sub_of(table, source);
        let target_badness_after = Self::badness_add_of(table, target, card);
        src_badness_after + target_badness_after- src_badness_before - target_badness_before
    }
    fn badness_sub(cards : &[Card]) -> i32 {
        Self::badness(&cards[0..=cards.len()-1])
    }
    fn badness_add(cards : &[Card], card : Card) -> i32 {
        let cards_2 : Vec<Card> = cards.iter().chain(vec![card].iter()).map(|c| *c).collect();
        Self::badness(&cards_2)
    }
    fn best_location_for_card(table:&Table, card : Card, other_than: GameObject) -> Option<GameObject> {
        for i in 0..=3 {
            if other_than != GameObject::Well(i) {
                if Self::can_place_well(table.well(i), &vec![card]) {
                    return Some(GameObject::Well(i));
                }
            }
        }
        let mut options : Vec<usize> = Vec::new();
        for i in 0..13 {
            if other_than != GameObject::Stack(i) {
                if Self::can_place_stack(table.stack(i), &vec![card], table) {
                    options.push(i );
                }
            }
        }
        options.sort_by(|a , b| Self::badness_add(&table.stack(*a).cards, card).cmp(&Self::badness_add(&table.stack(*b).cards, card)));
        options.first().map(|x| GameObject::Stack(*x))   
    }
}
impl Rules for BakersDozen {
    fn table_size() -> (u32,u32) { (338,320) }
    fn new_game(table: &mut Table) {
        let cards : Vec<Card> = Card::deck().into_iter().filter(|c| c.value < 13).collect();
        let mut kings = vec![Card {suit: Suit::Hearts, value: 13},Card {suit: Suit::Diamonds, value: 13},Card {suit: Suit::Clubs, value: 13},Card {suit: Suit::Spades, value: 13} ];
        let empty_vec = Vec::new();
        table.add_well((288,32), 0,&empty_vec);
        table.add_well((288,32+48),0, &empty_vec);
        table.add_well((288,32+48*2),0, &empty_vec);
        table.add_well((288,32+48*3),0, &empty_vec);
        let mut start = 0;
        let mut stacks_d = Vec::new();
        let mut kings_pos_vec : Vec<usize> = (1..=13).collect();
        kings_pos_vec.shuffle(&mut thread_rng());
        
        let kings_pos = &kings_pos_vec[0..4];
        for i in 1..=13 {
            let mut v = Vec::new();
            if kings_pos.contains(&i) {
                v.push(kings.pop().unwrap());
            }
            while v.len() < 4 {
                v.push(cards[start]);
                start += 1;
            }
            stacks_d.push(v);
        }
        for i in 1..=13 {
            table.add_stack((32 * ((i - 1) % 7) as i32 + 32 + (if i > 7 { 16 } else { 0}),32+ ((i as i32 -1)/ 7) * 136), &stacks_d[i-1] , 0);
        }
    }
    fn can_split_stack(stack: &Stack, position: usize, _ : &Table) -> bool {
        position < stack.cards.len() && position >= stack.cards.len() - 1       
    }
    fn can_skim_well(well: &Well) -> bool {
        well.cards.len() > 0
    }
    fn can_place_stack(stack: &Stack, cards: &[Card], _ : &Table) -> bool {        
        if let Some(c) = stack.cards.last () {
            cards[0].value + 1 == c.value
        } else {
            false
        }
    }
    fn can_place_well(well: &Well, cards: &[Card]) -> bool { 
        if well.cards.len() > 0 {
            cards.len() == 1 && cards[0].suit == well.cards[0].suit && cards[0].value == well.cards.len() as u8 + 1
        } else { 
            cards.len() == 1 && cards[0].value == 1
        }
        
    }
    fn placed_in_stack(_table: &mut Table, _stack_id: usize, _cards: usize) {
    }
    fn placed_in_well(_table: &mut Table, _well_id: usize, _cards: usize) {
    }
    fn deal_from_deck(_table: &mut Table, _deck_id: usize) {
    }
    fn stack_clicked(table: &mut Table, stack_id: usize, position: usize) {
        let card = &table.stack(stack_id).cards[position];       
        if let Some(loc) = Self::best_location_for_card(table, *card, GameObject::Stack(stack_id)) {                        
            table.shift_then(GameObject::Stack(stack_id), loc,1,Box::new(move |tbl| {
                match loc {
                    GameObject::Stack(i) => Self::placed_in_stack(tbl, i, 1),
                    GameObject::Well(i) => Self::placed_in_well(tbl, i, 1),
                    GameObject::Deck(_) => {},
                };
                tbl.end_move();
            }));
        }
    }
    fn well_clicked(_table: &mut Table, _well_id: usize) {
    }
    
    fn hint(table: &mut Table) {
        let mut moves: Vec<(GameObject, GameObject, Card)> = Vec::new();
        for i in 0..13 {
            if let Some(c) = table.stack(i).cards.last() {
                if let Some(dest) = Self::best_location_for_card(table, *c, GameObject::Stack(i)) {                    
                    moves.push((GameObject::Stack(i),dest,*c));
                }
            }
        }
        moves.sort_by(|(s1,d1, c1), (s2,d2, c2)| Self::badness_delta(table, *s1, *d1,*c1).cmp(&Self::badness_delta(table, *s2,*d2,*c2)) );
        if let Some((s,d,_)) = moves.first() {
            let dd = *d;
            let and_then : Box<dyn FnOnce(&mut Table)> = Box::new(move |tbl| {
                match dd {
                    GameObject::Well(w) => tbl.animate_highlight_well(w, (0,200,100)),
                    GameObject::Stack(w) => tbl.animate_highlight_stack(w, tbl.stack(w).cards.len()-1.min(tbl.stack(w).cards.len()), (0,200,100)),
                    _ => {}
                } 
                
            } );
            match *s {
                GameObject::Well(w) => table.animate_highlight_well_then(w, (100,0,200), and_then),
                GameObject::Stack(s) => { let pos = table.stack(s).cards.len() - 1; table.animate_highlight_stack_then(s, pos, (100,0,200), and_then)},
                _ => {}
            }

        } else {
        }
    }
    fn game_won(table: &Table) -> bool {
        table.stacks().iter().all(|s| s.cards.len() == 0)
    }
}
pub struct Cruel {}
impl Cruel {
    fn end_redeal(table:&mut Table) {        
        let mut cards_count = table.deck(0).cards.len();
        for i in (0..12).into_iter().rev() {
            if cards_count > i * 4 { 
                let num = 4.min(cards_count - i * 4);
                table.shift(GameObject::Deck(0), GameObject::Well(i), num);
                cards_count -= num;
            }
        }
        table.end_move();        
    }
    fn begin_redeal(table: &mut Table) {
        let mut moves = Vec::new();
        for i in 0..12 {
            if table.well(i).cards.len() > 0 { moves.push((GameObject::Well(i),table.well(i).cards.len())) };
        }
        table.multi_shift_then(moves, GameObject::Deck(0), Box::new(|tbl| Self::end_redeal(tbl)))        
    }
    fn best_location_for(table: &mut Table, w : usize) -> Option<usize> {
        let c = table.well(w).cards.last()?;        
        for i in table.wells.iter().rev() {
            if i.id != w && Self::can_place_well(i, &vec![*c][..]) {
                return Some(i.id)
            }
        }
        None
    }
}
impl Rules for Cruel {
    fn table_size() -> (u32,u32) { (256,256+32) }
    fn new_game(table: &mut Table) { 
        let mut cards = Card::deck();
        cards.shuffle(&mut thread_rng());
        let remaining_cards = cards.into_iter().filter(|x| x.value != 1);
        let empty_vec = Vec::new();
        //table.add_well((34,32), 0,&empty_vec);
        table.add_well((32,128), 0,&empty_vec);
        table.add_well((64,128), 0,&empty_vec);
        table.add_well((96,128), 0,&empty_vec);
        table.add_well((128,128), 0,&empty_vec);
        table.add_well((160,128), 0,&empty_vec);
        table.add_well((192,128), 0,&empty_vec);
        table.add_well((32,172), 0,&empty_vec);
        table.add_well((64,172), 0,&empty_vec);
        table.add_well((96,172), 0,&empty_vec);
        table.add_well((128,172), 0,&empty_vec);
        table.add_well((160,172), 0,&empty_vec);
        table.add_well((192,172), 0,&empty_vec);
        table.add_well((64,32), 0,&vec![Card {value: 1, suit: Suit::Hearts}][..]);
        table.add_well((96,32), 0,&vec![Card {value: 1, suit: Suit::Spades}][..]);
        table.add_well((128,32), 0,&vec![Card {value: 1, suit: Suit::Diamonds}][..]);
        table.add_well((160,32), 0,&vec![Card {value: 1, suit: Suit::Clubs}][..]);
        table.add_deck_with_emblem((112,84), &empty_vec);
        let mut idx = 0;
        for c in remaining_cards {
            table.add_cards_to(GameObject::Well(idx), &vec![c][..]);
            idx = (idx + 1) % 12;
        }
    }
    // no stacks in cruel
    fn can_split_stack(_: &Stack, _: usize, _: &Table) -> bool { false }
    fn can_place_stack(_: &Stack, _: &[Card], _ : &Table) -> bool { false }
    fn stack_clicked(_: &mut Table, _: usize, _: usize) { }
    fn placed_in_stack(_: &mut Table, _: usize, _: usize) { }

    fn can_place_well(w: &Well, cs: &[Card]) -> bool {
        if let [c] = cs {
            if let Some(t) = w.cards.last() {
                c.suit == t.suit && if w.id >= 12 {
                    c.value == t.value + 1
                } else {
                    t.value == c.value + 1
                }
            } else { false }
        } else { false }
    }
    fn can_skim_well(w: &Well) -> bool { w.cards.len() > 1 || w.id < 12 && w.cards.len() > 0 }
    fn game_won(tbl: &Table) -> bool { 
        tbl.well(12).cards.len() + tbl.well(13).cards.len() + tbl.well(14).cards.len() + tbl.well(15).cards.len() == 52
     }
    fn well_clicked(table: &mut Table, w: usize) { 
        if w < 12 && table.well(w).cards.len() > 0 {
            if let Some(l) = Self::best_location_for(table,w) {
                table.shift_then(GameObject::Well(w), GameObject::Well(l), 1, Box::new(|tbl| tbl.end_move()));
            }
        }
    }
    fn placed_in_well(_: &mut Table, _: usize, _: usize) { }

    fn deal_from_deck(tbl: &mut Table, _id: usize) {
        if tbl.deck(0).cards.len() > 0 {
            Self::end_redeal(tbl);
        } else {
            Self::begin_redeal(tbl);
        }
    }
    fn hint(table: &mut Table) {
        let mut good_moves = Vec::new();
        let mut okay_moves = Vec::new();
        for i in 0..12 {
            if let Some(j) = Self::best_location_for(table,i) {
                if j >= 12 {
                    good_moves.push((i,j))
                } else {
                    okay_moves.push((i,j))
                }
            }
        }
        // moves to top wells we want from lowest first
        good_moves.sort_by(|(i,_), (j,_)| table.well(*i).cards.last().map(|x| x.value).cmp(&table.well(*j).cards.last().map(|x| x.value)));
        // moves between bottom wells we want from highest first
        okay_moves.sort_by(|(j,_), (i,_)| table.well(*i).cards.last().map(|x| x.value).cmp(&table.well(*j).cards.last().map(|x| x.value)));
        if good_moves.is_empty() {
            good_moves = okay_moves;
        }
        if let Some((i,j)) = good_moves.first() {
            let jj = *j;
            table.animate_highlight_well_then(*i, (100,0,200), Box::new(move |tbl| tbl.animate_highlight_well(jj,(0,200,100))))
        } else {
            table.animate_highlight_deck(0, (200,54,10))
        }
    }
}
pub struct FreeCell {}
impl FreeCell {
    fn is_golden(cards:&[Card]) -> bool {
        if cards.len() == 0 { return false };
        let mut ret = true;
        for (x,y) in cards.iter().zip(cards.iter().skip(1)) {           
            ret &= x.value == y.value + 1 && match x.suit {
                Suit::Diamonds => y.suit == Suit::Clubs || y.suit == Suit::Spades,
                Suit::Hearts   => y.suit == Suit::Clubs || y.suit == Suit::Spades,
                Suit::Clubs    => y.suit == Suit::Diamonds || y.suit == Suit::Hearts,
                Suit::Spades   => y.suit == Suit::Diamonds || y.suit == Suit::Hearts,
            }
        }
        ret
    }
    fn nongolden_length(cards:&[Card]) -> usize {
        let mut top = cards.len() - 1.min(cards.len());
        let mut value = 0;
        while Self::is_golden(&cards[top..]) {
            value += 1;
            if top == 0 { break }
            top -= 1;
        }
        cards.len() - value
    }
    fn free_cells(tbl: &Table) -> usize {
        let mut count = 0;
        for i in 0..4 {
            if tbl.well(i).cards.len() == 0 { count += 1};
        }
        count
    }
    fn best_location_for_stack(table:&Table, cards: &[Card], other_than: GameObject) -> Option<GameObject> {
        let mut options : Vec<usize> = Vec::new();
        for i in 0..8 {
            if other_than != GameObject::Stack(i) {
                if Self::can_place_stack(table.stack(i), cards, table) {
                    options.push(i);
                }
            }
        }
        options.sort_by(|a , b| (Self::nongolden_length(&table.stack(*a).cards),table.stack(*a).cards.len() )
                       .cmp(&(Self::nongolden_length(&table.stack(*b).cards),table.stack(*b).cards.len() )));
        options.first().map(|x| GameObject::Stack(*x))
    }
    fn best_location_for(table:&Table, cards : &[Card], other_than: GameObject) -> Option<GameObject> {
        if cards.len() == 1 { 
            Self::best_location_for_card(table, cards[0], other_than)
        } else {
            Self::best_location_for_stack(table, cards, other_than)
        }
    }
    fn best_location_for_card(table:&Table, card : Card, other_than: GameObject) -> Option<GameObject> {
        for i in 4..8 {
            if other_than != GameObject::Well(i) {
                if Self::can_place_well(table.well(i), &vec![card]) {
                    return Some(GameObject::Well(i));
                }
            }
        }
        if let Some(s) = Self::best_location_for_stack(table, &vec![card], other_than) {
            Some(s)
        } else {
            for i in 0..4 {
                if other_than != GameObject::Well(i) {
                    if Self::can_place_well(table.well(i), &vec![card]) {
                        return Some(GameObject::Well(i));
                    }
                }
            }
            None
        }
    }
}
impl Rules for FreeCell {
    fn table_size() -> (u32,u32) { (338,320) }
    fn new_game(table: &mut Table) {
        let mut cards = Card::deck();
        cards.shuffle(&mut thread_rng());
        
        let empty_vec = Vec::new();
        table.add_well((34,32), 0,&empty_vec);
        table.add_well((66,32), 0,&empty_vec);
        table.add_well((98,32), 0,&empty_vec);
        table.add_well((130,32), 0,&empty_vec);
        table.add_well((176,32), 0,&empty_vec);
        table.add_well((208,32), 0,&empty_vec);
        table.add_well((240,32), 0,&empty_vec);
        table.add_well((272,32), 0,&empty_vec);
        let mut stacks_cards : [Vec<Card>;8] 
            = [Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new()];
        let mut idx = 0;
        for i in cards {
            stacks_cards[idx].push(i);
            idx = (idx + 1) % 8
        }
        for i in 1..=8 {
            table.add_stack((34*i,76), &stacks_cards[i as usize-1], 0);
        }
    }
    fn can_place_stack(stack: &Stack, cards: &[Card], _ : &Table) -> bool {        
        if let Some(c) = stack.cards.last () {
            cards[0].value + 1 == c.value && match cards[0].suit {
                Suit::Diamonds => c.suit == Suit::Clubs || c.suit == Suit::Spades,
                Suit::Hearts   => c.suit == Suit::Clubs || c.suit == Suit::Spades,
                Suit::Clubs    => c.suit == Suit::Diamonds || c.suit == Suit::Hearts,
                Suit::Spades   => c.suit == Suit::Diamonds || c.suit == Suit::Hearts,
            }
        } else {
            true
        }
    }
    fn can_place_well(well: &Well, cards: &[Card]) -> bool { 
        if well.id < 4 { return well.cards.len() == 0 && cards.len() == 1 };
        if well.cards.len() > 0 {
            cards.len() == 1 && cards[0].suit == well.cards[0].suit && cards[0].value == well.cards.len() as u8 + 1
        } else { 
            cards.len() == 1 && cards[0].value == 1
         }
        
    }
    fn can_skim_well(w: &Well) -> bool { 
        w.cards.len() > 0
    }
    fn can_split_stack(stack: &Stack, position: usize, tbl : &Table) -> bool { 
        position < stack.cards.len() && position >= stack.hidden_point && stack.cards.len() > 0 &&
            Self::is_golden(&stack.cards[position..]) && stack.cards.len() - position <= Self::free_cells(tbl) + 1
    }
    fn game_won(table: &Table) -> bool { 
        return table.well(4).cards.len() + table.well(5).cards.len() + table.well(6).cards.len() + table.well(7).cards.len() == 52
    }
    fn stack_clicked(table: &mut Table, stack_id: usize, position: usize) {
        let cards = &table.stack(stack_id).cards[position..];
        let l = cards.len();        
        if let Some(loc) = Self::best_location_for(table, cards, GameObject::Stack(stack_id)) {                        
            table.shift_then(GameObject::Stack(stack_id), loc,l,Box::new(move |tbl| {
                match loc {
                    GameObject::Stack(i) => Self::placed_in_stack(tbl, i, l),
                    GameObject::Well(i) => Self::placed_in_well(tbl, i, l),
                    GameObject::Deck(_) => {},
                };
                tbl.end_move();
            }));
        }
    }
    fn placed_in_stack(_: &mut Table, _: usize, _: usize) {  }
    fn placed_in_well(_: &mut Table, _: usize, _: usize) {  }
    fn deal_from_deck(_: &mut Table, _: usize) { }
    fn well_clicked(table: &mut Table, well_id: usize) {
        if let Some(card) = table.well(well_id).cards.last() {
            if let Some(loc) = Self::best_location_for_card(table, *card, GameObject::Well(well_id)) {
                table.shift_then(GameObject::Well(well_id), loc,1,Box::new(move |tbl| {
                    match loc {
                        GameObject::Stack(i) => Self::placed_in_stack(tbl, i, 1),
                        GameObject::Well(i) => Self::placed_in_well(tbl, i, 1),
                        GameObject::Deck(_) => {},
                    };
                    tbl.end_move();
                }));
            }
        }
    }
    fn hint(tbl: &mut Table) {
        // move to completed decks
        for i in 0..4 {
            if let Some(c) = tbl.well(i).cards.last() {
                for j in 4..8 {
                    if Self::can_place_well(tbl.well(j), &vec![*c][..]) {
                        tbl.animate_highlight_well_then(i, (100,0,200), Box::new(move |tbl| tbl.animate_highlight_well(j, (0,200,100))));
                        return
                    }
                }
            }
        }
        for i in 0..8 {
            if let Some(c) = tbl.stack(i).cards.last() {
                for j in 4..8 {
                    if Self::can_place_well(tbl.well(j), &vec![*c][..]) {
                        tbl.animate_highlight_stack_then(i, tbl.stack(i).cards.len()-1, (100,0,200), Box::new(move |tbl| tbl.animate_highlight_well(j, (0,200,100))));
                        return
                    }
                }
            }
        }
        //moves to stacks 
        // - move from cells
        // - move from other stacks
        let mut moves = Vec::new();
        for i in 0..4 {
            if let Some(c) = tbl.well(i).cards.last() {
                for j in 0..8 {
                    if Self::can_place_stack(tbl.stack(j), &vec![*c][..], tbl) {
                        moves.push((Self::nongolden_length(&tbl.stack(j).cards), (i,j)));                        
                    }
                }
            }
        }
        moves.sort();
        if let Some((_,(i,j))) = moves.first() {
            let pos = tbl.stack(*j).cards.len() - 1.min(tbl.stack(*j).cards.len());
            let jj = *j;
            tbl.animate_highlight_well_then(*i, (100,0,200), Box::new(move |tbl| tbl.animate_highlight_stack(jj,pos,(0,200,100))));
            return
        }
        let mut movesy = Vec::new();
        for i in 0..8 {
            for j in 1..=Self::free_cells(tbl)+1 {
                if j <= tbl.stack(i).cards.len() {
                    let pos = tbl.stack(i).cards.len() - j;
                    if Self::can_split_stack(tbl.stack(i), pos, tbl) {
                        if let Some(loc) = Self::best_location_for(tbl, &tbl.stack(i).cards[pos..], GameObject::Stack(i)) {
                            let score = match loc {
                                GameObject::Well(_) => (100,0),
                                GameObject::Deck(_) => (0,0),
                                GameObject::Stack(k) => (Self::nongolden_length(&tbl.stack(k).cards), tbl.stack(k).cards.len())
                            };
                            movesy.push((score, i, pos, loc));
                        }
                    }
                }
            }
        }
        movesy.sort_by(|(s,_,_,_)  , (t,_,_,_) | s.cmp(t));
        if let Some((_,i,pos,loc)) = movesy.first() {
            let locc = loc.clone();
            tbl.animate_highlight_stack_then(*i, *pos, (100,0,200), Box::new(move |tbl| 
                match locc {
                    GameObject::Well(w) => tbl.animate_highlight_well(w,(0,200,100)),
                    GameObject::Stack(j) =>  tbl.animate_highlight_stack(j,tbl.stack(j).cards.len()-1.min(tbl.stack(j).cards.len()),(0,200,100)),
                    GameObject::Deck(_) => ()
                }
            ));
        }

        //moves to cells
        // - move from stacks
    }
}
pub struct TriPeaks {}
impl TriPeaks {
    fn refresh_helper(table: &mut Table, child: usize, parent1: usize, parent2: usize) {
        if table.stack(parent1).cards.is_empty() && table.stack(parent2).cards.is_empty() {
            table.reveal(child, 1);
        }
    }
    fn refresh_stacks(table: &mut Table) {
        Self::refresh_helper(table, 0, 3, 4);
        Self::refresh_helper(table, 1, 5, 6);
        Self::refresh_helper(table, 2, 7, 8);
        Self::refresh_helper(table, 3, 9, 10);
        Self::refresh_helper(table, 4, 10, 11);
        Self::refresh_helper(table, 5, 12, 13);
        Self::refresh_helper(table, 6, 13, 14);
        Self::refresh_helper(table, 7, 15, 16);
        Self::refresh_helper(table, 8, 16, 17);
        Self::refresh_helper(table, 9, 18, 19);
        Self::refresh_helper(table, 10, 19, 20);
        Self::refresh_helper(table, 11, 20, 21);
        Self::refresh_helper(table, 12, 21, 22);
        Self::refresh_helper(table, 13, 22, 23);
        Self::refresh_helper(table, 14, 23, 24);
        Self::refresh_helper(table, 15, 24, 25);
        Self::refresh_helper(table, 16, 25, 26);
        Self::refresh_helper(table, 17, 26, 27);
    }
}
impl Rules for TriPeaks {

    fn table_size() -> (u32,u32) { (384,256+32) }
    fn new_game(table: &mut Table) {
        let cards = Card::deck();
        let (tableau_cards,rest) = cards.split_at(28);
        table.add_deck(((384/2)-32,192), rest);
        let empty_vec = Vec::new();
        table.add_well(((384/2),192), 0,&empty_vec);
        let mut start = 0;
        table.add_stack_nobase((80,32),&tableau_cards[start..start+1],1);
        start += 1;
        table.add_stack_nobase((80+96,32),&tableau_cards[start..start+1],1);
        start += 1;
        table.add_stack_nobase((80+192,32),&tableau_cards[start..start+1],1);
        start += 1;
        table.add_stack_nobase((64,64),&tableau_cards[start..start+1],1);
        start += 1;
        table.add_stack_nobase((96,64),&tableau_cards[start..start+1],1);
        start += 1;
        table.add_stack_nobase((160,64),&tableau_cards[start..start+1],1);
        start += 1;
        table.add_stack_nobase((192,64),&tableau_cards[start..start+1],1);
        start += 1;
        table.add_stack_nobase((256,64),&tableau_cards[start..start+1],1);
        start += 1;
        table.add_stack_nobase((288,64),&tableau_cards[start..start+1],1);
        start += 1;
        table.add_stack_nobase((48,96),&tableau_cards[start..start+1],1);
        start += 1;
        table.add_stack_nobase((80,96),&tableau_cards[start..start+1],1);
        start += 1;
        table.add_stack_nobase((112,96),&tableau_cards[start..start+1],1);
        start += 1;
        table.add_stack_nobase((144,96),&tableau_cards[start..start+1],1);
        start += 1;
        table.add_stack_nobase((176,96),&tableau_cards[start..start+1],1);
        start += 1;
        table.add_stack_nobase((208,96),&tableau_cards[start..start+1],1);
        start += 1;
        table.add_stack_nobase((240,96),&tableau_cards[start..start+1],1);
        start += 1;
        table.add_stack_nobase((272,96),&tableau_cards[start..start+1],1);
        start += 1;
        table.add_stack_nobase((304,96),&tableau_cards[start..start+1],1);
        start += 1;
        table.add_stack_nobase((32,128),&tableau_cards[start..start+1],0);
        start += 1;
        table.add_stack_nobase((64,128),&tableau_cards[start..start+1],0);
        start += 1;
        table.add_stack_nobase((96,128),&tableau_cards[start..start+1],0);
        start += 1;
        table.add_stack_nobase((128,128),&tableau_cards[start..start+1],0);
        start += 1;
        table.add_stack_nobase((160,128),&tableau_cards[start..start+1],0);
        start += 1;
        table.add_stack_nobase((192,128),&tableau_cards[start..start+1],0);
        start += 1;
        table.add_stack_nobase((224,128),&tableau_cards[start..start+1],0);
        start += 1;
        table.add_stack_nobase((256,128),&tableau_cards[start..start+1],0);
        start += 1;
        table.add_stack_nobase((288,128),&tableau_cards[start..start+1],0);
        start += 1;
        table.add_stack_nobase((320,128),&tableau_cards[start..start+1],0);
    }
    fn can_split_stack(stack: &Stack, pos: usize, _tbl: &Table) -> bool {
        pos < stack.cards.len() && pos >= stack.hidden_point
    }
    fn can_place_stack(_: &Stack, _: &[Card], _ : &Table) -> bool { false }
    fn can_place_well(w: &Well, cs: &[Card]) -> bool { Golf::can_place_well(w, cs) }
    fn can_skim_well(_: &Well) -> bool { false }
    fn game_won(table: &Table) -> bool { Golf::game_won(table) }  
    fn deal_from_deck(table: &mut Table, id: usize) { Golf::deal_from_deck(table, id) }
    fn placed_in_stack(_: &mut Table, _: usize, _: usize) { }
    fn placed_in_well(table: &mut Table, _: usize, _: usize) { Self::refresh_stacks(table) }
    fn stack_clicked(table: &mut Table, stack_id: usize, position: usize) { 
        let cards = &table.stack(stack_id).cards[position..];
        if let [c] = cards {
            if Self::can_place_well(table.well(0), &[*c]) {
                table.shift_then(GameObject::Stack(stack_id), GameObject::Well(0), 1, Box::new(move |tbl| {
                    Self::refresh_stacks(tbl);
                    tbl.end_move();
                }))
            }
        }        
    }
    fn well_clicked(_: &mut Table, _: usize) { }
    fn hint(tbl: &mut Table) { Golf::hint(tbl) }
}
pub struct Golf {}
impl Rules for Golf {
    
    fn table_size() -> (u32,u32) { (288,256+32) }
    fn new_game(table: &mut Table) {
        let cards = Card::deck();
        let (tableau_cards,rest) = cards.split_at(35);
        table.add_deck((128-16,160), rest);
        let empty_vec = Vec::new();
        table.add_well((128+16,160), 0,&empty_vec);
        let mut start = 0;
        for i in 1..=7 {
            
            table.add_stack((32 * i as i32,32), &tableau_cards[start..start+5] , 0);
            start += 5;
        }
    }
    fn can_split_stack(stack: &Stack, s: usize, _ : &Table) -> bool { stack.cards.len() > 0 && s == stack.cards.len() - 1 }
    fn can_place_stack(_: &Stack, _: &[Card], _ : &Table) -> bool { false }
    fn can_place_well(w: &Well, cs: &[Card]) -> bool {
        if let [t] = cs {
            if let Some(c) = w.cards.last() { 
                return t.value as i32 - c.value as i32 == 1 || c.value as i32 - t.value as i32 == 1 || c.value == 13 && t.value == 1 || c.value == 1 && t.value == 13
            } else {
                return true
            }
        } else {
            return false
        }
    }
    fn can_skim_well(_: &Well) -> bool { false }
    fn deal_from_deck(table: &mut Table, _: usize) {
        if table.deck(0).cards.len() > 0 {
            table.shift(GameObject::Deck(0), GameObject::Well(0), 1)
        }
    }
    fn placed_in_stack(_: &mut Table, _: usize, _: usize) {  }
    fn placed_in_well(_: &mut Table, _: usize, _: usize) { }
    fn well_clicked(_: &mut Table, _: usize) {  }
    fn game_won(table: &Table) -> bool {
        return table.well(0).cards.len() + table.deck(0).cards.len() == 52
    }
    fn stack_clicked(table: &mut Table, stack_id: usize, position: usize) {
        let cards = &table.stack(stack_id).cards[position..];
        if let [c] = cards {
            if Self::can_place_well(table.well(0), &[*c]) {
                table.shift_then(GameObject::Stack(stack_id), GameObject::Well(0), 1, Box::new(move |tbl| {
                    tbl.end_move();
                }))
            }
        }        
    }
    fn hint(tbl: &mut Table) {
        let mut available_stacks : Vec<(usize, usize)> = Vec::new();
        for i in tbl.stacks().iter().rev() {
            let len = i.cards.len() - i.hidden_point;
            if len > 0 {
                if Self::can_place_well(tbl.well(0), &i.cards[len-1..]) {
                    available_stacks.push((i.id,len))
                }
            }
        }
        available_stacks.sort_by(|(_,b), (_,d) | d.cmp(b) );
        if let Some ((s,_)) = available_stacks.first() {
            tbl.animate_highlight_stack(*s, tbl.stack(*s).cards.len()-1, (200,54,200))
        } else {
            tbl.animate_highlight_deck(0, (200,54,10))
        }
    }
}
pub trait SpiderVariant {
    fn suit_match(x: Suit, y: Suit) -> bool;
}
pub struct OneSuit{}
impl SpiderVariant for OneSuit {
    fn suit_match(_x: Suit,_y: Suit) -> bool { true }
}
pub struct TwoSuit{}
impl SpiderVariant for TwoSuit {
    fn suit_match(x: Suit,y: Suit) -> bool { 
        match x {
            Suit::Hearts => y == Suit::Hearts || y == Suit::Diamonds,
            Suit::Diamonds => y == Suit::Hearts || y == Suit::Diamonds,
            Suit::Spades => y == Suit::Spades || y == Suit::Clubs,
            Suit::Clubs => y == Suit::Spades || y == Suit::Clubs
        }
     }
}
pub struct FourSuit{}
impl SpiderVariant for FourSuit {
    fn suit_match(x: Suit,y: Suit) -> bool { x == y }
}
pub struct Spider<V:SpiderVariant> { _dummy: V }
impl <V:SpiderVariant>Spider<V> {
    fn is_golden(cards:&[Card]) -> bool {
        let mut ret = true;
        for (x,y) in cards.iter().zip(cards.iter().skip(1)) {           
            ret &= x.value == y.value + 1 && V::suit_match(x.suit, y.suit)
        }
        ret
    }
    fn longest_golden_run(cards:&[Card]) -> usize {
        let mut top = cards.len() - 1.min(cards.len());
        let mut value = 0;
        while Self::is_golden(&cards[top..]) {
            value += 1;
            if top == 0 { break }
            top -= 1;
        }
        value
    }
    fn free_well(table: &Table) -> usize {
        for i in 0..table.wells().len() {
            if table.well(i).cards.len() == 0 { return i };
        }
        return 0;
    }
    
}

impl <V:SpiderVariant>Rules for Spider<V> {
    fn table_size() -> (u32,u32) { (384,320) }
    fn new_game(table: &mut Table) {
        let mut cards = Card::deck();
        cards.append(&mut Card::deck());
        cards.shuffle(&mut thread_rng());
        let (deck_cards,rest) = cards.split_at(50);
        table.add_deck((32,32), deck_cards);
        let empty_vec = Vec::new();
        table.add_well((96,32), 0,&empty_vec);
        table.add_well((128,32), 0,&empty_vec);
        table.add_well((160,32), 0,&empty_vec);
        table.add_well((192,32), 0,&empty_vec);
        table.add_well((224,32), 0,&empty_vec);
        table.add_well((256,32), 0,&empty_vec);
        table.add_well((288,32), 0,&empty_vec);
        table.add_well((320,32), 0,&empty_vec);        
        let mut stacks_cards : [Vec<Card>;10] 
            = [Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new()];
        let mut idx = 0;
        for i in rest {
            stacks_cards[idx].push(*i);
            idx = (idx + 1) % 10
        }
        for i in 1..=10 {
            table.add_stack((32*i,76), &stacks_cards[i as usize-1], stacks_cards[i as usize-1].len()-1);
        }
    }
    fn game_won(table: &Table) -> bool {
        table.stacks().iter().all(|s| s.cards.len() == 0) && table.deck(0).cards.len() == 0
    }
    fn stack_clicked(table: &mut Table, stack_id: usize, position: usize) {
        if Self::can_split_stack(table.stack(stack_id), position, table) {
            let mut moves : Vec<(usize, usize, i32)> = Vec::new();
            let cards = &table.stack(stack_id).cards[position..];
            for i in 0..10 {
                if i != stack_id {
                    if Self::can_place_stack(table.stack(i), cards, table) {
                        let mut hypothetical : Vec<Card> = table.stack(i).cards.iter().skip(table.stack(i).hidden_point).cloned().collect();
                        hypothetical.extend(cards);
                        let g1 = Self::longest_golden_run(&hypothetical);
                        let g2 = Self::longest_golden_run(cards);
                        moves.push((i,  g1 - g2, - (hypothetical.len() as i32) + g1 as i32));
                    }
                }
            }
            let len = cards.len();
            moves.sort_by(|(_,x1,y1),(_,x2,y2)| { (*x2,*y2).cmp(&(*x1,*y1)) });
            if let Some((m,_,_)) = moves.first() {
                let dm = *m;
                table.shift_then(GameObject::Stack(stack_id), GameObject::Stack(dm), len, Box::new(move |tbl| {
                    Self::placed_in_stack(tbl, dm, len);
                    tbl.end_move();
                }));
            }
        }
    }
    fn well_clicked(_: &mut Table, _: usize) {}
    fn hint(table: &mut Table) {
        // first locate any complete sequences
        for i in 0..10 {
            if Self::longest_golden_run(&table.stack(i).cards) == 13 {
                let j = Self::free_well(table);
                table.animate_highlight_stack_then(i, table.stack(i).cards.len()-13, (200,54,200), Box::new(move |tbl| {
                    tbl.animate_highlight_well(j, (200,54,200));
                }));
                return;
            }
        }
        // next try stack->stack moves
        let mut moves : Vec<(usize, usize, usize,usize, i32)> = Vec::new();
        for src in 0..10 {
            let mut pos = 0;
            if table.stack(src).cards.len() != 0 {
                while !Self::can_split_stack(table.stack(src), pos,table) { pos += 1 };
                let cards = &table.stack(src).cards[pos..];
                for i in 0..10 {
                    if i != src {
                        if Self::can_place_stack(table.stack(i), cards, table) {
                            let mut hypothetical : Vec<Card> = table.stack(i).cards.iter().skip(table.stack(i).hidden_point).cloned().collect();
                            hypothetical.extend(cards);
                            let g1 = Self::longest_golden_run(&hypothetical);
                            let g2 = Self::longest_golden_run(cards);
                            moves.push((src,i,pos, g1 - g2, -(hypothetical.len() as i32) + g1 as i32));
                        }
                    }
                }
            }
        }
        moves.sort_by(|(_,_,_,x1,y1),(_,_,_,x2,y2)| { (*x2,*y2).cmp(&(*x1,*y1))});
        if let Some((s,d,m,_,_)) = moves.first() {
            let dd = *d;            
            table.animate_highlight_stack_then(*s, *m, (0,54,200), Box::new(move |tbl| {
                tbl.animate_highlight_stack(dd,tbl.stack(dd).cards.len(),(0,200,54))
            }));
            return;
        }
        // give up, highlight the deck
        table.animate_highlight_deck(0, (200,54,10));
    }
    fn deal_from_deck(table: &mut Table, deck_id: usize) {
        if !table.stacks().iter().any(|x| x.cards.len() == 0) {
            let mut s = 0;
            while let Some(_) = table.deck(deck_id).cards.last() {
                table.shift(GameObject::Deck(deck_id), GameObject::Stack(s),1);
                s += 1;
                if s == 10 { break };
            }
        }
    }
    fn placed_in_stack(table: &mut Table, stack_id : usize, _cards: usize)  {
        refresh_stacks(table);   
        if Self::longest_golden_run(&table.stack(stack_id).cards) == 13 {
            table.end_move();
            let dest = Self::free_well(table);
            table.shift(GameObject::Stack(stack_id), GameObject::Well(dest), 13);
            refresh_stacks(table);
        }
    }
    fn placed_in_well(table: &mut Table, _well_id : usize, _cards: usize)  {
        refresh_stacks(table);
    }
    fn can_place_stack(stack : &Stack, cards: &[Card], _ : &Table) -> bool {
        if let Some(n) = cards.first() {
            if let Some(p) = stack.cards.last() {
                p.value == n.value + 1
            } else { true }
        } else { false }
    }
    fn can_place_well(well : &Well, cards: &[Card]) -> bool {
        well.cards.len() == 0 && cards.len() == 13 && Self::is_golden(&cards[..])
    }
    fn can_split_stack(stack : &Stack, position: usize, _ : &Table) -> bool {
        position < stack.cards.len() && position >= stack.hidden_point && stack.cards.len() > 0 &&
            Self::is_golden(&stack.cards[position..])        
    }
    fn can_skim_well(_well: &Well) -> bool { false }
}
pub trait DrawSize {
    fn size_of_draw() -> usize;
}
pub struct LittleSpider {  }
impl LittleSpider {
    fn best_location_for_card(table:&Table, card : Card, other_than: GameObject) -> Option<GameObject> {        
        for i in 0..4 {
            if Self::can_place_well(table.well(i), &vec![card]) {
                return Some(GameObject::Well(i));
            }
        }
        let mut moves : Vec<(usize,  i32)> = Vec::new();
        for i in 0..8 {
            if other_than != GameObject::Stack(i)  {
                if Self::can_place_stack(table.stack(i), &vec![card], table) {
                    moves.push((i,  -(table.stack(i).cards.len() as i32)));
                }
            }
        }
        moves.sort_by(|(_,x1),(_,x2)| { x2.cmp(x1) });
        if let Some((m,_)) = moves.first() {
            return Some (GameObject::Stack(*m));
        }
        return None;
    }
}

impl Rules for LittleSpider {
    fn table_size() -> (u32,u32) { (320,320) }
    fn new_game(table: &mut Table) {
        let mut specials = vec![Card{value: 1, suit: Suit::Diamonds}, Card{value:1, suit:Suit::Hearts}, Card{value:13, suit:Suit::Clubs}, Card{value:13, suit:Suit::Spades}];
        specials.reverse();
        let cards : Vec<Card> = Card::deck().into_iter().filter(|c| !specials.contains(&c)).collect();
        
        
        let (deck_cards,rest) = cards.split_at(40);
        table.add_deck((32,32), deck_cards);
        table.add_well((96,32), 0,&vec![specials.pop().unwrap()]);
        table.add_well((128,32), 0,&vec![specials.pop().unwrap()]);
        table.add_well((160,32), 0,&vec![specials.pop().unwrap()]);
        table.add_well((192,32), 0,&vec![specials.pop().unwrap()]);
        let mut stacks_cards : [Vec<Card>;8] 
            = [Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new()];
        let mut idx = 0;
        for i in rest {
            stacks_cards[idx].push(*i);
            idx = idx + 1
        }
        for i in 1..=8 {
            table.add_stack((32*i,76), &stacks_cards[i as usize-1], stacks_cards[i as usize-1].len()-1);
        }
    }
    fn game_won(table: &Table) -> bool {
        table.stacks().iter().all(|s| s.cards.len() == 0) && table.deck(0).cards.len() == 0
    }
    fn stack_clicked(table: &mut Table, stack_id: usize, position: usize) {
        let cards = &table.stack(stack_id).cards[position..];
        let l = cards.len();        
        if l > 1 { return; };
        if let Some(loc) = Self::best_location_for_card(table, cards[0], GameObject::Stack(stack_id)) {                        
            table.shift_then(GameObject::Stack(stack_id), loc,l,Box::new(move |tbl| {
                match loc {
                    GameObject::Stack(i) => Self::placed_in_stack(tbl, i, l),
                    GameObject::Well(i) => Self::placed_in_well(tbl, i, l),
                    GameObject::Deck(_) => {},
                };
                tbl.end_move();
            }));
        }
        
    }
    fn well_clicked(_: &mut Table, _: usize) {}
    fn hint(table: &mut Table) {
        let mut moves: Vec<(GameObject, GameObject, usize)> = Vec::new();
        for i in 0..7 {
            if let Some(c) = table.stack(i).cards.last() {
                if let Some(l) = Self::best_location_for_card(table,*c, GameObject::Stack(i)) {
                    moves.push((GameObject::Stack(i),l,table.stack(i).cards.len()))
                }
            }
        }
        moves.sort_by(|(_s1,d1,c1), (_s2,d2,c2)| match (d1,d2) {
            (GameObject::Well(_),_) => Ordering::Less,
            (_,GameObject::Well(_)) => Ordering::Greater,
            (_,_) => c2.cmp(c1)
        });
        if let Some((s,d,_c)) = moves.first() {
            let dd = *d;
            let and_then : Box<dyn FnOnce(&mut Table)> = Box::new(move |tbl| {
                match dd {
                    GameObject::Well(w) => tbl.animate_highlight_well(w, (0,200,100)),
                    GameObject::Stack(w) => tbl.animate_highlight_stack(w, tbl.stack(w).cards.len()-1.min(tbl.stack(w).cards.len()), (0,200,100)),
                    _ => {}
                } 
                
            } );
            match *s {
                GameObject::Well(w) => table.animate_highlight_well_then(w, (100,0,200), and_then),
                GameObject::Stack(s) => { let pos = table.stack(s).cards.len() - 1; table.animate_highlight_stack_then(s, pos, (100,0,200), and_then)},
                _ => {}
            }

        } else {
            table.animate_highlight_deck(0, (200,68,25));
        }
    }
    fn deal_from_deck(table: &mut Table, deck_id: usize) {
        let mut s = 0;
        while let Some(_) = table.deck(deck_id).cards.last() {
            table.shift(GameObject::Deck(deck_id), GameObject::Stack(s),1);
            s += 1;
            if s == 8 { break };
        }
    }
    fn placed_in_stack(_table: &mut Table, _stack_id : usize, _cards: usize)  {
    }
    fn placed_in_well(_table: &mut Table, _well_id : usize, _cards: usize)  {
    }
    fn can_place_stack(stack : &Stack, cards: &[Card], table : &Table) -> bool {
        if table.deck(0).cards.len() != 0 {
            return false
        }
        if let Some(p) = stack.cards.last() {
            p.value == cards[0].value + 1 || p.value + 1 == cards[0].value
            || p.value == 1 && cards[0].value == 13 ||  p.value == 13 && cards[0].value == 1
        } else {
            false
        }
    }
    fn can_place_well(well : &Well, cards: &[Card]) -> bool {
        if cards.len() == 1 && well.cards.last().is_some() {
            let wc = well.cards.last().unwrap();
            if cards[0].suit != wc.suit { return false } 
            if cards[0].suit == Suit::Hearts || cards[0].suit == Suit::Diamonds {
                cards[0].value == wc.value + 1
            } else {
                wc.value == cards[0].value + 1
            }
        } else { false }
    }
    fn can_split_stack(stack : &Stack, position: usize, _ : &Table) -> bool {
        stack.cards.len() > 0 && position == stack.cards.len() - 1
    }
    fn can_skim_well(_well: &Well) -> bool { false }
}


pub struct OneDraw {}
pub struct ThreeDraw {}
impl DrawSize for OneDraw { fn size_of_draw () -> usize { 1 } }
impl DrawSize for ThreeDraw { fn size_of_draw () -> usize { 3 } }
pub struct Pyramid<V: DrawSize> {
    _dummy: V
}
impl <V:DrawSize> Pyramid<V> {
    fn value_of(table : &Table, obj: GameObject) -> u8 {
        match obj {
            GameObject::Deck(i) => table.deck(i).cards.last().map(|x| x.value).unwrap_or(127),
            GameObject::Stack(i) => table.stack(i).cards.last().map(|x| x.value).unwrap_or(127),
            GameObject::Well(i) => table.well(i).cards.last().map(|x| x.value).unwrap_or(127)
        }
    }
    fn handle_select(table: &mut Table, obj: GameObject) {        
        if let Some(n) = table.selection {
            let x = Self::value_of(table,obj) + Self::value_of(table, n);
            if x == 13 {
                table.shift(n,   GameObject::Well(1), 1);
                table.shift(obj, GameObject::Well(1), 1);
                table.deselect();
                table.end_move();
                return
            }
        }
        if Self::value_of(table,obj) == 13 {
            table.shift(obj, GameObject::Well(1),1);
            table.end_move();
        } else { table.select(obj) };
    }
    fn unobstructed(table : &Table, s: usize) -> bool {
        let (x,y) = match s {
            0 => (1,2),

            1 => (3,4),
            2 => (4,5),

            3 => (6,7),
            4 => (7,8),
            5 => (8,9),

            6 => (10,11),
            7 => (11,12),
            8 => (12,13),
            9 => (13,14),

            10 => (15,16),
            11 => (16,17),
            12 => (17,18),
            13 => (18,19),
            14 => (19,20),

            15 => (21,22),
            16 => (22,23),
            17 => (23,24),
            18 => (24,25),
            19 => (25,26),
            20 => (26,27),
            _  => (255,255)
        };
        (x == 255 || table.stack(x).cards.len() == 0) && (y == 255 || table.stack(y).cards.len() == 0)
    }
}
impl <V:DrawSize> Rules for Pyramid<V> {
    fn table_size() -> (u32,u32) { (384,256+32) }
    fn new_game(table: &mut Table) {
        let cards = Card::deck();
        let (tableau_cards,rest) = cards.split_at(28);
        let offset = if V::size_of_draw() == 3 { 24 } else {0};
        table.add_deck((384-64-32-offset,32), rest);
        let empty_vec = Vec::new();
        let mut start = 0;
        let locations = vec![ 
            (96,32), 
            (80,64), (112,64), 
            (64,96), (96,96), (128,96), 
            (48,128), (80,128), (112,128), (144,128),
            (32,160), (64,160), (96,160), (128,160), (160,160),
            (16,192), (48,192), (80,192), (112, 192), (144,192), (176,192),
            (0,228), (32,228), (64,228), (96,228), (128,228), (160, 228), (192,228),
        ];
        for (x,y) in locations {
            table.add_stack_nobase((x+32,y),&tableau_cards[start..start+1],0);
            start += 1;
        }        
        table.add_well((384-32-32-offset,32), V::size_of_draw()-1,&empty_vec);
        table.add_well((384-32-32,228), 0,&empty_vec);
    }
    fn can_split_stack(stack: &Stack, pos: usize, tbl: &Table) -> bool { stack.cards.len() > pos && Self::unobstructed(tbl, stack.id) }
    fn can_place_stack(_: &Stack, _: &[Card], _: &Table) -> bool { false }
    fn can_place_well(_: &Well, _: &[Card]) -> bool { false }
    fn can_skim_well(w: &Well) -> bool { w.cards.len() > 0 && w.id == 0 }
    fn game_won(table: &Table) -> bool { table.well(1).cards.len() + table.well(0).cards.len() + table.deck(0).cards.len() == 52 }
    fn placed_in_stack(_: &mut Table, _: usize, _: usize) { }
    fn placed_in_well(_: &mut Table, _: usize, _: usize) { }
    fn deal_from_deck(table: &mut Table, _deck_id: usize) {
        if table.deck(0).cards.len() > 0 {
            table.shift(GameObject::Deck(0), GameObject::Well(0), V::size_of_draw())
        } else {
            if table.well(0).cards.len() > 0 {
                table.shift(GameObject::Well(0), GameObject::Deck(0), table.well(0).cards.len());                
            }
        }
    }
    fn stack_clicked(table: &mut Table, id: usize, _pos: usize) {
        Self::handle_select(table, GameObject::Stack(id));
    }
    fn well_clicked(table: &mut Table, id: usize) {
        Self::handle_select(table, GameObject::Well(id));
    }
    fn hint(table: &mut Table) { 
        if Self::value_of(table,GameObject::Well(0)) == 13 {
            table.animate_highlight_well(0, (0,200,100));
            return
        }
        for i in 0..28 {
            if Self::unobstructed(table, i) {
                if Self::value_of(table, GameObject::Stack(i)) == 13 {
                    table.animate_highlight_stack(i,0,(0,200,100));
                    return
                }
                for j in i..28 {
                    if i != j && Self::unobstructed(table, j) {
                        if Self::value_of(table,GameObject::Stack(i)) + Self::value_of(table, GameObject::Stack(j)) == 13 {
                            table.animate_highlight_stack(i, 0, (0,200,100));
                            table.animate_highlight_stack(j, 0, (0,200,100));
                            return
                        }

                    }
                }
                if Self::value_of(table, GameObject::Stack(i)) + Self::value_of(table,GameObject::Well(0)) == 13 {
                    table.animate_highlight_stack(i, 0, (0,200,100));
                    table.animate_highlight_well(0, (0,200,100));
                    return
                }
            }
        }
        table.animate_highlight_deck(0, (200,0,100));
     }
}
pub struct Yukon{}
impl Yukon {}

impl Yukon {
    fn best_location_for_stack(table:&Table, cards: &[Card], other_than: GameObject) -> Option<GameObject> {
        let mut options : Vec<usize> = Vec::new();
        for i in 0..7 {
            if other_than != GameObject::Stack(i) {
                if Self::can_place_stack(table.stack(i), cards, table) {
                    options.push(i);
                }
            }
        }
        options.sort_by(|a , b| (table.stack(*b).cards.len() - table.stack(*b).hidden_point).cmp(&(table.stack(*a).cards.len() - table.stack(*a).hidden_point)) );
        options.first().map(|x| GameObject::Stack(*x))
    }
    fn best_location_for(table:&Table, cards : &[Card], other_than: GameObject) -> Option<GameObject> {
        if cards.len() == 1 { 
            Self::best_location_for_card(table, cards[0], other_than)
        } else {
            Self::best_location_for_stack(table, cards, other_than)
        }
    }
    fn best_location_for_card(table:&Table, card : Card, other_than: GameObject) -> Option<GameObject> {
        for i in 0..=3 {
            if other_than != GameObject::Well(i) {
                if Self::can_place_well(table.well(i), &vec![card]) {
                    return Some(GameObject::Well(i));
                }
            }
        }
        Self::best_location_for_stack(table, &vec![card], other_than)        
    }
}
impl Rules for Yukon {
    fn table_size() -> (u32,u32) { (288,320) }
    fn new_game(table: &mut Table) {
        let cards = Card::deck();
        let empty_vec = Vec::new();
        table.add_well((32,32), 0,&empty_vec);
        table.add_well((96,32),0, &empty_vec);
        table.add_well((160,32),0, &empty_vec);
        table.add_well((224,32),0, &empty_vec);
        let mut start = 0;
        for i in 1..=7 {
            let m = (i - 1 ) + if i == 1 { 1 } else { 5};
            table.add_stack((32 * i as i32,76), &cards[start..start+m] , i-1);
            start += m;
        }
    }
    fn can_split_stack(stack: &Stack, position: usize, _ : &Table) -> bool {
        position < stack.cards.len() && position >= stack.hidden_point        
    }
    fn can_skim_well(well: &Well) -> bool {
        well.cards.len() > 0
    }
    fn can_place_stack(stack: &Stack, cards: &[Card], _tbl: &Table) -> bool {        
        if let Some(c) = stack.cards.last () {
            cards[0].value + 1 == c.value && match cards[0].suit {
                Suit::Diamonds => c.suit == Suit::Clubs || c.suit == Suit::Spades,
                Suit::Hearts   => c.suit == Suit::Clubs || c.suit == Suit::Spades,
                Suit::Clubs    => c.suit == Suit::Diamonds || c.suit == Suit::Hearts,
                Suit::Spades   => c.suit == Suit::Diamonds || c.suit == Suit::Hearts,
            }
        } else {
            cards[0].value == 13
        }
    }
    fn can_place_well(well: &Well, cards: &[Card]) -> bool { 
        if well.cards.len() > 0 {
            cards.len() == 1 && cards[0].suit == well.cards[0].suit && cards[0].value == well.cards.len() as u8 + 1
        } else { 
            cards.len() == 1 && cards[0].value == 1
         }
        
    }
    fn placed_in_stack(table: &mut Table, _stack_id: usize, _cards: usize) {
        refresh_stacks(table);
    }
    fn placed_in_well(table: &mut Table, _well_id: usize, _cards: usize) {
        refresh_stacks(table);
    }
    fn deal_from_deck(_table: &mut Table, _deck_id: usize) {
    }
    fn stack_clicked(table: &mut Table, stack_id: usize, position: usize) {
        let cards = &table.stack(stack_id).cards[position..];
        let l = cards.len();        
        if let Some(loc) = Self::best_location_for(table, cards, GameObject::Stack(stack_id)) {                        
            table.shift_then(GameObject::Stack(stack_id), loc,l,Box::new(move |tbl| {
                match loc {
                    GameObject::Stack(i) => Self::placed_in_stack(tbl, i, l),
                    GameObject::Well(i) => Self::placed_in_well(tbl, i, l),
                    GameObject::Deck(_) => {},
                };
                tbl.end_move();
            }));
        }
    }
    fn well_clicked(table: &mut Table, well_id: usize) {
        if let Some(card) = table.well(well_id).cards.last() {
            if let Some(loc) = Self::best_location_for_card(table, *card, GameObject::Well(well_id)) {
                table.shift_then(GameObject::Well(well_id), loc,1,Box::new(move |tbl| {
                    match loc {
                        GameObject::Stack(i) => Self::placed_in_stack(tbl, i, 1),
                        GameObject::Well(i) => Self::placed_in_well(tbl, i, 1),
                        GameObject::Deck(_) => {},
                    };
                    tbl.end_move();
                }));
            }
        }
    }
    
    fn hint(table: &mut Table) {
        let mut moves: Vec<(GameObject, GameObject, usize)> = Vec::new();
        for i in 0..7 {
            if let Some(_) = table.stack(i).cards.last() {
                for b in table.stack(i).hidden_point..table.stack(i).cards.len() {
                    let cards = &table.stack(i).cards[b..];
                    if let Some(l) = Self::best_location_for(table,cards, GameObject::Stack(i)) {
                        moves.push((GameObject::Stack(i),l,cards.len()))
                    }
                }
            }
        }
        moves.sort_by(|(s1,d1,c1), (s2,d2,c2)| match (d1,d2) {
            (GameObject::Well(_),_) => Ordering::Less,
            (_,GameObject::Well(_)) => Ordering::Greater,
            (_,_) => match(s1,s2) {
                // TODO: This doesn't try to expose cards that would be addable to wells..
                (GameObject::Stack(src1), GameObject::Stack(src2)) => {
                        let b1 = table.stack(*src1).cards.len() - *c1 <= table.stack(*src1).hidden_point;
                        let b2 = table.stack(*src2).cards.len() - *c2 <= table.stack(*src2).hidden_point;
                        if b1 && !b2 { Ordering::Less }
                        else if b2 && !b1 { Ordering::Greater } else {
                        table.stack(*src2).hidden_point.cmp( &table.stack(*src1).hidden_point) 
                        }
                    },
                (_,_) => Ordering::Equal
            }
        });
        if let Some((s,d,c)) = moves.first() {
            let dd = *d;
            let and_then : Box<dyn FnOnce(&mut Table)> = Box::new(move |tbl| {
                match dd {
                    GameObject::Well(w) => tbl.animate_highlight_well(w, (0,200,100)),
                    GameObject::Stack(w) => tbl.animate_highlight_stack(w, tbl.stack(w).cards.len()-1.min(tbl.stack(w).cards.len()), (0,200,100)),
                    _ => {}
                } 
                
            } );
            match *s {
                GameObject::Well(w) => table.animate_highlight_well_then(w, (100,0,200), and_then),
                GameObject::Stack(s) => { let pos = table.stack(s).cards.len() - *c; table.animate_highlight_stack_then(s, pos, (100,0,200), and_then)},
                _ => {}
            }
        }
    }
    fn game_won(table: &Table) -> bool {
        table.stacks().iter().all(|s| s.cards.len() == 0)
    }
}
pub struct Klondike<V: DrawSize> {
    _dummy: V
}
impl <V:DrawSize>Klondike<V> {
    fn best_location_for_stack(table:&Table, cards: &[Card], other_than: GameObject) -> Option<GameObject> {
        let mut options : Vec<usize> = Vec::new();
        for i in 0..7 {
            if other_than != GameObject::Stack(i) {
                if Self::can_place_stack(table.stack(i), cards, table) {
                    options.push(i);
                }
            }
        }
        options.sort_by(|a , b| (table.stack(*b).cards.len() - table.stack(*b).hidden_point).cmp(&(table.stack(*a).cards.len() - table.stack(*a).hidden_point)) );
        options.first().map(|x| GameObject::Stack(*x))
    }
    fn best_location_for(table:&Table, cards : &[Card], other_than: GameObject) -> Option<GameObject> {
        if cards.len() == 1 { 
            Self::best_location_for_card(table, cards[0], other_than)
        } else {
            Self::best_location_for_stack(table, cards, other_than)
        }
    }
    fn best_location_for_card(table:&Table, card : Card, other_than: GameObject) -> Option<GameObject> {
        for i in 1..=4 {
            if other_than != GameObject::Well(i) {
                if Self::can_place_well(table.well(i), &vec![card]) {
                    return Some(GameObject::Well(i));
                }
            }
        }
        Self::best_location_for_stack(table, &vec![card], other_than)        
    }
}
impl <V:DrawSize> Rules for Klondike<V> {
    fn table_size() -> (u32,u32) { (288,320) }
    fn new_game(table: &mut Table) {
        let cards = Card::deck();
        let (tableau_cards,rest) = cards.split_at(28);
        table.add_deck((32,32), rest);
        let empty_vec = Vec::new();
        table.add_well((64,32), V::size_of_draw()-1,&empty_vec);
        table.add_well((128,32), 0,&empty_vec);
        table.add_well((160,32),0, &empty_vec);
        table.add_well((192,32),0, &empty_vec);
        table.add_well((224,32),0, &empty_vec);
        let mut start = 0;
        for i in 1..=7 {
            
            table.add_stack((32 * i as i32,76), &tableau_cards[start..start+i] , i-1);
            start += i;
        }
    }
    fn can_split_stack(stack: &Stack, position: usize, _ : &Table) -> bool {
        position < stack.cards.len() && position >= stack.hidden_point        
    }
    fn can_skim_well(well: &Well) -> bool {
        well.cards.len() > 0
    }
    fn can_place_stack(stack: &Stack, cards: &[Card], _tbl: &Table) -> bool {        
        if let Some(c) = stack.cards.last () {
            cards[0].value + 1 == c.value && match cards[0].suit {
                Suit::Diamonds => c.suit == Suit::Clubs || c.suit == Suit::Spades,
                Suit::Hearts   => c.suit == Suit::Clubs || c.suit == Suit::Spades,
                Suit::Clubs    => c.suit == Suit::Diamonds || c.suit == Suit::Hearts,
                Suit::Spades   => c.suit == Suit::Diamonds || c.suit == Suit::Hearts,
            }
        } else {
            cards[0].value == 13
        }
    }
    fn can_place_well(well: &Well, cards: &[Card]) -> bool { 
        if well.id == 0 { return false };
        if well.cards.len() > 0 {
            cards.len() == 1 && cards[0].suit == well.cards[0].suit && cards[0].value == well.cards.len() as u8 + 1
        } else { 
            cards.len() == 1 && cards[0].value == 1
         }
        
    }
    fn placed_in_stack(table: &mut Table, _stack_id: usize, _cards: usize) {
        refresh_stacks(table);
    }
    fn placed_in_well(table: &mut Table, _well_id: usize, _cards: usize) {
        refresh_stacks(table);
    }
    fn deal_from_deck(table: &mut Table, _deck_id: usize) {
        if table.deck(0).cards.len() > 0 {
            table.shift(GameObject::Deck(0), GameObject::Well(0), V::size_of_draw())
        } else {
            if table.well(0).cards.len() > 0 {
                table.shift(GameObject::Well(0), GameObject::Deck(0), table.well(0).cards.len());                
            }
        }
    }
    fn stack_clicked(table: &mut Table, stack_id: usize, position: usize) {
        let cards = &table.stack(stack_id).cards[position..];
        let l = cards.len();        
        if let Some(loc) = Self::best_location_for(table, cards, GameObject::Stack(stack_id)) {                        
            table.shift_then(GameObject::Stack(stack_id), loc,l,Box::new(move |tbl| {
                match loc {
                    GameObject::Stack(i) => Self::placed_in_stack(tbl, i, l),
                    GameObject::Well(i) => Self::placed_in_well(tbl, i, l),
                    GameObject::Deck(_) => {},
                };
                tbl.end_move();
            }));
        }
    }
    fn well_clicked(table: &mut Table, well_id: usize) {
        if let Some(card) = table.well(well_id).cards.last() {
            if let Some(loc) = Self::best_location_for_card(table, *card, GameObject::Well(well_id)) {
                table.shift_then(GameObject::Well(well_id), loc,1,Box::new(move |tbl| {
                    match loc {
                        GameObject::Stack(i) => Self::placed_in_stack(tbl, i, 1),
                        GameObject::Well(i) => Self::placed_in_well(tbl, i, 1),
                        GameObject::Deck(_) => {},
                    };
                    tbl.end_move();
                }));
            }
        }
    }
    
    fn hint(table: &mut Table) {
        let mut moves: Vec<(GameObject, GameObject, usize)> = Vec::new();
        for i in 0..7 {
            if let Some(c) = table.stack(i).cards.last() {
                let cards = &table.stack(i).cards[table.stack(i).hidden_point..];
                if cards.len() > 1 {
                    if let Some(GameObject::Well(w)) = Self::best_location_for_card(table, *c, GameObject::Stack(i)) {                    
                        moves.push((GameObject::Stack(i),GameObject::Well(w),1));
                    }
                }
                if let Some(l) = Self::best_location_for(table,cards, GameObject::Stack(i)) {
                    moves.push((GameObject::Stack(i),l,cards.len()))
                }
            }
        }
        if let Some(c) = table.well(0).cards.last() {
            if let Some(l) = Self::best_location_for_card(table, *c, GameObject::Well(0)) {
                moves.push((GameObject::Well(0),l,1))
            }
        }
        moves.sort_by(|(s1,d1,_), (s2,d2,_)| match (d1,d2) {
            (GameObject::Well(_),_) => Ordering::Less,
            (_,GameObject::Well(_)) => Ordering::Greater,
            (_,_) => match(s1,s2) {
                (GameObject::Well(_),GameObject::Stack(src2)) =>  table.stack(*src2).hidden_point.cmp(&1) ,
                (GameObject::Stack(src2),GameObject::Well(0)) =>  1.cmp(&table.stack(*src2).hidden_point) ,
                (GameObject::Stack(src1), GameObject::Stack(src2)) =>
                        table.stack(*src2).hidden_point.cmp( &table.stack(*src1).hidden_point),
                (_,_) => Ordering::Equal
            }
        });
        if let Some((s,d,c)) = moves.first() {
            let dd = *d;
            let and_then : Box<dyn FnOnce(&mut Table)> = Box::new(move |tbl| {
                match dd {
                    GameObject::Well(w) => tbl.animate_highlight_well(w, (0,200,100)),
                    GameObject::Stack(w) => tbl.animate_highlight_stack(w, tbl.stack(w).cards.len()-1.min(tbl.stack(w).cards.len()), (0,200,100)),
                    _ => {}
                } 
                
            } );
            match *s {
                GameObject::Well(w) => table.animate_highlight_well_then(w, (100,0,200), and_then),
                GameObject::Stack(s) => { let pos = table.stack(s).cards.len() - *c; table.animate_highlight_stack_then(s, pos, (100,0,200), and_then)},
                _ => {}
            }

        } else {
            table.animate_highlight_deck(0, (200,68,25));
        }
    }
    fn game_won(table: &Table) -> bool {
         
        table.stacks().iter().all(|s| s.cards.len() == 0) && table.deck(0).cards.len() == 0 && table.well(0).cards.len() == 0
    }
}