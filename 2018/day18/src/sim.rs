use std::mem;

use crate::{
    map::{LumberBlock, LumberMap},
    Error,
};

pub struct Simulator {
    tick: usize,
    map: LumberMap,
    temp_map: LumberMap,
    chg_mask: Vec<bool>,
    temp_chg_mask: Vec<bool>,
}

impl Simulator {
    pub fn new(map: LumberMap) -> Simulator {
        Simulator {
            tick: 0,
            temp_map: map.clone(),
            chg_mask: vec![true; map.len()],
            temp_chg_mask: vec![false; map.len()],
            map,
        }
    }

    pub fn simulate(&mut self, tick_count: usize) -> Result<(usize, usize), Error> {
        for _ in 0..tick_count {
            if !self.simulate_tick()? {
                break;
            }
            let (tree_count, lumberyard_count) = self.count_lumber();
            println!(
                "Tick #{}: tree count = {}, lumberyard count = {}",
                self.tick, tree_count, lumberyard_count
            );
        }

        Ok(self.count_lumber())
    }

    fn simulate_tick(&mut self) -> Result<bool, Error> {
        self.temp_chg_mask.fill(false);
        let mut has_changed = false;
        for i in 0..self.chg_mask.len() {
            let block = if !self.chg_mask[i] {
                *self.map.row_major_at(i)?
            } else {
                let (neighbor_ind_iter, neighbor_iter) = self.map.neighbor_8(i)?;
                let (cur_has_changed, block) = self.map.row_major_at(i)?.change(neighbor_iter);
                if cur_has_changed {
                    neighbor_ind_iter.for_each(|ind| self.temp_chg_mask[ind] = true);
                    has_changed = true;
                }

                block
            };

            *self.temp_map.row_major_at_mut(i)? = block;
        }

        mem::swap(&mut self.map, &mut self.temp_map);
        mem::swap(&mut self.chg_mask, &mut self.temp_chg_mask);
        self.tick += 1;
        Ok(has_changed)
    }

    fn count_lumber(&self) -> (usize, usize) {
        let mut tree_count = 0;
        let mut lumberyard_count = 0;
        for i in 0..self.map.len() {
            match self.map.row_major_at(i).unwrap() {
                LumberBlock::Tree => tree_count += 1,
                LumberBlock::Lumberyard => lumberyard_count += 1,
                _ => (),
            }
        }

        (tree_count, lumberyard_count)
    }
}
