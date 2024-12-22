use array_init::array_init;
use std::{cell::RefCell, collections::BTreeMap as Map, rc::Rc};

mod cell;
pub mod dice;
mod goti;
mod rang;

use self::{
    cell::{LudoCell as Box, LudoCellType},
    goti::LudoGoti,
};
pub use rang::Rang; 

#[derive(Debug, PartialEq, Eq)]
pub enum MoveResult {
    NormalMove((u8, u8)), // normal move
    Attacked((u8, u8)),   // attacked a goti already present there
    Unlocked,             // goti was unlocked
    Finished,             // goti finished move
}

pub struct LudoEngine {
    board: [[Box; 15]; 15],

    // Note1: Whenever needs to share goti, .clone() the Rc, else RefCell will be in shared state and .borrow_mut() may panic
    // Note2: ONLY use .borrow_mut(), at the moment you are making a change, else a non-mutable function call that tries to .borrow() may panic
    // Why Rc<RefCell<>> ? Because, RefCell can't be cloned (I dont want to derive Clone on LudoGoti)
    moving_gotis: Map<Rang, Vec<Rc<RefCell<LudoGoti>>>>,
    locked_gotis: Map<Rang, Vec<Rc<RefCell<LudoGoti>>>>,
    num_finished: Map<Rang, u8>,
    active_colours: Vec<Rang>,
    curr_colour: Rang,
}

impl LudoEngine {
    pub fn new(active_colours: Vec<Rang>) -> Self {
        if active_colours.is_empty() {
            panic!("No active colours to play 🥲");
        }

        
        let mut board: [[Box; 15]; 15] = array_init(|_| {
            array_init(|_| Box {
                cell_type: LudoCellType::NoUse,
                gotis: Vec::new(),
            })
        });

        // Order Matters: For eg. Default also marks some as SafeSpots
        // Defaults (ie. usual Path box)
        {
            for r in 6..9 {
                for c in 0..15 {
                    board[r][c].cell_type = LudoCellType::Default;
                    board[c][r].cell_type = LudoCellType::Default;
                }
            }

            // Mark middle square as NoUse again
            for r in 6..9 {
                for c in 6..9 {
                    board[r][c].cell_type = LudoCellType::NoUse;
                }
            }
        }

        // Safe Spots
        for (r, c) in [
            (1, 8),
            (2, 6),
            (6, 1),
            (6, 12),
            (8, 2),
            (8, 13),
            (12, 8),
            (13, 6),
        ] {
            board[r][c].cell_type = LudoCellType::SafeSpot;
        }

        let mut locked_gotis = Map::new();
        // Locked Locations:
        for colour in active_colours.iter() {
            let colour = *colour;
            locked_gotis.insert(colour, Vec::new());

            for (r, c) in Rang::GetLockedPositions(colour) {
                board[r as usize][c as usize].cell_type = LudoCellType::LockedPosition(colour);

                let goti_ref = Rc::new(RefCell::new(LudoGoti {
                    colour,
                    coords: (r, c),
                }));

                locked_gotis
                    .get_mut(&colour)
                    .unwrap()
                    .push(goti_ref.clone());

                // board should know also
                board[r as usize][c as usize].gotis.push(goti_ref);
            }
        }

        // Home Lane (the ending path to finish)
        {
            // Red
            for r in 9..14 {
                board[r][7].cell_type = LudoCellType::HomeLane(Rang::Red);
            }

            // Green
            for c in 1..6 {
                board[7][c].cell_type = LudoCellType::HomeLane(Rang::Green);
            }

            // Yellow
            for r in 1..6 {
                board[r][7].cell_type = LudoCellType::HomeLane(Rang::Yellow);
            }

            // Blue
            for c in 9..14 {
                board[7][c].cell_type = LudoCellType::HomeLane(Rang::Blue);
            }
        }

        let mut moving_gotis = Map::new();
        let mut num_finished = Map::new();
        for colour in active_colours.iter() {
            let colour = *colour;
            moving_gotis.insert(colour, Vec::new());
            num_finished.insert(colour, 0);
        }

        LudoEngine {
            curr_colour: *active_colours.first().unwrap(),
            active_colours,
            board,
            locked_gotis,
            moving_gotis,
            num_finished,
        }
    }

    pub fn get_board(&self) -> &[[Box; 15]; 15] {
        &self.board
    }

