use std::{fs, io, time::{self, Duration}};
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, KeyboardEnhancementFlags, PopKeyboardEnhancementFlags, PushKeyboardEnhancementFlags},
    execute,
};

mod timing;
use timing::StopWatchReadout;
mod text_gen;
mod prng;
use prng::Lcrng;
mod persist;
mod rendering; 

const WORD_HISTORY_LENGTH: usize = 10;
const AVERAGE_SPEED_BUFFER_SIZE: usize = 15_000;
// const MAX_DEBUG_MESSAGES: usize = 20;
const SAVE_INTERVAL_SECONDS: u64 = 1;
type NanoSeconds = u128;

fn main() -> Result<(), io::Error> {
    
    // Initialise the terminal 
    let mut terminal = ratatui::init();
    execute!(
        std::io::stdout(),
        PushKeyboardEnhancementFlags(
            KeyboardEnhancementFlags::REPORT_EVENT_TYPES
        )
    )?;
    
    // Debug Messages
    let mut debug_messages = Vec::new();
    
    // Save data related state
    let (mut session_record, mut text_generation_settings) = persist::load(&mut debug_messages);
    let total_time_today_at_the_start_of_the_session = session_record.total_time_today;
    let mut current_average_speed = 0.0;
    let mut average_speed_rolling_buffer = [0.0f64; AVERAGE_SPEED_BUFFER_SIZE];
    let mut average_speed_rolling_buffer_idx = 0usize;
    
    // Text generation
    let mut prng = Lcrng::new(prng::seed_from_time_now());
    let (mut typing_prompt_text, mut maybe_closing_char, mut line_indices) = text_gen::generate_typing_prompt(&mut prng, None, &text_generation_settings);

    // Character state
    let mut correct_character_count: usize = 0;
    let mut wrong_character_count: usize = 0;
    let mut letters: Vec<char> = typing_prompt_text.chars().collect();
    let mut text_length = letters.len();
    let mut next_character = letters[correct_character_count];
    let mut last_key_press: KeyEvent = KeyEvent::new(KeyCode::Null, KeyModifiers::NONE);

    // Time-keeping state
    let mut last_time = time::Instant::now();
    let mut interval_rolling_buffer: [NanoSeconds; WORD_HISTORY_LENGTH*5] = [0; WORD_HISTORY_LENGTH*5];
    let mut current_interval_history_idx: usize = 0;
    let mut is_first_pass = true;
    let start_time = time::Instant::now();
    let mut frame_time = Duration::new(0,0);
    let mut fps = 0f64;
    let mut cumulative_frame_time = Duration::new(0,0);
    let date_at_start_of_session = persist::get_todays_date_in_dhaka_time();

    rendering::draw_to_terminal(
        &mut terminal, 
        &typing_prompt_text,  
        correct_character_count, 
        wrong_character_count, 
        current_average_speed,
        session_record,
        frame_time,
        fps,
        line_indices.clone(),
        last_key_press,
        &debug_messages,
    )?;

    'app: loop {
        let debug_frame_start_time = time::Instant::now();

        // Check if event exists
        if event::poll(Duration::from_nanos(1000))? { 
            if let Event::Key(key) = event::read()? {
                last_key_press = key;
                match key {
                    // Esc quits the program
                    KeyEvent { code: KeyCode::Esc, modifiers: _, kind: KeyEventKind::Release, state: _ } => {
                        ratatui::restore();
                        execute!(
                            std::io::stdout(),
                            PopKeyboardEnhancementFlags
                        )?;
                        break 'app;
                    },
                    // Backspace deletes characters, entire words if Ctrl modifier is on
                    KeyEvent { code: KeyCode::Backspace, modifiers, kind: KeyEventKind::Release, state: _ } => {
                        let currently_committed_characters = correct_character_count + wrong_character_count;
                        let backspace_count;
                        if currently_committed_characters != 0 {
                            // we want to back up til the previous space character ('_' actually), to do so we iterate backwards until we find it.
                            let mut index = 0;
                            if modifiers.contains(KeyModifiers::CONTROL) {
                                for (i, chr) in letters[..currently_committed_characters].iter().enumerate().rev() {
                                    if *chr == '_' {
                                        index = i;
                                        break;
                                    }
                                } 
                                if index != 0 {
                                    backspace_count = (currently_committed_characters - index - 1).max(1);
                                } else {
                                    backspace_count = currently_committed_characters;
                                }
                            } else {
                                backspace_count = 1;
                            }
                            if backspace_count > wrong_character_count {
                                correct_character_count = correct_character_count.saturating_sub(backspace_count - wrong_character_count);
                            }
                            wrong_character_count = wrong_character_count.saturating_sub(backspace_count);
                        } 
                    },
                    // Insert allows one to regenerate the text on-screen
                    KeyEvent { code: KeyCode::Insert, modifiers: _, kind: KeyEventKind::Release, state: _ } => {
                        // Regenerate text
                        (typing_prompt_text, maybe_closing_char, line_indices) = text_gen::generate_typing_prompt(
                            &mut prng, 
                            maybe_closing_char, 
                            &text_generation_settings
                        );
                        correct_character_count = 0;
                        wrong_character_count = 0;
                        letters = typing_prompt_text.chars().collect();
                        text_length = letters.len();
                        is_first_pass = true;
                    }
                    // Main key presses for the typing
                    KeyEvent { code: KeyCode::Char(input), modifiers, kind: KeyEventKind::Release, state: _ } => {
                        match modifiers {
                            KeyModifiers::ALT => {
                                match input {
                                    // Toggle number generation
                                    'n' => { text_generation_settings.contains_numbers.toggle(); },
                                    // Toggle suffix characters
                                    's' => { text_generation_settings.contains_suffixes.toggle(); },
                                    // Toggle capital letters
                                    'c' => { text_generation_settings.contains_capital_letters.toggle(); },
                                    // Toggle enclosers (like brackets)
                                    'e' => { text_generation_settings.contains_enclosers.toggle(); },
                                    // Reset the current screeen
                                    'r' => { 
                                        fs::write(persist::TEMP_FILE_NAME, "").expect("File should be successfully written to.");
                                        if fs::read(persist::get_save_file_name_with_appropriate_extension(text_generation_settings)).is_ok() {
                                            (session_record, _) = persist::load(&mut debug_messages);
                                        }
                                    }
                                     _ => {}
                                }
                                // Regenerate text
                                (typing_prompt_text, maybe_closing_char, line_indices) = text_gen::generate_typing_prompt(
                                    &mut prng, 
                                    maybe_closing_char, 
                                    &text_generation_settings
                                );
                                correct_character_count = 0;
                                wrong_character_count = 0;
                                letters = typing_prompt_text.chars().collect();
                                text_length = letters.len();
                                is_first_pass = true;
                            },
                            KeyModifiers::NONE | KeyModifiers::SHIFT => {
                            if ((input == ' ' && next_character == '_') || (modifiers == KeyModifiers::SHIFT && input == next_character.to_ascii_lowercase()) || input == next_character) && wrong_character_count == 0 {
                                    correct_character_count += 1;
                                    correct_character_count = correct_character_count.min(text_length);
                                    let time_since_last_character;
                                    if is_first_pass {
                                        last_time = time::Instant::now();
                                        is_first_pass = false;
                                    } else {
                                        let this_time = time::Instant::now();
                                        time_since_last_character = this_time - last_time;
                                        last_time = this_time;
                                        interval_rolling_buffer[current_interval_history_idx] = time_since_last_character.as_nanos();
                                        current_interval_history_idx = (current_interval_history_idx + 1) % (WORD_HISTORY_LENGTH*5);
                                    }
                                } else if correct_character_count+wrong_character_count < text_length {
                                        wrong_character_count += 1
                                }
                            },
                            _ => {}
                        }
                    },
                    _ => {}
                }
            };

        }
        
        session_record.total_time_today = total_time_today_at_the_start_of_the_session + StopWatchReadout::from_duration(time::Instant::now() - start_time);
        if is_interval_history_full(interval_rolling_buffer) {
            let started_typing = correct_character_count > 0;
            if started_typing { 
                // Update the current and weighted average speed
                current_average_speed = calculate_average_speed(interval_rolling_buffer);
                average_speed_rolling_buffer[average_speed_rolling_buffer_idx] = current_average_speed;
                session_record.weighted_average_speed = calculate_weighted_average_speed(average_speed_rolling_buffer);
                average_speed_rolling_buffer_idx = (average_speed_rolling_buffer_idx + 1) % AVERAGE_SPEED_BUFFER_SIZE;
            }
            if current_average_speed >= session_record.all_time_best_speed { 
                session_record.all_time_best_speed = current_average_speed;
            }
        }

        rendering::draw_to_terminal(
            &mut terminal, 
            &typing_prompt_text, 
            correct_character_count, 
            wrong_character_count, 
            current_average_speed,
            session_record,
            frame_time,
            fps,
            line_indices.clone(),
            last_key_press,
            &debug_messages
        )?;
        
        if correct_character_count < text_length {
            next_character = letters[correct_character_count];
        } else {
            persist::save(session_record, text_generation_settings, date_at_start_of_session);
            // Regenerate text
            (typing_prompt_text, maybe_closing_char, line_indices) = text_gen::generate_typing_prompt(
                &mut prng, 
                maybe_closing_char, 
                &text_generation_settings
            );
            correct_character_count = 0;
            wrong_character_count = 0;
            letters = typing_prompt_text.chars().collect();
            text_length = letters.len();
            next_character = letters[correct_character_count];
            is_first_pass = true;
        }

        // End of frame stuff

        let debug_frame_end_time = time::Instant::now();
        frame_time = debug_frame_end_time - debug_frame_start_time;
        cumulative_frame_time += frame_time;
        fps = 10u128.pow(9) as f64 / frame_time.as_nanos() as f64;
        if cumulative_frame_time.as_secs() >= SAVE_INTERVAL_SECONDS {
            persist::temporary_save(session_record, text_generation_settings);
            cumulative_frame_time = Duration::new(0,0);
        }
    }
    
    Ok(())    
}

