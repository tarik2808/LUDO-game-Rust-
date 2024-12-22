use std::{cell::RefCell, rc::Rc};

use super::goti::LudoGoti;
use super::rang::Rang;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LudoCellType {
    Default,
    SafeSpot,
    LockedPosition(Rang),
    HomeLane(Rang),

    NoUse   // MUST not be mutated, such a cell will panic on invalid (eg. movedHere etc.)
}

pub struct LudoCell {
    pub cell_type: LudoCellType,
    pub gotis: Vec<Rc<RefCell<LudoGoti>>>
}
