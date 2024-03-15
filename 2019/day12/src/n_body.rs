use std::{
    fs::File,
    io::{BufRead, BufReader},
    iter::Sum,
    ops::{AddAssign, Div, Index, IndexMut, Neg, Sub},
    path::Path,
    str::FromStr,
};

use once_cell::sync::Lazy;
use regex::Regex;

use crate::Error;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Vec3 {
    v: [i32; 3],
}

impl Neg for Vec3 {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self::new(-self.v[0], -self.v[1], -self.v[2])
    }
}

impl Sub for Vec3 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        self - &rhs
    }
}

impl Sub<&Vec3> for Vec3 {
    type Output = Self;

    fn sub(self, rhs: &Vec3) -> Self::Output {
        Self::new(
            self.v[0] - rhs.v[0],
            self.v[1] - rhs.v[1],
            self.v[2] - rhs.v[2],
        )
    }
}

impl AddAssign for Vec3 {
    fn add_assign(&mut self, rhs: Self) {
        *self += &rhs;
    }
}

impl AddAssign<&Vec3> for Vec3 {
    fn add_assign(&mut self, rhs: &Vec3) {
        self.v[0] += rhs.v[0];
        self.v[1] += rhs.v[1];
        self.v[2] += rhs.v[2];
    }
}

impl Sum for Vec3 {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        let mut sum = Self::default();
        for b in iter {
            sum += b;
        }

        sum
    }
}

impl Div<i32> for Vec3 {
    type Output = Self;

    fn div(self, rhs: i32) -> Self::Output {
        Self::new(self.v[0] / rhs, self.v[1] / rhs, self.v[2] / rhs)
    }
}

impl Index<usize> for Vec3 {
    type Output = i32;

    fn index(&self, index: usize) -> &Self::Output {
        &self.v[index]
    }
}

impl IndexMut<usize> for Vec3 {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.v[index]
    }
}

impl Vec3 {
    fn new(x: i32, y: i32, z: i32) -> Self {
        Self { v: [x, y, z] }
    }

    fn zeros() -> Self {
        Self::new(0, 0, 0)
    }

    fn signum(&self) -> Self {
        Self::new(self.v[0].signum(), self.v[1].signum(), self.v[2].signum())
    }

    fn abs_sum(&self) -> u32 {
        self.v.iter().map(|n| n.abs_diff(0)).sum()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Body {
    pos: Vec3,
    vel: Vec3,
}

impl FromStr for Body {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        static BODY_PATTERN: Lazy<Regex> =
            Lazy::new(|| Regex::new(r"<x=([+-]?\d+), y=([+-]?\d+), z=([+-]?\d+)>").unwrap());

        BODY_PATTERN
            .captures(s)
            .map(|caps| {
                Body::with_pos(
                    str::parse::<i32>(&caps[1]).unwrap(),
                    str::parse::<i32>(&caps[2]).unwrap(),
                    str::parse::<i32>(&caps[3]).unwrap(),
                )
            })
            .ok_or(Error::InvalidBodyDescription(s.to_string()))
    }
}

impl Body {
    fn with_pos(pos_x: i32, pos_y: i32, pos_z: i32) -> Self {
        Self {
            pos: Vec3::new(pos_x, pos_y, pos_z),
            vel: Vec3::default(),
        }
    }

    fn grav_vel_from(&self, other: &Body) -> Vec3 {
        (other.pos - self.pos).signum()
    }

    fn vel_mut(&mut self) -> &mut Vec3 {
        &mut self.vel
    }

    fn apply_vel(&mut self) {
        self.pos += self.vel;
    }

    fn potential_energy(&self) -> u32 {
        self.pos.abs_sum()
    }

    fn kinetic_energy(&self) -> u32 {
        self.vel.abs_sum()
    }

    fn total_energy(&self) -> u32 {
        self.potential_energy() * self.kinetic_energy()
    }
}

#[cfg(not(any(feature = "use_avx2", feature = "multithread")))]
// General implementation
pub struct NBodySimulator {
    bodies: Vec<Body>,
}

#[cfg(not(any(feature = "use_avx2", feature = "multithread")))]
impl NBodySimulator {
    pub fn new(bodies: Vec<Body>) -> Self {
        Self { bodies }
    }