trait Toggleable {
    fn toggle(&mut self);
}

impl Toggleable for bool {
    fn toggle(&mut self) {
        *self ^= true; 
    }
}

fn _debug_string<T: std::fmt::Debug>(var: T, var_name: &str) -> String {
    format!("[{}:{}] {} = {:?}",std::file!(), std::line!(), var_name, &var)
}

fn is_interval_history_full(interval_history: [NanoSeconds; WORD_HISTORY_LENGTH*5]) -> bool {
    !(interval_history.contains(&0u128))
}

const NANOSECONDS_PER_MINUTE: f64 = 60.0e+9;

fn calculate_average_speed(interval_rolling_buffer: [NanoSeconds; WORD_HISTORY_LENGTH*5]) -> f64 {
    let nonzero_character_count = interval_rolling_buffer
        .iter()
        .filter(|it| {**it != 0})
        .count() as f64;
    let total = interval_rolling_buffer
        .iter()
        .sum::<u128>()
        .max(1) as f64;
    ( NANOSECONDS_PER_MINUTE * nonzero_character_count) / (total * 5.0)
}

fn calculate_weighted_average_speed(average_speed_rolling_buffer: [f64; AVERAGE_SPEED_BUFFER_SIZE]) -> f64 {
    let nonzero_character_count = average_speed_rolling_buffer
        .iter()
        .filter(|it| {**it != 0.0})
        .count() as f64;
    let total = average_speed_rolling_buffer
        .iter()
        .sum::<f64>()
        .max(1.0);
    total / (nonzero_character_count)
}

