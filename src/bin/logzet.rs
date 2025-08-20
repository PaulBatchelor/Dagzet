use regex::Regex;

/// Simple representation of a date
#[allow(dead_code)]
#[derive(Default)]
struct Date {
    month: u8,
    day: u8,
    year: u16,
    title: String,
    context: Option<String>,
    tags: Vec<String>,
}

/// Simple representation of a time
#[allow(dead_code)]
#[derive(Default)]
struct Time {
    hour: u8,
    minute: u8,
    title: String,
    tags: Vec<String>,
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

#[derive(Debug)]
#[allow(dead_code)]
enum StatementError {
    ParseError,
}

impl std::error::Error for StatementError {}

impl std::fmt::Display for StatementError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "TODO: implement display trait")
    }
}

fn title_and_tags(full_title: &str) -> (String, Vec<String>) {
    let (words, tags): (Vec<String>, Vec<String>) = full_title
        .split(' ')
        .map(|w| w.to_string())
        .partition(|w| !w.starts_with("#"));

    let title = words.join(" ");

    let tags: Vec<_> = tags
        .iter()
        .filter_map(|s| s.strip_prefix("#"))
        .map(|s| s.to_string())
        .collect();

    (title, tags)
}

impl TryFrom<String> for Statement {
    type Error = StatementError;
    fn try_from(value: String) -> Result<Statement, Self::Error> {
        if value.starts_with("---") {
            return Ok(Statement::Break);
        }

        if value.starts_with("@") {
            // It's a block, but which block type?

            // try to match it on a time
            let re_time = Regex::new(r"@(\d\d):(\d\d)\s+(.*)").unwrap();
            if let Some(caps) = re_time.captures(&value) {
                dbg!(&caps[1]);
                let hour = match str::parse::<u8>(&caps[1]) {
                    Ok(hour) => hour,
                    Err(_) => {
                        return Err(StatementError::ParseError);
                    }
                };
                let minute = match str::parse::<u8>(&caps[2]) {
                    Ok(minute) => minute,
                    Err(_) => {
                        return Err(StatementError::ParseError);
                    }
                };

                let (title, tags) = title_and_tags(&caps[3]);

                return Ok(Statement::Time(Time {
                    hour,
                    minute,
                    title,
                    tags,
                }));
            }
            // Try to match it on a Date
            let re_date = Regex::new(r"@(\d\d\d\d)-(\d\d)-(\d\d)(#[a-z]+)??\s+(.*)").unwrap();

            if let Some(caps) = re_date.captures(&value) {
                let year = match str::parse::<u16>(&caps[1]) {
                    Ok(year) => year,
                    Err(_) => {
                        return Err(StatementError::ParseError);
                    }
                };

                let month = match str::parse::<u8>(&caps[2]) {
                    Ok(month) => month,
                    Err(_) => {
                        return Err(StatementError::ParseError);
                    }
                };

                let day = match str::parse::<u8>(&caps[3]) {
                    Ok(day) => day,
                    Err(_) => {
                        return Err(StatementError::ParseError);
                    }
                };

                let context = if let Some(c) = caps.get(4) {
                    c.as_str().strip_prefix("#").map(|s| s.to_string())
                } else {
                    None
                };

                let (title, tags) = title_and_tags(&caps[5]);
                return Ok(Statement::Date(Date {
                    year,
                    month,
                    day,
                    title,
                    context,
                    tags,
                }));
            }
        }

        if value.starts_with("#!") {
            let args: Vec<_> = value
                .split_whitespace()
                .skip(1)
                .map(|s| s.to_string())
                .collect();
            return Ok(Statement::Command(Command { args }));
        }

        Err(StatementError::ParseError)
    }
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

#[allow(dead_code)]
// TODO: error handling, plz read that rust for rustaceans chapter
fn build_sessions(_stmts: Vec<Statement>) -> Vec<Session> {
    todo!()
}

fn main() {
    println!("hello logzet");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_statement_tryfrom_break() {
        let s = "---".to_string();
        let stmt: Option<Statement> = s.try_into().ok();
        assert!(stmt.is_some(), "Could not parse");
        assert!(
            matches!(stmt.unwrap(), Statement::Break),
            "Did not parse correctly"
        );
    }

    #[test]
    fn test_statement_tryfrom_time() {
        let timestr = "@12:34 this is a title".to_string();

        let time: Option<Statement> = timestr.try_into().ok();

        assert!(time.is_some(), "Could not parse");
        let time = time.unwrap();
        assert!(
            matches!(&time, Statement::Time(_)),
            "Did not parse correctly"
        );

        if let Statement::Time(time) = time {
            assert_eq!(time.hour, 12);
            assert_eq!(time.minute, 34);
            assert_eq!(&time.title, "this is a title");
        }
    }

    #[test]
    fn test_statement_tryfrom_time_with_tags() {
        let time1str = "@12:34 test title #tag1 #tag2".to_string();
        let time2str = "@12:34 test #tag3 title #tag1 #tag2  extra spaces #tag4 ...".to_string();

        let time1: Option<Statement> = time1str.try_into().ok();

        assert!(time1.is_some(), "Could not parse");
        let time1 = time1.unwrap();
        assert!(
            matches!(&time1, Statement::Time(_)),
            "Did not parse correctly"
        );

        if let Statement::Time(time) = time1 {
            assert_eq!(&time.title, "test title");
            let expected_tags: Vec<_> = ["tag1", "tag2"].into_iter().map(String::from).collect();
            assert_eq!(&time.tags, &expected_tags);
        }

        let time2: Option<Statement> = time2str.try_into().ok();

        assert!(time2.is_some(), "Could not parse");
        let time2 = time2.unwrap();
        assert!(
            matches!(&time2, Statement::Time(_)),
            "Did not parse correctly"
        );

        if let Statement::Time(time) = time2 {
            assert_eq!(&time.title, "test title  extra spaces ...");
            let expected_tags: Vec<_> = ["tag3", "tag1", "tag2", "tag4"]
                .into_iter()
                .map(String::from)
                .collect();
            assert_eq!(&time.tags, &expected_tags);
        }
    }

    #[test]
    fn test_statement_tryfrom_date() {
        let date1str = "@2025-08-18 Test Title";
        let date2str = "@2025-08-18#abc Test Title (with context name)";
        let date3str = "@2025-08-18 Title with hashtags? #tag1 #tag2";
        let date4str = "@2025-08-18#abcde Everything! #tag2 #tag1";

        let date1: Option<Statement> = date1str.to_string().try_into().ok();

        assert!(date1.is_some(), "Could not parse");
        let date1 = date1.unwrap();
        assert!(
            matches!(&date1, Statement::Date(_)),
            "Did not parse correctly"
        );

        if let Statement::Date(date) = date1 {
            assert_eq!(date.year, 2025);
            assert_eq!(date.month, 8);
            assert_eq!(date.day, 18);
            assert_eq!(&date.title, "Test Title");
        }

        let date2: Option<Statement> = date2str.to_string().try_into().ok();
        assert!(date2.is_some(), "Could not parse");
        let date2 = date2.unwrap();
        assert!(
            matches!(&date2, Statement::Date(_)),
            "Did not parse correctly"
        );

        if let Statement::Date(date) = date2 {
            assert_eq!(date.year, 2025);
            assert_eq!(date.month, 8);
            assert_eq!(date.day, 18);
            assert_eq!(&date.title, "Test Title (with context name)");
            assert!(date.context.is_some());

            if let Some(context) = &date.context {
                assert_eq!(context, "abc");
            }
        }

        let date3: Option<Statement> = date3str.to_string().try_into().ok();
        assert!(date3.is_some(), "Could not parse");
        let date3 = date3.unwrap();
        assert!(
            matches!(&date3, Statement::Date(_)),
            "Did not parse correctly"
        );

        if let Statement::Date(date) = date3 {
            assert_eq!(date.year, 2025);
            assert_eq!(date.month, 8);
            assert_eq!(date.day, 18);
            assert_eq!(&date.title, "Title with hashtags?");
            assert!(date.context.is_none());
            let expected_tags: Vec<_> = ["tag1", "tag2"].into_iter().map(String::from).collect();
            assert_eq!(&date.tags, &expected_tags);
        }

        let date4: Option<Statement> = date4str.to_string().try_into().ok();
        assert!(date4.is_some(), "Could not parse");
        let date4 = date4.unwrap();
        assert!(
            matches!(&date4, Statement::Date(_)),
            "Did not parse correctly"
        );

        if let Statement::Date(date) = date4 {
            assert_eq!(date.year, 2025);
            assert_eq!(date.month, 8);
            assert_eq!(date.day, 18);
            assert_eq!(&date.title, "Everything!");
            assert!(date.context.is_some());
            let expected_tags: Vec<_> = ["tag2", "tag1"].into_iter().map(String::from).collect();
            assert_eq!(&date.tags, &expected_tags);
            if let Some(context) = &date.context {
                assert_eq!(context, "abcde");
            }
        }
    }

    #[test]
    fn test_statement_tryfrom_command() {
        let cmdstr = "#! dz foo/bar";
        let cmd: Option<Statement> = cmdstr.to_string().try_into().ok();
        assert!(cmd.is_some(), "Could not parse");
        let cmd = cmd.unwrap();
        assert!(
            matches!(&cmd, Statement::Command(_)),
            "Did not parse correctly"
        );

        if let Statement::Command(cmd) = cmd {
            let expected_args: Vec<_> = ["dz", "foo/bar"].into_iter().map(String::from).collect();
            assert_eq!(&cmd.args, &expected_args);
        }
    }
}