    pub fn body_count(&self) -> usize {
        self.bodies.len()
    }

    pub fn nth_body(&self, n: usize) -> Body {
        self.bodies[n].clone()
    }

    pub fn step(&mut self) {
        let body_count = self.bodies.len();
        if body_count == 4 {
            return self.step_4_bodies();
        }

        for i in 0..body_count {
            for j in (i + 1)..body_count {
                let grav_vel = self.bodies[i].grav_vel_from(&self.bodies[j]);
                *self.bodies[i].vel_mut() += grav_vel;
                *self.bodies[j].vel_mut() += -grav_vel;
            }

            self.bodies[i].apply_vel();
        }
    }

    fn step_4_bodies(&mut self) {
        let mut delta_v0 = Vec3::zeros();
        let mut delta_v1 = Vec3::zeros();
        let mut delta_v2 = Vec3::zeros();
        let mut delta_v3 = Vec3::zeros();

        fn update(
            bodies: &mut [Body],
            body_ind0: usize,
            body_ind1: usize,
            delta_v_r_0: &mut Vec3,
            delta_v_r_1: &mut Vec3,
        ) {
            let delta_v = bodies[body_ind0].grav_vel_from(&bodies[body_ind1]);
            *delta_v_r_0 += delta_v;
            *delta_v_r_1 += -delta_v;
        }

        update(&mut self.bodies, 0, 1, &mut delta_v0, &mut delta_v1);
        update(&mut self.bodies, 0, 2, &mut delta_v0, &mut delta_v2);
        update(&mut self.bodies, 0, 3, &mut delta_v0, &mut delta_v3);
        update(&mut self.bodies, 1, 2, &mut delta_v1, &mut delta_v2);
        update(&mut self.bodies, 1, 3, &mut delta_v1, &mut delta_v3);
        update(&mut self.bodies, 2, 3, &mut delta_v2, &mut delta_v3);

        *self.bodies[0].vel_mut() += delta_v0;
        self.bodies[0].apply_vel();
        *self.bodies[1].vel_mut() += delta_v1;
        self.bodies[1].apply_vel();
        *self.bodies[2].vel_mut() += delta_v2;
        self.bodies[2].apply_vel();
        *self.bodies[3].vel_mut() += delta_v3;
        self.bodies[3].apply_vel();
    }

    pub fn potential_energy(&self) -> u32 {
        self.bodies.iter().map(|b| b.potential_energy()).sum()
    }

    pub fn kinetic_energy(&self) -> u32 {
        self.bodies.iter().map(|b| b.kinetic_energy()).sum()
    }

    pub fn total_energy(&self) -> u32 {
        self.bodies.iter().map(|b| b.total_energy()).sum()
    }

    pub fn center_pos(&self) -> Vec3 {
        self.bodies.iter().map(|b| b.pos).sum::<Vec3>() / (self.bodies.len() as i32)
    }

    pub fn vel_sum(&self) -> Vec3 {
        self.bodies.iter().map(|b| b.vel).sum::<Vec3>()
    }
}

#[cfg(feature = "multithread")]
pub use multithread::NBodySimulator;

#[cfg(feature = "multithread")]
mod multithread {
    use rayon::prelude::*;

    use super::{Body, Vec3};

    struct Bodies1D {
        pos_v: Vec<i32>,
        vel_v: Vec<i32>,
    }
    // Implementation using multiple threads to update each dimension.
    pub struct NBodySimulator {
        dimensions: [Bodies1D; 3],
    }

    impl NBodySimulator {
        pub fn new(bodies: Vec<Body>) -> Self {
            let dim_x = Bodies1D {
                pos_v: bodies.iter().map(|b| b.pos[0]).collect(),
                vel_v: bodies.iter().map(|b| b.vel[0]).collect(),
            };
            let dim_y = Bodies1D {
                pos_v: bodies.iter().map(|b| b.pos[1]).collect(),
                vel_v: bodies.iter().map(|b| b.vel[1]).collect(),
            };
            let dim_z = Bodies1D {
                pos_v: bodies.iter().map(|b| b.pos[2]).collect(),
                vel_v: bodies.iter().map(|b| b.vel[2]).collect(),
            };

            Self {
                dimensions: [dim_x, dim_y, dim_z],
            }
        }

