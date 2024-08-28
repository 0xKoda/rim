use std::fs::{File, OpenOptions};
use std::io::{self, BufRead, BufReader, Write};
use std::path::Path;
use std::time::Duration;
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::screen::AlternateScreen;
use termion::color;
use termion::screen::IntoAlternateScreen;


enum Mode {
    Normal,
    Insert,
    Command,
}

struct Editor {
    lines: Vec<String>,
    cursor: (usize, usize),
    mode: Mode,
    file_path: String,
    status_message: String,
    scroll_offset: usize,
}

impl Editor {
    fn new(file_path: &str) -> io::Result<Self> {
        let path = Path::new(file_path);
        let lines = if path.exists() {
            let file = File::open(path)?;
            BufReader::new(file).lines().collect::<io::Result<Vec<String>>>()?
        } else {
            vec![String::new()]
        };

        Ok(Editor {
            lines,
            cursor: (0, 0),
            mode: Mode::Normal,
            file_path: file_path.to_string(),
            status_message: String::new(),
            scroll_offset: 0,
        })
    }

    fn run(&mut self) -> io::Result<()> {
        let stdout = io::stdout().into_raw_mode()?;
        let mut screen = stdout.into_alternate_screen()?;
        let mut stdin = termion::async_stdin().keys();

        self.display(&mut screen)?;

        loop {
            if let Some(Ok(key)) = stdin.next() {
                if self.handle_key(key)? {
                    break;
                }
                self.display(&mut screen)?;
            }
            std::thread::sleep(Duration::from_millis(10));
        }

        write!(screen, "{}", termion::cursor::Show)?;
        Ok(())
    }

    fn display(&self, screen: &mut AlternateScreen<termion::raw::RawTerminal<io::Stdout>>) -> io::Result<()> {
        write!(screen, "{}{}", termion::clear::All, termion::cursor::Goto(1, 1))?;

        let (width, height) = termion::terminal_size()?;
        let visible_lines = (height - 2) as usize;
        let line_number_width = 4;
        let content_width = width as usize - line_number_width - 3; // 3 for the separator and padding

        for (i, line) in self.lines.iter().enumerate().skip(self.scroll_offset).take(visible_lines) {
            // Line number
            write!(screen, "{}", termion::cursor::Goto(1, (i - self.scroll_offset + 1) as u16))?;
            write!(screen, "{}{:>4} │ ", color::Fg(color::LightBlue), i + 1)?;

            // Line content
            write!(screen, "{}", color::Fg(color::Reset))?;
            if line.len() > content_width {
                writeln!(screen, "{}...", &line[..content_width - 3])?;
            } else {
                writeln!(screen, "{}", line)?;
            }
        }

        self.draw_status_bar(screen)?;

        // Update cursor position
        let cursor_y = (self.cursor.0 - self.scroll_offset + 1) as u16;
        let cursor_x = (self.cursor.1 + line_number_width + 3) as u16;
        write!(screen, "{}{}", termion::cursor::Goto(cursor_x, cursor_y), termion::cursor::Show)?;

        screen.flush()?;
        Ok(())
    }

    fn draw_status_bar(&self, screen: &mut AlternateScreen<termion::raw::RawTerminal<io::Stdout>>) -> io::Result<()> {
        let (width, height) = termion::terminal_size()?;
        write!(
            screen,
            "{}{}{}",
            termion::cursor::Goto(1, height - 1),
            color::Bg(color::Blue),
            " ".repeat(width as usize)
        )?;

        write!(
            screen,
            "{}{}-- {} -- {}:{} --{}{}{}",
            termion::cursor::Goto(1, height),
            color::Fg(color::White),
            match self.mode {
                Mode::Normal => "NORMAL",
                Mode::Insert => "INSERT",
                Mode::Command => "COMMAND",
            },
            self.cursor.0 + 1,
            self.cursor.1 + 1,
            self.status_message,
            color::Fg(color::Reset),
            color::Bg(color::Reset)
        )?;
        Ok(())
    }

