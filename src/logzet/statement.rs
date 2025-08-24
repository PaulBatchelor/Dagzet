use crate::logzet::{Command, Date, DateKey, TextLine, Time, TimeKey};
use regex::Regex;

#[derive(Debug)]
#[allow(dead_code)]
pub enum StatementError {
    ParseError,
}

/// A granular unit of information, typically represented as a line of text
#[allow(dead_code)]
#[derive(Clone)]
pub enum Statement {
    Date(Date),
    Time(Time),
    Break,
    TextLine(TextLine),
    PreTextLine(TextLine),
    Command(Command),
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
                    key: TimeKey { hour, minute },
                    title,
                    tags,
                    ..Default::default()
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
                    key: DateKey {
                        year,
                        month,
                        day,
                        context,
                    },
                    title,
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
