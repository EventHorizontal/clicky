use std::{io::{Error, Stdout}, time::Duration};

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{backend::CrosstermBackend, layout::{Alignment, Constraint, Direction, Layout}, style::{Color, Style}, symbols::line::VERTICAL, text::{Line, Span, Text}, widgets::{Block, Borders, Paragraph, Tabs, Wrap}, CompletedFrame, Terminal};

use crate::persist::SessionRecord;

const EMPTY_STRING: String = String::new();

#[allow(clippy::too_many_arguments)]
pub(crate) fn draw_to_terminal<'a>(
    terminal: &'a mut Terminal<CrosstermBackend<Stdout>>,
    typing_prompt_text: &str,
    correct_character_count: usize,
    wrong_character_count: usize,
    current_average_speed: f64,
    session_record: SessionRecord,
    frame_time: Duration,
    fps: f64,
    line_indices: Vec<usize>,
    last_key_press: KeyEvent,
    debug_messages: &Vec<String>,
) -> Result<CompletedFrame<'a>, Error> {
    terminal.draw(|f| {        
        
        let chunks = if cfg!(debug_assertions) {
           Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints(
                    [Constraint::Length(4), Constraint::Min(3), Constraint::Length(3)].as_ref()
                )
                .split(f.area())
        } else {
            Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints(
                    [Constraint::Length(4), Constraint::Min(3)].as_ref()
                )
                .split(f.area())
        };
        
        // Bottom Bar widget
        let items = Span::raw(
            format![ "Current Average Speed: {0:.2} | Weighted Average Speed: {1:.2} | Total Time Today: {2} | All Time Best Speed: {3:.2} | FPS: {4:7.2} | Frame time: {5:5.2}ms", current_average_speed, session_record.weighted_average_speed, session_record.total_time_today, session_record.all_time_best_speed, fps, frame_time.as_nanos() as f64 / 10u128.pow(6) as f64]);
        let stats = Paragraph::new(items)
            .block(Block::default().title(" Statistics ")
            .borders(Borders::ALL))
            .style(
                Style::default()
                .fg(Color::White)
                .bg(Color::Black)
            )
            .wrap(Wrap { trim: false } );
        f.render_widget(stats, chunks[0]);

        // Paragraph widget

        // 1. Split the text into lines and record how many correct, wrong and untyped words there are on each line
        let mut lines = Vec::new();
        let mut previous_index = 0;
        let mut correct_characters_left = correct_character_count;
        let mut wrong_charcters_left = wrong_character_count;
        for current_index in line_indices {
            let line = &typing_prompt_text[previous_index..current_index];
            let line_length = line.len();
            let correct_characters_on_line = correct_characters_left.min(line_length); 
            let untyped_characters = line_length - correct_characters_on_line;
            let wrong_characters_on_line = (untyped_characters).min(wrong_charcters_left);
            correct_characters_left -= correct_characters_on_line;
            wrong_charcters_left -= wrong_characters_on_line;
            lines.push((line, correct_characters_on_line, wrong_characters_on_line));
            previous_index = current_index;
        }

        
        // 2. Turn the lines into Line.
        let mut formatted_text = Vec::new();
        for (line, correct_characters_on_line, wrong_characters_on_line) in lines.into_iter() {
            let spans_for_line = Line::from(vec![
                Span::styled(&line[0..correct_characters_on_line], Style::default().fg(Color::Green)), 
                Span::styled(&line[correct_characters_on_line..correct_characters_on_line + wrong_characters_on_line], Style::default().fg(Color::Red)), 
                Span::raw(&line[correct_characters_on_line + wrong_characters_on_line..]),
                ]
            );
            formatted_text.push(spans_for_line);
        }
        formatted_text.push(Line::from(Span::raw("")));
        if let KeyEvent { code: KeyCode::Char(input), modifiers, kind: KeyEventKind::Release, state: _ } = last_key_press {
            formatted_text.push(
                Line::from(
                    Span::styled(if modifiers == KeyModifiers::NONE {
                        format!["Last Key Pressed: {}", input]
                    } else {
                        format!["Last Key Pressed: {:?} + {}", modifiers, input]
                    }, Style::default().fg(Color::Blue))
                )
            );
        } else {
            formatted_text.push(
                Line::from(
                    Span::styled("Last Key Pressed: None", Style::default().fg(Color::Blue))
                )
            );
        }
        
        // 3. Construct the paragraph block 
        let paragraph = Paragraph::new(formatted_text)
            .block(Block::default()
            .title(" Typing Text ")
            .borders(Borders::ALL))
            .style(
                Style::default()
                .fg(Color::White)
                .bg(Color::Black)
            )
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: false });
        f.render_widget(paragraph, chunks[1]);

        // 4. Construct the Debug Message box if building in Debug Mode 
        let empty = &EMPTY_STRING;
        if cfg!(debug_assertions) {
            let paragraph = Paragraph::new(Span::raw(debug_messages.last().unwrap_or(empty)))
                .block(Block::default()
                .title(" Debug Output ")
                .borders(Borders::ALL))
                .style(
                    Style::default()
                    .fg(Color::White)
                    .bg(Color::Black)
                )
                .alignment(Alignment::Center)
                .wrap(Wrap { trim: false });
            f.render_widget(paragraph, chunks[2]);
        }
    })
}
