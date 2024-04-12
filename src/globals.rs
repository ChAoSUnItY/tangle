use std::process::abort;
use std::sync::OnceLock;

use std::{
    fs::File,
    io::{BufRead, BufReader, Error},
    path::PathBuf,
};

pub static SOURCE: OnceLock<String> = OnceLock::new();

fn read_source_file(file_path: impl Into<PathBuf>) -> Result<String, Error> {
    let file_path: PathBuf = file_path.into();
    let file = File::open(file_path.clone())?;
    let mut builder = String::with_capacity(file.metadata()?.len() as usize);
    let reader = BufReader::new(file);

    for line in reader.lines() {
        let line = line?;

        if line.starts_with("#include \"") {
            let mut include_path = file_path.clone();
            include_path.pop();
            let path = line
                .bytes()
                .skip(10)
                .take_while(|c| *c as char != '"')
                .map(|c| c as char)
                .collect::<String>();
            include_path.push(path);

            let sub_file_content = read_source_file(include_path)?;

            builder.push_str(&sub_file_content);
        } else {
            builder.push_str(&line);
        }

        builder.push('\n');
    }

    Ok(builder)
}

pub fn get_source() -> &'static str {
    SOURCE.get_or_init(|| read_source_file("shecc/src/main.c").unwrap())
}

pub fn error(msg: &str, pos: usize) -> ! {
    let source = get_source();
    let mut offset = pos;
    let start_idx: usize;
    let end_idx: usize;

    while source.as_bytes()[offset] != b'\n' {
        offset -= 1;
    }

    start_idx = offset + 1;
    offset = pos;

    while source.as_bytes()[offset] != b'\n' {
        offset += 1;
    }

    end_idx = offset;

    println!("{}", &source[start_idx..end_idx]);
    println!("{}^ {msg}", " ".repeat(pos - start_idx));
    abort()
}