    fn handle_key(&mut self, key: Key) -> io::Result<bool> {
        match self.mode {
            Mode::Normal => match key {
                Key::Char('q') => return Ok(true),
                Key::Char('i') => self.mode = Mode::Insert,
                Key::Char(':') => {
                    self.mode = Mode::Command;
                    self.status_message.clear();
                },
                Key::Up => self.move_cursor_up(),
                Key::Down => self.move_cursor_down(),
                Key::Left => self.move_cursor_left(),
                Key::Right => self.move_cursor_right(),
                _ => {}
            },
            Mode::Insert => match key {
                Key::Esc => self.mode = Mode::Normal,
                Key::Char('\n') => self.insert_newline(),
                Key::Char(c) => self.insert_char(c),
                Key::Backspace => self.delete_char(),
                Key::Up => self.move_cursor_up(),
                Key::Down => self.move_cursor_down(),
                Key::Left => self.move_cursor_left(),
                Key::Right => self.move_cursor_right(),
                _ => {}
            },
            Mode::Command => match key {
                Key::Char('\n') => return self.execute_command(),
                Key::Esc => {
                    self.mode = Mode::Normal;
                    self.status_message.clear();
                }
                Key::Char(c) => self.status_message.push(c),
                Key::Backspace => { self.status_message.pop(); }
                _ => {}
            },
        }
        Ok(false)
    }

    fn move_cursor_up(&mut self) {
        if self.cursor.0 > 0 {
            self.cursor.0 -= 1;
            self.cursor.1 = self.cursor.1.min(self.lines[self.cursor.0].len());
            if self.cursor.0 < self.scroll_offset {
                self.scroll_offset = self.cursor.0;
            }
        }
    }

    fn move_cursor_down(&mut self) {
        if self.cursor.0 < self.lines.len() - 1 {
            self.cursor.0 += 1;
            self.cursor.1 = self.cursor.1.min(self.lines[self.cursor.0].len());
            let (_, height) = termion::terminal_size().unwrap();
            if self.cursor.0 >= self.scroll_offset + height as usize - 3 {
                self.scroll_offset = self.cursor.0.saturating_sub(height as usize - 3);
            }
        }
    }

    fn move_cursor_left(&mut self) {
        if self.cursor.1 > 0 {
            self.cursor.1 -= 1;
        } else if self.cursor.0 > 0 {
            self.cursor.0 -= 1;
            self.cursor.1 = self.lines[self.cursor.0].len();
        }
    }

    fn move_cursor_right(&mut self) {
        if self.cursor.1 < self.lines[self.cursor.0].len() {
            self.cursor.1 += 1;
        } else if self.cursor.0 < self.lines.len() - 1 {
            self.cursor.0 += 1;
            self.cursor.1 = 0;
        }
    }

    fn execute_command(&mut self) -> io::Result<bool> {
        match self.status_message.as_str() {
            "w" => self.save()?,
            "q" => return Ok(true),
            "wq" => {
                self.save()?;
                return Ok(true);
            }
            _ => self.status_message = "Invalid command".to_string(),
        }
        self.mode = Mode::Normal;
        self.status_message.clear();
        Ok(false)
    }

    fn insert_char(&mut self, c: char) {
        let line = &mut self.lines[self.cursor.0];
        line.insert(self.cursor.1, c);
        self.cursor.1 += 1;
    }

    fn insert_newline(&mut self) {
        let new_line = self.lines[self.cursor.0][self.cursor.1..].to_string();
        self.lines[self.cursor.0].truncate(self.cursor.1);
        self.cursor.0 += 1;
        self.lines.insert(self.cursor.0, new_line);
        self.cursor.1 = 0;
    }

    fn delete_char(&mut self) {
        if self.cursor.1 > 0 {
            let line = &mut self.lines[self.cursor.0];
            line.remove(self.cursor.1 - 1);
            self.cursor.1 -= 1;
        } else if self.cursor.0 > 0 {
            let current_line = self.lines.remove(self.cursor.0);
            self.cursor.0 -= 1;
            self.cursor.1 = self.lines[self.cursor.0].len();
            self.lines[self.cursor.0].push_str(&current_line);
        }
    }

    fn save(&mut self) -> io::Result<()> {
        let mut file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(&self.file_path)?;

        for line in &self.lines {
            writeln!(file, "{}", line)?;
        }
        self.status_message = "File saved".to_string();
        Ok(())
    }
}

fn main() -> io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        println!("Usage: {} <file_path>", args[0]);
        return Ok(());
    }

    let mut editor = Editor::new(&args[1])?;
    editor.run()
}