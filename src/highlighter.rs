use syntect::easy::HighlightLines;
use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSet;
use syntect::util::{as_24_bit_terminal_escaped, LinesWithEndings};

pub struct Highlighter<'a> {
    syntax_set: SyntaxSet,
    theme_set: ThemeSet,
    theme_name: &'a str,
}

impl<'a> Highlighter<'a> {
    pub fn new(theme_name: &'a str) -> Self {
        let syntax_set = SyntaxSet::load_defaults_newlines();
        let theme_set = ThemeSet::load_defaults();

        Self {
            syntax_set,
            theme_set,
            theme_name,
        }
    }

    pub fn print(&mut self, content: &str) {
        let syntax = self.syntax_set.find_syntax_by_extension("md").unwrap();
        let mut h = HighlightLines::new(syntax, &self.theme_set.themes[self.theme_name]);
        for line in LinesWithEndings::from(content) {
            let ranges: Vec<_> = h.highlight(line, &self.syntax_set);
            let escaped = as_24_bit_terminal_escaped(&ranges[..], false);
            print!("{}", escaped);
        }
    }
}
