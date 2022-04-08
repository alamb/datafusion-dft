// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.  See the NOTICE file
// distributed with this work for additional information
// regarding copyright ownership.  The ASF licenses this file
// to you under the Apache License, Version 2.0 (the
// "License"); you may not use this file except in compliance
// with the License.  You may obtain a copy of the License at
//
//   http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing,
// software distributed under the License is distributed on an
// "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied.  See the License for the
// specific language governing permissions and limitations
// under the License.

use log::debug;
use std::cmp;
use std::fs::File;
use std::io::{self, BufRead, BufReader};

use unicode_width::UnicodeWidthStr;

use crate::app::core::AppReturn;
use crate::app::datafusion::context::QueryResultsMeta;
use crate::app::error::Result;

const MAX_EDITOR_LINES: u16 = 17;

/// Single line of text in SQL Editor and cursor over it
#[derive(Debug)]
pub struct Line {
    text: io::Cursor<String>,
}

impl Default for Line {
    fn default() -> Line {
        Line {
            text: io::Cursor::new(String::new()),
        }
    }
}

impl Line {
    pub fn new(text: String) -> Self {
        Line {
            text: io::Cursor::new(text),
        }
    }
}

/// All lines in SQL Editor and cursor location
#[derive(Debug, Default)]
pub struct Input {
    pub lines: Vec<Line>,
    /// Current line in editor
    pub current_row: u16,
    /// Current column in editor
    pub cursor_column: u16,
}

impl Input {
    pub fn combine_lines(&self) -> String {
        let text: Vec<&str> = self
            .lines
            .iter()
            // Replace tabs with spaces
            .map(|line| line.text.get_ref().as_str())
            .collect();
        text.join("")
    }

    pub fn combine_visible_lines(&self) -> String {
        let start = if self.current_row < MAX_EDITOR_LINES {
            0
        } else {
            self.current_row - MAX_EDITOR_LINES
        } as usize;

        let end = (start + (MAX_EDITOR_LINES as usize) + 1) as usize;

        let text: Vec<&str> = if start == 0 {
            // debug!("Combining all lines");
            self.lines
                .iter()
                .map(|line| line.text.get_ref().as_str())
                .collect()
        } else {
            debug!("Combining visible lines: start({}) to end({})", start, end);
            self.lines[start..end]
                .iter()
                .map(|line| line.text.get_ref().as_str())
                .collect()
        };

        text.join("")
    }

    pub fn append_char(&mut self, c: char) -> Result<AppReturn> {
        if self.lines.is_empty() {
            let line = Line::default();
            self.lines.push(line)
        }
        match c {
            '\n' => self.new_line(),
            '\t' => {
                self.lines[self.current_row as usize]
                    .text
                    .get_mut()
                    .push_str("    ");
                self.cursor_column += 4
            }
            _ => {
                self.lines[self.current_row as usize]
                    .text
                    .get_mut()
                    .insert(self.cursor_column as usize, c);
                debug!(
                    "Line after appending {:?} : {:?}",
                    c,
                    self.lines[self.current_row as usize].text.get_ref()
                );
                self.cursor_column += 1;
            }
        }
        Ok(AppReturn::Continue)
    }

    pub fn pop(&mut self) -> Option<char> {
        self.lines[self.current_row as usize].text.get_mut().pop()
    }

    pub fn up_row(&mut self) -> Result<AppReturn> {
        if self.current_row > 0 {
            match self.lines[self.current_row as usize]
                .text
                .get_ref()
                .is_empty()
            {
                true => {
                    self.current_row -= 1;
                    let new_row_width =
                        self.lines[self.current_row as usize].text.get_ref().width() as u16;
                    self.cursor_column = new_row_width;
                }
                false => {
                    let previous_col = self.cursor_column;
                    self.current_row -= 1;
                    let new_row_width =
                        self.lines[self.current_row as usize].text.get_ref().width() as u16;
                    let new_col = cmp::min(previous_col, new_row_width);
                    self.cursor_column = new_col;
                }
            }
        }
        Ok(AppReturn::Continue)
    }

    pub fn down_row(&mut self) -> Result<AppReturn> {
        if self.lines.is_empty() {
            return Ok(AppReturn::Continue);
        } else if self.current_row + 1 < self.lines.len() as u16 {
            let previous_col = self.cursor_column;
            self.current_row += 1;
            let new_row_width = self.lines[self.current_row as usize].text.get_ref().width() as u16;
            let new_col = cmp::min(previous_col, new_row_width);
            self.cursor_column = new_col;
        }
        Ok(AppReturn::Continue)
    }

    pub fn next_char(&mut self) -> Result<AppReturn> {
        if self.lines.is_empty()
            || self.cursor_column
                == self.lines[self.current_row as usize].text.get_ref().width() as u16
        {
            return Ok(AppReturn::Continue);
        } else if (self.cursor_column + 1
            == self.lines[self.current_row as usize].text.get_ref().width() as u16)
            && (self.current_row as usize != self.lines.len() - 1)
        {
            self.current_row += 1;
            self.cursor_column = 0
        } else {
            self.cursor_column += 1
        }
        Ok(AppReturn::Continue)
    }

