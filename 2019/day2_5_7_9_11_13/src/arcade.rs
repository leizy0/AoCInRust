use std::{
    cmp,
    collections::HashMap,
    error,
    fmt::Display,
    io::{self, Stdout, Write},
    iter,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    thread,
    time::{Duration, Instant},
};

use crossterm::{
    cursor,
    event::{poll, read, Event, KeyCode, KeyEventKind},
    execute, queue, style,
    terminal::{self, ClearType},
};
use int_enum::IntEnum;
use once_cell::sync::Lazy;
use rayon::ThreadPool;

use crate::int_code::io::{InputPort, OutputPort};

#[derive(Debug)]
pub enum Error {
    IncompleteTile(usize),
    InvalidTileId(i64),
    InvalidTilePos(i64, i64),
    InvalidScore(i64),
    TerminalError(io::Error),
    NotEnoughTerminalSpace(u32, u32, u32, u32),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::IncompleteTile(len) => write!(f, "Incomplete tile info(expects three integers) when parsing {}(th) tile", len + 1),
            Error::InvalidTileId(id) => write!(f, "Invalid tile id({}) encountered", id),
            Error::InvalidTilePos(x, y) => write!(f, "Invalid tile position({}, {}) encountered, expects a pair of non-negative integers", x, y),
            Error::InvalidScore(s) => write!(f, "Given invalid game score({})", s),
            Error::TerminalError(ioe) => write!(f, "Failed to control terminal(stdout), get error({})", ioe),
            Error::NotEnoughTerminalSpace(real_row_n, real_col_n, expect_row_n, expect_col_n) => write!(f, "Terminal({} x {}) hasn't enough space to render, expect {} rows x {} columns", real_row_n, real_col_n, expect_row_n, expect_col_n),
        }
    }
}

impl error::Error for Error {}

type GameInfo = [i64; 3];

#[repr(u8)]
#[derive(Debug, Default, Clone, Copy, IntEnum, PartialEq, Eq, Hash)]
pub enum TileId {
    #[default]
    Empty = 0,
    Wall = 1,
    Block = 2,
    HorizontalPaddle = 3,
    Ball = 4,
}

pub struct Tile {
    x: u32,
    y: u32,
    id: TileId,
}

impl Tile {
    fn from_info(info: &GameInfo) -> Result<Self, Error> {
        let x = u32::try_from(info[0]).map_err(|_| Error::InvalidTilePos(info[0], info[1]))?;
        let y =
            u32::try_from(info[1]).map_err(|_| Error::InvalidTilePos(info[0] as i64, info[1]))?;
        let id = TileId::from_int(info[2] as u8).map_err(|_| Error::InvalidTileId(info[2]))?;

        Ok(Self { x, y, id })
    }
}

#[derive(Debug, Clone)]
pub struct TileBuffer {
    row_count: u32,
    col_count: u32,
    buffer: Vec<TileId>,
}

impl TileBuffer {
    pub fn new(row_count: u32, col_count: u32) -> Self {
        Self {
            row_count,
            col_count,
            buffer: vec![TileId::Empty; (row_count * col_count) as usize],
        }
    }

    pub fn size(&self) -> (u32, u32) {
        (self.row_count, self.col_count)
    }

    pub fn first_pos(&self, id: TileId) -> Option<BallPosition> {
        for (ind, &this_id) in self.buffer.iter().enumerate() {
            if id == this_id {
                return Some(self.tile_pos(ind));
            }
        }

        None
    }

    pub fn tile(&self, x: u32, y: u32) -> &TileId {
        &self.buffer[self.tile_ind(x, y)]
    }

    pub fn tile_mut(&mut self, x: u32, y: u32) -> &mut TileId {
        let ind = self.tile_ind(x, y);
        &mut self.buffer[ind]
    }

    pub fn copy_from(&mut self, other: &Self) {
        self.row_count = other.row_count;
        self.col_count = other.col_count;
        self.buffer
            .resize_with(other.buffer.len(), Default::default);
        self.buffer.copy_from_slice(&other.buffer);
    }

