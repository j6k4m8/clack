use crate::sound::{SoundManager, Tone, Utterance};
use crate::Document;
use crate::Row;
use crate::Terminal;
use std::env;
use std::time::Duration;
use std::time::Instant;
use termion::color;
use termion::event::Key;

const VERSION: &str = env!("CARGO_PKG_VERSION");

const STATUS_FG_COLOR: color::Rgb = color::Rgb(63, 63, 63);
const STATUS_BG_COLOR: color::Rgb = color::Rgb(239, 239, 239);

#[derive(PartialEq)]
enum QuitStatus {
    Default,
    Confirming,
    Quitting,
}

#[derive(Default)]
pub struct Position {
    pub x: usize,
    pub y: usize,
}

pub struct Editor {
    should_quit: QuitStatus,
    should_draw_ui: bool,
    wrap_arrow_key_navigation: bool,
    terminal: Terminal,
    cursor_position: Position,
    offset: Position,
    document: Document,
    status_message: StatusMessage,
    sound_manager: SoundManager,
}

enum Mode {
    Editing,
}

struct StatusMessage {
    text: String,
    time: Instant,
}

impl StatusMessage {
    fn from(message: String) -> Self {
        Self {
            time: Instant::now(),
            text: message,
        }
    }
}

impl Editor {
    pub fn run(&mut self) {
        self.change_mode(Mode::Editing);
        loop {
            if let Err(error) = self.refresh_screen() {
                die(error);
            }
            if self.should_quit == QuitStatus::Quitting {
                break;
            }
            let input_handler = self.process_keypress();
            match input_handler {
                Err(error) => die(error),
                _ => (),
            };
            self.sound_manager.speak_next_or_wait();
        }
    }

    pub fn default() -> Self {
        let args: Vec<String> = env::args().collect();
        let mut initial_status = String::from("Ctrl-S = save | Ctrl-Q = quit");
        let document = if args.len() > 1 {
            let file_name = &args[1];
            let doc = Document::open(&file_name);
            if doc.is_ok() {
                doc.unwrap()
            } else {
                initial_status = format!("ERR: Could not open file: {}", file_name);
                Document::default()
            }
        } else {
            Document::default()
        };

        Self {
            should_quit: QuitStatus::Default,
            should_draw_ui: true,
            wrap_arrow_key_navigation: false,
            terminal: Terminal::default().expect("Failed to initialize terminal"),
            cursor_position: Position::default(),
            document,
            offset: Position::default(),
            status_message: StatusMessage::from(initial_status),
            sound_manager: SoundManager::new(),
        }
    }

    fn refresh_screen(&mut self) -> Result<(), std::io::Error> {
        if !self.should_draw_ui {
            return Terminal::flush();
        }
        Terminal::cursor_hide();
        Terminal::cursor_position(&Position { x: 0, y: 0 });
        if self.should_quit == QuitStatus::Quitting {
            Terminal::clear_screen();
        } else {
            self.draw_rows();
            self.draw_status_bar();
            self.draw_message_bar();
            Terminal::cursor_position(&Position {
                x: self.cursor_position.x.saturating_sub(self.offset.x),
                y: self.cursor_position.y.saturating_sub(self.offset.y),
            });
        }
        Terminal::cursor_show();
        Terminal::flush()
    }
    fn process_keypress(&mut self) -> Result<bool, std::io::Error> {
        // TODO: Modal editing.
        let pressed_key = Terminal::read_key()?;
        match pressed_key {
            Key::Ctrl('q') => {
                if self.document.is_dirty() && self.should_quit == QuitStatus::Default {
                    self.should_quit = QuitStatus::Confirming;
                    self.status_message = StatusMessage::from("Quit? (Ctrl-Q)".to_string());
                    self.sound_manager
                        .interrupt_and_play(Box::new(Utterance::from("Quit without saving?")));
                } else {
                    self.should_quit = QuitStatus::Quitting;
                    self.sound_manager
                        .interrupt_and_play(Box::new(Utterance::from("Goodbye!")));
                }
            }
            Key::Ctrl('s') => self.save(),

            Key::Alt(';') => {
                // Say the current location:
                self.sound_manager
                    .interrupt_and_play(Box::new(Utterance::from(
                        format!(
                            "Row {}, column {}",
                            self.cursor_position.y + 1,
                            self.cursor_position.x + 1
                        )
                        .as_str(),
                    )));
            }
            Key::Alt('l') => {
                // Say the current row.
                let default = &Row::from("");
                let row = self
                    .document
                    .get_row(self.cursor_position.y)
                    .unwrap_or(default);
                row.play_blocking(&mut self.sound_manager);
            }

            Key::Alt('.') => {
                // Spell the current word.
                let default = &Row::from("");
                let row = self
                    .document
                    .get_row(self.cursor_position.y)
                    .unwrap_or(default);
                let word = row.get_word_at(self.cursor_position.x).unwrap_or_default();
                // Add a space in between each letter.
                let letters_with_spaces = word
                    .chars()
                    .map(|c| format!("{}, ", c))
                    .collect::<Vec<String>>()
                    .join("");
                self.sound_manager
                    .play_and_wait(Box::new(Utterance::from(letters_with_spaces.as_str())));
            }

            Key::Char(c) => {
                if c == '\n' {
                    self.speak_current_row();
                } else if !c.is_alphanumeric() {
                    self.speak_current_word();
                }
                self.document.insert(&self.cursor_position, c);
                self.move_cursor(Key::Right);
            }

            // Deletion:
            Key::Delete => self.document.delete(&self.cursor_position),
            Key::Backspace => {
                if self.cursor_position.x > 0 || self.cursor_position.y > 0 {
                    self.move_cursor(Key::Left);
                    self.document.delete(&self.cursor_position);
                }
            }

            // TODO: Wordwise navigation.
            Key::Up
            | Key::Down
            | Key::Left
            | Key::Right
            | Key::PageUp
            | Key::PageDown
            | Key::End
            | Key::Home => self.move_cursor(pressed_key),

            _ => return Ok(false),
        }
        self.scroll();
        Ok(true)
    }

