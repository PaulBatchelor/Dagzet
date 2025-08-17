/// Simple representation of a date
#[allow(dead_code)]
struct Date {
    month: u8,
    day: u8,
    year: u16,
}

/// Simple representation of a time
#[allow(dead_code)]
struct Time {
    hour: u8,
    minute: u8,
}

/// A single line of text
#[allow(dead_code)]
struct TextLine {
    text: String,
}

/// A command
#[allow(dead_code)]
struct Command {
    args: Vec<String>,
}

/// A granular unit of information, typically represented as a line of text
#[allow(dead_code)]
enum Statement {
    Date(Date),
    Time(Time),
    Break,
    TextLine(TextLine),
    PreTextLine(TextLine),
    Command(Command),
}

/// A line range
#[allow(dead_code)]
struct LineRange {
    start: usize,
    end: Option<usize>,
}

/// Used for source mapping
#[allow(dead_code)]
struct Location<T> {
    filename: String,
    lines: LineRange,
    data: T,
}

#[allow(dead_code)]
enum Block {
    Text(String),
    PreText(String),
}

#[allow(dead_code)]
struct Entry {
    time: Time,
    blocks: Vec<Block>,
}

#[allow(dead_code)]
struct Session {
    date: Date,
    context: Option<String>,
    entries: Vec<Entry>,
}

fn main() {
    println!("hello logzet");
}