        pub fn body_count(&self) -> usize {
            self.dimensions[0].pos_v.len()
        }

        pub fn nth_body(&self, n: usize) -> Body {
            Body {
                pos: Vec3::new(
                    self.dimensions[0].pos_v[n],
                    self.dimensions[1].pos_v[n],
                    self.dimensions[2].pos_v[n],
                ),
                vel: Vec3::new(
                    self.dimensions[0].vel_v[n],
                    self.dimensions[1].vel_v[n],
                    self.dimensions[2].vel_v[n],
                ),
            }
        }

        pub fn step(&mut self) {
            self.dimensions.par_iter_mut().for_each(|b1d| {
                let pos_v = &mut b1d.pos_v;
                let vel_v = &mut b1d.vel_v;
                let body_count = pos_v.len();
                assert!(
                    pos_v.len() == vel_v.len(),
                    "Length of position vector and velocity vector should be equal"
                );
                for i in 0..body_count {
                    for j in (i + 1)..body_count {
                        let delta_vel = (pos_v[j] - pos_v[i]).signum();
                        vel_v[i] += delta_vel;
                        vel_v[j] -= delta_vel;
                    }

                    pos_v[i] += vel_v[i];
                }
            });
        }

        pub fn potential_energy(&self) -> u32 {
            (0..self.body_count())
                .into_iter()
                .map(|i| self.nth_body(i).potential_energy())
                .sum()
        }

        pub fn kinetic_energy(&self) -> u32 {
            (0..self.body_count())
                .into_iter()
                .map(|i| self.nth_body(i).kinetic_energy())
                .sum()
        }

        pub fn total_energy(&self) -> u32 {
            (0..self.body_count())
                .into_iter()
                .map(|i| self.nth_body(i).total_energy())
                .sum()
        }

        pub fn center_pos(&self) -> Vec3 {
            let count = self.body_count();
            Vec3::new(
                self.dimensions[0].pos_v.iter().sum::<i32>() / count as i32,
                self.dimensions[1].pos_v.iter().sum::<i32>() / count as i32,
                self.dimensions[2].pos_v.iter().sum::<i32>() / count as i32,
            )
        }

        pub fn vel_sum(&self) -> Vec3 {
            Vec3::new(
                self.dimensions[0].vel_v.iter().sum(),
                self.dimensions[1].vel_v.iter().sum(),
                self.dimensions[2].vel_v.iter().sum(),
            )
        }
    }
}

#[cfg(all(
    any(target_arch = "x86", target_arch = "x86_64"),
    target_feature = "avx2",
    feature = "use_avx2"
))]
pub use avx2::NBodySimulator;

#[cfg(all(
    any(target_arch = "x86", target_arch = "x86_64"),
    target_feature = "avx2",
    feature = "use_avx2"
))]
mod avx2 {
    use super::{Body, Vec3};
    use aligned_vec::AVec;
    use std::iter;

    pub struct NBodySimulator {
        count: usize,
        pos_x: AVec<i32>,
        pos_y: AVec<i32>,
        pos_z: AVec<i32>,
        vel_x: AVec<i32>,
        vel_y: AVec<i32>,
        vel_z: AVec<i32>,
        acc_x: AVec<i32>,
        acc_y: AVec<i32>,
        acc_z: AVec<i32>,
        pad_mask: AVec<i32>,
    }