    pub fn previous_char(&mut self) -> Result<AppReturn> {
        if (self.cursor_column == 0) && (self.current_row > 0) {
            self.current_row -= 1;
            self.cursor_column = self.lines[self.current_row as usize].text.get_ref().width() as u16
        } else if self.cursor_column > 0 {
            self.cursor_column -= 1
        }
        Ok(AppReturn::Continue)
    }

    pub fn backspace(&mut self) -> Result<AppReturn> {
        debug!("Backspace entered. Input Before: {:?}", self);
        match self.lines[self.current_row as usize]
            .text
            .get_ref()
            .is_empty()
        {
            true => {
                self.up_row()?;
                // Pop newline character
                self.pop();
            }
            false => {
                self.lines[self.current_row as usize]
                    .text
                    .get_mut()
                    .remove((self.cursor_column - 1) as usize);
                self.cursor_column -= 1
            }
        };
        debug!("Input After: {:?}", self);
        Ok(AppReturn::Continue)
    }

    pub fn clear(&mut self) -> Result<AppReturn> {
        let lines = Vec::<Line>::new();
        self.lines = lines;
        self.current_row = 0;
        self.cursor_column = 0;
        Ok(AppReturn::Continue)
    }

    pub fn tab(&mut self) -> Result<AppReturn> {
        self.append_char('\t')
    }

    fn new_line(&mut self) {
        if self.cursor_is_at_line_beginning() {
            debug!("Cursor at line beginning");
            let line_empty = self.lines[self.current_row as usize]
                .text
                .get_ref()
                .is_empty();

            let line = if line_empty {
                Line::default()
            } else {
                let text = self.lines[self.current_row as usize].text.get_ref().clone();
                Line::new(text)
            };

            self.lines[self.current_row as usize] = Line::new(String::from('\n'));
            if self.on_last_line() {
                self.lines.push(line);
            } else {
                self.lines.insert((self.current_row + 1) as usize, line)
            }
            self.current_row += 1;
            self.cursor_column = 0;
            debug!("Lines: {:?}", self.lines);
        } else if self.cursor_is_at_line_end() {
            debug!("Cursor at line end");
            self.lines[self.current_row as usize]
                .text
                .get_mut()
                .push('\n');
            let line = Line::default();
            if self.on_last_line() {
                self.lines.push(line);
            } else {
                self.lines.insert((self.current_row + 1) as usize, line)
            }
            self.current_row += 1;
            self.cursor_column = 0;
        } else if self.cursor_is_in_line_middle() {
            debug!("Cursor in middle of line");
            let new_line: String = self.lines[self.current_row as usize]
                .text
                .get_mut()
                .drain((self.cursor_column as usize)..)
                .collect();
            self.lines[self.current_row as usize]
                .text
                .get_mut()
                .push('\n');

            debug!("New line: {}", new_line);
            let line = Line::new(new_line);
            if self.on_last_line() {
                self.lines.push(line);
            } else {
                self.lines.insert((self.current_row + 1) as usize, line)
            }
            self.current_row += 1;
            self.cursor_column = 0;
        } else {
            debug!("Unhandled")
        }
    }

    fn cursor_is_at_line_end(&self) -> bool {
        let len = self.lines[self.current_row as usize].text.get_ref().len();
        self.cursor_column as usize == len
    }

    fn cursor_is_at_line_beginning(&self) -> bool {
        self.cursor_column == 0
    }

    fn cursor_is_in_line_middle(&self) -> bool {
        let len = self.lines[self.current_row as usize].text.get_ref().len();
        (self.cursor_column > 0) && ((self.cursor_column as usize) < len)
    }

    fn on_last_line(&self) -> bool {
        let res = self.current_row as usize == self.lines.len() - 1;
        debug!("On last line: {}", res);
        res
    }
}

/// The entire editor and it's state
pub struct Editor {
    /// Current value of the input box
    pub input: Input,
    /// Flag if SQL statement was terminated with ';'
    pub sql_terminated: bool,
    /// History of QueryResultMeta
    pub history: Vec<QueryResultsMeta>,
}
impl Default for Editor {
    fn default() -> Editor {
        let input = Input::default();
        Editor {
            input,
            history: Vec::new(),
            sql_terminated: false,
        }
    }
}

impl Editor {
    pub fn get_cursor_row(&self) -> u16 {
        if self.input.current_row < MAX_EDITOR_LINES {
            self.input.current_row
        } else {
            MAX_EDITOR_LINES
        }
    }

    pub fn get_cursor_column(&self) -> u16 {
        self.input.cursor_column
    }

    pub fn load_file(&mut self, file: File) -> Result<()> {
        let buf = BufReader::new(file);
        let mut lines = Vec::new();
        for line in buf.lines() {
            let mut line = line?;
            debug!("Line: {}", line);
            line.push('\n');
            let line = line.replace('\t', "    ");
            lines.push(Line::new(line));
        }
        self.input.lines = lines;
        Ok(())
    }
}
