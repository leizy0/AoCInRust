use std::fmt;
use std::fs;
use std::io::{self, BufRead, Write};

fn main() {
    let input_path = "input.txt";
    let input_file =
        fs::File::open(input_path).expect(&format!("Failed to open input file({})", input_path));
    let input_lines: Vec<String> = io::BufReader::new(input_file)
        .lines()
        .map(|l| l.unwrap())
        .collect();

    let mut simulator = CTSimulator::new(input_lines);
    loop {
        if let Err(ctse) = simulator.sim_tick() {
            println!(
                "Carts collide at pos({}), after {} ticks",
                ctse.pos(),
                simulator.elapsed()
            );
            break;
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug)]
struct Coordinate {
    y: u32,
    x: u32,
}

impl fmt::Display for Coordinate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}, {}", self.x, self.y)
    }
}

impl Coordinate {
    pub fn shift(&mut self, x_offset: i32, y_offset: i32) {
        let shift_x = self.x as i32 + x_offset;
        let shift_y = self.y as i32 + y_offset;
        if shift_x < 0 || shift_y < 0 {
            panic!(format!(
                "{} is shifted to invalid result({}, {})",
                self, shift_x, shift_y
            ));
        }

        self.x = shift_x as u32;
        self.y = shift_y as u32;
    }
}

#[test]
fn test_coord_ord() {
    assert!(Coordinate { x: 0, y: 1 } > Coordinate { x: 0, y: 0 });
    assert!(Coordinate { x: 1, y: 1 } > Coordinate { x: 0, y: 0 });
    assert!(Coordinate { x: 0, y: 1 } > Coordinate { x: 1, y: 0 });

    assert!(Coordinate { x: 0, y: 0 } == Coordinate { x: 0, y: 0 });
    assert!(Coordinate { x: 1, y: 0 } > Coordinate { x: 0, y: 0 });
    assert!(Coordinate { x: 0, y: 0 } < Coordinate { x: 1, y: 0 });
}

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
enum Direction {
    North,
    South,
    West,
    East,
}

impl Direction {
    pub fn coord_offset(&self) -> (i32, i32) {
        match self {
            Direction::North => (0, -1),
            Direction::South => (0, 1),
            Direction::West => (-1, 0),
            Direction::East => (1, 0),
        }
    }
}

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
enum Turn {
    Left,
    Straight,
    Right,
}

impl Turn {
    pub fn turned_dir(&self, cur_dir: Direction) -> Direction {
        match cur_dir {
            Direction::North => match self {
                Turn::Left => Direction::West,
                Turn::Straight => Direction::North,
                Turn::Right => Direction::East,
            },
            Direction::South => match self {
                Turn::Left => Direction::East,
                Turn::Straight => Direction::South,
                Turn::Right => Direction::West,
            },
            Direction::West => match self {
                Turn::Left => Direction::South,
                Turn::Straight => Direction::West,
                Turn::Right => Direction::North,
            },
            Direction::East => match self {
                Turn::Left => Direction::North,
                Turn::Straight => Direction::East,
                Turn::Right => Direction::South,
            },
        }
    }
}

struct Cart {
    last_pos: Coordinate,
    pos: Coordinate,
    dir: Direction,
    cur_turn: Turn,
}

impl Cart {
    pub fn new(coord: Coordinate, desc: char) -> Self {
        Cart {
            last_pos: coord,
            pos: coord,
            dir: match desc {
                '^' => Direction::North,
                'v' => Direction::South,
                '<' => Direction::West,
                '>' => Direction::East,
                _ => panic!(format!("Invalid cart description({})", desc)),
            },
            cur_turn: Turn::Left,
        }
    }

    pub fn coord(&self) -> Coordinate {
        self.pos
    }

    pub fn last_coord(&self) -> Coordinate {
        self.last_pos
    }

    pub fn go_ahead(&mut self, track: Track) {
        self.dir = match track {
            Track::Empty => panic!("Cart({:?}) is derailed!"),
            Track::HStraight | Track::VStraight | Track::UpCurve | Track::DownCurve => {
                track.next_dir(self.dir)
            }
            Track::Intersection => track.turn_dir(self.dir, self.turn()),
        };

        let (x_offset, y_offset) = self.dir.coord_offset();
        self.last_pos = self.pos;
        self.pos.shift(x_offset, y_offset);
    }

