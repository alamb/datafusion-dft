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

// Enumeration of the possibilities for each key stroke
// Key type => Letter or special character
// Cursor location => Beginning of line, middle of line, end of line
// Text around cursor => Before cursor, on cursor, after cursor
// Lines around cursor => First line, middle line, last line

use log::debug;

use crate::app::core::{App, AppReturn, InputMode};
use crate::app::error::Result;
use crate::events::Key;

pub async fn edit_mode_handler<'logs>(app: &mut App<'logs>, key: Key) -> Result<AppReturn> {
    debug!(
        "{} Entered, current row / col: {} / {}",
        key, app.editor.input.current_row, app.editor.input.cursor_column
    );
    match key {
        Key::Enter => app.editor.input.append_char('\n'),
        Key::Char(c) => match c {
            ';' => {
                let result = app.editor.input.append_char(c);
                app.editor.sql_terminated = true;
                result
            }
            _ => app.editor.input.append_char(c),
        },
        Key::Left => app.editor.input.previous_char(),
        Key::Right => app.editor.input.next_char(),
        Key::Up => app.editor.input.up_row(),
        Key::Down => app.editor.input.down_row(),
        Key::Tab => app.editor.input.tab(),
        Key::Backspace => app.editor.input.backspace(),
        Key::Esc => {
            app.input_mode = InputMode::Normal;
            Ok(AppReturn::Continue)
        }
        _ => Ok(AppReturn::Continue),
    }
}
