use std::{
    cmp::{Ordering, Reverse},
    collections::HashSet,
};

use crate::{unit::{Army, Group}, Error};

pub enum Cheat {
    ArmyAttackBoost{army_ind: usize, boost_point: usize},
}

pub struct Simulator {
    cheat: Option<Cheat>,
}

impl Simulator {
    pub fn new() -> Self {
        Self {cheat: None}
    }

    pub fn with_cheat(cheat: Cheat) -> Simulator {
        Self{cheat: Some(cheat)}
    }

    pub fn simulate(&self, armies: &mut [&mut Army]) -> Result<usize, Error> {
        self.apply_cheat(armies);
        let mut round_ind = 0;
        while Self::left_army_inds(armies).len() > 1 {
            // println!("Round #{}:", round_ind);
            let mut groups = armies
                .iter_mut()
                .enumerate()
                .flat_map(|(a_ind, a)| a.groups_mut().map(move |g| (a_ind, g)))
                .collect::<Vec<_>>();
            // groups
            //     .iter()
            //     .enumerate()
            //     .for_each(|(g_ind, (_, g))| println!("Group#{}: {} unit(s)", g_ind, g.count()));

            let attack_pairs = Self::select_target(&groups);
            if attack_pairs.is_empty() {
                return Err(Error::SimulationInDraw);
            }
            // println!("attack_pairs: {:?}", attack_pairs);
            Self::apply_attack(attack_pairs, &mut groups);
            round_ind += 1;
            // println!();
        }

        Self::left_army_inds(armies).first().copied().ok_or(Error::NoArmyLeft)
    }

    fn apply_cheat(&self, armies: &mut [&mut Army]) {
        if let Some(cheat) = &self.cheat {
            match cheat {
                Cheat::ArmyAttackBoost { army_ind, boost_point } => 
                    armies[*army_ind].groups_mut().for_each(|g| {
                        let new_dmg_point = g.unit().damage_point() + boost_point;
                        g.unit_mut().set_damage_point(new_dmg_point);})
            }
        }
    }

    fn select_target(groups: &Vec<(usize, &mut Group)>) -> Vec<(usize, usize)> {
        let mut prepare_groups = groups
            .iter()
            .enumerate()
            .filter(|(_, (_, g))| g.count() > 0)
            .collect::<Vec<_>>();
        prepare_groups.sort_unstable_by(|(_, (_, lg)), (_, (_, rg))|
            // Decreasing order
            match rg.effective_power().cmp(&lg.effective_power()) {
                Ordering::Equal => rg.unit().initiative().cmp(&lg.unit().initiative()),
                other => other,
            });

        let mut selected_ind = HashSet::new();
        let mut attack_pairs = Vec::new();
        for (g_ind, (army_ind, group)) in prepare_groups {
            let attackee = groups
                .iter()
                .enumerate()
                .filter(|(g_ind, (a_ind, g))| {
                    !selected_ind.contains(g_ind) && g.count() > 0 && a_ind != army_ind
                })
                .max_by(|(_, (_, lg)), (_, (_, rg))| {
                    let l_damage =
                        lg.true_damage(group.effective_power(), group.unit().damage_type());
                    let r_damage =
                        rg.true_damage(group.effective_power(), group.unit().damage_type());
                    match l_damage.cmp(&r_damage) {
                        Ordering::Equal => match lg.effective_power().cmp(&rg.effective_power()) {
                            Ordering::Equal => lg.unit().initiative().cmp(&rg.unit().initiative()),
                            other => other,
                        },
                        other => other,
                    }
                });

            if let Some((attackee_ind, (_, attackee))) = attackee {
                let expect_damage =
                    attackee.true_damage(group.effective_power(), group.unit().damage_type());
                if expect_damage > 0 {
                    attack_pairs.push((g_ind, attackee_ind));
                    selected_ind.insert(attackee_ind);
                }
            }
        }

        attack_pairs
    }

    fn apply_attack(mut attack_pairs: Vec<(usize, usize)>, groups: &mut Vec<(usize, &mut Group)>) {
        attack_pairs.sort_unstable_by_key(|(attack_ind, _)| {
            Reverse(groups[*attack_ind].1.unit().initiative())
        });
        for (attacker_ind, attackee_ind) in attack_pairs {
            assert!(groups[attacker_ind].0 != groups[attackee_ind].0);
            if groups[attacker_ind].1.count() > 0 && groups[attackee_ind].1.count() > 0 {
                let damage_point = groups[attacker_ind].1.effective_power();
                let damage_type = groups[attacker_ind].1.unit().damage_type();
                // println!("Group#{} attack Group#{}", attacker_ind, attackee_ind);
                groups[attackee_ind].1.attack_by(damage_point, damage_type);
            }
        }
    }

    fn left_army_inds(armies: &[&mut Army]) -> Vec<usize> {
        armies
            .iter()
            .enumerate()
            .map(|(ind, a)| (ind, a.groups().map(|g| g.count()).sum::<usize>()))
            .filter(|(_, uc)| *uc > 0)
            .map(|(ind, _)| ind)
            .collect::<Vec<_>>()
    }
}
