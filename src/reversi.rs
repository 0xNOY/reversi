use std::{
    cell::RefCell,
    fmt::Display,
    io::{Stdin, Stdout, Write},
    ops::{Index, IndexMut},
    thread::sleep,
    time::Duration,
};

pub const BOARD_SIZE: usize = 8;

#[derive(Clone, Copy, Debug)]
pub struct Point(pub usize, pub usize);

impl Point {
    fn shift(&self, xd: isize, yd: isize) -> Self {
        Self(
            (self.0 as isize + xd) as usize,
            (self.1 as isize + yd) as usize,
        )
    }

    fn is_in_board(&self) -> bool {
        BOARD_SIZE > self.0 && BOARD_SIZE > self.1
    }
}

impl Index<usize> for Point {
    type Output = usize;
    fn index(&self, index: usize) -> &Self::Output {
        match index {
            0 => &self.0,
            1 => &self.1,
            _ => panic!("out of range: {}", index),
        }
    }
}

impl IndexMut<usize> for Point {
    fn index_mut(&mut self, index: usize) -> &mut usize {
        match index {
            0 => &mut self.0,
            1 => &mut self.1,
            _ => panic!("out of range: {}", index),
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum Stone {
    Black,
    White,
}

impl Display for Stone {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Black => "●",
                Self::White => "○",
            }
        )
    }
}

impl From<usize> for Stone {
    fn from(i: usize) -> Self {
        if i % 2 == 0 {
            Self::Black
        } else {
            Self::White
        }
    }
}

#[derive(Clone, Copy)]
pub enum Error {
    Uncontinuable,
    OutOfBoard,
    StoneAlreadyExists,
    NoChainOccurred,
    PlayerError(PlayerError),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Error::PlayerError(e) => e.to_string(),
                _ => "error occurred!".to_owned(),
            }
        )
    }
}

#[derive(Clone, Copy)]
pub struct Board([[Option<Stone>; BOARD_SIZE]; BOARD_SIZE]);

impl Board {
    fn new() -> Self {
        Board([[None; BOARD_SIZE]; BOARD_SIZE])
    }

    fn put(&mut self, point: Point, stone: Stone) -> Result<(), Error> {
        if !point.is_in_board() {
            return Err(Error::OutOfBoard);
        } else if self.get(&point).is_some() {
            return Err(Error::StoneAlreadyExists);
        }

        let chain = self.scan_stones_turning(point, stone, false);
        if 0 >= chain.len() {
            return Err(Error::NoChainOccurred);
        }
        self.0[point.0][point.1] = Some(stone);
        for p in chain {
            self.0[p.0][p.1] = Some(stone);
        }
        Ok(())
    }

    pub fn get(&self, point: &Point) -> Option<Stone> {
        match self.0.get(point.0) {
            Some(line) => *line.get(point.1).unwrap_or(&None),
            None => None,
        }
    }

    /// return (black_stones_number, white_stones_number)
    pub fn count_stones(&self) -> (usize, usize) {
        let mut result = (0, 0);

        for line in self.0 {
            for cell in line {
                if let Some(stone) = cell {
                    match stone {
                        Stone::Black => result.0 += 1,
                        Stone::White => result.1 += 1,
                    }
                }
            }
        }

        result
    }

    pub fn scan_stones_turning(
        &self,
        org_point: Point,
        org_stone: Stone,
        should_scan_one_only: bool, // 裏返る列が一列見つかったら探査を終了するか
    ) -> Vec<Point> {
        let mut result = Vec::new();

        for xd in -1..=1 {
            for yd in -1..=1 {
                let mut trace = Vec::with_capacity(BOARD_SIZE);
                trace.push(org_point);
                for i in 1..BOARD_SIZE {
                    let point = trace[i - 1].shift(xd, yd);
                    match self.get(&point) {
                        Some(stone) => {
                            if stone == org_stone {
                                result.extend_from_slice(&trace[1..]);
                                if should_scan_one_only && result.len() > 0 {
                                    return result;
                                }
                                break;
                            }
                        }
                        None => break,
                    }
                    trace.push(point);
                }
            }
        }
        result
    }

    pub fn scan_cells_placeable(&self, stone: Stone) -> Vec<Point> {
        let mut result = Vec::new();

        for x in 0..BOARD_SIZE {
            for y in 0..BOARD_SIZE {
                if self.0[x][y].is_none() {
                    if self.scan_stones_turning(Point(x, y), stone, true).len() > 0 {
                        result.push(Point(x, y));
                    }
                }
            }
        }

        result
    }

