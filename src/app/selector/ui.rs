use std::cmp::min;
use std::io::{self, Write, stderr};
use std::sync::mpsc::{self, Receiver, Sender, TryRecvError};
use std::thread;
use std::time::Duration;

use crossterm::event::poll;
use crossterm::terminal::Clear;
use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{Event, KeyCode, KeyEvent, KeyModifiers, read},
    execute,
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen, enable_raw_mode, size},
};

use crate::app::selector::core::{MatchResult, Matcher};

fn draw_outline(stderr: &mut impl Write, cols: u16, rows: u16) -> io::Result<()> {
    execute!(
        stderr,
        SetForegroundColor(Color::Rgb {
            r: 100,
            g: 100,
            b: 100
        })
    )?;

    execute!(stderr, MoveTo(0, 0), Print("╭"))?;
    for _ in 1..cols - 1 {
        execute!(stderr, Print("─"))?;
    }
    execute!(stderr, MoveTo(cols - 1, 0), Print("╮"))?;

    for c in 1..rows - 1 {
        execute!(stderr, MoveTo(cols - 1, c), Print("│"))?;
    }
    execute!(stderr, MoveTo(cols - 1, rows - 1), Print("╯"))?;

    for c in 1..rows - 1 {
        execute!(stderr, MoveTo(0, c), Print("│"))?;
    }
    execute!(stderr, MoveTo(0, rows - 1), Print("╰"))?;

    for _ in 1..cols - 1 {
        execute!(stderr, Print("─"))?;
    }

    execute!(stderr, ResetColor)?;

    Ok(())
}

fn draw_count(stderr: &mut impl Write, match_count: usize, items_count: usize) -> io::Result<()> {
    execute!(
        stderr,
        SetForegroundColor(Color::Rgb {
            r: 110,
            g: 110,
            b: 110
        }),
        MoveTo(4, 2),
        Print(format!("{}/{}", match_count, items_count)),
    )?;

    execute!(stderr, ResetColor)?;

    Ok(())
}

fn draw_input(stderr: &mut impl Write, input: &str) -> io::Result<()> {
    execute!(
        stderr,
        MoveTo(2, 1),
        SetForegroundColor(Color::Blue),
        Print("> "),
        ResetColor,
        Print(input),
        Print("█"),
    )?;

    execute!(stderr, ResetColor)?;

    Ok(())
}

fn draw_items(
    stderr: &mut impl Write,
    selected_index: usize,
    results: &[&MatchResult],
) -> io::Result<()> {
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
                stderr,
                MoveTo(2, (r_i + 3) as u16),
                SetForegroundColor(Color::Red),
                Print("█"),
                ResetColor
            )?;
        }
        execute!(stderr, MoveTo(4, (r_i + 3) as u16), Print(line),)?;
    }
    Ok(())
}

pub fn select(mut matcher: Matcher) -> io::Result<Option<String>> {
    enable_raw_mode()?;
    execute!(stderr(), EnterAlternateScreen, Hide)?;

    let (cols, rows) = size()?;

    let mut input = String::new();
    let mut selected_index = 0;
    let mut start_pos = 0;

    let max_items = rows as usize - 4;
    let all_items = matcher.items.len();
    let scroll_next_pos = rows as usize - 11;
    let scroll_previous_pos = 5;

    let (query_tx, query_rx): (Sender<String>, Receiver<String>) = mpsc::channel();
    let (result_tx, result_rx): (Sender<Vec<MatchResult>>, Receiver<Vec<MatchResult>>) =
        mpsc::channel();
    let mut result: Vec<MatchResult> = Vec::new();

    thread::spawn(move || {
        while let Ok(input) = query_rx.recv() {
            let result = matcher.fuzzy_match(&input);
            if result_tx.send(result).is_err() {
                break;
            }
        }
    });

    let _ = query_tx.send(input.clone());
    let mut needs_redraw = true;

    let select = loop {
        loop {
            match result_rx.try_recv() {
                Ok(new_result) => {
                    result = new_result;
                    selected_index = 0;
                    start_pos = 0;
                    needs_redraw = true;
                }
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => break,
            }
        }

        let items: Vec<_> = result.iter().skip(start_pos).take(max_items).collect();

        if needs_redraw {
            let mut stderr = stderr().lock();
            execute!(stderr, Clear(terminal::ClearType::All))?;
            draw_outline(&mut stderr, cols, rows)?;
            draw_input(&mut stderr, &input)?;
            draw_count(&mut stderr, result.len(), all_items)?;
            draw_items(&mut stderr, selected_index, &items[..])?;
            stderr.flush()?;
            needs_redraw = false;
        }

        if poll(Duration::from_millis(16))? {
            needs_redraw = true;
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
                        selected_index = min(max_items, result.len()) - 1;
                        start_pos = result.len().saturating_sub(max_items);
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
                    if result.len().saturating_sub(max_items) == start_pos {
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
                    let _ = query_tx.send(input.clone());
                }
                Event::Key(KeyEvent {
                    code: KeyCode::Char(c),
                    ..
                }) => {
                    input.push(c);
                    let _ = query_tx.send(input.clone());
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
        }
    };

    terminal::disable_raw_mode()?;
    execute!(stderr(), LeaveAlternateScreen, Show)?;

    Ok(select)
}
