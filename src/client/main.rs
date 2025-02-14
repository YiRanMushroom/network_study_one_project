use networking::logic::input_parser::{parse_input, InputToken};
use std::io::stdin;

fn main() {
    let mut input = String::new();
    loop {
        stdin().read_line(&mut input).unwrap();
        let tokens = parse_input(&input).unwrap_or_default();
        if tokens.len() == 1 && tokens[0] == InputToken::General("stop".to_string()) {
            break;
        }

        for (idx, token) in tokens.iter().enumerate() {
            println!("{}: {:?}", idx, token);
        }

        input.clear()
    }
}
