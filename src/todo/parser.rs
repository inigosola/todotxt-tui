mod line;
mod line_block;
mod parts;
use line::Line;
use line_block::LineBlock;
use parts::Parts;

use super::{ToDo, ToDoData};
use crate::error::{ToDoError, ToDoRes};
use std::str::FromStr;
use tui::style::Style;

pub struct Parser {
    lines: Vec<Line>,
}

impl Parser {
    fn read_block(iter: &mut std::str::Chars<'_>, delimiter: char) -> String {
        let mut read = String::default();
        loop {
            let c = match iter.next() {
                Some(c) => c,
                None => break, // TODO errror, block is not closed
            };
            match c {
                '\\' => read.push(iter.next().unwrap()),
                c if c == delimiter => break,
                _ => read.push(c),
            };
        }
        read
    }

    fn parse(template: &str) -> ToDoRes<Vec<Line>> {
        let mut ret = Vec::new();
        let mut line = Line::default();
        let mut act = String::default();
        let mut iter = template.chars().into_iter();
        loop {
            let c = match iter.next() {
                Some(c) => c,
                None => break,
            };
            match c {
                '[' => {
                    let block = Parser::read_block(&mut iter, ']');
                    line.add_span(&act)?;
                    act = String::default();
                    let mut style = None;
                    match iter.next() {
                        Some('(') => style = Some(Parser::read_block(&mut iter, ')')),
                        Some('\\') => act.push(iter.next().unwrap()),
                        Some(ch) => act.push(ch),
                        None => {
                            act = block;
                            break;
                        }
                    }
                    line.add_span_styled(&block, style)?;
                }
                '\\' => act.push(iter.next().unwrap()),
                '\n' => {
                    line.add_span(&act)?;
                    act = String::default();
                    ret.push(line);
                    line = Line::default();
                }
                _ => act.push(c),
            }
        }
        line.add_span(&act)?;
        ret.push(line);
        Ok(ret)
    }

    pub fn fill(&self, todo: &ToDo) -> Vec<Vec<(String, Style)>> {
        self.lines.iter().map(|line| line.fill(todo)).collect()
    }
}

impl FromStr for Parser {
    type Err = ToDoError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Ok(Parser {
            lines: Parser::parse(value)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::Line;
    use tui::style::Color;
    use tui::style::Modifier;

    #[test]
    fn read_block() {
        let mut iter = "block to parse]".chars();
        assert_eq!(&Parser::read_block(&mut iter, ']'), "block to parse");
        assert_eq!(&iter.collect::<String>(), "");

        let mut iter = "Some style block)".chars();
        assert_eq!(
            &Parser::read_block(&mut iter, ')'),
            "Some style block"
        );
        assert_eq!(&iter.collect::<String>(), "");

        let mut iter = "block to parse] some other text".chars();
        assert_eq!(&Parser::read_block(&mut iter, ']'), "block to parse");
        assert_eq!(&iter.collect::<String>(), " some other text");

        let mut iter = "block to parse \\] with some \\\\ escapes]".chars();
        assert_eq!(
            &Parser::read_block(&mut iter, ']'),
            "block to parse ] with some \\ escapes"
        );
        assert_eq!(&iter.collect::<String>(), "");
    }

    #[test]
    fn parse() -> ToDoRes<()> {
        assert_eq!(Parser::parse("")?[0], Line::default());
        assert_eq!(
            Parser::parse("some text")?[0],
            Line(vec![LineBlock {
                parts: vec![Parts::Text("some text".to_string())],
                style: Style::default()
            }])
        );
        assert_eq!(
            Parser::parse("some text \\[ with escapes \\]")?[0],
            Line(vec![LineBlock {
                parts: vec![Parts::Text("some text [ with escapes ]".to_string())],
                style: Style::default()
            }])
        );
        assert_eq!(
            Parser::parse("[some text](red)")?[0],
            Line(vec![LineBlock {
                parts: vec![Parts::Text("some text".to_string())],
                style: Style::default().fg(Color::Red)
            }])
        );
        assert_eq!(
            Parser::parse("[some text] and another text")?[0],
            Line(vec![
                LineBlock {
                    parts: vec![Parts::Text("some text".to_string())],
                    style: Style::default()
                },
                LineBlock {
                    parts: vec![Parts::Text(" and another text".to_string())],
                    style: Style::default()
                }
            ])
        );
        assert_eq!(
            Parser::parse("[some text]\\[ and escaped text \\]")?[0],
            Line(vec![
                LineBlock {
                    parts: vec![Parts::Text("some text".to_string())],
                    style: Style::default()
                },
                LineBlock {
                    parts: vec![Parts::Text("[ and escaped text ]".to_string())],
                    style: Style::default()
                }
            ])
        );
        assert_eq!(
            Parser::parse("[some text]")?[0],
            Line(vec![LineBlock {
                parts: vec![Parts::Text("some text".to_string())],
                style: Style::default()
            }])
        );
        assert_eq!(
            Parser::parse("[some text](red) text between [another text](blue bold)")?[0],
            Line(vec![
                LineBlock {
                    parts: vec![Parts::Text("some text".to_string())],
                    style: Style::default().fg(Color::Red)
                },
                LineBlock {
                    parts: vec![Parts::Text(" text between ".to_string())],
                    style: Style::default()
                },
                LineBlock {
                    parts: vec![Parts::Text("another text".to_string())],
                    style: Style::default()
                        .fg(Color::Blue)
                        .add_modifier(Modifier::BOLD)
                }
            ])
        );
        let parse = Parser::parse("some text\nnew line")?;
        assert_eq!(parse.len(), 2);
        assert_eq!(
            parse[0],
            Line(vec![LineBlock {
                parts: vec![Parts::Text("some text".to_string())],
                style: Style::default()
            }])
        );
        assert_eq!(
            parse[1],
            Line(vec![LineBlock {
                parts: vec![Parts::Text("new line".to_string())],
                style: Style::default()
            }])
        );

        Ok(())
    }
}