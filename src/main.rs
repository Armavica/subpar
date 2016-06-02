extern crate rustc_serialize;
extern crate docopt;

use std::fmt;
use std::io::{self, Read};
use docopt::Docopt;

const USAGE: &'static str = "
Subpar is a filter for paragraph reformatting.

Usage: subpar [options]

Options:
  -h, --help            Print this message.
  -l, --last            Make the last line as long as the others.
  -w, --width <width>   No line in the output may contain more than <width>
                        characters (newline excluded) [default: 79].
";

#[derive(Debug, RustcDecodable)]
struct Args {
    flag_last: bool,
    flag_width: usize,
}

#[derive(Debug)]
enum Word<'a> {
    Normal(&'a str),
    EndOfSentence(&'a str),
}

// Returns a vector of paragraphs (vectors of words)
fn tokenize(input: &str) -> Vec<Vec<Word>> {
    let endings = ".!?…";
    let mut text = Vec::new();
    let mut paragraph = Vec::new();
    let mut last_word: Option<&str> = None;
    let mut many_spaces = false;
    let mut newlines = 0;
    for line in input.lines() {
        for word in line.split(' ') {
            if word.is_empty() {
                many_spaces = true;
            } else {
                if let Some(last_word) = last_word {
                    if last_word.ends_with(|c| endings.contains(c)) &&
                       (many_spaces || newlines > 0) {
                        paragraph.push(Word::EndOfSentence(last_word));
                    } else {
                        paragraph.push(Word::Normal(last_word));
                    }
                    many_spaces = false;
                    if newlines > 1 {
                        text.push(paragraph);
                        paragraph = Vec::new();
                    }
                    newlines = 0;
                }
                last_word = Some(word);
            }
        }
        newlines += 1;
    }
    if let Some(last_word) = last_word {
        paragraph.push(Word::EndOfSentence(last_word));
    }
    text.push(paragraph);
    text
}

// Returns a vector of vectors such that `lengths[i][j]` is the length
// of a line starting with word `i` and ending with word `i+j`.
fn line_lengths(line: &[Word]) -> Vec<Vec<usize>> {
    let mut lengths = Vec::with_capacity(line.len() * line.len());
    let mut length;
    for i in 0..line.len() {
        length = 0usize;
        let mut tmp = Vec::with_capacity(line.len() - i);
        for word in line[i..].iter() {
            match *word {
                Word::Normal(ref w) => {
                    length += w.chars().count();
                    tmp.push(length);
                    length += 1;
                }
                Word::EndOfSentence(w) => {
                    length += w.chars().count();
                    tmp.push(length);
                    length += 2;
                }
            }
        }
        lengths.push(tmp);
    }
    lengths
}

fn badness(line_length: usize, width: usize) -> usize {
    if line_length > width {
        1_000_000 * (line_length - width)
    } else {
        (width - line_length).pow(3)
    }
}

// Contains a vector of lines, each line being a slice of words.
struct Paragraph<'a> {
    paragraph: Vec<&'a [Word<'a>]>,
    maxwidth: usize,
}

impl<'a> fmt::Display for Paragraph<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for words in &self.paragraph {
            let mut line = String::new();
            for word in words.iter() {
                match *word {
                    Word::Normal(ref w) => line.push_str(&format!("{} ", w)),
                    Word::EndOfSentence(ref w) => line.push_str(&format!("{}  ", w)),
                }
            }
            let line = line.trim_right();
            try!(writeln!(f, "{}", line));
        }
        Ok(())
    }
}

impl<'a> fmt::Debug for Paragraph<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for words in &self.paragraph {
            let mut line = String::new();
            for word in words.iter() {
                match *word {
                    Word::Normal(ref w) => line.push_str(&format!("{} ", w)),
                    Word::EndOfSentence(ref w) => line.push_str(&format!("{}  ", w)),
                }
            }
            let line = line.trim_right();
            try!(write!(f, "{}", line));
            if self.maxwidth >= line.chars().count() {
                for _ in 0..self.maxwidth - line.chars().count() {
                    try!(write!(f, " "));
                }
                try!(write!(f, "|{}", self.maxwidth));
            }
            try!(writeln!(f, ""));
        }
        Ok(())
    }
}

fn reformat<'a>(text: &'a [Word<'a>], args: &Args) -> Paragraph<'a> {
    let width = args.flag_width;
    let last = args.flag_last;
    let n = text.len();

    // Optimize the length of the lines independently (DP).
    let mut dp = Vec::with_capacity(text.len() * text.len());
    let lengths = line_lengths(text);
    dp.push((0, 0));
    for i in (0..n).rev() {
        let mut minbadness = None;
        for j in 1..n - i + 1 {
            let length = lengths[i][j - 1];
            let mut localbad = badness(length, width) + dp[n - j - i].0;
            if !last && i + j == n {
                // last line
                if width / 4 < length && length < width {
                    localbad /= 100;
                }
            }
            match minbadness {
                None => minbadness = Some((localbad, j)),
                Some((m, _)) if localbad < m => minbadness = Some((localbad, j)),
                _ => {}
            }
        }
        dp.push(minbadness.unwrap());
    }

    // Exploit the DP result to split the lines and produce a paragraph.
    let mut paragraph = Vec::with_capacity(dp.len() - 1);
    let mut nb = 0;
    let mut i = 0;
    dp.reverse();
    dp.pop();
    for (_, k) in dp.into_iter() {
        if nb == 0 {
            paragraph.push(&text[i..i + k]);
            i += k;
            nb = k;
        }
        nb -= 1;
    }
    Paragraph {
        paragraph: paragraph,
        maxwidth: width,
    }
}

fn main() {
    let args: Args = Docopt::new(USAGE)
        .and_then(|dopt| dopt.decode())
        .unwrap_or_else(|e| e.exit());

    let mut input = String::new();
    if io::stdin().read_to_string(&mut input).is_ok() {
        for paragraph in &tokenize(&input) {
            println!("{}", reformat(&paragraph, &args));
        }
    } else {
        println!("subpar: Error reading stdin.");
    }
}
