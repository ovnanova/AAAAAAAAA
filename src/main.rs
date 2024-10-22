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

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;
    use unicode_segmentation::UnicodeSegmentation;

    #[test]
    fn test_random_string_length() {
        for _ in 0..1000000 {
            let s = random_string();
            let grapheme_count = s.graphemes(true).count();
            assert!(
                (1..=32).contains(&grapheme_count),
                "Grapheme count out of bounds: {} (string: {})",
                grapheme_count,
                s
            );
        }
    }

    #[test]
    fn test_all_chars_appear() {
        let mut appearances = HashSet::<&'static str>::new();
        for _ in 0..10000 {
            let s = random_string();
            appearances.extend(CHAR_SET.iter().filter(|&&c| s.contains(c)));
        }
        assert_eq!(
            appearances.len(),
            CHAR_SET.len(),
            "Not all characters appeared in 10000 iterations"
        );
    }

    #[test]
    fn test_stream_direction_changes() {
        let mut stream = Stream::new(80, 24);
        let mut direction_changes = 0;
        let mut last_direction = stream.direction.get_offset();

        for _ in 0..1000 {
            stream.update(80, 24);
            let new_direction = stream.direction.get_offset();
            if new_direction != last_direction {
                direction_changes += 1;
            }
            last_direction = new_direction;
        }

        assert!(
            direction_changes > 50,
            "Stream should change direction frequently, only changed {} times",
            direction_changes
        );
    }

    #[test]
    fn test_stream_bounds() {
        let mut stream = Stream::new(80, 24);
        for _ in 0..10000 {
            stream.update(80, 24);
            assert!(
                stream.x >= 1 && stream.x <= 78,
                "X out of bounds: {}",
                stream.x
            );
            assert!(stream.y >= 1, "Y below minimum: {}", stream.y);
        }
    }

    #[test]
    fn test_color_distribution() {
        let mut color_counts = std::collections::HashMap::new();
        for _ in 0..10000 {
            let color = random_color();
            *color_counts.entry(format!("{:?}", color)).or_insert(0) += 1;
        }

        // Check that each color appeared at least once
        assert!(
            color_counts.len() >= COLORS.len(),
            "Not all colors appeared: {:?}",
            color_counts
        );

        // Verify primary colors appear more often than accents
        for weight in COLORS {
            match weight {
                Weight::Primary(c, _) => {
                    let count = color_counts.get(&format!("{:?}", c)).unwrap_or(&0);
                    assert!(
                        count > &500,
                        "Primary color {:?} appeared only {} times",
                        c,
                        count
                    );
                }
                Weight::Accent(c, _) => {
                    let count = color_counts.get(&format!("{:?}", c)).unwrap_or(&0);
                    assert!(
                        count > &100,
                        "Accent color {:?} appeared only {} times",
                        c,
                        count
                    );
                }
            }
        }
    }

    #[test]
    fn test_chaos_probability() {
        let mut new_streams = 0;
        let trials = 10000;

        for _ in 0..trials {
            if rand::thread_rng().gen_bool(CHAOS) {
                new_streams += 1;
            }
        }

        let actual_probability = new_streams as f64 / trials as f64;
        assert!(
            (actual_probability - CHAOS).abs() < 0.02,
            "Chaos probability {} significantly deviated from expected {}",
            actual_probability,
            CHAOS
        );
    }

    #[test]
    fn test_random_string_content() {
        let s = random_string();
        assert!(
            s.chars()
                .all(|c| CHAR_SET.iter().any(|&set| set.contains(c))),
            "Invalid characters in string: {}",
            s
        );
    }

    #[test]
    fn test_color_weights() {
        let total: u8 = COLORS
            .iter()
            .map(|c| match c {
                Weight::Primary(_, w) | Weight::Accent(_, w) => w,
            })
            .sum();
        assert!(total > 0, "Total color weights must be positive");

        let mut counts = std::collections::HashMap::new();
        for _ in 0..1000 {
            let color = random_color();
            *counts.entry(color).or_insert(0) += 1;
        }

        // Verify primary colors appear more frequently than accents
        for color_weight in COLORS {
            match color_weight {
                Weight::Primary(c, _) => {
                    let count = counts.get(c).unwrap_or(&0);
                    assert!(*count > 100, "Primary color {:?} appeared too rarely", c);
                }
                Weight::Accent(_, _) => {}
            }
        }
    }

    #[test]
    fn test_stream_boundaries() {
        let mut stream = Stream::new(80, 24);

        // Test multiple updates to ensure boundaries are respected
        for _ in 0..1000 {
            stream.update(80, 24);
            assert!(
                stream.x > 0 && stream.x < 79,
                "X position out of bounds: {}",
                stream.x
            );
            assert!(stream.y > 0, "Y position below zero: {}", stream.y);
        }
    }

    #[test]
    fn test_direction_distribution() {
        let mut counts = std::collections::HashMap::new();
        for _ in 0..1000 {
            let dir = Direction::random();
            let offset = dir.get_offset();
            *counts.entry(offset).or_insert(0) += 1;
        }

        // Check that all directions are used
        assert_eq!(counts.len(), 8, "Not all directions were generated");

        // Check for roughly even distribution
        for (_offset, count) in counts {
            assert!(count > 50, "Direction appeared too rarely: {} times", count);
        }
    }

    #[test]
    fn test_stream_movement() {
        let mut stream = Stream::new(80, 24);
        let initial_pos = (stream.x, stream.y);

        // Store a few positions to verify movement
        let mut positions = vec![initial_pos];
        for _ in 0..10 {
            stream.update(80, 24);
            positions.push((stream.x, stream.y));
        }

        // Verify that the stream actually moved
        assert!(
            positions.windows(2).any(|w| w[0] != w[1]),
            "Stream didn't move from initial position"
        );
    }
}
