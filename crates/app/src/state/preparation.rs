//! Real startup stages reported by the background preparation worker.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub(crate) enum PreparationStage {
    Settings,
    Languages,
    Components,
    Updates,
}

impl PreparationStage {
    pub(crate) fn from_progress(progress: u8) -> Self {
        match progress {
            1 => Self::Languages,
            2 => Self::Components,
            3 => Self::Updates,
            _ => Self::Settings,
        }
    }
}
