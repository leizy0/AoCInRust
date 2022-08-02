use std::{ops::Range, collections::LinkedList};

use crate::{map::{UnderGroundMap, WaterMap, FlowEndType, WaterRowRange, BlockRange}, Position, Error};

pub struct Simulator {
    under_map: UnderGroundMap
}

impl Simulator {
    pub fn new(under_map: UnderGroundMap) -> Simulator {
        Simulator { under_map }
    }

    pub fn simulate(&self, water_src: &Position, vert_range: &Range<usize>) -> Result<WaterMap, Error> {
        let mut src_queue = LinkedList::new();
        src_queue.push_back(*water_src);
        let mut water_map = WaterMap::new(&vert_range);
        let mut blocked_map = self.under_map.clone();
        while let Some(src) = src_queue.pop_front() {
            // map::log_map_text(&self.under_map, &water_map)?;
            // println!();

            let mut cur_r = src.r;
            if blocked_map.is_blocked(cur_r, src.c) {
                // Water source in rest water, ignore it
                continue;
            }
            while !blocked_map.is_blocked(cur_r, src.c) && cur_r < vert_range.end {
                water_map.add_row_range(cur_r, &WaterRowRange::reach(&BlockRange::beg_len(src.c, 1)));
                cur_r += 1;
            }

            // Exceed vertical limit, drop this source
            if cur_r >= vert_range.end {
                // map::log_map_text(&self.under_map, &water_map)?;
                // println!();
                continue;
            }

            // Above clay, flow to left and right
            cur_r -= 1;
            let mut has_leak = false;
            while !has_leak {
                // Flow to left and right
                let (left_end, right_end) = blocked_map.flow_hort(cur_r, src.c)?;
                for end in [&left_end, &right_end] {
                    if let FlowEndType::Leak = end.end_type {
                        has_leak = true;
                        let new_src_pos = Position::new(cur_r, end.ind);
                        if !src_queue.iter().find(|p| **p == new_src_pos).is_some() {
                            src_queue.push_back(new_src_pos);
                        }
                    }
                }

                // Ignore two end points
                let range = BlockRange::beg_end(left_end.ind + 1, right_end.ind);
                if has_leak {
                    water_map.add_row_range(cur_r, &WaterRowRange::reach(&range));
                } else {
                    blocked_map.add_row_range(cur_r, &range);
                    water_map.add_row_range(cur_r, &WaterRowRange::rest(&range));
                }
                cur_r -= 1;
            }
        }

        Ok(water_map)
    }
}
