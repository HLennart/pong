use core::{sync::atomic::{AtomicU8, Ordering}};


#[repr(u8)]
#[derive(PartialEq)]
pub enum StateLocation {
    Menu,
    Running,
    Paused
}

pub struct GlobalStateLocation (AtomicU8);

impl Default for GlobalStateLocation {
    fn default() -> Self {
        Self (AtomicU8::new(StateLocation::Menu.into()))
    }
}

impl GlobalStateLocation {
    pub fn set(&self, s: StateLocation) {
        let val: u8 = s.into();
        // I hope Relaxed is right for one-off writes :)
        self.0.store(val, Ordering::Relaxed);
    }

    pub fn read(&self) -> StateLocation {
        (&self.0).into()
        
    }
}

impl Into<StateLocation> for u8 {
    fn into(self) -> StateLocation {
        match self {
            0 => StateLocation::Menu,
            1 => StateLocation::Running,
            2 => StateLocation::Paused,
            _ => panic!("Invalid StateLocation")
        }
    }
}

impl From<StateLocation> for u8 {
    fn from(s: StateLocation) -> Self {
        match s {
            StateLocation::Menu => 0,
            StateLocation::Running => 1,
            StateLocation::Paused => 2,
        }
    }
}


impl From<&AtomicU8> for StateLocation {
    fn from(atomic_u8: &AtomicU8) -> Self {
        // We *probably* don't care about memory Ordering here.
        // Again, im not sure.
        // Memory ordering is hard with smooth brain
        let val = atomic_u8.load(Ordering::Relaxed).into();

        match val {
            0 => Self::Menu,
            1 => Self::Running,
            2 => Self::Paused,
            _ => panic!("Invalid StateLocation")
        }
    }
}