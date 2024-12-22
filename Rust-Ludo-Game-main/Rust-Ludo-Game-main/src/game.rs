mod player;

use std::io::{stdin, stdout, Write};

use crate::display::Display;
use crate::engine::MoveResult;
use crate::engine::{dice::roll, LudoEngine, Rang};

use crossterm::style::Color;
use player::Player;

pub struct LudoGame {
    engine: LudoEngine, // actual logic
    display: Display,
    active_players: Vec<Player>, // order matters !
}

impl LudoGame {
    pub fn new() -> Self {
        let display = Display::new();
        let mut active_players = Vec::new();
        let mut active_colours = Vec::new();

        let player_names = display.get_player_names();
        let colors = [Rang::Red, Rang::Green, Rang::Yellow, Rang::Blue];

        for (i, name) in player_names.iter().enumerate() {
            if !name.is_empty() {
                active_colours.push(colors[i]);
            }
        }

        let engine = LudoEngine::new(active_colours);

        for (i, name) in player_names.iter().enumerate() {
            if !name.is_empty() {
                active_players.push(Player {
                    name: name.clone(),
                    colour: colors[i],
                })
            }
        }

        if active_players.is_empty() {
            Display::splash_screen("No players entered", Some(Color::Red));
            std::thread::sleep(std::time::Duration::from_secs(10));
            panic!("No players entered");
        }

        LudoGame {
            active_players,
            display,
            engine,
        }
    }

    fn update_display(&self) {
        // `display` component requires this
        let mut display_content = Vec::new();
        let board = &self.engine.get_board();

        for (i, row) in board.iter().enumerate() {
            for (j, cell) in row.iter().enumerate() {
                if !cell.gotis.is_empty() {
                    // Invariant: Assuming all gotis in one cell, even if multiple, are of same color
                    let mut content = match cell.gotis[0].borrow().colour {
                        Rang::Red => '🔴',
                        Rang::Green => '🟢',
                        Rang::Yellow => '🟡',
                        Rang::Blue => '🔵',
                    }
                    .to_string();

                    if cell.gotis.len() > 1 {
                        content.push_str(&cell.gotis.len().to_string())
                    }

                    display_content.push(((i as u8, j as u8), content));

                    // Note: Not handling case of multiple gotis of different colors, in same cell, eg. "RG", "RGRB" which should be shown as "R2GB"
                }
            }
        }

        for player in self.active_players.iter() {
            if self.engine.is_finished(player.colour) {
                display_content.push((Rang::GetEndCoord(player.colour), "👑".to_string()));
            }
        }

        self.display.update_display(display_content.clone());
    }

    pub fn play(&mut self) {
        self.update_display();

        // Will be updated at end of each iteration
        let mut player_index = 0;
        let mut same_player_next_chance;
        loop {
            // for player in self.active_players.iter() {
            same_player_next_chance = false; // may later be modified, if for eg. '6', or finishes etc.

            let player = &self.active_players[player_index];

            if self.engine.is_game_finished() {
                break;
            } else if self.engine.is_finished(player.colour) {
                continue;
            }

            self.engine.set_current_colour(player.colour);
            self.display.set_player(&player.name);
            self.update_display();

            print!("Press Enter to Roll: ");
            stdout().flush().unwrap();
            // ignore input till Enter
            let mut ignore_buf = String::new();
            stdin().read_line(&mut ignore_buf).unwrap();
            ignore_buf.clear();

            let roll = roll();

            println!("Roll Output - {:?}", roll);

            if roll == 6 {
                same_player_next_chance = true;
            }

            let movable_gotis = self.engine.get_movable_gotis(player.colour, roll);

            if (roll == 6 && self.engine.get_num_locked(player.colour).unwrap() > 0)
                || !movable_gotis.is_empty()
            {
                println!("Chose from these options: ");

                let mut i = 0;
                if roll == 6 && self.engine.get_num_locked(player.colour).unwrap() > 0 {
                    println!("0. Unlock New Goti (just type 0)");
                    i += 1;
                }

                for c in movable_gotis.iter() {
                    println!("{}. [{}][{}]", i, c.0, c.1);
                    i += 1;
                }

                let mut input = String::new();
                stdin().read_line(&mut input).expect("Failed to read input");

                let trimmed = input.trim();
                let mut chosen_option = match trimmed.parse::<u8>() {
                    Ok(i) => i,
                    Err(_) => {
                        println!("Not a option: {:?}", trimmed);
                        println!("Repeating...");
                        // Probable bug: Dice output wont be same next time

                        std::thread::sleep(std::time::Duration::from_secs(1));
                        continue;
                    }
                };

                let mut skip_rest = true;

                if roll == 6 && self.engine.get_num_locked(player.colour).unwrap() > 0 {
                    same_player_next_chance = true;

                    if chosen_option == 0 {
                        self.engine.unlock_goti(player.colour)
                            .expect("No Goti to unlock... this is a bug, please report at https://github.com/ludo-game-self.engine/issues");
                    } else {
                        chosen_option -= 1;
                        skip_rest = false;
                    }
                } else {
                    skip_rest = false;
                }

                if !skip_rest {
                    // Choice is one of `movable_gotis`
                    match movable_gotis.get(chosen_option as usize) {
                        Some(start_coord) => {
                            let result = self.engine.move_goti(player.colour, *start_coord, roll)
                                        .expect("Could not move, although .get_movable_gotis() said i can :(...  this is a bug, please report at https://github.com/ludo-game-self.engine/issues");

                            match result {
                                MoveResult::Attacked(_)
                                | MoveResult::Finished
                                | MoveResult::Unlocked => {
                                    same_player_next_chance = true;
                                }
                                MoveResult::NormalMove(_) => {}
                            };
                        }
                        None => {
                            println!("Invalid choice: {:?}", chosen_option);
                        }
                    }
                }
            } else {
                println!("No possible moves...");
            }

            if same_player_next_chance == false {
                // Next player
                player_index = (player_index + 1) % self.active_players.len();
            }

            std::thread::sleep(std::time::Duration::from_secs(1));
        }

        // !TODO
    }
}

impl Drop for LudoGame {
    fn drop(&mut self) {
        self.display.end_display();
    }
}
