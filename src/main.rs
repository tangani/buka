use crossterm::{event::*, cursor};
// use crossterm::terminal::ClearType;
use crossterm::{event, execute, terminal};
// use crossterm::event::{Event, KeyCode, KeyEvent};
use std::io::stdout;
use std::time::Duration;

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

    fn process_key_press(&self) -> crossterm::Result<bool> {
        match self.reader.read_key()? {
            KeyEvent {
                code: KeyCode::Char('z'),
                modifiers: event::KeyModifiers::CONTROL,
            } => return Ok(false),
            _ => {}
        }
        Ok(true)
    }

    fn run(&self) -> crossterm::Result<bool> {
        self.output.refresh_screen()?;
        self.process_key_press()
    }
}


struct Output;

impl Output {
    fn new() -> Self {
        Self
    }

    fn clear_screen() -> crossterm::Result<()> {
        execute!(stdout(), terminal::Clear(terminal::ClearType::All))?;
        execute!(stdout(), cursor::MoveTo(0, 0))
    }

    fn refresh_screen(&self) -> crossterm::Result<()> {
        Self::clear_screen()
    }
}


fn main() -> crossterm::Result<()> {

    let _clean_up: CleanUp = CleanUp;
    terminal::enable_raw_mode()?;
    let editor: Editor = Editor::new();
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