    fn tile_ind(&self, x: u32, y: u32) -> usize {
        (y * self.col_count + x) as usize
    }

    fn tile_pos(&self, ind: usize) -> BallPosition {
        debug_assert!(ind < self.buffer.len());
        let ind = u32::try_from(ind).unwrap();
        BallPosition(ind % self.col_count, ind / self.col_count)
    }
}

#[derive(Debug, PartialEq, Eq)]
enum ScreenState {
    Rendering,
    TileUpdated,
}

static TILE_CHAR_MAP: Lazy<HashMap<TileId, char>> = Lazy::new(|| {
    let mut m = HashMap::new();
    m.insert(TileId::Empty, ' ');
    m.insert(TileId::Wall, 'X');
    m.insert(TileId::Block, '=');
    m.insert(TileId::HorizontalPaddle, '-');
    m.insert(TileId::Ball, 'O');

    m
});

pub struct Screen {
    buffer: TileBuffer,
    score: u32,
    term: Stdout,
    state: ScreenState,
}

impl Screen {
    pub fn from_ints<I: Iterator<Item = i64>>(mut tiles_info: I) -> Result<Self, Error> {
        let mut tiles = Vec::new();
        let mut max_row_ind = 0;
        let mut max_col_ind = 0;
        loop {
            if let Some(x) = tiles_info.next() {
                let y = tiles_info
                    .next()
                    .ok_or(Error::IncompleteTile(tiles.len()))?;
                let tile_id = tiles_info
                    .next()
                    .ok_or(Error::IncompleteTile(tiles.len()))?;
                tiles.push(Tile::from_info(&[x, y, tile_id])?);
                max_row_ind = cmp::max(max_row_ind, y as u32);
                max_col_ind = cmp::max(max_col_ind, x as u32);
            } else {
                break;
            }
        }

        let row_count = max_row_ind + 1;
        let col_count = max_col_ind + 1;
        let mut buffer = TileBuffer::new(row_count, col_count);
        for Tile { x, y, id } in &tiles {
            *buffer.tile_mut(*x, *y) = *id;
        }

        Ok(Self {
            buffer,
            score: 0,
            term: io::stdout(),
            state: ScreenState::Rendering,
        })
    }

    pub fn count_id(&self, id: TileId) -> usize {
        let (row_n, col_n) = self.buffer.size();
        (0..col_n)
            .flat_map(|x| (0..row_n).map(move |y| (x, y)))
            .map(|(x, y)| self.buffer.tile(x, y))
            .filter(|&&t_id| t_id == id)
            .count()
    }

    pub fn score(&self) -> u32 {
        self.score
    }

    fn update_state(&mut self, info: &GameInfo) -> Result<(), Error> {
        match Tile::from_info(info) {
            Ok(Tile { x, y, id }) => {
                *self.buffer.tile_mut(x, y) = id;
                if self.state == ScreenState::Rendering {
                    self.state = ScreenState::TileUpdated;
                }
            }
            Err(_) if info[0] == -1 && info[1] == 0 => {
                if let Ok(score) = u32::try_from(info[2]) {
                    self.score = score;
                    self.render().map_err(Error::TerminalError)?;
                } else {
                    return Err(Error::InvalidScore(info[2]));
                }
            }
            Err(e) => return Err(e),
        };

        Ok(())
    }

    fn refresh(&mut self) -> Result<(), Error> {
        if self.state == ScreenState::TileUpdated {
            self.state = ScreenState::Rendering;
        }

        self.render().map_err(Error::TerminalError)
    }