    impl NBodySimulator {
        pub fn new(bodies: Vec<Body>) -> Self {
            let pad_count = 8 - bodies.len() % 8;
            let total_len = bodies.len() + pad_count;
            fn fill_aligned_vec(_: usize, iter: impl Iterator<Item = i32>) -> AVec<i32> {
                AVec::from_iter(32, iter)
            }

            Self {
                count: bodies.len(),
                pos_x: fill_aligned_vec(
                    total_len,
                    bodies
                        .iter()
                        .map(|b| b.pos[0])
                        .chain(iter::repeat(0).take(pad_count)),
                ),
                pos_y: fill_aligned_vec(
                    total_len,
                    bodies
                        .iter()
                        .map(|b| b.pos[1])
                        .chain(iter::repeat(0).take(pad_count)),
                ),
                pos_z: fill_aligned_vec(
                    total_len,
                    bodies
                        .iter()
                        .map(|b| b.pos[2])
                        .chain(iter::repeat(0).take(pad_count)),
                ),
                vel_x: fill_aligned_vec(
                    total_len,
                    bodies
                        .iter()
                        .map(|b| b.vel[0])
                        .chain(iter::repeat(0).take(pad_count)),
                ),
                vel_y: fill_aligned_vec(
                    total_len,
                    bodies
                        .iter()
                        .map(|b| b.vel[1])
                        .chain(iter::repeat(0).take(pad_count)),
                ),
                vel_z: fill_aligned_vec(
                    total_len,
                    bodies
                        .iter()
                        .map(|b| b.vel[2])
                        .chain(iter::repeat(0).take(pad_count)),
                ),
                acc_x: fill_aligned_vec(total_len, iter::repeat(0).take(total_len)),
                acc_y: fill_aligned_vec(total_len, iter::repeat(0).take(total_len)),
                acc_z: fill_aligned_vec(total_len, iter::repeat(0).take(total_len)),
                pad_mask: fill_aligned_vec(
                    total_len,
                    iter::repeat(-1)
                        .take(bodies.len())
                        .chain(iter::repeat(0).take(pad_count)),
                ),
            }
        }

        pub fn body_count(&self) -> usize {
            self.count
        }

        pub fn nth_body(&self, n: usize) -> Body {
            Body {
                pos: Vec3::new(self.pos_x[n], self.pos_y[n], self.pos_z[n]),
                vel: Vec3::new(self.vel_x[n], self.vel_y[n], self.vel_z[n]),
            }
        }

        pub fn step(&mut self) {
            #[cfg(target_arch = "x86")]
            use std::arch::x86::*;
            #[cfg(target_arch = "x86_64")]
            use std::arch::x86_64::*;

            let lane_count = 8;
            let block_count = self.pos_x.len() / lane_count;
            let pos_pointers = [
                self.pos_x.as_mut_ptr(),
                self.pos_y.as_mut_ptr(),
                self.pos_z.as_mut_ptr(),
            ];
            let vel_pointers = [
                self.vel_x.as_mut_ptr(),
                self.vel_y.as_mut_ptr(),
                self.vel_z.as_mut_ptr(),
            ];
            let acc_pointers = [
                self.acc_x.as_mut_ptr(),
                self.acc_y.as_mut_ptr(),
                self.acc_z.as_mut_ptr(),
            ];

            unsafe {
                let ones = _mm256_set1_epi32(1);
                for d in 0..3 {
                    // Iterate on dimension x, y, z
                    // Calculate acceleration of every body in system.
                    for i in 0..self.count {
                        // Iterate on i_th body
                        let body_pos_lanes = _mm256_set1_epi32(*pos_pointers[d].add(i));
                        let mut this_body_acc = _mm256_set1_epi32(0);
                        for b in 0..block_count {
                            // Iterate on b_th block of body data
                            let block_mask = _mm256_load_si256(
                                self.pad_mask.as_ptr().add(b * lane_count) as *const _,
                            );
                            let this_body_pos = _mm256_and_si256(body_pos_lanes, block_mask);
                            let other_bodies_pos =
                                _mm256_load_si256(pos_pointers[d].add(b * lane_count) as *const _);
                            let body_pos_diff = _mm256_sub_epi32(other_bodies_pos, this_body_pos);
                            let this_block_acc = _mm256_sign_epi32(ones, body_pos_diff);
                            this_body_acc = _mm256_add_epi32(this_body_acc, this_block_acc);
                        }
                        *acc_pointers[d].add(i) = hsum_8x32(this_body_acc);
                    }

                    // Update velocity and position in this dimension.
                    for b2 in 0..block_count {
                        let block_mask = _mm256_load_si256(
                            self.pad_mask.as_ptr().add(b2 * lane_count) as *const _,
                        );
                        let this_block_acc = _mm256_maskload_epi32(
                            acc_pointers[d].add(b2 * lane_count) as *const _,
                            block_mask,
                        );
                        let mut this_block_vel = _mm256_maskload_epi32(
                            vel_pointers[d].add(b2 * lane_count) as *const _,
                            block_mask,
                        );
                        this_block_vel = _mm256_add_epi32(this_block_vel, this_block_acc);
                        let mut this_block_pos = _mm256_maskload_epi32(
                            pos_pointers[d].add(b2 * lane_count) as *const _,
                            block_mask,
                        );
                        this_block_pos = _mm256_add_epi32(this_block_pos, this_block_vel);

                        _mm256_maskstore_epi32(
                            vel_pointers[d].add(b2 * lane_count),
                            block_mask,
                            this_block_vel,
                        );
                        _mm256_maskstore_epi32(
                            pos_pointers[d].add(b2 * lane_count),
                            block_mask,
                            this_block_pos,
                        );
                    }
                }
            }
        }

