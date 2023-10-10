use std::path::{Path,PathBuf};
use std::fs::File;

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};


pub struct ScoreTable {
    scores: [[u32; 3]; super::MAX_LEVEL as usize],
    filename: PathBuf,
}

impl ScoreTable {
    pub fn new(filename: &Path) -> ::std::io::Result<ScoreTable> {
        let mut it = ScoreTable {
            scores: [[0; 3]; super::MAX_LEVEL as usize],
            filename: filename.to_path_buf(),
        };
        match File::open(&it.filename) {
            Ok(mut file) => {
                for i in 0..super::MAX_LEVEL as usize {
                    for j in 0..3 {
                        it.scores[i][j] = file.read_u32::<LittleEndian>()?;
                    }
                }
            }
            Err(_) => {}
        }
        Ok(it)
    }

    pub fn save_scores(&self) -> ::std::io::Result<()> {
        let mut file = File::create(&self.filename)?;
        for i in 0..super::MAX_LEVEL as usize {
            for j in 0..3 {
                file.write_u32::<LittleEndian>(self.scores[i][j])?;
            }
        }
        Ok(())
    }
    pub fn get_top_score(&self, c: &super::Config) -> u32 {
        self.scores[c.level as usize][c.speed.to_index()]
    }
    pub fn clear_score(&mut self, c:&super::Config) -> ::std::io::Result<()> {
        self.scores[c.level as usize][c.speed.to_index()] = 0;
        self.save_scores()?;
        Ok(())
    }

    pub fn update_scores(&mut self, c: &super::Config, score: u32) -> ::std::io::Result<()> {
        if self.scores[c.level as usize][c.speed.to_index()] < score {
            self.scores[c.level as usize][c.speed.to_index()] = score;
            self.save_scores()?;
        }
        Ok(())
    }
}
