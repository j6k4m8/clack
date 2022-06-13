use std::cmp;
use unicode_segmentation::UnicodeSegmentation;

#[derive(Default)]
pub struct Row {
    string: String,
    len: usize,
}

impl From<&str> for Row {
    fn from(slice: &str) -> Self {
        let mut row = Row {
            string: String::from(slice),
            len: 0,
        };
        row.update_len();
        row
    }
}

impl Row {
    /// Render a row to a string.
    ///
    /// # Arguments
    ///
    /// * `start` - The index of the first character to render.
    /// * `end` - The index of the last character to render.
    ///
    /// # Returns
    ///
    /// A string containing the rendered row.
    ///
    pub fn render(&self, start: usize, end: usize) -> String {
        let end = cmp::min(end, self.string.len());
        let start = cmp::min(start, end);
        let mut result = String::new();
        for grapheme in self.string[..]
            .graphemes(true)
            .skip(start)
            .take(end - start)
        {
            if grapheme == "\t" {
                result.push_str(" ");
            } else {
                result.push_str(grapheme);
            }
        }
        result
    }

    /// Get the length of the row (cached)
    ///
    /// # Returns
    ///
    /// The length of the row.
    ///
    pub fn len(&self) -> usize {
        self.len
    }

    /// Get the row contents as a string.
    ///
    /// # Returns
    ///
    /// The row contents as a string.
    ///
    pub fn as_str(&self) -> &str {
        &self.string
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn update_len(&mut self) {
        self.len = self.string[..].graphemes(true).count();
    }

    pub fn insert(&mut self, at: usize, c: char) {
        if at >= self.len() {
            self.string.push(c);
            // let mut result: String = self.string[..].graphemes(true).take(at).collect();
            self.len += 1;
            return;
        }
        let mut result: String = String::new();
        let mut length = 0;
        for (index, grapheme) in self.string[..].graphemes(true).enumerate() {
            length += 1;
            if index == at {
                length += 1;
                result.push(c);
            }
            result.push_str(grapheme);
        }
        self.len = length;
        self.string = result;
    }

    pub fn delete(&mut self, at: usize) {
        if at >= self.len() {
            return;
        }
        let mut result: String = String::new();
        let mut length = 0;
        for (index, grapheme) in self.string[..].graphemes(true).enumerate() {
            if index != at {
                length += 1;
                result.push_str(grapheme);
            }
        }
        self.len = length;
        self.string = result;
    }

    pub fn append(&mut self, new: &Self) {
        self.string = format!("{}{}", self.string, new.string);
        self.update_len();
    }

    pub fn split(&mut self, at: usize) -> Self {
        let mut row: String = String::new();
        let mut length = 0;
        let mut split_row: String = String::new();
        let mut split_length = 0;

        for (index, grapheme) in self.string[..].graphemes(true).enumerate() {
            if index < at {
                length += 1;
                row.push_str(grapheme);
            } else {
                split_length += 1;
                split_row.push_str(grapheme);
            }
        }

        self.len = length;
        self.string = row;
        Self {
            string: split_row,
            len: split_length,
        }
    }

    pub fn as_bytes(&self) -> &[u8] {
        self.string.as_bytes()
    }
}