    /** @note Will always return `true` for a colour that is not playing */
    pub(crate) fn is_finished(&self, colour: Rang) -> bool {
        if self.active_colours.contains(&colour) == false {
            true
        } else {
            self.moving_gotis.get(&colour).unwrap().is_empty()
                && self.locked_gotis.get(&colour).unwrap().is_empty()
        }
    }

    pub(crate) fn set_current_colour(&mut self, colour: Rang) {
        self.curr_colour = colour;
    }

    // Returns Err(()), if no locked goti present
    pub fn unlock_goti(&mut self, colour: Rang) -> Result<(), ()> {
        let locked_positions = Rang::GetLockedPositions(colour);

        // SAFETY: If the attacked_goti was moving, that means atleast 1 locked_positions must be empty, so unwrap() wont panic
        let i = locked_positions
            .iter()
            .position(|coord| {
                self.board[coord.0 as usize][coord.1 as usize]
                    .gotis
                    .is_empty()
                    == false
            })
            .unwrap();
        let locked_coord = locked_positions[i];

        match self.move_goti(colour, locked_coord, 6) {
            Ok(res) => {
                debug_assert!(res == MoveResult::Unlocked);
                Ok(())
            }
            Err(_) => Err(()),
        }
    }

    // Invariant: Expects that atleast 1 goti of `colour` is present at `start_coord`
    pub fn move_goti(
        &mut self,
        colour: Rang,
        start_coords: (u8, u8),
        dist: u8,
    ) -> Result<MoveResult, String> {
        let final_coords = match self.is_move_possible(colour, start_coords, dist) {
            Some(coord) => coord,
            None => {
                return Err("Move not possible".to_string());
            }
        };

        // Invariant: cell.gotis has a goti of `colour`
        let start_cell_goti_index = {
            let start_cell = &self.board[start_coords.0 as usize][start_coords.1 as usize];

            match start_cell
                .gotis
                .iter()
                .position(|g| g.borrow().colour == colour)
            {
                Some(i) => i,
                None => {
                    return Err(format!(
                        "Goti of colour: {:?} doesn't exist at {:?}",
                        colour, start_coords
                    ))
                }
            }
        };

        let was_attack = {
            let cell = &self.board[final_coords.0 as usize][final_coords.1 as usize];

            // If it is a SafeSpot, attack can not happen
            // If it is any other cell where two colors can meet,
            // then presence of any other colour = presence of enemy... 
            (cell.cell_type != LudoCellType::SafeSpot) && (
                cell.gotis.iter().position(|g|  g.borrow().colour != colour).is_some()
            )
        };
        let finished = final_coords == Rang::GetEndCoord(colour);
        let unlocked = final_coords == Rang::GetStartCoord(colour);

        // Mutable changes here; This MUST be an atomic change, either all or none
        {
            // SAFETY: goti_index == None, already handled, so .unwrap() won't panic
            // Cloned `goti`, ab atleast ek reference h goti ka always
            let start_cell = &self.board[start_coords.0 as usize][start_coords.1 as usize];

            let goti = start_cell.gotis.get(start_cell_goti_index).unwrap().clone();
            {
                goti.borrow_mut().coords = final_coords;
            }

            if unlocked {
                // Remove goti from locked_gotis (& start_cell), and put in moving gotis (& dest_cell)
                {
                    let start_cell =
                        &mut self.board[start_coords.0 as usize][start_coords.1 as usize];
                    start_cell.gotis.remove(start_cell_goti_index);
                }
                {
                    let dest_cell =
                        &mut self.board[final_coords.0 as usize][final_coords.1 as usize];
                    dest_cell.gotis.push(goti.clone());
                }

                let goti_index = self
                    .locked_gotis
                    .get(&colour)
                    .unwrap()
                    .iter()
                    .position(|g| g == &goti)
                    .unwrap();
                self.locked_gotis
                    .get_mut(&colour)
                    .unwrap()
                    .remove(goti_index);

                self.moving_gotis
                    .get_mut(&colour)
                    .unwrap()
                    .push(goti.clone());
            } else if finished {
                // Remove goti from everywhere, moving gotis (& start_cell)
                let start_cell = &mut self.board[start_coords.0 as usize][start_coords.1 as usize];
                start_cell.gotis.remove(start_cell_goti_index);

                // SAFETY: all these .unwrap() here assume presence of goti in self.moving_gotis also, which must be the case, else it's critical internal bug
                let goti_index = self
                    .moving_gotis
                    .get(&colour)
                    .unwrap()
                    .iter()
                    .position(|g| g == &goti)
                    .unwrap();
                self.moving_gotis
                    .get_mut(&colour)
                    .unwrap()
                    .remove(goti_index);

                *self.num_finished.get_mut(&goti.borrow().colour).unwrap() += 1;
            } else if was_attack {
                // Remove goti from start_cell, and put in dest_cell
                {
                    let start_cell =
                        &mut self.board[start_coords.0 as usize][start_coords.1 as usize];
                    start_cell.gotis.remove(start_cell_goti_index);
                }
                {
                    let dest_cell =
                        &mut self.board[final_coords.0 as usize][final_coords.1 as usize];
                    dest_cell.gotis.push(goti.clone());
                }

                
                // TIP2: Can add more logic inside the lambda for special rules, for eg. here all gotis of different colors are enemies
                // AND, Remove all attacked_gotis from dest_cell, and put in locked_gotis
                loop {
                    let attacked_goti = {
                        // SAFETY: `goti_to_remove_idx` is a valid index in `dest_cells.gotis`, so unwraps won't panic
                        let dest_cell =
                            &mut self.board[final_coords.0 as usize][final_coords.1 as usize];
                        let goti_to_remove_idx = dest_cell
                            .gotis
                            .iter()
                            .position(|g| g.borrow().colour != colour);

                        if goti_to_remove_idx.is_none() {
                            break;
                        }

                        let goti_ref = dest_cell
                            .gotis
                            .get(goti_to_remove_idx.unwrap())
                            .unwrap()
                            .clone();

                        dest_cell.gotis.remove(goti_to_remove_idx.unwrap());

                        goti_ref
                    };

                    let locked_positions = Rang::GetLockedPositions(attacked_goti.borrow().colour);

                    // SAFETY: If the attacked_goti was moving, that means atleast 1 locked_positions must be empty, so unwrap() wont panic
                    let i = locked_positions
                        .iter()
                        .position(|coord| {
                            self.board[coord.0 as usize][coord.1 as usize]
                                .gotis
                                .is_empty()
                        })
                        .unwrap();
                    let empty_locked_cell = &mut self.board[locked_positions[i].0 as usize]
                        [locked_positions[i].1 as usize];

                    attacked_goti.borrow_mut().coords = locked_positions[i];
                    empty_locked_cell.gotis.push(attacked_goti);
                }
            } else {
                // Normal move
                // Remove goti from start_cell, and put in dest_cell
                {
                    let start_cell =
                        &mut self.board[start_coords.0 as usize][start_coords.1 as usize];
                    start_cell.gotis.remove(start_cell_goti_index);
                }
                {
                    let dest_cell =
                        &mut self.board[final_coords.0 as usize][final_coords.1 as usize];
                    dest_cell.gotis.push(goti.clone());
                }
            }
        }

        let start_cell = &self.board[start_coords.0 as usize][start_coords.1 as usize];

        if was_attack {
            Ok(MoveResult::Attacked(final_coords))
        } else if finished {
            Ok(MoveResult::Finished)
        } else if unlocked {
            Ok(MoveResult::Unlocked)
        } else {
            match start_cell.cell_type {
                LudoCellType::Default | LudoCellType::SafeSpot => {
                    Ok(MoveResult::NormalMove(final_coords))
                }
                LudoCellType::HomeLane(c) => {
                    if c == colour {
                        Ok(MoveResult::NormalMove(final_coords))
                    } else {
                        panic!(
                            ".is_move_possible() galat coordinate diya: {:?}, rang: {:?} 🥺!!",
                            final_coords, colour
                        )
                    }
                }
                _ => panic!(
                    ".is_move_possible() galat coordinate diya: {:?}, rang: {:?} 🥺!!",
                    final_coords, colour
                ),
            }
        }
    }

