use std::time::{Duration, Instant};

const INPUT: u32 = 7803;
const GRID_BEGIN: u32 = 1;
const GRID_END: u32 = 300;
const GRID_ROW_N: u32 = GRID_END - GRID_BEGIN + 1;
const GRID_COLUMN_N: u32 = GRID_END - GRID_BEGIN + 1;

fn main() {
    let fuel_grid = FuelGrid::new(GRID_ROW_N, GRID_COLUMN_N, INPUT);

    let start_time = Instant::now();
    let (max_cell_x, max_cell_y, max_cell_size, max_cell_level) = (GRID_BEGIN..=GRID_END)
        .map(|s| {
            println!(
                "{}: compute cell size {}",
                start_time.elapsed().as_secs(),
                s
            );
            let (max_cell_x, max_cell_y, max_cell_level) = fuel_grid.max_cell(s, s);
            (max_cell_x, max_cell_y, s, max_cell_level)
        })
        .max_by_key(|(_x, _y, _s, l)| l.clone())
        .unwrap();

    println!(
        "Max square cell in {} x {} grid starts at ({}, {}), with size({}) and a total power of {}",
        GRID_COLUMN_N,
        GRID_ROW_N,
        max_cell_x + 1,
        max_cell_y + 1,
        max_cell_size,
        max_cell_level
    );
}

struct FuelGrid {
    level_mat: Vec<i32>,
    row_n: u32,
    col_n: u32,
    serial_n: u32,
}

impl FuelGrid {
    pub fn new(a_row_n: u32, a_col_n: u32, a_serial_n: u32) -> FuelGrid {
        let grid_iter = (1..=a_row_n).flat_map(|y| (1..=a_col_n).map(move |x| (x, y)));
        let levels: Vec<i32> = grid_iter
            .map(|(x, y)| FuelGrid::comp_level(x, y, INPUT))
            .collect();

        FuelGrid {
            level_mat: levels,
            row_n: a_row_n,
            col_n: a_col_n,
            serial_n: a_serial_n,
        }
    }

    pub fn max_cell(&self, cell_row_n: u32, cell_col_n: u32) -> (u32, u32, i32) {
        if cell_col_n > self.col_n || cell_row_n > self.row_n {
            panic!(format!("Invalid cell size({} X {}), it's bigger than whole grid size({} X {}) in some dimensions", cell_row_n, cell_col_n, self.row_n, self.col_n));
        }

        let h_cell_n = self.col_n - cell_col_n + 1;
        let v_cell_n = self.row_n - cell_row_n + 1;
        (0..h_cell_n)
            .flat_map(|x| {
                (0..v_cell_n).map(move |y| (x, y, self.cell_sum(x, y, cell_row_n, cell_col_n)))
            })
            .max_by_key(|(_x, _y, v)| v.clone())
            .unwrap()
    }

    fn cell_sum(&self, cell_x: u32, cell_y: u32, cell_row_n: u32, cell_col_n: u32) -> i32 {
        let cell_x_end = cell_x + cell_col_n;
        let cell_y_end = cell_y + cell_row_n;
        (cell_x..cell_x_end)
            .flat_map(|x| (cell_y..cell_y_end).map(move |y| self.get(x, y)))
            .sum()
    }

    fn get(&self, x: u32, y: u32) -> i32 {
        if x >= self.col_n || y >= self.row_n {
            panic!(format!(
                "Failed to get value of coordinate({}, {}), index out of bound",
                x, y
            ));
        }

        let vec_ind = (y * self.col_n + x) as usize;
        self.level_mat[vec_ind]
    }

    fn comp_level(x: u32, y: u32, serial_n: u32) -> i32 {
        let rack_id = x + 10;
        let pre_result = (rack_id * y + serial_n) * rack_id;

        let pre_level = if pre_result < 100 {
            0
        } else {
            pre_result % 1000 / 100
        };

        pre_level as i32 - 5
    }
}

#[test]
fn test_comp_level() {
    let level0 = FuelGrid::comp_level(122, 79, 57);
    assert_eq!(level0, -5);

    let level1 = FuelGrid::comp_level(217, 196, 39);
    assert_eq!(level1, 0);

    let level2 = FuelGrid::comp_level(101, 153, 71);
    assert_eq!(level2, 4);

    let level3 = FuelGrid::comp_level(3, 5, 8);
    assert_eq!(level3, 4);
}
