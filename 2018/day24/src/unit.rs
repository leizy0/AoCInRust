use std::{
    collections::{HashMap, HashSet},
    fs::File,
    hash::Hash,
    io::{BufRead, BufReader},
    path::Path,
};

use once_cell::sync::Lazy;
use regex::Regex;

use crate::Error;

#[derive(Debug, Clone)]
pub struct Army {
    groups: Vec<Group>,
}

impl Army {
    pub fn groups(&self) -> impl Iterator<Item = &Group> + '_ {
        self.groups.iter()
    }

    pub fn groups_mut(&mut self) -> impl Iterator<Item = &mut Group> + '_ {
        self.groups.iter_mut()
    }
}

#[derive(Debug, Clone)]
pub struct Group {
    unit: Unit,
    count: usize,
}

impl TryFrom<&str> for Group {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        static GROUP_PATTERN: Lazy<Regex> =
            Lazy::new(|| Regex::new(r"(\d+) units\s+(.+)").unwrap());
        let caps = GROUP_PATTERN
            .captures(value)
            .ok_or(Error::NotMatchGroupPattern(value.to_string()))?;
        let count = caps[1].parse::<usize>().unwrap();
        let unit = Unit::try_from(&caps[2])?;
        Ok(Group { unit, count })
    }
}

impl Group {
    pub fn count(&self) -> usize {
        self.count
    }

    pub fn unit(&self) -> &Unit {
        &self.unit
    }

    pub fn effective_power(&self) -> usize {
        self.unit.dmg_point * self.count
    }

    pub fn true_damage(&self, dmg_point: usize, dmg_type: DamageType) -> usize {
        let dmg_ratio = self.unit.dmg_ratio(dmg_type);
        dmg_point * dmg_ratio
    }

    pub fn attack_by(&mut self, damage_point: usize, damage_type: DamageType) {
        let true_damage = self.true_damage(damage_point, damage_type);
        let dead_count = true_damage / self.unit.health_point();
        self.count = self.count.checked_sub(dead_count).unwrap_or(0);
    }

    pub fn unit_mut(&mut self) -> &mut Unit {
        &mut self.unit
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DamageType {
    Bludgeoning,
    Cold,
    Fire,
    Radiation,
    Slashing,
}

impl TryFrom<&str> for DamageType {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        static DAMAGE_TYPE_NAME_MAP: Lazy<HashMap<&'static str, DamageType>> = Lazy::new(|| {
            HashMap::from([
                ("bludgeoning", DamageType::Bludgeoning),
                ("cold", DamageType::Cold),
                ("fire", DamageType::Fire),
                ("radiation", DamageType::Radiation),
                ("slashing", DamageType::Slashing),
            ])
        });

        DAMAGE_TYPE_NAME_MAP
            .get(value)
            .copied()
            .ok_or(Error::UnknownDamageType(value.to_string()))
    }
}

#[derive(Debug, Clone)]
pub struct Unit {
    health_point: usize,
    immune_dmg_types: HashSet<DamageType>,
    weak_dmg_types: HashSet<DamageType>,
    dmg_point: usize,
    dmg_type: DamageType,
    initiative: usize,
}

impl TryFrom<&str> for Unit {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        static UNIT_PATTERN: Lazy<Regex> = Lazy::new(|| {
            Regex::new(r"each with (\d+) hit points (\(.+\))?\s*with an attack that does (\d+) (\w+) damage at initiative (\d+)").unwrap()
        });
        static IMM_PATTERN: Lazy<Regex> =
            Lazy::new(|| Regex::new(r"immune to ([\w\s,]+)").unwrap());
        static WEAK_PATTERN: Lazy<Regex> = Lazy::new(|| Regex::new(r"weak to ([\w\s,]+)").unwrap());
        let caps = UNIT_PATTERN
            .captures(value)
            .ok_or(Error::NotMatchWithUnitPattern(value.to_string()))?;
        let health_point = caps[1].parse::<usize>().unwrap();
        let def_trait_text = caps.get(2).map(|m| m.as_str()).unwrap_or("");
        let immune_dmg_types = Self::parse_defense_trait(def_trait_text, &IMM_PATTERN, 1)?;
        let weak_dmg_types = Self::parse_defense_trait(def_trait_text, &WEAK_PATTERN, 1)?;

        let dmg_point = caps[3].parse::<usize>().unwrap();
        let dmg_type = DamageType::try_from(&caps[4])?;
        let initiative = caps[5].parse::<usize>().unwrap();

        Ok(Self {
            health_point,
            immune_dmg_types,
            weak_dmg_types,
            dmg_point,
            dmg_type,
            initiative,
        })
    }
}

impl Unit {
    fn parse_defense_trait(
        value: &str,
        pattern: &Regex,
        type_cap_ind: usize,
    ) -> Result<HashSet<DamageType>, Error> {
        pattern
            .captures(value)
            .map(|c| Self::parse_dmg_type_list(&c[type_cap_ind]))
            .unwrap_or(Ok(HashSet::new()))
    }

    fn parse_dmg_type_list(value: &str) -> Result<HashSet<DamageType>, Error> {
        static DAMAGE_TYPE_PATTERN: Lazy<Regex> = Lazy::new(|| Regex::new(r"\w+").unwrap());
        DAMAGE_TYPE_PATTERN
            .find_iter(value)
            .map(|m| DamageType::try_from(m.as_str()))
            .collect::<Result<HashSet<_>, _>>()
    }

    pub fn health_point(&self) -> usize {
        self.health_point
    }

    pub fn initiative(&self) -> usize {
        self.initiative
    }

    pub fn damage_point(&self) -> usize {
        self.dmg_point
    }

    pub fn set_damage_point(&mut self, damage_point: usize) {
        self.dmg_point = damage_point;
    }

    pub fn damage_type(&self) -> DamageType {
        self.dmg_type
    }

    pub fn dmg_ratio(&self, dmg_type: DamageType) -> usize {
        if self.weak_dmg_types.contains(&dmg_type) {
            2
        } else if self.immune_dmg_types.contains(&dmg_type) {
            0
        } else {
            1
        }
    }
}

pub fn load_army<P>(input_path: P) -> Result<Army, Error>
where
    P: AsRef<Path>,
{
    let input_file = File::open(input_path).map_err(Error::IOError)?;
    let reader = BufReader::new(input_file);
    let groups = reader
        .lines()
        .map(|l| {
            l.map_err(Error::IOError)
                .and_then(|l| Group::try_from(l.as_str()))
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(Army { groups })
}
