#[macro_export]
macro_rules! prty {
    ($($args:tt)*) => {
        println!("[ \x1b[4;32mOk!\x1b[m ] {}", format!($($args)*))
    };
}

#[macro_export]
macro_rules! largeprty {
    ($($args:tt)*) => {
        println!("[ \x1b[4;32mImportant!\x1b[m ] ---------- [ \x1b[4;32mImportant!\x1b[m ]\n\x1b[100m{}\x1b[m\n[ \x1b[4;33mEND!\x1b[m ] ---------- [ \x1b[4;33mEND!\x1b[m ]", format!($($args)*))
    };
}


#[macro_export]
macro_rules! wprty {
    ($($args:tt)*) => {
        println!("[ \x1b[4;33mWARN!\x1b[m ] {}", format!($($args)*))
    };
}

#[macro_export]
macro_rules! eprty {
    ($($args:tt)*) => {
        eprintln!("[ \x1b[4;31mERR!\x1b[m ] {}", format!($($args)*))
    };
}