/// Terminal styling constants and ANSI codes for P2P chat client

// ANSI color codes for terminal
pub const COLOR_RESET: &str = "\x1b[0m";
pub const COLOR_GREEN: &str = "\x1b[32m";
pub const COLOR_BLUE: &str = "\x1b[34m";
pub const COLOR_YELLOW: &str = "\x1b[33m";
pub const COLOR_CYAN: &str = "\x1b[36m";
pub const COLOR_BOLD: &str = "\x1b[1m";
pub const COLOR_DIM: &str = "\x1b[2m";
pub const COLOR_WHITE: &str = "\x1b[37m";

// Box drawing characters (unused but kept for future UI enhancements)
pub const BOX_HORIZONTAL: &str = "─";
pub const BOX_VERTICAL: &str = "│";
pub const BOX_TOP_LEFT: &str = "┌";
pub const BOX_TOP_RIGHT: &str = "┐";
pub const BOX_BOTTOM_LEFT: &str = "└";
pub const BOX_BOTTOM_RIGHT: &str = "┘";
pub const BOX_CROSS: &str = "┼";
pub const BOX_T_DOWN: &str = "┬";
pub const BOX_T_UP: &str = "┴";
pub const BOX_T_RIGHT: &str = "├";
pub const BOX_T_LEFT: &str = "┤";
