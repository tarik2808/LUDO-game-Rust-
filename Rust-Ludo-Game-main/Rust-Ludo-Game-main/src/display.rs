use crossterm::{
    self, cursor,
    style::{self, Color, Stylize},
    terminal, ExecutableCommand, QueueableCommand,
};
use std::{io::{stdout, Write, stdin}, thread, time::Duration};

mod parts;  

pub struct Display {
    player_name: String,
}

impl Display {
    pub fn new() -> Self {
        let display = Display {
            player_name: String::new(),
        };

        stdout().execute(terminal::SetTitle("Ludo-The_Game")).unwrap();
        //Display::splash_screen("This is the Ludo game", None);
        thread::sleep(Duration::from_millis(1200));

        display
    }

    pub fn get_player_names(&self) -> [String; 4] {
        let hor_char = "─";
        let vert_char = "│";

        let message = "Enter names of the Players (Leave empty if not playing)";
        let mut names = [String::new(), String::new(), String::new(), String::new()];
        let colors = ["🔴","🟢","🟡","🔵"];

        let (columns, rows) = terminal::size().unwrap();
        let original_position = cursor::position().unwrap();

        let mut stdout = stdout();

        self.header();

        stdout
            .queue(cursor::MoveTo(1,3)).unwrap()
            .queue(style::Print(format!("┌{}┐", hor_char.repeat(columns as usize -3)))).unwrap();

        for i in 4..rows {
            stdout
                .queue(cursor::MoveTo(1,i)).unwrap()
                .queue(style::Print(vert_char)).unwrap()
                .queue(cursor::MoveToColumn(columns)).unwrap()
                .queue(style::Print(vert_char)).unwrap()
                .queue(cursor::MoveToNextLine(1)).unwrap()
                ;
        }

        stdout
            .queue(cursor::MoveTo(1,rows-1)).unwrap()
            .queue(style::Print(format!("└{}┘", hor_char.repeat(columns as usize -3)))).unwrap()
            ;

        stdout
            .queue(cursor::MoveTo(((columns as usize - message.len()) as u16)/2, (rows - 5)/2)).unwrap();

        // Print Message
        stdout.queue(style::PrintStyledContent(message.bold())).unwrap();

        for (i,name) in names.iter_mut().enumerate() {
            stdout
                .queue(cursor::MoveToNextLine(1)).unwrap()
                .queue(cursor::MoveToColumn((columns/2) - "Player".len() as u16 -4 -1 )).unwrap()
                .queue(style::Print(format!("{} Player{} : ", colors[i], i+1))).unwrap()
                ;
            stdout.flush().unwrap();

            if stdin().read_line(name).is_err() {
                panic!("Failed to read name");
            }

            // Equivalent to `name = name.trim()`
            name.clone_from(&name.trim().to_string());

            stdout.queue(cursor::MoveToPreviousLine(1)).unwrap();   // It goes onto next line due to 'Enter' pressed by user
        }

        // Restore Cursor Position
        stdout.queue(cursor::MoveTo(original_position.0, original_position.1)).unwrap();
        stdout.flush().unwrap();

        names
    }

    pub fn ensure_terminal_size() {
        let (mut curr_cols, mut curr_rows) = terminal::size().unwrap();

        // Min 44 rows
        // Min 100 columns
        while curr_cols < 100 || curr_rows < 44 {
            Display::splash_screen("Please Zoom Out or Stretch to make the terminal window larger.", Some(Color::Red));

            thread::sleep(Duration::from_millis(50));
            (curr_cols, curr_rows) = terminal::size().unwrap();
        }
    }

    pub fn set_player(&mut self, name: &str) {
        self.player_name = name.to_string();
    }

    // This way, all 3 components: game, engine & display are separate
    pub fn update_display(&self, board_contents: Vec<((u8,u8), String)> ) {
        Display::ensure_terminal_size();
    
        let player_name = &self.player_name;
    
        let mut stdout = stdout();
    
        stdout
            .queue(terminal::Clear(terminal::ClearType::All))
            .unwrap()
            .queue(cursor::Hide)
            .unwrap();
    
        let (columns, _rows) = match terminal::size() {
            Ok(size) => (size.0 as usize, size.1 as usize),
            Err(e) => panic!("{:?}", e),
        };
    
        let h_scale: u16 = 3;
        let v_scale: u16 = 1;
    
        self.header();
        let ((board_start_col, board_start_row),(board_end_col, board_end_row))
            = self.board_design(h_scale, v_scale);
        self.update_according_to_ludo_board(board_start_col, board_start_row, h_scale, v_scale, board_contents);
    
        // Print player name at the top left corner
        stdout
            .queue(cursor::MoveTo(1, 1)).unwrap()
            .queue(style::Print(player_name))
            .unwrap();
    
        // Move to the next line
        stdout.queue(cursor::MoveToNextLine(1)).unwrap();
    
        // Print a separator line
        stdout.queue(style::Print("±".repeat(columns))).unwrap();
    
        stdout
            .queue(cursor::MoveTo(board_end_col, board_end_row)).unwrap();
    
        if stdout.flush().is_err() {
            terminal::disable_raw_mode().unwrap();
            panic!("Couldn't print board");
        }
    
        stdout.execute(cursor::Show).unwrap();
    }
    

    pub fn end_display(&mut self) {
        if terminal::disable_raw_mode().is_err() { /* Ignore */ };
    }
}