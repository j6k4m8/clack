use crate::{
    sound::{self, Audible, SoundManager, Tone, Utterance},
    utils::string_to_speakable_tokens,
};
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
                // TODO: This is bad
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

    fn get_tokens_and_indices(&self) -> Vec<(usize, &str)> {
        // Split on non-alphanumeric characters.
        let bounds = self.string.split_word_bound_indices();
        return bounds.into_iter().collect();
    }

    pub fn get_word_at(&self, at: usize) -> Option<&str> {
        // Split on tokens
        for (start, token) in self.get_tokens_and_indices().iter() {
            if start + token.len() > at {
                return Some(&self.string[*start..*start + token.len()]);
            }
        }
        None
    }

    pub fn play_blocking(&self, manager: &mut SoundManager) {
        // Represent leading tabs with tones.
        let indent_level = self.string.chars().take_while(|c| *c == '\t').count();
        // TODO: Space indent fixed size:
        let indent_space_level = self.string.chars().take_while(|c| *c == ' ').count() / 4;
        let indent_level = indent_level + indent_space_level;
        let duration = 0.15;
        let volume: f32 = 0.5;
        for indent in 0..indent_level {
            manager.play_and_wait(Box::new(Tone::new(
                *sound::PENTATONIC_SCALE
                    .get(indent % sound::PENTATONIC_SCALE.len())
                    .unwrap(),
                duration,
                volume,
            )));
        }

        // Play the rest of the row:
        let utterance = Utterance::new(string_to_speakable_tokens(&self.string, None));
        manager.play_and_wait(Box::new(utterance))
    }

    pub fn play(&self, manager: &mut SoundManager) {
        // Represent leading tabs with tones.
        let indent_level = self.string.chars().take_while(|c| *c == '\t').count();
        // TODO: Space indent fixed size:
        let indent_space_level = self.string.chars().take_while(|c| *c == ' ').count() / 4;
        let indent_level = indent_level + indent_space_level;
        // TONES:
        // D: 36.6666 E: 41.15625 F#: 46.40625 A: 55 B: 61.875
        let duration = 0.15;
        let volume: f32 = 0.5;
        let tones = vec![
            Tone::new(8.0 * 36.6666, duration, volume),
            Tone::new(8.0 * 41.15625, duration, volume),
            Tone::new(8.0 * 46.40625, duration, volume),
            Tone::new(8.0 * 55.0, duration, volume),
            Tone::new(8.0 * 61.875, duration, volume),
        ];
        for indent in 0..indent_level {
            manager.play_and_wait(Box::new(*tones.get(indent % tones.len()).unwrap()));
        }

        // Play the rest of the row:
        let utterance = Utterance::new(string_to_speakable_tokens(&self.string, None));
        manager.play(Box::new(utterance))
    }

    pub fn find(&self, query: &str) -> Option<usize> {
        let matching_byte_index = self.string.find(query);
        if let Some(matching_byte_index) = matching_byte_index {
            for (grapheme_index, (byte_index, _)) in
                self.string[..].grapheme_indices(true).enumerate()
            {
                if matching_byte_index == byte_index {
                    return Some(grapheme_index);
                }
            }
        }
        None
    }
}