    fn collide_with(&self, other: &Cart) -> bool {
        // Two situation
        // First, two cart have the same current position
        if self.pos == other.pos {
            return true;
        }

        // Second, this cart's last position is position of other,
        // and the last positon of other is this cart's current posiiton
        if self.pos == other.last_pos && self.last_pos == other.pos {
            return true;
        }

        false
    }

    fn turn(&mut self) -> Turn {
        let res_turn = self.cur_turn;
        self.cur_turn = match self.cur_turn {
            Turn::Left => Turn::Straight,
            Turn::Straight => Turn::Right,
            Turn::Right => Turn::Left,
        };

        res_turn
    }
}

#[test]
fn test_cart_turn() {
    let mut cart = Cart::new(Coordinate { x: 0, y: 0 }, '>');
    assert_eq!(cart.turn(), Turn::Left);
    assert_eq!(cart.turn(), Turn::Straight);
    assert_eq!(cart.turn(), Turn::Right);
    assert_eq!(cart.turn(), Turn::Left);
}

#[test]
fn test_cart_go_straight() {
    // East
    let mut cart = Cart::new(Coordinate { x: 10, y: 10 }, '>');
    cart.go_ahead(Track::HStraight);
    assert_eq!(cart.coord(), Coordinate { x: 11, y: 10 });

    // West
    let mut cart = Cart::new(Coordinate { x: 10, y: 10 }, '<');
    cart.go_ahead(Track::HStraight);
    assert_eq!(cart.coord(), Coordinate { x: 9, y: 10 });

    // North
    let mut cart = Cart::new(Coordinate { x: 10, y: 10 }, '^');
    cart.go_ahead(Track::VStraight);
    assert_eq!(cart.coord(), Coordinate { x: 10, y: 9 });

    // South
    let mut cart = Cart::new(Coordinate { x: 10, y: 10 }, 'v');
    cart.go_ahead(Track::VStraight);
    assert_eq!(cart.coord(), Coordinate { x: 10, y: 11 });
}

#[test]
fn test_cart_go_up_curve() {
    // North
    let mut cart = Cart::new(Coordinate { x: 10, y: 10 }, '^');
    cart.go_ahead(Track::UpCurve);
    assert_eq!(cart.coord(), Coordinate { x: 11, y: 10 });

    // South
    let mut cart = Cart::new(Coordinate { x: 10, y: 10 }, 'v');
    cart.go_ahead(Track::UpCurve);
    assert_eq!(cart.coord(), Coordinate { x: 9, y: 10 });

    // West
    let mut cart = Cart::new(Coordinate { x: 10, y: 10 }, '<');
    cart.go_ahead(Track::UpCurve);
    assert_eq!(cart.coord(), Coordinate { x: 10, y: 11 });

    // East
    let mut cart = Cart::new(Coordinate { x: 10, y: 10 }, '>');
    cart.go_ahead(Track::UpCurve);
    assert_eq!(cart.coord(), Coordinate { x: 10, y: 9 });
}

#[test]
fn test_cart_go_down_curve() {
    // North
    let mut cart = Cart::new(Coordinate { x: 10, y: 10 }, '^');
    cart.go_ahead(Track::DownCurve);
    assert_eq!(cart.coord(), Coordinate { x: 9, y: 10 });

    // South
    let mut cart = Cart::new(Coordinate { x: 10, y: 10 }, 'v');
    cart.go_ahead(Track::DownCurve);
    assert_eq!(cart.coord(), Coordinate { x: 11, y: 10 });

    // West
    let mut cart = Cart::new(Coordinate { x: 10, y: 10 }, '<');
    cart.go_ahead(Track::DownCurve);
    assert_eq!(cart.coord(), Coordinate { x: 10, y: 9 });

    // East
    let mut cart = Cart::new(Coordinate { x: 10, y: 10 }, '>');
    cart.go_ahead(Track::DownCurve);
    assert_eq!(cart.coord(), Coordinate { x: 10, y: 11 });
}