#[test]
fn weighted_average() {
    let mut dummy_rolling_buffer = [0.0; AVERAGE_SPEED_BUFFER_SIZE];
    dummy_rolling_buffer[0] = 50.1;
    dummy_rolling_buffer[1] = 50.0;
    dummy_rolling_buffer[2] = 49.9;
    dummy_rolling_buffer[3] = 50.4;
    dummy_rolling_buffer[4] = 51.4;
    dummy_rolling_buffer[5] = 32.1;
    dummy_rolling_buffer[6] = 34.0;
    dummy_rolling_buffer[7] = 50.2;
    dummy_rolling_buffer[8] = 50.5;
    dummy_rolling_buffer[11] = 49.9;
    dummy_rolling_buffer[99] = 50.0;
    dummy_rolling_buffer[100] = 50.1;
    dummy_rolling_buffer[101] = 50.12;
    dummy_rolling_buffer[102] = 50.9;
    dummy_rolling_buffer[103] = 51.9;
    dummy_rolling_buffer[104] = 50.3;
    dummy_rolling_buffer[105] = 77.0;
    dummy_rolling_buffer[106] = 50.0;
    dummy_rolling_buffer[117] = 48.0;
    assert_eq!(calculate_weighted_average_speed(dummy_rolling_buffer), 49.832631578947364);
}