        pub fn potential_energy(&self) -> u32 {
            (0..self.count)
                .into_iter()
                .map(|i| self.nth_body(i).potential_energy())
                .sum()
        }

        pub fn kinetic_energy(&self) -> u32 {
            (0..self.count)
                .into_iter()
                .map(|i| self.nth_body(i).kinetic_energy())
                .sum()
        }

        pub fn total_energy(&self) -> u32 {
            (0..self.count)
                .into_iter()
                .map(|i| self.nth_body(i).total_energy())
                .sum()
        }

        pub fn center_pos(&self) -> Vec3 {
            Vec3::new(
                self.pos_x[0..self.count].iter().sum::<i32>() / self.count as i32,
                self.pos_y[0..self.count].iter().sum::<i32>() / self.count as i32,
                self.pos_z[0..self.count].iter().sum::<i32>() / self.count as i32,
            )
        }

        pub fn vel_sum(&self) -> Vec3 {
            Vec3::new(
                self.vel_x[0..self.count].iter().sum(),
                self.vel_y[0..self.count].iter().sum(),
                self.vel_z[0..self.count].iter().sum(),
            )
        }
    }

    // The following two function is for calculating 8 32-bit integers' sum, copied from Peter Cordes' answer at https://stackoverflow.com/questions/60108658/fastest-method-to-calculate-sum-of-all-packed-32-bit-integers-using-avx512-or-av
    unsafe fn hsum_epi32_avx(n: std::arch::x86_64::__m128i) -> i32 {
        #[cfg(target_arch = "x86")]
        use std::arch::x86::*;
        #[cfg(target_arch = "x86_64")]
        use std::arch::x86_64::*;

        let hi64 = _mm_unpackhi_epi64(n, n);
        let sum64 = _mm_add_epi32(hi64, n);
        let hi32 = _mm_shuffle_epi32::<0xB1>(sum64); // _MM_PERM_CDBA
        let sum32 = _mm_add_epi32(sum64, hi32);
        _mm_cvtsi128_si32(sum32)
    }

    unsafe fn hsum_8x32(n: std::arch::x86_64::__m256i) -> i32 {
        #[cfg(target_arch = "x86")]
        use std::arch::x86::*;
        #[cfg(target_arch = "x86_64")]
        use std::arch::x86_64::*;

        let sum128 = _mm_add_epi32(_mm256_castsi256_si128(n), _mm256_extracti128_si256::<1>(n));
        hsum_epi32_avx(sum128)
    }

    #[test]
    fn test_hsum_8x32() {
        let test_value = (0..8).into_iter().collect::<Vec<i32>>();
        unsafe {
            #[cfg(target_arch = "x86")]
            use std::arch::x86::*;
            #[cfg(target_arch = "x86_64")]
            use std::arch::x86_64::*;
            let value = _mm256_loadu_si256(test_value.as_ptr() as *const _);
            let sum = hsum_8x32(value);
            assert_eq!(sum, 28);
        }
    }
}

pub fn read_n_body<P>(path: P) -> Result<Vec<Body>, Error>
where
    P: AsRef<Path>,
{
    let file = File::open(path).map_err(Error::IOError)?;
    let reader = BufReader::new(file);
    reader
        .lines()
        .map(|r| r.map_err(Error::IOError).and_then(|s| Body::from_str(&s)))
        .collect::<Result<Vec<_>, Error>>()
}
