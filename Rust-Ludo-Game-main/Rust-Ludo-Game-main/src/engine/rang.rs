#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Rang {
    Red,    
    Green,  
    Yellow, 
    Blue    
}

#[allow(non_snake_case)]
impl Rang {
    pub(crate) fn GetStartCoord(colour: Rang) -> (u8, u8) {
        match colour {
            Self::Red => (13,6),
            Self::Green => (6,1),
            Self::Yellow => (1,8),
            Self::Blue => (8,13),
        }
    }

    pub(crate) fn GetEndCoord(colour: Rang) -> (u8, u8) {
        match colour {
            Self::Red => (8,7),
            Self::Green => (7,6),
            Self::Yellow => (6,7),
            Self::Blue => (7,8),
        }
    }

    pub fn GetHomeTurn(colour: Rang) -> ((u8,u8),(u8,u8)) {
        match colour {
            Self::Red => ((14,7), (13,7)),
            Self::Green => ((7,0), (7,1)),
            Self::Yellow => ((0,7), (1,7)),
            Self::Blue => ((7,14), (7,13))
        }
    }

    pub fn GetLockedPositions(colour: Rang) -> [(u8,u8); 4] {
        match colour {
            Self::Red => [(10, 1), (10, 4), (13, 1), (13, 4)],
            Self::Green => [(1, 1), (1, 4), (4, 1), (4, 4)],
            Self::Yellow => [(1, 10), (1, 13), (4, 10), (4, 13)],
            Self::Blue => [(10, 10), (10, 13), (13, 10), (13, 13)]
        }
    }
}
