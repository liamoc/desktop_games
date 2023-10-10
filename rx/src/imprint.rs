use std::ops::{Index, IndexMut};

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Cell<A> {
    Empty,
    Filled(A),
}
impl <A> Cell<A> {
    pub fn is_empty(&self) -> bool {
        match *self {
            Cell::Empty => true,
            _ => false
        }
    }
    pub fn as_option(self) -> Option<A> {
        match self {
            Cell::Empty => None,
            Cell::Filled(x) => Some(x)
        }        
    }
}

pub struct Imprint<A> {
    footprint: Vec<Cell<A>>,
    width: usize,
    height: usize,
}
impl <A : Copy> Clone for Imprint<A> {
    fn clone(&self) -> Self {
        Imprint {
            footprint: self.footprint.clone(),
            width: self.width,
            height: self.height,
        }
    }
}
impl <A : Copy> Imprint<A> {

    pub fn empty(width: usize, height: usize) -> Imprint<A> {
        Imprint {
            footprint: vec![Cell::Empty; width * height],
            width: width,
            height: height,
        }
    }

    pub fn from_footprint<F>(print: &[&[u8]], style: F) -> Imprint<A>  where F : Fn(u8)-> Cell<A>{
        let h = print.len();
        let w = print[0].len();
        let mut it = Imprint::empty(w, h);
        for y in 0..h {
            for x in 0..w {
                it[(x, y)] = if print[y][x] > 0 { style(print[y][x]) } else { Cell::Empty };
            }
        }
        it
    }

    pub fn full_lines(&self, results: &mut Vec<usize>) -> bool {
        let mut all = false;
        for y in 0..self.height {
            let mut row = true;
            for x in 0..self.width {
                row = row && !self[(x, y)].is_empty()
            }
            if row {
                results.push(y)
            }
            all = all || row;
        }
        all
    }


    pub fn size(&self) -> (usize, usize) {
        return (self.width, self.height);
    }




    pub fn accepts(&self, other: &Imprint<A>, (x0, y0): (i32, i32)) -> bool {
        for y in 0..other.height {
            for x in 0..other.width {
                if !other[(x, y)].is_empty() {
                    let xx = x0 + x as i32;
                    let yy = y0 + y as i32;
                    if xx >= self.width as i32 || yy >= self.height as i32 || xx < 0 || yy < 0 {
                        return false;
                    }
                    if !self[(xx as usize, yy as usize)].is_empty() {
                        return false;
                    }
                }
            }
        }
        return true;
    }

    pub fn all_clear(&self, range: usize) -> bool {
        for y in 0..range {
            for x in 0..self.width {
                if !self[(x, y)].is_empty() {
                    return false;
                }
            }
        }
        true
    }

    pub fn stamp(&mut self, other: &Imprint<A>, (x0, y0): (i32, i32)) {
        for y in 0..other.height {
            for x in 0..other.width {
                if !other[(x, y)].is_empty() {
                    let xx = x0 + x as i32;
                    let yy = y0 + y as i32;
                    if xx < self.width as i32 && yy < self.height as i32 && xx >= 0 && yy >= 0 {
                        self[(xx as usize, yy as usize)] = other[(x, y)];
                    }
                }
            }
        }
    }
}

impl <A> Index<(usize, usize)> for Imprint<A> {
    type Output = Cell<A>;
    fn index<'a>(&'a self, (x, y): (usize, usize)) -> &'a Cell<A> {
        &self.footprint[y * self.width + x]
    }
}

impl <A> IndexMut<(usize, usize)> for Imprint<A> {
    fn index_mut<'a>(&'a mut self, (x, y): (usize, usize)) -> &'a mut Cell<A> {
        &mut self.footprint[y * self.width + x]
    }
}
