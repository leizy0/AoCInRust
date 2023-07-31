use std::{
    cell::RefCell,
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
};

use crate::Error;

#[derive(Debug)]
pub struct Asteroid {
    x: usize,
    y: usize,
}

pub struct AsteroidMap {
    asteroids: Vec<Asteroid>,
    position_map: Vec<Vec<Option<usize>>>,
    detect_matrix: RefCell<Vec<Vec<Option<bool>>>>,
    detect_counts: RefCell<Vec<Option<usize>>>,
}

impl TryFrom<&[String]> for AsteroidMap {
    type Error = Error;

    fn try_from(value: &[String]) -> Result<Self, Self::Error> {
        let row_count = value.len();
        let mut map = Self {
            asteroids: Vec::new(),
            position_map: Vec::with_capacity(row_count),
            detect_matrix: RefCell::new(Vec::new()),
            detect_counts: RefCell::new(Vec::new()),
        };

        for (r_ind, s) in value.iter().enumerate() {
            let mut row = Vec::new();
            for (c_ind, c) in s.chars().enumerate() {
                match c {
                    '#' => {
                        map.asteroids.push(Asteroid { x: c_ind, y: r_ind });
                        row.push(Some(map.asteroids.len() - 1));
                        map.detect_counts.borrow_mut().push(None);
                    }
                    '.' => row.push(None),
                    _ => return Err(Error::InvalidCharacterInMap(r_ind, c_ind, c)),
                }
            }

            map.position_map.push(row);
        }

        let asteroid_count = map.asteroids.len();
        for a_ind in 0..(asteroid_count - 1) {
            map.detect_matrix
                .borrow_mut()
                .push(vec![None; asteroid_count - a_ind - 1]);
        }

        Ok(map)
    }
}

impl AsteroidMap {
    pub fn asteroid_count(&self) -> usize {
        self.asteroids.len()
    }

    pub fn asteroid(&self, ind: usize) -> Option<&Asteroid> {
        self.asteroids.get(ind)
    }

    pub fn detect_count(&self, ind: usize) -> Option<usize> {
        self.detect_counts.borrow_mut().get_mut(ind).and_then(|o| {
            if o.is_none() {
                let mut detect_count = 0;
                for a_ind in (0..self.asteroids.len()).filter(|&n| n != ind) {
                    if self.can_detect(ind, a_ind) {
                        detect_count += 1;
                    }
                }

                *o = Some(detect_count);
            }

            *o
        })
    }

    fn can_detect(&self, ind0: usize, ind1: usize) -> bool {
        if ind0 == ind1 {
            return true;
        }

        self.get_detection(ind0, ind1).unwrap_or_else(|| {
            let asteroid0 = &self.asteroids[ind0];
            let asteroid1 = &self.asteroids[ind1];
            let offset_x =
                i32::try_from(asteroid0.x).unwrap() - i32::try_from(asteroid1.x).unwrap();
            let offset_y =
                i32::try_from(asteroid0.y).unwrap() - i32::try_from(asteroid1.y).unwrap();
            let offset_gcd = gcd(offset_x, offset_y);
            let offset_unit_x = offset_x / offset_gcd;
            let offset_unit_y = offset_y / offset_gcd;

            self.perform_detection(ind0, offset_unit_x, offset_unit_y);
            self.perform_detection(ind0, -offset_unit_x, -offset_unit_y);

            self.get_detection(ind0, ind1).unwrap()
        })
    }

    fn get_detection(&self, ind0: usize, ind1: usize) -> Option<bool> {
        self.detection_index(ind0, ind1).and_then(|(r_ind, c_ind)| {
            self.detect_matrix
                .borrow()
                .get(r_ind)
                .and_then(|v| v.get(c_ind).and_then(|ob| *ob))
        })
    }

    fn set_detection(&self, ind0: usize, ind1: usize, detectable: bool) {
        self.detection_index(ind0, ind1).map(|(r_ind, c_ind)| {
            self.detect_matrix
                .borrow_mut()
                .get_mut(r_ind)
                .map(|v| v.get_mut(c_ind).map(|ob| *ob = Some(detectable)));
        });
    }

    fn detection_index(&self, ind0: usize, ind1: usize) -> Option<(usize, usize)> {
        if ind0 >= self.asteroids.len() || ind1 >= self.asteroids.len() || ind0 == ind1 {
            None
        } else {
            let ind_min = ind0.min(ind1);
            let ind_max = ind0.max(ind1);
            let r_ind = ind_min;
            let c_ind = ind_max - ind_min - 1;

            Some((r_ind, c_ind))
        }
    }

    fn perform_detection(&self, start_ind: usize, offset_x: i32, offset_y: i32) {
        if start_ind >= self.asteroids.len() {
            return;
        }

        let mut detected = false;
        let asteroid = &self.asteroids[start_ind];
        let mut cur_x = i32::try_from(asteroid.x).unwrap();
        let mut cur_y = i32::try_from(asteroid.y).unwrap();
        loop {
            cur_x += offset_x;
            cur_y += offset_y;
            if self.exceed_border(cur_x, cur_y) {
                break;
            }

            self.asteroid_index(cur_x, cur_y).map(|ind| {
                if self.get_detection(ind, start_ind).is_none() {
                    self.set_detection(ind, start_ind, !detected);
                    if !detected {
                        detected = true;
                    }
                }
            });
        }
    }

    fn exceed_border(&self, x: i32, y: i32) -> bool {
        x < 0
            || y < 0
            || y as usize >= self.position_map.len()
            || x as usize >= self.position_map[y as usize].len()
    }

    fn asteroid_index(&self, x: i32, y: i32) -> Option<usize> {
        if self.exceed_border(x, y) {
            None
        } else {
            self.position_map[y as usize][x as usize]
        }
    }
}

pub fn read_map<P>(path: P) -> Result<AsteroidMap, Error>
where
    P: AsRef<Path>,
{
    let input_file = File::open(path).map_err(Error::IOError)?;
    let reader = BufReader::new(input_file);
    let lines = reader
        .lines()
        .map(|r| r.map_err(Error::IOError))
        .collect::<Result<Vec<_>, Error>>()?;

    AsteroidMap::try_from(lines.as_slice())
}

fn gcd(n0: i32, n1: i32) -> i32 {
    let n0 = n0.abs();
    let n1 = n1.abs();
    let mut large = n0.max(n1);
    let mut small = n0.min(n1);

    while small != 0 {
        let rem = large % small;
        large = small;
        small = rem;
    }

    large
}
