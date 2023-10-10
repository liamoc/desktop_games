use imprint::{Imprint, Cell};
use rand::{Rng};
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum PieceColor {
  Red, Yellow,Green
}
impl PieceColor {
  pub fn to_index(&self) -> usize {
    match *self {
      PieceColor::Red => 0,
      PieceColor::Yellow => 1,
      PieceColor::Green => 2
    }
  }
}
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Piece {
    CapH(PieceColor, PieceColor),
    CapV(PieceColor, PieceColor),
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum FieldTile {
    Orphan(PieceColor), CapLeft(PieceColor), CapRight(PieceColor), CapTop(PieceColor), CapBottom(PieceColor), Virus(PieceColor)
}
impl FieldTile {
  pub fn color(&self) -> PieceColor {
    match *self {
      FieldTile::CapBottom(c) => c,
      FieldTile::CapLeft(c) => c,
      FieldTile::CapRight(c) => c,
      FieldTile::CapTop(c) => c,
      FieldTile::Orphan(c) => c,
      FieldTile::Virus(c) => c
    }
  }
}

impl Piece {
    pub fn rand<R: Rng>(rng: &mut R) -> Piece {
        let c1: u8 = rng.gen_range(0, 3);
        let c2: u8 = rng.gen_range(0, 3);
        
        let col1= match c1 {
            0 => PieceColor::Red,
            1 => PieceColor::Green,
            _ => PieceColor::Yellow,
        };
        let col2= match c2 {
          0 => PieceColor::Red,
          1 => PieceColor::Green,
          _ => PieceColor::Yellow,
        };
        return Piece::CapH(col1,col2)
    }
    pub fn imprint(&self) -> Imprint<FieldTile> {
        match *self {
          Piece::CapH(c1,c2 ) => {
            Imprint::from_footprint(
              &[&[0, 0],
                       &[1, 2]],
              |i| if i == 1 { Cell::Filled(FieldTile::CapLeft(c1))} else { Cell::Filled(FieldTile::CapRight(c2))})
          },
          Piece::CapV(c1,c2 ) => {
            Imprint::from_footprint(
              &[&[1, 0],
                       &[2, 0]],
              |i| if i == 1 { Cell::Filled(FieldTile::CapTop(c1))} else { Cell::Filled(FieldTile::CapBottom(c2))})
          },
        }
    }
    pub fn rotate_r(&self) -> Piece {
        match *self {
            Piece::CapH(c1,c2) => Piece::CapV(c1, c2),
            Piece::CapV(c1,c2 ) => Piece::CapH(c2,c1)
        }
    }
    pub fn rotate_l(&self) -> Piece {
        self.rotate_r().rotate_r().rotate_r()
    }
}
