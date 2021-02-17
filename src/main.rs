#![feature(bindings_after_at)]

use notify::{self, Watcher};

mod parser;
mod unit_expr;

mod expr;
use expr::val::Val;

mod statement;
use statement::State;

mod latex;

use std::io::Write;

#[cfg(test)]
fn full_eval(s: &str) -> Val {
    use crate::parser::*;
    use pest::Parser;
    use statement::Scope;

    let scope = Scope::default();

    parse_expr(
        MathParser::parse(Rule::expression, s)
            .unwrap()
            .next()
            .unwrap(),
    )
    .eval(&scope)
}

#[cfg(test)]
mod tests {
    use std::convert::TryFrom;

    use super::*;
    use crate::expr::unit::Unit;

    #[test]
    fn test_basic() {
        assert_eq!(full_eval("5 - 3").to_string(), "2".to_string());
        // dbg!(full_eval("5 kg"));
        assert_eq!(full_eval("5 kg").to_string(), "5000 g".to_string());
        assert_eq!(full_eval("5 grams").to_string(), "5 g".to_string());
        assert_eq!(
            full_eval("5 grams + 4 grams").to_string(),
            "9 g".to_string()
        );
        assert_eq!(
            full_eval("5")
                .with_unit(&Unit::try_from("N").unwrap())
                .to_string(),
            "5000 m g s^-2".to_string()
        );
        assert_eq!(
            full_eval("5 kilograms + 4 grams").to_string(),
            "5004 g".to_string()
        );
        assert_eq!(
            full_eval("5 meters * 4 grams").to_string(),
            "20 m g".to_string()
        );
        assert_eq!(
            full_eval("5 meters / 4 grams").to_string(),
            "5/4 m g^-1".to_string()
        );
    }
}

fn main() {
    let args = std::env::args().collect::<Vec<String>>();
    let filename = &args[1];

    let (tx, rx) = std::sync::mpsc::channel();
    let mut watcher = notify::PollWatcher::with_delay_ms(tx, 500).unwrap();

    watcher
        .watch(filename, notify::RecursiveMode::NonRecursive)
        .unwrap();

    let rebuild = || {
        println!("rebuilding pdf");
        let contents = std::fs::read_to_string(filename).unwrap();

        let mut state = State::new(&contents);

        state.exec();

        let mut md_file = tempfile::NamedTempFile::new().unwrap();
        write!(md_file, "{}", state.output).unwrap();

        let mut pandoc = pandoc::new();
        pandoc.set_input_format(pandoc::InputFormat::Latex, Vec::new());
        pandoc.add_input(&md_file.path());

        pandoc.set_output(pandoc::OutputKind::File(args[2].to_string().into()));
        pandoc.execute().unwrap();
        println!("done rebuilding pdf");
    };

    rebuild();

    loop {
        match rx.recv() {
            Ok(_) => rebuild(),
            Err(e) => println!("watch error: {:?}", e),
        }
    }
}
