use colorize::AnsiColor;

static mut VERBOSE_FLAG: usize = 0;
const VERBOSE_PRELUDE: &str = "{}";

fn update_prelude() -> String {
    "".to_string()
}
fn _verbose_println(msg: &str) {
    unsafe {
        if VERBOSE_FLAG >= 1 {
            let pre = update_prelude();
            println!("V) {}: {}", pre, msg.to_string().yellow())
        }
    }
}
fn _very_verbose_println(msg: &str) {
    unsafe {
        if VERBOSE_FLAG >= 2 {
            println!("{} {}", VERBOSE_PRELUDE, msg.to_string().b_yellow())
        }
    }
}

fn _very_very_verbose_println(msg: &str) {
    unsafe {
        if VERBOSE_FLAG >= 3 {
            println!("{} {}", VERBOSE_PRELUDE, msg.to_string().b_yellowb())
        }
    }
}

pub fn set_verbosity(mode: usize) {
    unsafe { VERBOSE_FLAG = mode }
}

#[macro_export]
macro_rules! verbose_println {
    ($($arg:tt)*) => (crate::_verbose_println(&format!($($arg)*)));
}
#[macro_export]
macro_rules! very_verbose_println {
    ($($arg:tt)*) => (crate::_very_verbose_println(&format!($($arg)*)));
}
#[macro_export]
macro_rules! very_very_verbose_println {
    ($($arg:tt)*) => (crate::_very_very_verbose_println(&format!($($arg)*)));
}
