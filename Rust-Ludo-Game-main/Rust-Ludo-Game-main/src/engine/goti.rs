use super::Rang;
use debug_print::debug_println;

#[derive(PartialEq, Eq)]
pub struct LudoGoti {
    pub colour: Rang,
    pub coords: (u8,u8)
}

impl Drop for LudoGoti {
    fn drop(&mut self) {
        debug_println!("(Ignore this, if another panic happened before this) Dropping {:?}, was at {:?}", self.colour, self.coords);
        debug_assert!(self.coords == Rang::GetEndCoord(self.colour), "Goti was not at finish location when dropped");
    }
}
