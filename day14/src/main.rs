const INPUT: usize = 846601;

fn main() {
    let mut combinator = RecipeCombinator::new(&[3, 7]);
    let predict_recipe_scores = combinator.get_score_in_range(INPUT, INPUT + 10);

    print!("10 score after input index({}) is: ", INPUT);
    for score in predict_recipe_scores {
        print!("{}", score);
    }
    println!();
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
fn test_combinator() {
    let mut combinator = RecipeCombinator::new(&[3, 7]);
    assert_eq!(
        combinator.get_score_in_range(0, 20),
        &[3, 7, 1, 0, 1, 0, 1, 2, 4, 5, 1, 5, 8, 9, 1, 6, 7, 7, 9, 2]
    );
}