    fn change_mode(&mut self, mode: Mode) {
        // self.document.change_mode(mode);
        self.sound_manager
            .play_and_wait(Box::new(Tone::new(440.0, 0.06, 0.5)));
        self.sound_manager
            .play_and_wait(Box::new(Tone::new(440.0 * 3.0 / 2.0, 0.1, 0.5)));
    }

    fn speak_current_word(&mut self) {
        let default = &Row::from("");
        let row = self
            .document
            .get_row(self.cursor_position.y)
            .unwrap_or(default);
        let word = row
            .get_word_at(self.cursor_position.x.saturating_sub(1))
            .unwrap_or_default();
        self.sound_manager
            .play_and_wait(Box::new(Utterance::from(word)));
    }

    fn speak_current_row(&mut self) {
        let default = &Row::from("");
        let row = self
            .document
            .get_row(self.cursor_position.y)
            .unwrap_or(default);
        row.play(&mut self.sound_manager);
    }

    fn prompt(&mut self, prompt: &str) -> Result<Option<String>, std::io::Error> {
        let mut result = String::new();
        loop {
            self.status_message = StatusMessage::from(format!("{}{}", prompt, result));
            self.refresh_screen()?;
            match Terminal::read_key()? {
                Key::Backspace => result.truncate(result.len().saturating_sub(1)),
                Key::Char('\n') => break,
                Key::Char(c) => {
                    if !c.is_control() {
                        result.push(c);
                    }
                }
                Key::Esc => {
                    result.truncate(0);
                    break;
                }
                _ => (),
            }
        }
        self.status_message = StatusMessage::from(String::new());
        if result.is_empty() {
            return Ok(None);
        }
        Ok(Some(result))
    }

    fn save(&mut self) {
        if self.document.file_name.is_none() {
            self.sound_manager
                .play_and_wait(Box::new(Utterance::from("Save as ")));
            let new_name = self.prompt("Save as: ").unwrap_or(None);
            if new_name.is_none() {
                self.status_message = StatusMessage::from("Save aborted.".to_string());
                self.sound_manager
                    .interrupt_and_play(Box::new(Utterance::from("Save aborted.")));
                return;
            }
            self.document.file_name = new_name;
        }

        if self.document.save().is_ok() {
            self.sound_manager
                .interrupt_and_play(Box::new(Utterance::from("Saved. ")));

            self.status_message = StatusMessage::from("File saved successfully.".to_string());
            self.sound_manager
                .interrupt_and_play(Box::new(Utterance::from(
                    format!("Saved {}.", self.document.file_name.as_ref().unwrap()).as_str(),
                )));
        } else {
            self.status_message = StatusMessage::from("Error writing file!".to_string());
            self.sound_manager
                .interrupt_and_play(Box::new(Utterance::from("Error writing file!")));
        }
    }

    fn scroll(&mut self) {
        let Position { x, y } = self.cursor_position;
        let width = self.terminal.size().width as usize;
        let height = self.terminal.size().height as usize;
        let mut offset = &mut self.offset;
        if y < offset.y {
            offset.y = y;
        } else if y >= offset.y.saturating_add(height) {
            offset.y = y.saturating_sub(height).saturating_add(1);
        }
        if x < offset.x {
            offset.x = x;
        } else if x >= offset.x.saturating_add(width) {
            offset.x = x.saturating_sub(width).saturating_add(1);
        }
    }

