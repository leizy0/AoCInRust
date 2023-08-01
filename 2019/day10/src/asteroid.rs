use std::{
    cell::RefCell,
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
};

use crate::Error;

#[derive(Debug, Clone, Copy)]
pub struct Asteroid {
    x: usize,
    y: usize,
}

impl Asteroid {
    fn offset_to(&self, other: &Asteroid) -> (i32, i32) {
        (
            i32::try_from(other.x).unwrap() - i32::try_from(self.x).unwrap(),
            i32::try_from(other.y).unwrap() - i32::try_from(self.y).unwrap(),
        )
    }

    fn simplest_offset_to(&self, other: &Asteroid) -> (i32, i32) {
        let (offset_x, offset_y) = self.offset_to(other);
        let offset_gcd = gcd(offset_x, offset_y);

        (offset_x / offset_gcd, offset_y / offset_gcd)
    }
}

pub struct AsteroidMap {
    asteroids: Vec<Asteroid>,
    position_map: Vec<Vec<Option<usize>>>,
    detect_matrix: RefCell<Vec<Vec<Option<usize>>>>,
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
                for a_ind in (0..self.asteroid_count()).filter(|&n| n != ind) {
                    if self.detect_ind(ind, a_ind) == 1 {
                        detect_count += 1;
                    }
                }

                *o = Some(detect_count);
            }

            *o
        })
    }

    pub fn vaporize_order(&self, ind: usize) -> Option<Vec<usize>> {
        if ind >= self.asteroid_count() {
            None
        } else {
            let mut coords = (0..self.asteroid_count())
                .filter(|&n| n != ind)
                .map(|a_ind| {
                    (
                        a_ind,
                        VaporizationCoordinate::new(
                            self.asteroid(ind).unwrap(),
                            self.asteroid(a_ind).unwrap(),
                        ),
                        self.detect_ind(ind, a_ind),
                    )
                })
                .collect::<Vec<_>>();
            coords.sort_unstable_by_key(|(_, vc, _)| *vc);
            coords.sort_by_key(|(_, _, detect_ind)| *detect_ind);
            Some(
                coords
                    .iter()
                    .map(|(a_ind, _, _)| *a_ind)
                    .collect::<Vec<_>>(),
            )
        }
    }

    fn detect_ind(&self, ind0: usize, ind1: usize) -> usize {
        if ind0 == ind1 {
            return 0;
        }

        self.get_detection(ind0, ind1).unwrap_or_else(|| {
            let asteroid0 = &self.asteroids[ind0];
            let asteroid1 = &self.asteroids[ind1];
            let (offset_unit_x, offset_unit_y) = asteroid0.simplest_offset_to(asteroid1);

            self.perform_detection(ind0, offset_unit_x, offset_unit_y);
            self.perform_detection(ind0, -offset_unit_x, -offset_unit_y);

            self.get_detection(ind0, ind1).unwrap()
        })
    }

    fn get_detection(&self, ind0: usize, ind1: usize) -> Option<usize> {
        self.detection_index(ind0, ind1).and_then(|(r_ind, c_ind)| {
            self.detect_matrix
                .borrow()
                .get(r_ind)
                .and_then(|v| v.get(c_ind).and_then(|ob| *ob))
        })
    }

    fn set_detection(&self, ind0: usize, ind1: usize, detect_ind: usize) {
        self.detection_index(ind0, ind1).map(|(r_ind, c_ind)| {
            self.detect_matrix
                .borrow_mut()
                .get_mut(r_ind)
                .map(|v| v.get_mut(c_ind).map(|ob| *ob = Some(detect_ind)));
        });
    }

    fn detection_index(&self, ind0: usize, ind1: usize) -> Option<(usize, usize)> {
        if ind0 >= self.asteroid_count() || ind1 >= self.asteroid_count() || ind0 == ind1 {
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
        if start_ind >= self.asteroid_count() {
            return;
        }

        let mut detect_ind = 1;
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
                    self.set_detection(ind, start_ind, detect_ind);
                }
                detect_ind += 1;
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

#[derive(PartialEq, Eq, Clone, Copy)]
struct VaporizationCoordinate {
    vap_offset_x: i32,
    vap_offset_y: i32,
}

impl VaporizationCoordinate {
    fn new(center_asteroid: &Asteroid, vap_asteroid: &Asteroid) -> Self {
        let (sim_offset_x, sim_offset_y) = center_asteroid.simplest_offset_to(vap_asteroid);
        Self {
            vap_offset_x: sim_offset_x,
            vap_offset_y: sim_offset_y,
        }
    }

    fn with_offset(vap_offset_x: i32, vap_offset_y: i32) -> Self {
        Self {
            vap_offset_x,
            vap_offset_y,
        }
    }

    fn vap_deg(&self) -> f32 {
        let x = self.vap_offset_x as f32;
        let y = self.vap_offset_y as f32;
        let res = y.atan2(x) + std::f32::consts::FRAC_PI_2;
        if res < 0.0 {
            res + std::f32::consts::PI * 2.0
        } else {
            res
        }
    }
}

impl PartialOrd for VaporizationCoordinate {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for VaporizationCoordinate {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.vap_deg().total_cmp(&other.vap_deg())
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

#[test]
fn test_vc_vap_degree_axis() {
    assert!(f_eq(
        VaporizationCoordinate::with_offset(0, -1).vap_deg(),
        0.0
    ));
    assert!(f_eq(
        VaporizationCoordinate::with_offset(1, 0).vap_deg(),
        std::f32::consts::FRAC_PI_2
    ));
    assert!(f_eq(
        VaporizationCoordinate::with_offset(0, 1).vap_deg(),
        std::f32::consts::PI
    ));
    assert!(f_eq(
        VaporizationCoordinate::with_offset(-1, 0).vap_deg(),
        std::f32::consts::PI * 3.0 / 2.0
    ));
}

#[test]
fn test_vc_vap_degree_pi_over_4() {
    assert!(f_eq(
        VaporizationCoordinate::with_offset(1, -1).vap_deg(),
        std::f32::consts::FRAC_PI_4
    ));
    assert!(f_eq(
        VaporizationCoordinate::with_offset(1, 1).vap_deg(),
        std::f32::consts::FRAC_PI_4 * 3.0
    ));
    assert!(f_eq(
        VaporizationCoordinate::with_offset(-1, 1).vap_deg(),
        std::f32::consts::FRAC_PI_4 * 5.0
    ));
    assert!(f_eq(
        VaporizationCoordinate::with_offset(-1, -1).vap_deg(),
        std::f32::consts::FRAC_PI_4 * 7.0
    ));
}

fn f_eq(l: f32, r: f32) -> bool {
    (l - r).abs() <= std::f32::EPSILON
}