    /**
     * Invariant: start_coord is a valid coordinate for a goti to exist on the board
     * Note: This does NOT check if goti of such `colour` exists on `start_coord`, for easier debugging or other use by the programmer
     * @returns Some(final_coords), if move possible
     *          None, otherwise if not possible
     */
    pub fn is_move_possible(
        &self,
        colour: Rang,
        start_coord: (u8, u8),
        mut dist: u8,
    ) -> Option<(u8, u8)> {
        if self.board[start_coord.0 as usize][start_coord.1 as usize].cell_type
            == LudoCellType::NoUse
        {
            panic!("Invalid start coord: {:?}", start_coord)
        }

        let goti_is_locked = Rang::GetLockedPositions(colour).contains(&start_coord);

        if goti_is_locked {
            return if dist == 6 {
                Some(Rang::GetStartCoord(colour))
            } else {
                None
            };
        }

        let mut final_coord = start_coord;

        while dist != 0 {
            
            final_coord = LudoEngine::get_next_coord(colour, final_coord);

            if final_coord.0 > 14
                || final_coord.1 > 14
                || ((self.board[final_coord.0 as usize][final_coord.1 as usize].cell_type
                    == LudoCellType::NoUse)
                    && (final_coord != Rang::GetEndCoord(colour)))
            {
                return None;
            }

            dist -= 1;
        }

        Some(final_coord)
    }