#[test]
fn test_cart_go_intersection() {
    // North
    let mut cart = Cart::new(Coordinate { x: 10, y: 10 }, '^');
    cart.go_ahead(Track::Intersection); // Go left, toward west
    assert_eq!(cart.coord(), Coordinate { x: 9, y: 10 });
    cart.go_ahead(Track::Intersection); // Go Straight, toward west
    assert_eq!(cart.coord(), Coordinate { x: 8, y: 10 });
    cart.go_ahead(Track::Intersection); // Go Right, toward north
    assert_eq!(cart.coord(), Coordinate { x: 8, y: 9 });
    cart.go_ahead(Track::Intersection); // Go Left, toward west
    assert_eq!(cart.coord(), Coordinate { x: 7, y: 9 });

    // South
    let mut cart = Cart::new(Coordinate { x: 10, y: 10 }, 'v');
    cart.go_ahead(Track::Intersection); // Go left, toward east
    assert_eq!(cart.coord(), Coordinate { x: 11, y: 10 });
    cart.go_ahead(Track::Intersection); // Go Straight, toward east
    assert_eq!(cart.coord(), Coordinate { x: 12, y: 10 });
    cart.go_ahead(Track::Intersection); // Go Right, toward south
    assert_eq!(cart.coord(), Coordinate { x: 12, y: 11 });
    cart.go_ahead(Track::Intersection); // Go Left, toward east
    assert_eq!(cart.coord(), Coordinate { x: 13, y: 11 });

    // West
    let mut cart = Cart::new(Coordinate { x: 10, y: 10 }, '<');
    cart.go_ahead(Track::Intersection); // Go left, toward south
    assert_eq!(cart.coord(), Coordinate { x: 10, y: 11 });
    cart.go_ahead(Track::Intersection); // Go Straight, toward south
    assert_eq!(cart.coord(), Coordinate { x: 10, y: 12 });
    cart.go_ahead(Track::Intersection); // Go Right, toward west
    assert_eq!(cart.coord(), Coordinate { x: 9, y: 12 });
    cart.go_ahead(Track::Intersection); // Go Left, toward south
    assert_eq!(cart.coord(), Coordinate { x: 9, y: 13 });

    // East
    let mut cart = Cart::new(Coordinate { x: 10, y: 10 }, '>');
    cart.go_ahead(Track::Intersection); // Go left, toward north
    assert_eq!(cart.coord(), Coordinate { x: 10, y: 9 });
    cart.go_ahead(Track::Intersection); // Go Straight, toward north
    assert_eq!(cart.coord(), Coordinate { x: 10, y: 8 });
    cart.go_ahead(Track::Intersection); // Go Right, toward east
    assert_eq!(cart.coord(), Coordinate { x: 11, y: 8 });
    cart.go_ahead(Track::Intersection); // Go Left, toward north
    assert_eq!(cart.coord(), Coordinate { x: 11, y: 7 });
}

struct TrackMap {
    row_n: u32,
    col_n: u32,
    track_mat: Vec<Track>,
}

impl TrackMap {
    pub fn new(row_n: u32, col_n: u32) -> Self {
        let track_mat = vec![Track::Empty; (row_n * col_n) as usize];
        TrackMap {
            row_n,
            col_n,
            track_mat,
        }
    }

    pub fn row_n(&self) -> u32 {
        self.row_n
    }

    pub fn col_n(&self) -> u32 {
        self.col_n
    }

    pub fn at(&self, coord: Coordinate) -> Track {
        if !self.is_valid_coord(coord) {
            panic!("Invalid coordiante({}), can't get track", coord);
        }

        let ind = self.coord_to_ind(coord);
        self.track_mat[ind as usize]
    }

    pub fn set_track(&mut self, coord: Coordinate, desc: char) {
        if !self.is_valid_coord(coord) {
            panic!("Invalid coordinate({}), can't set track({})", coord, desc);
        }

        let ind = self.coord_to_ind(coord);
        self.track_mat[ind as usize] = match desc {
            '|' => Track::VStraight,
            '-' => Track::HStraight,
            '/' => Track::UpCurve,
            '\\' => Track::DownCurve,
            '+' => Track::Intersection,
            _ => panic!(format!("Invalid track description({})", desc)),
        }
    }

    pub fn row(&self, r_ind: u32) -> &[Track] {
        let start = (self.col_n() * r_ind) as usize;
        let end = start + self.col_n() as usize;
        &self.track_mat[start..end]
    }

    fn coord_to_ind(&self, coord: Coordinate) -> u32 {
        coord.y * self.col_n + coord.x
    }

    fn is_valid_coord(&self, coord: Coordinate) -> bool {
        coord.x < self.col_n && coord.y < self.row_n
    }
}

#[derive(Copy, Clone, Debug)]
enum Track {
    Empty,
    HStraight,
    VStraight,
    UpCurve,
    DownCurve,
    Intersection,
}

