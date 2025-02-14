use serde::{Deserialize, Serialize};
use std::*;

#[derive(Debug, PartialEq, Clone, PartialOrd, Serialize, Deserialize)]
pub enum InputToken {
    General(String),
    String(String),
    Integer(i64),
    Float(f64),
}

fn parse_token(input: String) -> InputToken {
    // String was handled else where
    let int_res = input.parse::<i64>();
    if int_res.is_ok() {
        return InputToken::Integer(int_res.unwrap());
    }
    let float_res = input.parse::<f64>();
    if float_res.is_ok() {
        return InputToken::Float(float_res.unwrap());
    }
    InputToken::General(input)
}
pub fn parse_input(input: &str) -> Result<Vec<InputToken>, String> {
    let mut tokens: Vec<InputToken> = Vec::new();
    let mut buffer = String::new();
    let mut idx = 0;
    while idx < input.len() {
        let c = input.chars().nth(idx);
        if c.is_none() {
            return Err("Unexpected end of input".to_string());
        }
        let c = c.unwrap();
        match c {
            '"' => {
                if !buffer.is_empty() {
                    return Err("Unexpected quote".to_string());
                }
                handle_string(&input, &mut idx, &mut buffer)?;
                tokens.push(InputToken::String(std::mem::take(&mut buffer)));
            }
            c if c.is_whitespace() => {
                if !buffer.is_empty() {
                    tokens.push(parse_token(std::mem::take(&mut buffer)));
                    buffer.clear();
                }
                idx += c.len_utf8();
            }
            c => {
                buffer.push(c);
                idx += c.len_utf8();
            }
        }
    }

    if !buffer.is_empty() {
        tokens.push(parse_token(buffer));
    }

    Ok(tokens)
}

fn handle_string(str: &&str, idx: &mut usize, buffer: &mut String) -> Result<(), String> {
    let c = str.chars().nth(*idx).unwrap();
    assert_eq!(c, '"');

    *idx += c.len_utf8();
    while *idx < str.len() {
        let c = str.chars().nth(*idx).unwrap();
        match c {
            '"' => {
                *idx += c.len_utf8();
                return Ok(());
            }
            '\\' => {
                handle_escape(str, idx, buffer)?;
            }
            _ => {
                buffer.push(c);
                *idx += c.len_utf8();
            }
        }
    }

    Err("Unexpected end of input".to_string())
}

fn handle_escape(str: &&str, idx: &mut usize, buffer: &mut String) -> Result<(), String> {
    assert_eq!(str.chars().nth(*idx).unwrap(), '\\');
    *idx += '\\'.len_utf8();
    let c = str.chars().nth(*idx).unwrap();
    match c {
        '"' => {
            buffer.push('"');
            *idx += c.len_utf8();
        }
        '\\' => {
            buffer.push('\\');
            *idx += c.len_utf8();
        }
        'n' => {
            buffer.push('\n');
            *idx += c.len_utf8();
        }
        't' => {
            buffer.push('\t');
            *idx += c.len_utf8();
        }
        'r' => {
            buffer.push('\r');
            *idx += c.len_utf8();
        }
        '0' => {
            buffer.push('\0');
            *idx += c.len_utf8();
        }
        'u' => {
            let mut hex = String::new();
            *idx += c.len_utf8();
            for _ in 0..4 {
                let c = str.chars().nth(*idx).unwrap();
                if !c.is_ascii_hexdigit() {
                    return Err("Invalid unicode escape".to_string());
                }
                hex.push(c);
                *idx += c.len_utf8();
            }
            let code = u32::from_str_radix(&hex, 16);
            if code.is_err() {
                return Err("Invalid unicode escape".to_string());
            }
            let code = code.unwrap();
            buffer.push(char::from_u32(code).unwrap());
        }
        _ => {
            *idx += c.len_utf8();
            return Err("Invalid escape character".to_string());
        }
    }

    Ok(())
}

#[cfg(test)]
mod test {
    #[test]
    fn test_handle_escape() {
        let str = r#""\n\t\r\0\"\\\u0041""#;
        let mut idx = 1;
        let mut buffer = String::new();
        super::handle_escape(&str, &mut idx, &mut buffer).unwrap();
        assert_eq!(buffer, "\n");
        buffer.clear();
        super::handle_escape(&str, &mut idx, &mut buffer).unwrap();
        assert_eq!(buffer, "\t");
        buffer.clear();
        super::handle_escape(&str, &mut idx, &mut buffer).unwrap();
        assert_eq!(buffer, "\r");
        buffer.clear();
        super::handle_escape(&str, &mut idx, &mut buffer).unwrap();
        assert_eq!(buffer, "\0");
        buffer.clear();
        super::handle_escape(&str, &mut idx, &mut buffer).unwrap();
        assert_eq!(buffer, "\"");
        buffer.clear();
        super::handle_escape(&str, &mut idx, &mut buffer).unwrap();
        assert_eq!(buffer, "\\");
        buffer.clear();
        super::handle_escape(&str, &mut idx, &mut buffer).unwrap();
        assert_eq!(buffer, "A");
    }

    #[test]
    fn test_handle_string() {
        let str = r#""Hello, \"world\"!\t\n\u0042\"""#;
        let mut idx = 0;
        let mut buffer = String::new();
        super::handle_string(&str, &mut idx, &mut buffer).unwrap();
        assert_eq!(buffer, "Hello, \"world\"!\t\nB\"");
    }

    #[test]
    fn test_parse_input() {
        let input = r#"Hello, "world\u0042\"!" 42 3.14"#;
        let tokens = super::parse_input(input).unwrap();
        assert_eq!(tokens.len(), 4);
        match &tokens[0] {
            super::InputToken::General(s) => assert_eq!(s, "Hello,"),
            _ => panic!("Unexpected token"),
        }
        match &tokens[1] {
            super::InputToken::String(s) => assert_eq!(s, "worldB\"!"),
            _ => panic!("Unexpected token"),
        }
        match &tokens[2] {
            super::InputToken::Integer(i) => assert_eq!(i, &42),
            _ => panic!("Unexpected token"),
        }
        match &tokens[3] {
            super::InputToken::Float(f) => assert_eq!(f, &3.14),
            _ => panic!("Unexpected token"),
        }
    }
}