    // Returns coords of movable gotis
    // Prevent from leaking internal ds
    // Note: This does NOT handle the case of UNLOCK, handle it yourself then call engine.unlock_goti()
    pub fn get_movable_gotis(&self, colour: Rang, dist: u8) -> Vec<(u8, u8)> {
        let mut start_coords = Vec::new();
        {
            let gotis = match self.moving_gotis.get(&colour) {
                Some(gotis) => gotis,
                None => return vec![],
            };

            for goti in gotis {
                if self
                    .is_move_possible(colour, goti.borrow().coords, dist)
                    .is_some()
                {
                    start_coords.push(goti.borrow().coords);
                }
            }
        }

        start_coords
    }

    // This may return NoUse coord
    fn get_next_coord(colour: Rang, coord: (u8, u8)) -> (u8, u8) {
        // arranged as: (start_coord, next_coord)
        let turns = [
            // Outer turns
            ((0, 6), (0, 7)),
            ((0, 8), (1, 8)),
            ((6, 0), (6, 1)),
            ((6, 14), (7, 14)),
            ((8, 0), (7, 0)),
            ((8, 14), (8, 13)),
            ((14, 6), (13, 6)),
            ((14, 8), (14, 7)),
            // Inner turns
            ((9, 6), (8, 5)),
            ((6, 5), (5, 6)),
            ((8, 9), (9, 8)),
            ((5, 8), (6, 9)),
        ];

        // Check if on outer or inner corners
        for (current, next) in turns {
            if coord == current {
                return next;
            }
        }

        // Check Home turns
        {
            let (current, next) = Rang::GetHomeTurn(colour);
            if coord == current {
                return next;
            }
        }

        // Handling rest cases (can return invalid locations, as in function description)
        if coord.0 == 6 {
            return (coord.0, coord.1 + 1);
        } else if coord.0 == 7 {
            // ie. (7,0)
            if coord.1 == 0 {
                return (coord.0 - 1, coord.1);
            } else if coord.1 < 6 {
                return (coord.0, coord.1 + 1);
            } else if coord.1 == 14 {
                return (coord.0 + 1, coord.1);
            } else if coord.1 > 8 {
                return (coord.0, coord.1 - 1);
            }
        } else if coord.0 == 8 {
            return (coord.0, coord.1 - 1);
        }

        if coord.1 == 6 {
            return (coord.0 - 1, coord.1);
        } else if coord.1 == 7 {
            // ie. (0,7)
            if coord.0 == 0 {
                return (coord.0, coord.1 + 1);
            } else if coord.0 < 6 {
                return (coord.0 + 1, coord.1);
            } else if coord.0 == 14 {
                return (coord.0, coord.1 - 1);
            } else if coord.0 > 8 {
                return (coord.0 - 1, coord.1);
            }
        } else if coord.1 == 8 {
            return (coord.0 + 1, coord.1);
        }

        panic!("Invalid coordinate: {:?}", coord);
    }

    // If any player isn't finished return false, else true
    // It is upto the game to exit if for eg. you want it to exit even if one player isn't finished
    pub(crate) fn is_game_finished(&self) -> bool {
        for colour in self.active_colours.iter() {
            if self.is_finished(*colour) == false {
                return false;
            }
        }

        true
    }

    // Note: returns None for non-playing colors
    pub(crate) fn get_num_locked(&self, colour: Rang) -> Option<u8> {
        match self.locked_gotis.get(&colour) {
            Some(v) => Some(v.len() as u8),
            None => None
        }
    }
}
