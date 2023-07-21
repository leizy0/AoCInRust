use std::{
    collections::HashMap,
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
    str::FromStr,
};

use once_cell::sync::Lazy;
use regex::Regex;

use crate::Error;

pub struct OrbitTree {
    new_obj_id: usize,
    orbit_links: Vec<usize>,
    object_map: HashMap<Object, usize>,
    objects: Vec<Object>,
}

impl OrbitTree {
    fn new() -> OrbitTree {
        OrbitTree {
            new_obj_id: 1,
            orbit_links: vec![0],
            object_map: HashMap::new(),
            objects: vec!["".to_string()],
        }
    }

    pub fn obj_iter(&self) -> ObjectIterator {
        ObjectIterator::new(self)
    }

    pub fn orbit_count_of(&self, obj: &Object) -> Option<usize> {
        self.object_map.get(obj).map(|&id| {
            let mut orbit_count = 0;
            let mut cur_id = id;
            loop {
                let orbit_id = self.orbit_links[cur_id];
                if orbit_id == Self::null_obj_id() {
                    break;
                }

                orbit_count += 1;
                cur_id = orbit_id;
            }

            orbit_count
        })
    }

    fn add_orbit(&mut self, orbit: Orbit) -> Result<(), Error> {
        let orbited_id = self.add_obj(orbit.orbited);
        let orbiter_id = self.add_obj(orbit.orbiter);

        if self.orbit_links[orbiter_id] != Self::null_obj_id() {
            Err(Error::RewriteOrbitLink(
                self.objects[orbiter_id].clone(),
                self.objects[self.orbit_links[orbiter_id]].clone(),
                self.objects[orbited_id].clone(),
            ))
        } else {
            self.orbit_links[orbiter_id] = orbited_id;

            Ok(())
        }
    }

    fn add_obj(&mut self, obj: Object) -> usize {
        *self.object_map.entry(obj.clone()).or_insert_with(|| {
            let cur_id = self.new_obj_id;
            self.objects.push(obj.clone());
            self.orbit_links.push(Self::null_obj_id());
            self.new_obj_id += 1;

            cur_id
        })
    }

    fn null_obj_id() -> usize {
        0
    }
}

pub struct ObjectIterator<'a> {
    orbit_tree: &'a OrbitTree,
    obj_ind: usize,
}

impl<'a> Iterator for ObjectIterator<'a> {
    type Item = &'a Object;

    fn next(&mut self) -> Option<Self::Item> {
        let cur_ind = self.obj_ind;
        self.obj_ind += 1;
        self.orbit_tree.objects.get(cur_ind)
    }
}

impl<'a> ObjectIterator<'a> {
    fn new(orbit_tree: &'a OrbitTree) -> Self {
        ObjectIterator {
            orbit_tree,
            obj_ind: 1,
        }
    }
}

pub type Object = String;

struct Orbit {
    orbited: Object,
    orbiter: Object,
}

impl FromStr for Orbit {
    type Err = Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        static ORBIT_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"(\w+)\)(\w+)").unwrap());

        let caps = ORBIT_REGEX
            .captures(value)
            .ok_or(Error::InvalidOrbitSpec(value.to_string()))?;
        Ok(Orbit {
            orbited: caps[1].to_string(),
            orbiter: caps[2].to_string(),
        })
    }
}

pub fn read_orbits<P>(path: P) -> Result<OrbitTree, Error>
where
    P: AsRef<Path>,
{
    let orbit_file = File::open(path).map_err(Error::IOError)?;
    let reader = BufReader::new(orbit_file);
    let mut orbit_tree = OrbitTree::new();
    reader
        .lines()
        .map(|l| {
            orbit_tree.add_orbit(
                l.map_err(Error::IOError)
                    .and_then(|s| Orbit::from_str(&s))?,
            )
        })
        .collect::<Result<_, Error>>()?;

    Ok(orbit_tree)
}
