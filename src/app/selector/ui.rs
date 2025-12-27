use std::cmp::min;
use std::io::{self, stderr};

use crossterm::terminal::Clear;
use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{Event, KeyCode, KeyEvent, KeyModifiers, read},
    execute,
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen, enable_raw_mode, size},
};

use crate::app::selector::core::{MatchResult, Matcher};

fn draw_outline(cols: u16, rows: u16) -> io::Result<()> {
    execute!(
        stderr(),
        SetForegroundColor(Color::Rgb {
            r: 100,
            g: 100,
            b: 100
        })
    )?;

    execute!(stderr(), MoveTo(0, 0), Print("╭"))?;
    for _ in 1..cols - 1 {
        execute!(stderr(), Print("─"))?;
    }
    execute!(stderr(), MoveTo(cols - 1, 0), Print("╮"))?;

    for c in 1..rows - 1 {
        execute!(stderr(), MoveTo(cols - 1, c), Print("│"))?;
    }
    execute!(stderr(), MoveTo(cols - 1, rows - 1), Print("╯"))?;

    for c in 1..rows - 1 {
        execute!(stderr(), MoveTo(0, c), Print("│"))?;
    }
    execute!(stderr(), MoveTo(0, rows - 1), Print("╰"))?;

    for _ in 1..cols - 1 {
        execute!(stderr(), Print("─"))?;
    }

    execute!(stderr(), ResetColor)?;

    Ok(())
}

fn draw_input(cols: u16, input: &str) -> io::Result<()> {
    execute!(
        stderr(),
        MoveTo(2, 1),
        SetForegroundColor(Color::Blue),
        Print("> "),
        ResetColor,
        Print(input),
        Print("█"),
    )?;

    execute!(
        stderr(),
        MoveTo(4, 2),
        SetForegroundColor(Color::Rgb {
            r: 100,
            g: 100,
            b: 100
        })
    )?;

    for _ in 4..cols - 3 {
        execute!(stderr(), Print("─"),)?;
    }

    execute!(stderr(), ResetColor)?;

    Ok(())
}

fn draw_items(selected_index: usize, results: &[&MatchResult]) -> io::Result<()> {
    for (r_i, result) in results.iter().enumerate() {
        let is_selected = r_i == selected_index;
        let mut line = String::new();

        for (c_i, char) in result.item.char_indices() {
            if result.hits.contains(&c_i) {
                line.push_str("\x1b[32m");
            } else {
                line.push_str("\x1b[0m");
            }

            if is_selected {
                line.push_str("\x1b[40m");
            }

            line.push(char);
        }
        line.push_str("\x1b[0m");

        if is_selected {
            execute!(
                stderr(),
                MoveTo(2, (r_i + 3) as u16),
                SetForegroundColor(Color::Red),
                Print("█"),
                ResetColor
            )?;
        }
        execute!(stderr(), MoveTo(4, (r_i + 3) as u16), Print(line),)?;
    }
    Ok(())
}

pub fn select(matcher: Matcher) -> io::Result<Option<String>> {
    enable_raw_mode()?;
    execute!(stderr(), EnterAlternateScreen, Hide)?;

    let (cols, rows) = size()?;

    let mut input = String::new();
    let mut selected_index = 0;
    let mut start_pos = 0;

    let max_items = rows as usize - 4;
    let scroll_next_pos = rows as usize - 11;
    let scroll_previous_pos = 5;

    let select = loop {
        execute!(stderr(), Clear(terminal::ClearType::All))?;

        draw_outline(cols, rows)?;
        draw_input(cols, &input)?;

        let result = matcher.fuzzy_match(&input);
        let items: Vec<_> = result.iter().skip(start_pos).take(max_items).collect();
        draw_items(selected_index, &items[..])?;

        let all_items = result.len();

        match read()? {
            Event::Key(KeyEvent {
                code: KeyCode::Esc, ..
            }) => break None,
            Event::Key(KeyEvent {
                code: KeyCode::Char('c'),
                modifiers: KeyModifiers::CONTROL,
                ..
            }) => break None,
            Event::Key(KeyEvent {
                code: KeyCode::Char('p'),
                modifiers: KeyModifiers::CONTROL,
                ..
            }) => {
                if items.is_empty() {
                    continue;
                }

                if selected_index == 0 {
                    selected_index = min(max_items, all_items) - 1;
                    start_pos = all_items.saturating_sub(max_items);
                    continue;
                }

                if start_pos == 0 {
                    selected_index -= 1;
                    continue;
                }

                if selected_index > scroll_previous_pos {
                    selected_index -= 1;
                } else {
                    start_pos -= 1;
                }
            }
            Event::Key(KeyEvent {
                code: KeyCode::Char('n'),
                modifiers: KeyModifiers::CONTROL,
                ..
            }) => {
                if items.is_empty() {
                    continue;
                }

                if selected_index == items.len() - 1 {
                    selected_index = 0;
                    start_pos = 0;
                    continue;
                }
                if all_items.saturating_sub(max_items) == start_pos {
                    selected_index += 1;
                    continue;
                }

                if selected_index <= scroll_next_pos {
                    selected_index += 1;
                } else {
                    start_pos += 1;
                }
            }
            Event::Key(KeyEvent {
                code: KeyCode::Backspace,
                ..
            }) => {
                input.pop();
                selected_index = 0;
                start_pos = 0;
            }
            Event::Key(KeyEvent {
                code: KeyCode::Char(c),
                ..
            }) => {
                input.push(c);
                selected_index = 0;
                start_pos = 0;
            }
            Event::Key(KeyEvent {
                code: KeyCode::Enter,
                ..
            }) => {
                if items.is_empty() {
                    break None;
                }
                break Some(items[selected_index].item.clone());
            }
            _ => continue,
        }
    };

    terminal::disable_raw_mode()?;
    execute!(stderr(), LeaveAlternateScreen, Show)?;

    Ok(select)
}
