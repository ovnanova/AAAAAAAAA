use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{self, Event, KeyCode, KeyEvent},
    execute, queue,
    style::{Color, SetForegroundColor},
    terminal::{disable_raw_mode, enable_raw_mode, size},
};
use rand::Rng;
use std::{
    io::{self, stdout, Write},
    sync::atomic::{AtomicBool, Ordering},
    sync::Arc,
    thread,
    time::Duration,
};

const CHAR_SET: &[&str] = &[
    "A̵̦̦̓͌͗͛̕",
    "A",
    "₳",
    "░A░",
    "A҉",
    "Ⱥ",
    "A̷",
    "A̲",
    "A̳",
    "A̾",
    "A͎",
    "A͓̽",
    "𝔸",
    "ᴀ",
    "∀",
];

const CHAOS: f64 = 0.20;

#[derive(Clone, Copy)]
enum Weight {
    Primary(Color, u8),
    Accent(Color, u8),
}

const COLORS: &[Weight] = &[
    Weight::Accent(Color::AnsiValue(0), 10),
    Weight::Accent(Color::AnsiValue(18), 10),
    Weight::Accent(Color::AnsiValue(29), 10),
    Weight::Accent(Color::AnsiValue(39), 10),
    Weight::Accent(Color::AnsiValue(128), 10),
    Weight::Accent(Color::AnsiValue(199), 10),
    Weight::Accent(Color::AnsiValue(206), 10),
    Weight::Primary(Color::AnsiValue(255), 30),
];

enum Direction {
    Left,
    Right,
    Up,
    Down,
    UpLeft,
    UpRight,
    DownLeft,
    DownRight,
}

impl Direction {
    fn random() -> Self {
        let mut rng = rand::thread_rng();
        match rng.gen_range(0..8) {
            0 => Direction::Left,
            1 => Direction::Right,
            2 => Direction::Up,
            3 => Direction::Down,
            4 => Direction::UpLeft,
            5 => Direction::UpRight,
            6 => Direction::DownLeft,
            _ => Direction::DownRight,
        }
    }

    fn get_offset(&self) -> (i16, i16) {
        match self {
            Direction::Left => (-1, 0),
            Direction::Right => (1, 0),
            Direction::Up => (0, -1),
            Direction::Down => (0, 1),
            Direction::UpLeft => (-1, -1),
            Direction::UpRight => (1, -1),
            Direction::DownLeft => (-1, 1),
            Direction::DownRight => (1, 1),
        }
    }
}

struct Stream {
    x: u16,
    y: u16,
    direction: Direction,
}

impl Stream {
    fn new(max_x: u16, max_y: u16) -> Self {
        let mut rng = rand::thread_rng();
        Self {
            x: rng.gen_range(0..max_x),
            y: rng.gen_range(0..max_y),
            direction: Direction::random(),
        }
    }

    fn update(&mut self, max_x: u16, _max_y: u16) {
        let mut rng = rand::thread_rng();
        let (dx, dy) = self.direction.get_offset();

        let new_x = self.x as i16 + dx;
        let new_y = self.y as i16 + dy;

        if new_x <= 0 || new_x >= max_x as i16 - 2 || new_y <= 0 || rng.gen_bool(0.1) {
            // Removed the max_y boundary check to allow scrolling
            self.direction = Direction::random();
            if new_x <= 0 {
                self.x = 1;
            }
            if new_x >= max_x as i16 - 1 {
                self.x = max_x - 2;
            }
            if new_y <= 0 {
                self.y = 1;
            }
        } else {
            self.x = new_x as u16;
            self.y = new_y as u16;
        }
    }
}

fn random_string() -> String {
    let mut rng = rand::thread_rng();
    let length = rng.gen_range(1..=16);
    (0..length)
        .map(|_| CHAR_SET[rng.gen_range(0..CHAR_SET.len())])
        .collect()
}

fn random_color() -> Color {
    let total_weight: u8 = COLORS
        .iter()
        .map(|c| match c {
            Weight::Primary(_, w) | Weight::Accent(_, w) => w,
        })
        .sum();

    let mut rng = rand::thread_rng();
    let mut choice = rng.gen_range(0..total_weight);

    for color_weight in COLORS {
        let weight = match color_weight {
            Weight::Primary(_, w) | Weight::Accent(_, w) => w,
        };
        if choice < *weight {
            return match color_weight {
                Weight::Primary(c, _) | Weight::Accent(c, _) => *c,
            };
        }
        choice -= weight;
    }

    Color::White
}

fn main() -> io::Result<()> {
    let running = Arc::new(AtomicBool::new(true));
    let paused = Arc::new(AtomicBool::new(false));

    let printer_running = running.clone();
    let printer_paused = paused.clone();

    enable_raw_mode()?;
    execute!(stdout(), Hide)?;

    let printer_thread = thread::spawn(move || {
        let mut stdout = stdout();
        let mut rng = rand::thread_rng();
        let mut streams = Vec::new();

        while printer_running.load(Ordering::SeqCst) {
            if !printer_paused.load(Ordering::SeqCst) {
                if let Ok((max_x, max_y)) = size() {
                    if rng.gen_bool(CHAOS) {
                        streams.push(Stream::new(max_x, max_y));
                    }

                    for stream in &mut streams {
                        stream.update(max_x, max_y);
                        queue!(
                            stdout,
                            MoveTo(stream.x, stream.y),
                            SetForegroundColor(random_color()),
                            crossterm::style::Print(random_string())
                        )
                        .unwrap();
                    }
                    stdout.flush().unwrap();
                }

                if streams.len() > 20 {
                    streams.remove(0);
                }

                thread::sleep(Duration::from_millis(50));
            } else {
                thread::sleep(Duration::from_millis(50));
            }
        }
    });

    while running.load(Ordering::SeqCst) {
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(KeyEvent { code, .. }) = event::read()? {
                match code {
                    KeyCode::Char('q') | KeyCode::Char('Q') => {
                        running.store(false, Ordering::SeqCst);
                        break;
                    }
                    KeyCode::Char(' ') => {
                        let current = paused.load(Ordering::SeqCst);
                        paused.store(!current, Ordering::SeqCst);
                        if !current {
                            execute!(
                                stdout(),
                                MoveTo(0, 0),
                                crossterm::style::Print(
                                    "*PAUSED* (press [SPACE] to resume, [q] to quit)"
                                )
                            )?;
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    execute!(stdout(), Show)?;
    disable_raw_mode()?;
    printer_thread.join().unwrap();
    execute!(
        stdout(),
        SetForegroundColor(Color::Reset),
        crossterm::terminal::Clear(crossterm::terminal::ClearType::All),
        MoveTo(0, 0)
    )?;

    Ok(())
}
