use std::{
    fs::File,
    io::{BufRead, BufReader},
    ops::{AddAssign, Neg, Sub, Div},
    path::Path,
    str::FromStr, iter::Sum,
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

impl Vec3 {
    fn new(x: i32, y: i32, z: i32) -> Self {
        Self { v: [x, y, z] }
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

pub struct NBodySimulator {
    bodies: Vec<Body>,
}

impl NBodySimulator {
    pub fn new(bodies: Vec<Body>) -> Self {
        Self { bodies }
    }

    pub fn bodies(&self) -> &[Body] {
        &self.bodies
    }

    pub fn step(&mut self) {
        let body_count = self.bodies.len();
        for i in 0..body_count {
            for j in (i + 1)..body_count {
                let grav_vel = self.bodies[i].grav_vel_from(&self.bodies[j]);
                *self.bodies[i].vel_mut() += grav_vel;
                *self.bodies[j].vel_mut() += -grav_vel;
            }

            self.bodies[i].apply_vel();
        }
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