    pub fn format(
        &self,
        empty_symbol: char,
        black_symbol: char,
        white_symbol: char,
        hint_symbol: char,
        stone_for_hint: Option<Stone>,
        x_index: Option<[char; BOARD_SIZE]>,
        y_index: Option<[char; BOARD_SIZE]>,
    ) -> Vec<Vec<char>> {
        let mut result = vec![vec![empty_symbol; BOARD_SIZE]; BOARD_SIZE];

        for x in 0..BOARD_SIZE {
            for y in 0..BOARD_SIZE {
                match self.0[x][y] {
                    Some(stone) => {
                        result[y][x] = match stone {
                            Stone::Black => black_symbol,
                            Stone::White => white_symbol,
                        }
                    }
                    None => continue,
                }
            }
        }

        match stone_for_hint {
            Some(stone) => {
                for point in self.scan_cells_placeable(stone) {
                    result[point.1][point.0] = hint_symbol;
                }
            }
            None => (),
        }

        if let Some(ia) = y_index {
            // ia.iter()
            //     .zip((0..result.len()))
            //     .map(|(i, li)| result[li].insert(0, *i));
            for (li, c) in ia.iter().enumerate() {
                result[li].insert(0, *c);
            }
        }

        if let Some(ia) = x_index {
            let mut iv = ia.to_vec();
            if y_index.is_some() {
                iv.insert(0, ' ');
            }
            result.insert(0, iv);
        }

        result
    }
}

impl Default for Board {
    fn default() -> Self {
        let center = BOARD_SIZE / 2 - 1;
        let mut board = Self::new();
        for x in center..center + 2 {
            for y in center..center + 2 {
                board.0[x][y] = Some(Stone::from(x + y));
            }
        }
        board
    }
}

impl Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let index: [char; BOARD_SIZE] = (0..BOARD_SIZE)
            .map(|i| i.to_string().chars().next().unwrap())
            .collect::<Vec<char>>()
            .try_into()
            .unwrap();
        let chars = self.format(
            '+',
            Stone::Black.to_string().chars().next().unwrap(),
            Stone::White.to_string().chars().next().unwrap(),
            '@',
            None,
            Some(index),
            Some(index),
        );

        write!(
            f,
            "{}",
            chars
                .iter()
                .map(|line| line
                    .iter()
                    .map(|c| c.to_string())
                    .collect::<Vec<String>>()
                    .join(" "))
                .collect::<Vec<String>>()
                .join("\n")
        )
    }
}

#[derive(Clone, Copy)]
pub struct GameContext {
    pub round: usize,
    pub board: Board,
    pub error: Option<Error>,
    pub is_done: bool,
}

impl GameContext {
    fn new() -> Self {
        Self {
            round: 0,
            board: Board::default(),
            error: None,
            is_done: false,
        }
    }
}

#[derive(Clone, Copy)]
pub enum PlayerError {
    Uncontinuable,
    UserInputParseFailure,
}

impl Display for PlayerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                _ => "error occurred!",
            }
        )
    }
}

pub trait Player {
    fn make_move(&self, ctx: &GameContext) -> Result<Point, PlayerError>;
    fn win(&self, ctx: &GameContext);
    fn lose(&self, ctx: &GameContext);
}

pub struct Game<B: Player, W: Player> {
    black_player: B,
    white_player: W,
    context: GameContext,
}

impl<B: Player, W: Player> Game<B, W> {
    pub fn new(black_player: B, white_player: W) -> Self {
        Self {
            black_player: black_player,
            white_player: white_player,
            context: GameContext::new(),
        }
    }

    pub fn get_context(&self) -> &GameContext {
        &self.context
    }

    pub fn make_move(&mut self) -> GameContext {
        let player_movement = if self.context.round % 2 == 0 {
            self.black_player.make_move(&self.context)
        } else {
            self.white_player.make_move(&self.context)
        };

        match player_movement {
            Ok(point) => {
                if let Err(e) = self
                    .context
                    .board
                    .put(point, Stone::from(self.context.round))
                {
                    self.context.error = Some(e);
                } else {
                    if 0 >= self
                        .context
                        .board
                        .scan_cells_placeable(Stone::from(self.context.round + 1))
                        .len()
                    {
                        if 0 >= self
                            .context
                            .board
                            .scan_cells_placeable(Stone::from(self.context.round))
                            .len()
                        {
                            self.context.is_done = true;
                            let (black, white) = self.context.board.count_stones();
                            if black > white {
                                self.black_player.win(&self.context);
                                self.white_player.lose(&self.context);
                            } else {
                                self.black_player.lose(&self.context);
                                self.white_player.win(&self.context);
                            }
                        } else {
                            self.context.round += 1;
                        }
                    }
                    self.context.round += 1;
                    self.context.error = None;
                }
            }
            Err(e) => match e {
                PlayerError::Uncontinuable => {
                    self.context.error = Some(Error::Uncontinuable);
                    self.context.is_done = true
                }
                _ => self.context.error = Some(Error::PlayerError(e)),
            },
        }

        self.context
    }

