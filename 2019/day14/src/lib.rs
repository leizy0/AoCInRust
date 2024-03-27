use std::{
    collections::HashMap,
    fmt::Display,
    fs::File,
    io::{self, BufRead, BufReader},
    path::Path,
};

use once_cell::sync::Lazy;
use regex::Regex;

#[derive(Debug)]
pub enum Error {
    IOError(io::Error),
    InvalidReactionStr(String),
    MissReaction(Chemical),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::IOError(ioe) => write!(f, "I/O error({})", ioe),
            Error::InvalidReactionStr(s) => {
                write!(f, "Failed to construct reaction, get invalid string({})", s)
            }
            Error::MissReaction(c) => write!(f, "Miss reaction output chemical({})", c),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Chemical(String);

impl Chemical {
    pub fn new(name: &str) -> Self {
        Self(name.to_string())
    }
}

impl Display for Chemical {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub struct Material {
    chemical: Chemical,
    unit_n: u64,
}

impl Material {
    pub fn new(chemical: Chemical, unit_n: u64) -> Self {
        Self { chemical, unit_n }
    }

    pub fn chemical(&self) -> &Chemical {
        &self.chemical
    }

    pub fn text_pattern() -> String {
        r"(\d+)\s*(\w+)".to_string()
    }
}

struct Reaction {
    output: Material,
    inputs: Vec<Material>,
}

impl TryFrom<&str> for Reaction {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        static REACTION_REGEX: Lazy<Regex> = Lazy::new(|| {
            Regex::new(&format!(
                r"({mat_pattern},?\s*)+\s*=>\s*({mat_pattern}){{1}}",
                mat_pattern = Material::text_pattern()
            ))
            .unwrap()
        });
        static MATERIAL_REGEX: Lazy<Regex> =
            Lazy::new(|| Regex::new(&Material::text_pattern()).unwrap());
        if REACTION_REGEX.is_match(value) {
            let mut materials = MATERIAL_REGEX
                .captures_iter(value)
                .map(|caps| {
                    let unit_n = u64::from_str_radix(caps[1].as_ref(), 10).unwrap();
                    let chemical = Chemical(caps[2].to_string());
                    Material::new(chemical, unit_n)
                })
                .collect::<Vec<_>>();

            let output = materials.pop().unwrap();
            Ok(Reaction {
                inputs: materials,
                output,
            })
        } else {
            Err(Error::InvalidReactionStr(value.to_string()))
        }
    }
}

impl Reaction {
    pub fn output(&self) -> &Material {
        &self.output
    }
}

pub struct ReactionMap {
    map: HashMap<Chemical, Reaction>,
}

impl ReactionMap {
    pub fn has(&self, chemical: &Chemical) -> bool {
        self.map.contains_key(chemical)
    }

    pub fn synthesize(&self, target_chemical: &Chemical, mut ore_unit_n: u64) -> Result<(u64, u64), Error> {
        let mut left_mats = HashMap::new();
        let mut target_unit_n = 0;
        loop {
            let used_ore_unit_n = self.decompose_one_unit(target_chemical, &mut left_mats)?;
            if used_ore_unit_n > ore_unit_n {
                return Ok((target_unit_n, ore_unit_n))
            }

            target_unit_n += 1;
            ore_unit_n -= used_ore_unit_n;
            if target_unit_n % 10000 == 0 {
                println!("Synthesize {} unit(s) {}, {} ORE left.", target_unit_n, target_chemical, ore_unit_n);
            }
        }
    }

    pub fn decompose(&self, target_material: &Material) -> Result<u64, Error> {
        let mut left_mats = HashMap::new();
        let mut ore_unit_n = 0;
        for _ in 0..target_material.unit_n {
            ore_unit_n += self.decompose_one_unit(&target_material.chemical, &mut left_mats)?;
        }

        Ok(ore_unit_n)
    }

    pub fn decompose_one_unit(&self, target_chemical: &Chemical, left_mats: &mut HashMap<Chemical, u64>) -> Result<u64, Error> {
        let mut decom_mats = HashMap::from([(target_chemical.clone(), 1)]);
        let final_chemical = Chemical::new("ORE");
        while decom_mats.len() > 1
            || !decom_mats
                .iter()
                .next()
                .is_some_and(|(c, _)| c == &final_chemical)
        {
            let decom_chem = decom_mats
                .iter()
                .filter(|(c, _)| **c != final_chemical)
                .map(|(c, _)| c.clone())
                .next()
                .unwrap();
            let mut decom_unit_n = decom_mats.remove(&decom_chem).unwrap();
            // If there are some material left from earlier reaction, take out its count from this reaction
            if let Some(mut left_unit_n) = left_mats.get(&decom_chem).copied() {
                if left_unit_n >= decom_unit_n {
                    // Left material can cover this decomposition, so cancel later reaction.
                    left_unit_n -= decom_unit_n;
                    if left_unit_n == 0 {
                        left_mats.remove(&decom_chem);
                    } else {
                        *left_mats.get_mut(&decom_chem).unwrap() = left_unit_n;
                    }

                    continue;
                } else {
                    // Subtract the amount needs to decompose.
                    decom_unit_n -= left_unit_n;
                    left_mats.remove(&decom_chem);
                }
            }

            // Decompose this material, borrow the least amount of output material(input of decomposition) of reaction if need to
            let decom_reaction = self
                .map
                .get(&decom_chem)
                .ok_or(Error::MissReaction(decom_chem.clone()))?;
            let decom_input_unit_n = decom_reaction.output.unit_n;
            let reaction_count = (decom_unit_n + decom_input_unit_n - 1) / decom_input_unit_n;
            for decom_output in &decom_reaction.inputs {
                let cur_output_unit_n = decom_output.unit_n * reaction_count;
                *decom_mats.entry(decom_output.chemical.clone()).or_insert(0) += cur_output_unit_n;
            }

            // Record borrowed amount, can be taken out to compensate later decomposition.
            let left_decom_unit_n = reaction_count * decom_input_unit_n - decom_unit_n;
            if left_decom_unit_n > 0 {
                left_mats
                    .entry(decom_chem.clone())
                    .or_insert(left_decom_unit_n);
            }
        }

        Ok(decom_mats[&final_chemical])
    }
}

pub fn parse_reactions<P>(path: P) -> Result<ReactionMap, Error>
where
    P: AsRef<Path>,
{
    let input_file = File::open(path).map_err(Error::IOError)?;
    let reader = BufReader::new(input_file);
    let map = reader
        .lines()
        .map(|l| {
            l.map_err(Error::IOError).and_then(|s| {
                Reaction::try_from(s.as_str()).map(|r| (r.output().chemical.clone(), r))
            })
        })
        .collect::<Result<HashMap<_, _>, Error>>()?;

    Ok(ReactionMap { map })
}