    fn render(&mut self) -> io::Result<()> {
        if self.state != ScreenState::Rendering {
            return Ok(());
        }

        // Render with render buffer
        let (term_col_n, term_row_n) = terminal::size()?;
        let term_col_n = term_col_n as u32;
        let term_row_n = term_row_n as u32;
        let (buf_row_n, buf_col_n) = self.buffer.size();
        if term_col_n < buf_col_n || term_row_n < buf_row_n {
            return Err(io::Error::new(
                io::ErrorKind::Unsupported,
                Error::NotEnoughTerminalSpace(term_row_n, term_col_n, buf_row_n, buf_col_n),
            ));
        }

        let tile_horz_margin_n = (term_col_n - buf_col_n) / 2;
        let score_str = format!("Score: {}", self.score);
        let score_horz_margin_n = (term_col_n as usize - score_str.len()) / 2;
        queue!(
            self.term,
            terminal::Clear(ClearType::All),
            cursor::MoveTo(0, 1)
        )?;
        for y in 0..buf_row_n {
            let line = iter::repeat(' ')
                .take(tile_horz_margin_n as usize)
                .chain((0..buf_col_n).map(|x| TILE_CHAR_MAP[self.buffer.tile(x, y)]))
                .collect::<String>();
            queue!(self.term, style::Print(line), cursor::MoveToNextLine(1))?;
        }
        queue!(
            self.term,
            cursor::MoveToNextLine(1),
            style::Print(" ".repeat(score_horz_margin_n) + &score_str),
            cursor::MoveToNextLine(1)
        )?;
        self.term.flush()
    }
}

#[repr(i8)]
#[derive(Debug, Default, Clone, Copy, IntEnum, PartialEq, Eq)]
pub enum JoystickState {
    Left = -1,
    #[default]
    Neutral = 0,
    Right = 1,
}

pub trait Player {
    fn prepare(&mut self);
    fn finish(&mut self);
    fn action(&mut self, buffer: &TileBuffer) -> JoystickState;
}

struct Joystick {
    state: Arc<Mutex<JoystickState>>,
    is_working: Arc<AtomicBool>,
}

impl Joystick {
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(JoystickState::Neutral)),
            is_working: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn start(&mut self, context: Arc<ThreadPool>) {
        if self.is_working.load(Ordering::Relaxed) {
            return;
        }

        let is_working = self.is_working.clone();
        let state = self.state.clone();
        self.is_working.store(true, Ordering::Release);
        context.spawn(move || {
            while is_working.load(Ordering::Acquire) {
                if poll(Duration::from_millis(100)).unwrap() {
                    match read().unwrap() {
                        Event::Key(ke) => match ke.code {
                            KeyCode::Left => {
                                let mut state = state.lock().unwrap();
                                if ke.kind == KeyEventKind::Press
                                    && *state == JoystickState::Neutral
                                {
                                    *state = JoystickState::Left;
                                } else if ke.kind == KeyEventKind::Release
                                    && *state == JoystickState::Left
                                {
                                    *state = JoystickState::Neutral;
                                }
                            }
                            KeyCode::Right => {
                                let mut state = state.lock().unwrap();
                                if ke.kind == KeyEventKind::Press
                                    && *state == JoystickState::Neutral
                                {
                                    *state = JoystickState::Right;
                                } else if ke.kind == KeyEventKind::Release
                                    && *state == JoystickState::Right
                                {
                                    *state = JoystickState::Neutral;
                                }
                            }
                            _ => (),
                        },
                        _ => (),
                    }
                }
            }
        });
    }

    pub fn stop(&mut self) {
        self.is_working.store(false, Ordering::Relaxed);
    }

    pub fn state(&self) -> JoystickState {
        *self.state.lock().unwrap()
    }
}

pub struct ManualPlayer {
    joystick: Joystick,
    context: Arc<ThreadPool>,
}

impl ManualPlayer {
    pub fn new(context: Arc<ThreadPool>) -> Self {
        Self {
            joystick: Joystick::new(),
            context,
        }
    }
}

impl Player for ManualPlayer {
    fn prepare(&mut self) {
        self.joystick.start(self.context.clone());
    }

    fn finish(&mut self) {
        self.joystick.stop();
    }

