use anyhow::{bail, ensure, Context, Result};

use clap::Parser;
use std::fs::File;
use std::io::{stdin, BufRead, BufReader};

#[derive(Parser)]
#[clap(
    name = "My RPN program",
    version = "1.0.0",
    author = "Shota",
    about = "Super awesome sample RPN calculator"
    )]
struct Opts {
    /// Sets the level of verbosity
    #[clap(short, long)]
    verbose: bool,

    /// Formulas written in RPN
    #[clap(name = "FILE")]
    formula_file: Option<String>,
}

struct RpnCalculator(bool);

impl RpnCalculator {
    pub fn new(verbose: bool) -> Self {
        Self(verbose)
    } 

    pub fn eval(&self, formula: &str) -> Result<i32> {
        let mut tokens = formula.split_whitespace().rev().collect::<Vec<_>>();
        self.eval_inner(&mut tokens)
    }

    // eval_innerの中でstackの中身をいじるので、可変な参照
    fn eval_inner(&self, tokens: &mut Vec<&str>) -> Result<i32> {
        let mut stack = Vec::new();
        let mut pos = 0;

        while let Some(token) = tokens.pop() {
            pos += 1;

            if let Ok(x) = token.parse::<i32>() {
                stack.push(x);
            } else {
                // 最後にpushした数値、数値出なかったらerror出力
                let y = stack.pop().context(format!("invalid syntax at {}", pos))?;
                // その前にpushした数値
                let x = stack.pop().context(format!("invalid syntax at {}", pos))?;

                // 今のtoken、記号の場合はx,yを使って計算する
                let res = match token {
                    "+" => x + y,
                    "-" => x - y,
                    "*" => x * y,
                    "/" => x / y,
                    "%" => x % y,
                    _ => bail!("invalid token at {}", pos),
                };
                stack.push(res);
            }
            // -v オプションが設定されている場合、この時点でのtoken, stackをプリントする
            if self.0 {
                println!("{:?} {:?}", tokens, stack);
            }
        }
        // stackを処理した結果、stackの中身は数字１文字になるはず。そうでない場合はinvalid
        // suntaxをエラー出力　
        ensure!(stack.len() == 1, "invalid syntax");
        Ok(stack[0])
    }
}

fn main() -> Result<()> {
    let opts = Opts::parse();

    if let Some(path) = opts.formula_file {
        let f = File::open(path)?;
        let reader = BufReader::new(f);
        run(reader, opts.verbose)

    } else {
        // println!("No file is specified.");
        let stdin = stdin();
        let reader = stdin.lock();
        run(reader, opts.verbose)
    }
}

fn run<R: BufRead>(reader: R, verbose: bool) -> Result<()> {
    let calc = RpnCalculator::new(verbose);
    
    for line in reader.lines() {
        let line = line?;
        match calc.eval(&line) {
            Ok(answer) => println!("{}", answer),
            Err(e) => println!("{:?}", e),
        }
    }
    Ok(()) // nullを返す？
}

#[cfg(test)]
mod tests {
    // testモジュールの親スコープで定義されている構造体など使えるようにする
    use super::*;

    #[test]
    fn test_ok() {
        let calc = RpnCalculator::new(false);
        assert_eq!(calc.eval("5").unwrap(), 5);
        assert_eq!(calc.eval("50").unwrap(), 50);
        assert_eq!(calc.eval("-50").unwrap(), -50);

        assert_eq!(calc.eval("2 3 +").unwrap(), 5);
        assert_eq!(calc.eval("2 3 *").unwrap(), 6);
    }

    #[test]
    #[should_panic]
    fn test_ng() {
        let calc = RpnCalculator::new(false);
        calc.eval("1 1 ^");
    }
}
