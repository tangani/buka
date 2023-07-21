use crossterm::terminal::ClearType;
use crossterm::{event::*, cursor};
use crossterm::{event, execute, queue, terminal};
use std::io;
use std::path::Path;
use std::{cmp, env, fs, result};
use std::io::{ErrorKind, Result, stdout, Write};
use std::time::Duration;


const VERSION: &str = "1.0.0";

struct CleanUp;

impl Drop for CleanUp {
    fn drop(&mut self) {
        terminal::disable_raw_mode().expect("Unable to disable Raw mode");
        Output::clear_screen().expect("Error");
    }
}


struct Reader;


impl Reader {
    fn read_key(&self) -> crossterm::Result<KeyEvent> {
        loop {
            if event::poll(Duration::from_millis(500))? {
                if let Event::Key(event) = event::read()? {
                    return Ok(event);
                }
            }
        }
    }
}


struct Editor {
    reader: Reader,
    output: Output,
}


impl Editor {
    fn new() -> Self {
        Self {
            reader: Reader,
            output: Output::new(),
        }
    }

    fn key_press(&mut self) -> crossterm::Result<bool> {
        match self.reader.read_key()? {
            KeyEvent {
                code: KeyCode::Char('z'),
                modifiers: event::KeyModifiers::CONTROL,
            } => return Ok(false),
            KeyEvent {
                code: direction @ (
                    KeyCode::Up
                    | KeyCode::Down
                    | KeyCode::Left
                    | KeyCode::Right
                    | KeyCode::Home
                    | KeyCode::End
                ),
                modifiers: KeyModifiers::NONE,
            } => self.output.move_cursor(direction),
            KeyEvent {
                code: val @ (KeyCode::PageUp | KeyCode::PageDown),
                modifiers: KeyModifiers::NONE
            } => (0..self.output.win_size.1).for_each(|_| {
                self.output.move_cursor(if matches!(val, KeyCode::PageUp) {
                    KeyCode::Up
                } else {
                    KeyCode::Down
                });
            }),
            _ => unimplemented!()
        }
        Ok(true)
    }

    fn run(&mut self) -> crossterm::Result<bool> {
        self.output.refresh_screen()?;
        self.key_press()
    }
}


struct Output {
    win_size: (usize, usize),
    editor_contents: EditorContents,
    editor_rows: EditorRows,
    cursor_controller: CursorController,
}

impl Output {
    fn new() -> Self {
        let win_size: (usize, usize) = terminal::size()
            .map(|(x, y)| (x as usize, y as usize))
            .unwrap();
        Self {
            win_size,
            editor_contents: EditorContents::new(),
            editor_rows: EditorRows::new(),
            cursor_controller: CursorController::new(win_size),
        }
    }

    fn clear_screen() -> crossterm::Result<()> {
        execute!(stdout(), terminal::Clear(terminal::ClearType::All))?;
        execute!(stdout(), cursor::MoveTo(0, 0))
    }

    fn draw_rows(&mut self) {
        let screen_rows: usize = self.win_size.1;
        let screen_columns: usize = self.win_size.0;
        for i in 0..screen_rows {
            let file_row = i + self.cursor_controller.row_offset;
            if file_row >= self.editor_rows.number_of_rows() {
                if self.editor_rows.number_of_rows() == 0 && i == screen_rows / 3 {
                    let mut welcome = format!("Buka editor - VERSION: {}", VERSION);
                    if welcome.len() > screen_columns {
                        welcome.truncate(screen_columns);
                    }
                    let mut padding = (screen_columns - welcome.len()) / 2;
                    if padding != 0 {
                        self.editor_contents.push('~');
                        padding -= 1;
                    }
                    (0..padding).for_each(|_| self.editor_contents.push(' '));
                    self.editor_contents.push_str(&welcome);
                } else {
                    self.editor_contents.push('~');
                }
            } else {
                let len = cmp::min(self.editor_rows.get_row(file_row).len(), screen_columns);
                self.editor_contents
                    .push_str(&self.editor_rows.get_row(file_row)[..len])
            }
            queue!(
                self.editor_contents,
                terminal::Clear(ClearType::UntilNewLine)
            )
            .unwrap();
            if i < screen_rows - 1 {
                self.editor_contents.push_str("\r\n");
            }
        }
    }