    fn action(&mut self, _buffer: &TileBuffer) -> JoystickState {
        self.joystick.state()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BallPosition(u32, u32);

pub struct AutoPlayer {
    last_ball_pos: Option<BallPosition>,
    pred_ball_pos: Option<BallPosition>,
    test_buffer: TileBuffer,
}

impl AutoPlayer {
    pub fn new() -> Self {
        Self {
            last_ball_pos: None,
            pred_ball_pos: None,
            test_buffer: TileBuffer::new(0, 0),
        }
    }

    fn pred_ball_pos(
        &mut self,
        ball_pos: &BallPosition,
        buffer: &TileBuffer,
    ) -> Option<BallPosition> {
        if self.last_ball_pos.is_none() {
            return None;
        }

        self.test_buffer.copy_from(buffer);
        let BallPosition(last_ball_x, last_ball_y) = self.last_ball_pos.unwrap();
        let BallPosition(ball_x, ball_y) = *ball_pos;
        let mut ball_vel_x = ball_x as i64 - last_ball_x as i64;
        let mut ball_vel_y = ball_y as i64 - last_ball_y as i64;
        let mut have_bounce = true;
        let mut next_ball_x = 0u32;
        let mut next_ball_y = 0u32;
        while have_bounce {
            next_ball_x = u32::try_from(ball_x as i64 + ball_vel_x).unwrap();
            next_ball_y = u32::try_from(ball_y as i64 + ball_vel_y).unwrap();
            // Vertical tile
            let mut no_verticle_bounce = false;
            let vert_tile = self.test_buffer.tile_mut(ball_x, next_ball_y);
            match *vert_tile {
                TileId::Block => {
                    ball_vel_y = -ball_vel_y;
                    *vert_tile = TileId::Empty;
                }
                TileId::Wall | TileId::HorizontalPaddle => ball_vel_y = -ball_vel_y,
                _ => no_verticle_bounce = true,
            }

            // Horizontal tile
            let mut no_horizontal_bounce = false;
            let horz_tile = self.test_buffer.tile_mut(next_ball_x, ball_y);
            match *horz_tile {
                TileId::Block => {
                    ball_vel_x = -ball_vel_x;
                    *horz_tile = TileId::Empty;
                }
                TileId::Wall => ball_vel_x = -ball_vel_x,
                _ => no_horizontal_bounce = true,
            }

            // Diagonal tile
            let mut no_diagonal_bounce = false;
            if no_horizontal_bounce && no_verticle_bounce {
                let diag_tile = self.test_buffer.tile_mut(next_ball_x, next_ball_y);
                match *diag_tile {
                    TileId::Block => {
                        ball_vel_x = -ball_vel_x;
                        ball_vel_y = -ball_vel_y;
                        *diag_tile = TileId::Empty;
                    }
                    TileId::Wall | TileId::HorizontalPaddle => {
                        ball_vel_x = -ball_vel_x;
                        ball_vel_y = -ball_vel_y;
                    }
                    _ => no_diagonal_bounce = true,
                }
            } else {
                no_diagonal_bounce = true;
            }

            have_bounce = !no_horizontal_bounce || !no_diagonal_bounce || !no_diagonal_bounce;
        }

        Some(BallPosition(next_ball_x, next_ball_y))
    }
}

impl Player for AutoPlayer {
    fn prepare(&mut self) {
        self.last_ball_pos = None;
        self.pred_ball_pos = None;
    }

    fn finish(&mut self) {}

    fn action(&mut self, buffer: &TileBuffer) -> JoystickState {
        let ball_pos = buffer
            .first_pos(TileId::Ball)
            .expect("Failed to find the ball in game");
        let BallPosition(paddle_pos_x, paddle_pos_y) = buffer
            .first_pos(TileId::HorizontalPaddle)
            .expect("Failed to find the paddle in game");
        let paddle_move_delta = if paddle_pos_x != ball_pos.0 || paddle_pos_y != ball_pos.1 + 1 {
            // Ball isn't exactly on the paddle
            if let Some(ball_next_pos) = self.pred_ball_pos(&ball_pos, buffer) {
                self.pred_ball_pos = Some(ball_next_pos);
                ball_next_pos.0 as i64 - paddle_pos_x as i64
            } else {
                0i64
            }
        } else {
            0i64
        };

        self.last_ball_pos = Some(BallPosition(ball_pos.0.into(), ball_pos.1.into()));
        JoystickState::from_int(paddle_move_delta.signum() as i8).unwrap()
    }
}

struct GameInfoBuffer {
    buffer: GameInfo,
    cur_ind: u8,
}

impl GameInfoBuffer {
    fn new() -> Self {
        Self {
            buffer: [0; 3],
            cur_ind: 0,
        }
    }

    fn push_and_get(&mut self, value: i64) -> Option<GameInfo> {
        self.buffer[self.cur_ind as usize] = value;
        self.cur_ind += 1;

        if self.cur_ind >= 3 {
            self.cur_ind = 0;
            Some(self.buffer)
        } else {
            None
        }
    }
}

pub struct ArcadeCabinet<P: Player> {
    frame_interval: Duration,
    last_refresh_time: Option<Instant>,
    game_info_buffer: GameInfoBuffer,
    player: P,
    screen: Arc<Mutex<Screen>>,
    context: Arc<ThreadPool>,
}

impl<P: Player> ArcadeCabinet<P> {
    pub fn new(fps: u32, screen: Screen, player: P, context: Arc<ThreadPool>) -> Self {
        assert!(fps <= 1000, "FPS greater than 1000 isn't supported");
        ArcadeCabinet {
            frame_interval: Duration::from_millis((1000 / fps) as u64),
            last_refresh_time: None,
            game_info_buffer: GameInfoBuffer::new(),
            player,
            screen: Arc::new(Mutex::new(screen)),
            context,
        }
    }

    pub fn start(&mut self) -> Result<(), Error> {
        Self::configure_term(io::stdout()).map_err(Error::TerminalError)?;
        self.player.prepare();
        let mut screen = self.screen.lock().unwrap();
        screen.refresh()?;
        Ok(())
    }

    pub fn stop(&mut self) -> Result<(), Error> {
        self.player.finish();
        let mut screen = self.screen.lock().unwrap();
        screen.refresh()?;
        Self::restore_term(io::stdout()).map_err(Error::TerminalError)?;
        Ok(())
    }

    pub fn check_screen<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&Screen) -> R,
    {
        f(&self.screen.lock().unwrap())
    }

    fn update_game_state(&mut self, value: i64) -> Result<(), Error> {
        if let Some(game_info) = self.game_info_buffer.push_and_get(value) {
            self.screen.lock().unwrap().update_state(&game_info)?;
        }

        Ok(())
    }

    fn configure_term(mut term: Stdout) -> io::Result<()> {
        execute!(term, terminal::EnterAlternateScreen)?;
        terminal::enable_raw_mode()?;
        queue!(
            term,
            style::ResetColor,
            terminal::Clear(ClearType::All),
            cursor::Hide,
            cursor::MoveTo(0, 0)
        )?;
        term.flush()
    }

    fn restore_term(mut term: Stdout) -> io::Result<()> {
        execute!(
            term,
            style::ResetColor,
            cursor::Show,
            terminal::LeaveAlternateScreen
        )?;
        terminal::disable_raw_mode()
    }
}

impl<P: Player> InputPort for ArcadeCabinet<P> {
    fn get(&mut self) -> Option<i64> {
        let screen = self.screen.clone();
        self.context.spawn(move || {
            screen
                .lock()
                .unwrap()
                .refresh()
                .expect("Failed to refresh screen when poll input");
        });

        if let Some(last_refresh_time) = self.last_refresh_time {
            let now = Instant::now();
            let refresh_interval = now.duration_since(last_refresh_time);
            let sleep_interval = self.frame_interval.saturating_sub(refresh_interval);
            if !sleep_interval.is_zero() {
                thread::sleep(sleep_interval);
            }
            self.last_refresh_time = Some(now);
        } else {
            self.last_refresh_time = Some(Instant::now());
        }

        let screen = self.screen.clone();
        let action = self.player.action(&screen.lock().unwrap().buffer);
        Some(action.int_value().into())
    }

    fn reg_proc(&mut self, _proc_id: usize) {}
}

impl<P: Player> OutputPort for ArcadeCabinet<P> {
    fn put(&mut self, value: i64) -> Result<(), crate::Error> {
        self.update_game_state(value)
            .map_err(|e| crate::Error::IOProcessError(e.to_string()))
    }

    fn wait_proc_id(&self) -> Option<usize> {
        None
    }
}
