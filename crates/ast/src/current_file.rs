use std::cell::Cell;

thread_local! {
    /// Current file to be used when generating Pos.
    static CURRENT_FILE_OF_POS: Cell<usize> = const { Cell::new(0) };
}

pub fn get_current_file_of_pos() -> usize {
    CURRENT_FILE_OF_POS.with(|v| v.get())
}

/// Set current file number to be used when generating Pos.
pub fn set_current_file_of_pos(file: usize) {
    CURRENT_FILE_OF_POS.with(|cell| cell.set(file));
}
