use crate::{Position, Row};
use std::{fs, io::Write};

#[derive(Default)]
pub struct Document {
    rows: Vec<Row>,
    pub file_name: Option<String>,
    dirty: bool,
}

impl Document {
    pub fn open(filename: &str) -> Result<Self, std::io::Error> {
        let file = fs::read_to_string(filename)?;
        let rows = file.split('\n').map(|row| Row::from(row)).collect();
        Ok(Self {
            rows,
            file_name: Some(filename.to_string()),
            dirty: false,
        })
    }

    pub fn get_row(&self, index: usize) -> Option<&Row> {
        self.rows.get(index)
    }

    pub fn row_count(&self) -> usize {
        self.rows.len()
    }

    fn insert_newline(&mut self, at: &Position) {
        if at.y > self.rows.len() {
            // The cursor is in a space that doesn't exist.
            return;
        }
        if at.y == self.row_count() {
            self.rows.push(Row::default());
            return;
        }

        let new_row = self.rows.get_mut(at.y).unwrap().split(at.x);
        self.rows.insert(at.y + 1, new_row);
    }

    pub fn insert(&mut self, at: &Position, c: char) {
        if at.y > self.row_count() {
            return;
        }
        self.dirty = true;
        if c == '\n' {
            // The user wants to insert a newline; create a new row:
            self.insert_newline(at);
            return;
        }
        if at.y == self.row_count() {
            let mut row = Row::default();
            // Override the at.x because the row is new.
            // TODO: Could allow editing in empty space by padding.
            row.insert(0, c);
            self.rows.push(row);
        } else {
            let row = self.rows.get_mut(at.y).unwrap();
            row.insert(at.x, c);
        }
    }

    pub fn delete(&mut self, at: &Position) {
        let len = self.rows.len();
        if at.y >= len {
            return;
        }

        self.dirty = true;
        if at.x == self.rows.get_mut(at.y).unwrap().len() && at.y + 1 < len {
            let next_row = self.rows.remove(at.y + 1);
            let row = self.rows.get_mut(at.y).unwrap();
            row.append(&next_row);
        } else {
            let row = self.rows.get_mut(at.y).unwrap();
            row.delete(at.x);
        }
    }

    pub fn save(&mut self) -> Result<(), std::io::Error> {
        if let Some(file_name) = &self.file_name {
            let mut file = fs::File::create(file_name)?;
            for row in &self.rows {
                file.write_all(row.as_bytes())?;
                file.write_all(b"\n")?;
            }
            self.dirty = false;
        }
        Ok(())
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    pub fn find(&self, query: &str) -> Option<Position> {
        for (y, row) in self.rows.iter().enumerate() {
            if let Some(x) = row.find(query) {
                return Some(Position { x, y });
            }
        }
        None
    }
}
