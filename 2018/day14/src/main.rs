use std::cmp::max;

const INPUT: usize = 846601;
const INPUT_SEQ: [u32; 6] = [8, 4, 6, 6, 0, 1];

fn main() {
    let mut combinator = RecipeCombinator::new(&[3, 7]);
    let inupt_ind = combinator.find_seq(&INPUT_SEQ);
    println!("Found input sequence ({:?}) after {} recipes", INPUT_SEQ, inupt_ind);
}

struct RecipeCombinator {
    recipe_list: Vec<u32>,
    elf_1_cur_ind: usize,
    elf_2_cur_ind: usize,
}

impl RecipeCombinator {
    pub fn new(init_score: &[u32]) -> RecipeCombinator {
        RecipeCombinator {
            recipe_list: init_score.to_vec(),
            elf_1_cur_ind: 0,
            elf_2_cur_ind: 1,
        }
    }

    pub fn get_score_in_range(&mut self, start: usize, end: usize) -> &[u32] {
        assert!(start <= end);
        if end > self.recipe_list.len() {
            self.comp_scores_until(end);
        }

        &self.recipe_list[start..end]
    }

    pub fn find_seq(&mut self, seq: &[u32]) -> usize {
        let mut start_ind = 0;
        loop {
            if let Some(found_ind) = find_seq(&self.recipe_list, seq, start_ind as usize) {
                return found_ind;
            }

            self.comb_recipe_once();
            // Because in one combination, new scores' length is at most 2, so minus 1 here
            start_ind = max(0, (self.recipe_list.len() as isize) - (seq.len() as isize) - 1);
        }
    }

    fn comp_scores_until(&mut self, end: usize) {
        while self.recipe_list.len() < end {
            self.comb_recipe_once();
        }
    }

    fn comb_recipe_once(&mut self) {
        let elf_1_score = self.recipe_list[self.elf_1_cur_ind];
        let elf_2_score = self.recipe_list[self.elf_2_cur_ind];
        let recipe_sum = elf_1_score + elf_2_score;
        if recipe_sum >= 10 {
            self.recipe_list.push(recipe_sum / 10);
            self.recipe_list.push(recipe_sum % 10);
        } else {
            self.recipe_list.push(recipe_sum);
        }

        self.elf_1_cur_ind =
            (self.elf_1_cur_ind + (elf_1_score as usize) + 1) % self.recipe_list.len();
        self.elf_2_cur_ind =
            (self.elf_2_cur_ind + (elf_2_score as usize) + 1) % self.recipe_list.len();
    }
}

#[test]
fn test_range_compute() {
    let mut combinator = RecipeCombinator::new(&[3, 7]);
    assert_eq!(
        combinator.get_score_in_range(0, 20),
        &[3, 7, 1, 0, 1, 0, 1, 2, 4, 5, 1, 5, 8, 9, 1, 6, 7, 7, 9, 2]
    );
}

#[test]
fn test_find_seq() {
    let mut combinator = RecipeCombinator::new(&[3, 7]);
    
    assert_eq!(9, combinator.find_seq(&[5, 1, 5, 8, 9]));
    assert_eq!(5, combinator.find_seq(&[0, 1, 2, 4, 5]));
    assert_eq!(18, combinator.find_seq(&[9, 2, 5, 1, 0]));
    assert_eq!(2018, combinator.find_seq(&[5, 9, 4, 1, 4]));
}

fn find_seq<T>(vec: &Vec<T>, seq: &[T], start_ind: usize) -> Option<usize> 
where T : PartialEq
{
    if start_ind > vec.len() {
        return None;
    }

    if (vec.len() - start_ind) < seq.len() {
        return None;
    }

    for i in start_ind..(vec.len() - seq.len() + 1) {
        let mut is_found = true;
        for j in 0..(seq.len()) {
            if vec[i + j] != seq[j] {
                is_found = false;
                break;
            }
        }

        if is_found {
            return Some(i);
        }
    }

    None
}