    fn refresh_screen(&mut self) -> crossterm::Result<()> {
        self.cursor_controller.scroll();
        queue!(
            self.editor_contents,
            cursor::Hide,
            cursor::MoveTo(0, 0)
        )?;
        self.draw_rows();
        let cursor_x = self.cursor_controller.cursor_x;
        let cursor_y = self.cursor_controller.cursor_y - self.cursor_controller.row_offset;
        queue!(
            self.editor_contents,
            cursor::MoveTo(cursor_x as u16, cursor_y as u16),
            cursor::Show
        )?;
        self.editor_contents.flush()
    }

    fn move_cursor(&mut self, direction: KeyCode) {
        self.cursor_controller.move_cursor(direction, self.editor_rows.number_of_rows());
    }
}


struct EditorContents {
    content: String,
}


impl EditorContents {
    fn new() -> Self {
        Self { content: String::new(), }
    }

    fn push (&mut self, ch: char) {
        self.content.push(ch)
    }

    fn push_str(&mut self, string: &str) {
        self.content.push_str(string)
    }
}


impl io::Write for EditorContents {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        match std::str::from_utf8(buf) {
            Ok(s) => {
                self.content.push_str(s);
                Ok(s.len())
            }
            Err(_) => Err(ErrorKind::WriteZero.into()),
        }
    }

    fn flush(&mut self) -> Result<()> {
        let out: result::Result<(), io::Error> = write!(stdout(), "{}", self.content);
        stdout().flush()?;
        self.content.clear();
        out
    }
}


struct CursorController {
    cursor_x: usize,
    cursor_y: usize,
    screen_columns: usize,
    screen_rows: usize,
    row_offset: usize,
    // columns_offset: usize,
}


impl  CursorController {
    fn new(win_size: (usize, usize)) -> CursorController {
        Self {
            cursor_x: 0,
            cursor_y: 0,
            screen_columns: win_size.0,
            screen_rows: win_size.1,
            row_offset: 0,
            // columns_offset: 0,
        }
    }

    fn move_cursor(&mut self, direction: KeyCode, number_of_rows: usize) {
        match direction {
            KeyCode::Up => {
                // self.cursor_y = self.cursor_y.saturating_sub(1)
                if self.cursor_y != 0 {
                    self.cursor_y -= 1;
                }
            }
            KeyCode::Left => {
                if self.cursor_x != 0 {
                    self.cursor_x -= 1;
                }
            }
            KeyCode::Down => {
                if self.cursor_y != number_of_rows {
                    self.cursor_y += 1;
                }
            }
            KeyCode::Right => {
                if self.cursor_x != self.screen_columns -1 {
                    self.cursor_x += 1;
                }
            }
            KeyCode::End => self.cursor_x = self.screen_columns - 1,
            KeyCode::Home => self.cursor_x = 0,
            KeyCode::PageUp => self.cursor_y = 0,
            KeyCode::PageDown => self.cursor_y = self.screen_rows - 1,
            _ => unimplemented!(),
        }
    }

    fn scroll(&mut self) {
        self.row_offset = cmp::min(self.row_offset, self.cursor_y);
        if self.cursor_y >= self.row_offset + self.screen_rows {
            self.row_offset = self.cursor_y - self.screen_rows + 1;
        }
    }
}



struct EditorRows {
    row_contents: Vec<Box<str>>
}


impl EditorRows {
    fn new() -> Self {
        let mut arg: env::Args = env::args();

        match arg.nth(1) {
            None => Self {
                row_contents: Vec::new(),
            },
            Some(file) => Self::from_file(file.as_ref()),
        }
    }

    fn from_file(file: &Path) -> Self {
        let file_contents = fs::read_to_string(file).expect("Unable to read file");
        Self {
            row_contents: file_contents.lines().map(|it| it.into()).collect(),
        }
    }

    fn number_of_rows(&self) -> usize {
        self.row_contents.len()
    }

    fn get_row(&self, at: usize) -> &str {
        &self.row_contents[at]
    }
}


fn main() -> crossterm::Result<()> {

    let _clean_up: CleanUp = CleanUp;
    terminal::enable_raw_mode()?;
    let mut editor: Editor = Editor::new();
    while editor.run()? {}

    // loop {
    //     if event::poll(Duration::from_millis(500)).expect("Error") {
    //         if let Event::Key(event) = event::read()? {
    //             match event {
    //                 KeyEvent {
    //                     code: KeyCode::Char('z'),
    //                     modifiers: event::KeyModifiers::CONTROL,
    //                 } => break,
    //                 _ => {
    //                     // todo!()
    //                 }
    //             }
    //             println!("{:?}\r", event);
    //         }
    //     } else {
    //         println!("No input yet\r");
    //     }
    // }

    Ok(())
}