    pub fn start(&mut self) -> &GameContext {
        while !self.context.is_done {
            self.make_move();
        }
        &self.context
    }
}

impl<B: Player, W: Player> Iterator for Game<B, W> {
    type Item = GameContext;

    fn next(&mut self) -> Option<Self::Item> {
        if self.context.is_done {
            None
        } else {
            Some(self.make_move())
        }
    }
}

pub struct CliPlayer {
    stdin: Stdin,
    stdout: RefCell<Stdout>,
}

impl CliPlayer {
    pub fn new(stdin: Stdin, stdout: Stdout) -> Self {
        Self {
            stdin: stdin,
            stdout: RefCell::new(stdout),
        }
    }

    fn print(&self, text: &str) {
        let o = &mut self.stdout.borrow_mut();
        write!(o, "{}", text);
        o.flush();
    }

    pub fn format_board(ctx: &GameContext) -> String {
        let board_index: [char; BOARD_SIZE] = (1..=BOARD_SIZE)
            .map(|i| i.to_string().chars().next().unwrap())
            .collect::<Vec<char>>()
            .try_into()
            .unwrap();
        ctx.board
            .format(
                '+',
                Stone::Black.to_string().chars().next().unwrap(),
                Stone::White.to_string().chars().next().unwrap(),
                '@',
                Some(Stone::from(ctx.round)),
                Some(board_index),
                Some(board_index),
            )
            .iter()
            .map(|line| {
                line.iter()
                    .map(|c| c.to_string())
                    .collect::<Vec<String>>()
                    .join(" ")
            })
            .collect::<Vec<String>>()
            .join("\n")
    }

    fn clear_terminal(&self) {
        write!(self.stdout.borrow_mut(), "\x1b[2J");
    }
}

impl Default for CliPlayer {
    fn default() -> Self {
        Self::new(std::io::stdin(), std::io::stdout())
    }
}

impl Player for CliPlayer {
    fn make_move(&self, ctx: &GameContext) -> Result<Point, PlayerError> {
        if let Some(e) = ctx.error {
            self.print(match e {
                Error::PlayerError(PlayerError::Uncontinuable) => "Unrecoverable Error Occurred",
                Error::Uncontinuable => "Unrecoverable Error Occurred",
                _ => "Invalid Value\n[x] [y] <= ",
            });
        } else {
            self.clear_terminal();
            self.print(&format!(
                "TURN OF {}\n{}\n[x] [y] <= ",
                Stone::from(ctx.round),
                Self::format_board(ctx)
            ));
        }
        let mut point = Point(0, 0);

        let mut input = String::new();
        self.stdin.read_line(&mut input);
        input.remove(input.len() - 1);
        for (i, pr) in input
            .split_whitespace()
            .map(|s| s.parse::<usize>())
            .enumerate()
        {
            if i > 1 {
                break;
            }

            match pr {
                Ok(c) => {
                    if c > 0 {
                        point[i] = c - 1
                    } else {
                        return Err(PlayerError::UserInputParseFailure);
                    }
                }
                Err(e) => {
                    return Err(PlayerError::UserInputParseFailure);
                }
            }
        }

        Ok(point)
    }

    fn win(&self, ctx: &GameContext) {
        self.clear_terminal()
    }
    fn lose(&self, ctx: &GameContext) {
        self.clear_terminal()
    }
}

pub struct WeekBot;

impl Player for WeekBot {
    fn make_move(&self, ctx: &GameContext) -> Result<Point, PlayerError> {
        if let Some(e) = ctx.error {
            match e {
                _ => return Err(PlayerError::Uncontinuable),
            }
        }

        let mut chains = vec![None; BOARD_SIZE * 3];
        for x in 0..BOARD_SIZE {
            for y in 0..BOARD_SIZE {
                let org_point = Point(x, y);
                if ctx.board.get(&org_point).is_some() {
                    continue;
                }
                let mut chain =
                    ctx.board
                        .scan_stones_turning(org_point, Stone::from(ctx.round), false);
                let chain_len = chain.len();
                chain.insert(0, org_point);
                chains[chain_len] = Some(chain);
            }
        }

        chains.reverse();
        for co in chains {
            if let Some(c) = co {
                sleep(Duration::from_millis(10));
                return Ok(c[0]);
            }
        }

        Err(PlayerError::Uncontinuable)
    }

    fn win(&self, ctx: &GameContext) {}
    fn lose(&self, ctx: &GameContext) {}
}