    fn move_cursor(&mut self, key: Key) {
        let term_height = self.terminal.size().height as usize;
        let Position { mut y, mut x } = self.cursor_position;
        let starting_y = y;
        let height = self.document.row_count();
        let mut width = if let Some(row) = self.document.get_row(y) {
            row.len()
        } else {
            0
        };
        match key {
            Key::Up => {
                if y == 0 {
                    self.play_blocked_navigation_sound();
                }
                y = y.saturating_sub(1);
            }
            Key::Down => {
                if y < height {
                    y = y.saturating_add(1);
                }
            }
            Key::Left => {
                if x > 0 {
                    x -= 1;
                } else if y > 0 && self.wrap_arrow_key_navigation {
                    y -= 1;
                    if let Some(row) = self.document.get_row(y) {
                        x = row.len();
                    } else {
                        x = 0;
                    }
                } else {
                    self.play_blocked_navigation_sound();
                }
            }
            Key::Right => {
                if x < width {
                    x += 1;
                } else if y < height && self.wrap_arrow_key_navigation {
                    y += 1;
                    x = 0;
                } else {
                    self.play_blocked_navigation_sound();
                }
            }
            Key::PageUp => {
                y = if y > term_height {
                    y.saturating_sub(term_height)
                } else {
                    0
                }
            }
            Key::PageDown => {
                y = if y.saturating_add(term_height) < height {
                    y.saturating_add(term_height)
                } else {
                    height
                }
            }
            Key::Home => x = 0,
            Key::End => x = width,
            _ => (),
        }
        width = if let Some(row) = self.document.get_row(y) {
            row.len()
        } else {
            0
        };
        if x > width {
            x = width;
        }

        let ending_y = y;
        self.cursor_position = Position { x, y };
        if starting_y != ending_y {
            self.speak_current_row()
        }
    }

    fn play_blocked_navigation_sound(&mut self) {
        self.sound_manager.play_and_wait(Box::new(Tone {
            frequency: 440.0,
            duration: 0.2,
            volume: 0.5,
        }));
        // self.sound_manager.play(Box::new(Utterance::from("no.")));
    }

    fn draw_welcome_message(&self) {
        let mut welcome_message = format!("clack {}", VERSION);
        let width = self.terminal.size().width as usize;
        let len = welcome_message.len();
        let padding = width.saturating_sub(len) / 2;
        let spaces = " ".repeat(padding.saturating_sub(1));
        welcome_message = format!("~{}{}", spaces, welcome_message);
        welcome_message.truncate(width);
        println!("{}\r", welcome_message);
    }

    fn draw_rows(&self) {
        let height = self.terminal.size().height;
        for terminal_row in 0..height {
            Terminal::clear_current_line();
            if let Some(row) = self
                .document
                .get_row(self.offset.y.saturating_add(terminal_row.into()))
            {
                self.draw_row(row);
            } else if self.document.row_count() == 0 && terminal_row == height / 3 {
                self.draw_welcome_message();
            } else {
                println!("~\r");
            }
        }
    }

    fn draw_row(&self, row: &Row) {
        let width = self.terminal.size().width as usize;
        let start = self.offset.x;
        let end = self.offset.x.saturating_add(width);
        println!("{}\r", row.render(start, end))
    }

    fn draw_status_bar(&self) {
        let mut status;
        let width = self.terminal.size().width as usize;
        let modified_indicator = if self.document.is_dirty() { "*" } else { "" };
        let mut file_name = "[No Name]".to_string();
        if let Some(name) = &self.document.file_name {
            file_name = name.clone();
            file_name.truncate(20);
        }
        status = format!(
            "{} - {} lines{}",
            file_name,
            self.document.row_count(),
            modified_indicator
        );
        let line_indicator = format!(
            "{}/{}",
            self.cursor_position.y.saturating_add(1),
            self.document.row_count()
        );
        let len = status.len() + line_indicator.len();
        status.push_str(&" ".repeat(width.saturating_sub(len)));
        status = format!("{}{}", status, line_indicator);
        Terminal::set_bg_color(STATUS_BG_COLOR);
        Terminal::set_fg_color(STATUS_FG_COLOR);
        println!("{}\r", status);
        Terminal::reset_bg_color();
        Terminal::reset_fg_color();
    }

    fn draw_message_bar(&self) {
        Terminal::clear_current_line();
        let message = &self.status_message;
        if Instant::now() - message.time < Duration::new(5, 0) {
            let mut text = message.text.clone();
            text.truncate(self.terminal.size().width as usize);
            print!("{}", text);
        }
    }
}

fn die(e: std::io::Error) {
    Terminal::clear_screen();
    panic!("{}", e);
}