impl Track {
    pub fn next_dir(&self, cur_dir: Direction) -> Direction {
        match self {
            Track::HStraight => match cur_dir {
                Direction::West | Direction::East => cur_dir,
                _ => panic!(format!(
                    "{:?} isn't direct along horizontal straight track",
                    cur_dir
                )),
            },
            Track::VStraight => match cur_dir {
                Direction::South | Direction::North => cur_dir,
                _ => panic!(format!(
                    "{:?} isn't direct along vertical straight track",
                    cur_dir
                )),
            },
            Track::UpCurve => match cur_dir {
                Direction::North => Direction::East,
                Direction::South => Direction::West,
                Direction::West => Direction::South,
                Direction::East => Direction::North,
            },
            Track::DownCurve => match cur_dir {
                Direction::North => Direction::West,
                Direction::South => Direction::East,
                Direction::West => Direction::North,
                Direction::East => Direction::South,
            },
            Track::Empty => panic!("Empty track has no next direction"),
            Track::Intersection => panic!(
                "Intersection has three next direction, should given turn info(use turn_dir)"
            ),
        }
    }

    pub fn turn_dir(&self, cur_dir: Direction, turn: Turn) -> Direction {
        match self {
            Track::Intersection => turn.turned_dir(cur_dir),
            _ => panic!(format!("{:?} can't turn direction", self)),
        }
    }
}

struct CTSimulator {
    map: TrackMap,
    cart_list: Vec<Cart>,
    tick_n: u32,
}

type CTSimResult = Result<(), CTSimError>;

enum CTSimErrorType {
    Collision,
}

struct CTSimError {
    err_type: CTSimErrorType,
    pos: Coordinate,
}

impl CTSimError {
    pub fn pos(&self) -> Coordinate {
        self.pos
    }
}

impl CTSimulator {
    pub fn new(desc: Vec<String>) -> Self {
        let row_n = desc.len();
        let col_n = desc[0].chars().count();
        let mut map = TrackMap::new(row_n as u32, col_n as u32);
        let mut carts = Vec::new();
        for (y, row) in desc.iter().enumerate() {
            let this_col_n = row.chars().count();
            if this_col_n != col_n {
                panic!(format!(
                    "Inconsistent # of track map column({}), expect all are {}",
                    this_col_n, col_n
                ));
            }

            for (x, c) in row.chars().enumerate() {
                let coord = Coordinate {
                    x: x as u32,
                    y: y as u32,
                };
                match c {
                    '|' | '-' | '/' | '\\' | '+' => map.set_track(coord, c),

                    '^' | 'v' | '<' | '>' => {
                        carts.push(Cart::new(coord, c));
                        match c {
                            '^' | 'v' => {
                                map.set_track(coord, '|');
                            }
                            '<' | '>' => {
                                map.set_track(coord, '-');
                            }
                            _ => panic!("Never go here, under protection of outer match branch"),
                        }
                    }
                    ' ' => (),
                    _ => panic!(format!("Invalid map description({}), at {}", c, coord)),
                }
            }
        }

        CTSimulator {
            map: map,
            cart_list: carts,
            tick_n: 0,
        }
    }

    pub fn sim_tick(&mut self) -> CTSimResult {
        self.cart_list.sort_unstable_by_key(|c| c.coord());

        for ind in 0..(self.cart_list.len()) {
            let cart = &mut self.cart_list[ind];
            cart.go_ahead(self.map.at(cart.coord()));
            self.check_cart_collision(ind)?;
        }

        self.tick_n += 1;
        Ok(())
    }

    pub fn elapsed(&self) -> u32 {
        self.tick_n
    }

    pub fn dump_map(&self, file_path: &str) -> io::Result<()> {
        let output_file = fs::File::create(file_path)?;
        let mut writer = io::BufWriter::new(output_file);
        let line_n = self.map.row_n();
        for i in 0..line_n {
            let mut line: Vec<u8> = self
                .map
                .row(i)
                .iter()
                .map(|t| match t {
                    Track::Empty => b' ',
                    Track::HStraight => b'-',
                    Track::VStraight => b'|',
                    Track::Intersection => b'+',
                    Track::UpCurve => b'/',
                    Track::DownCurve => b'\\',
                })
                .collect();
            line.push(b'\n');

            writer.write(line.as_slice())?;
        }

        Ok(())
    }

    fn check_cart_collision(&self, ind: usize) -> CTSimResult {
        let check_cart = &self.cart_list[ind];
        for (i, cart) in self.cart_list.iter().enumerate() {
            if ind != i && cart.coord() == check_cart.coord() {
                return Err(CTSimError {
                    err_type: CTSimErrorType::Collision,
                    pos: check_cart.coord(),
                });
            }
        }

        Ok(())
    }
}
