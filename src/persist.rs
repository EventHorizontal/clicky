use std::fs;
use chrono::{NaiveDate, Timelike};

use crate::timing::StopWatchReadout;
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct SessionRecord {
    pub all_time_best_speed: f64,
    pub weighted_average_speed: f64,
    pub total_time_today: StopWatchReadout,
    pub date: NaiveDate,
}

impl SessionRecord {
    fn new() -> Self {
        SessionRecord {
            all_time_best_speed: 0.0,
            weighted_average_speed: 0.0,
            total_time_today: StopWatchReadout::new(),
            date: get_todays_date_in_dhaka_time(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct TextGenerationSettings {
    pub contains_numbers: bool,
    pub contains_suffixes: bool,
    pub contains_enclosers: bool,
    pub contains_capital_letters: bool,
}

impl TextGenerationSettings {
    pub fn default() -> Self { 
        Self { 
            contains_numbers: true, 
            contains_suffixes: true, 
            contains_enclosers: true,
            contains_capital_letters: true,
        } 
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SaveData {
    session_records: Vec<SessionRecord>, 
    text_generation_settings: TextGenerationSettings
}

/// Returns date in DD-MM-YYYY format
pub fn get_todays_date_in_dhaka_time() -> NaiveDate {
    use chrono_tz::Asia::Dhaka;
    let date_time = chrono::Utc::now();
    let dhaka_time = date_time.with_timezone(&Dhaka);
    dhaka_time.date_naive()
}


fn has_day_rolled_over(date_at_start_of_session: NaiveDate) -> bool {
    date_at_start_of_session != get_todays_date_in_dhaka_time()
}

#[test]
fn test_current_date() {
    println!("{:?}", get_todays_date_in_dhaka_time());
}

// pub const SAVE_FILE_NAME: &str = "clicky_save_data.json";
pub const TEMP_FILE_NAME: &str = "clicky_temp_data.json";

pub fn save(session_record: SessionRecord, text_generation_settings: TextGenerationSettings, date_at_start_of_session: NaiveDate) {
    //TODO: What to do if the day rolls over to the next mid-session?
    let file_name = get_save_file_name_with_appropriate_extension(text_generation_settings);
    match fs::read_to_string(&file_name) {
        Ok(data) => {
            let mut save_data: SaveData = serde_json::from_str(data.as_str()).expect("parsing should be successful.");
            { // scope for limiting references inside savedata
                let saved_session_records = &mut save_data.session_records; 
                let data_length = saved_session_records.len() - 1;
                let last_session_record = &mut saved_session_records[data_length];
                if last_session_record.date == get_todays_date_in_dhaka_time() && !has_day_rolled_over(date_at_start_of_session) {
                    // Overwrite the last session if it is on the same day
                    last_session_record.total_time_today = session_record.total_time_today;
                    if session_record.all_time_best_speed != 0.0 {
                        last_session_record.all_time_best_speed = session_record.all_time_best_speed; 
                    }
                    if session_record.weighted_average_speed != 0.0 {
                        last_session_record.weighted_average_speed = session_record.weighted_average_speed; 
                    }
                } else if has_day_rolled_over(date_at_start_of_session) {
                    // Push entry for tomorrow
                    let mut new_session_record = session_record;
                    new_session_record.date = get_todays_date_in_dhaka_time();
                    new_session_record.total_time_today = StopWatchReadout::from_millis(chrono::Utc::now().num_seconds_from_midnight() as u128 * 1000);
                    saved_session_records.push(new_session_record);
                } else {
                    // Push new entry
                    saved_session_records.push(session_record);
                }
            }
            save_data.text_generation_settings = text_generation_settings;
            let config_data = serde_json::to_string(&text_generation_settings).expect("TextGenerationSettings should be serializable.");
            fs::write("clicky_config.txt", config_data).expect("Config should be successfully written to.");
            let save_data = serde_json::to_string(&save_data).expect("SaveData should be serializable.");
            fs::write(file_name, save_data).expect("Save should be successfully written to.");
        },
        Err(_) => {
            let save_data = SaveData {
                session_records: vec![session_record],
                text_generation_settings,
            };
            let save_data = serde_json::to_string(&save_data).expect("The data should be serializable.");
            fs::write(file_name, save_data).expect("File should be successfully written to.");
        },
    }
}

pub fn get_save_file_name_with_appropriate_extension(text_generation_settings: TextGenerationSettings) -> String {
    let mut file_name = "clicky_save_data".to_string();
    if text_generation_settings.contains_numbers {
        file_name += "_num";
    }
    if text_generation_settings.contains_suffixes {
        file_name += "_suf"   
    }
    if text_generation_settings.contains_enclosers {
        file_name += "_enc"
    }
    if text_generation_settings.contains_capital_letters {
        file_name += "_cap"
    }
    file_name += ".json";
    file_name
}

pub fn temporary_save(session_record: SessionRecord, text_generation_settings: TextGenerationSettings) {
    let save_data = SaveData {
        session_records: vec![session_record],
        text_generation_settings,
    };
    let save_data = serde_json::to_string(&save_data).expect("The data should be serializable.");
    fs::write(TEMP_FILE_NAME, save_data).expect("File should be successfully written to.");
}

#[test]
fn save_data_format() {
    // save(
    //     SessionRecord { 
    //         all_time_best_speed: 1.1, 
    //         weighted_average_speed: 1.1, 
    //         total_time_today: StopWatchReadout::new(), 
    //         date: get_todays_date_in_dhaka_time() 
    //     }, 
    //     TextGenerationSettings {
    //             contains_numbers: false,
    //             contains_suffixes: true,
    //             contains_enclosers: false,
    //             contains_capital_letters: true,
    //         })
}

pub fn load(debug_messages: &mut Vec<String>) -> (SessionRecord, TextGenerationSettings) {
    let config = fs::read_to_string("clicky_config.txt").unwrap_or("".to_string());
    let text_generation_settings: TextGenerationSettings = serde_json::from_str(&config).unwrap_or(TextGenerationSettings::default());
    let file_name = get_save_file_name_with_appropriate_extension(text_generation_settings);
    debug_messages.push(format!["loading {}", file_name]);
    if let Ok(data) = fs::read_to_string(file_name) {
            let mut save_data: SaveData = serde_json::from_str(data.as_str()).expect("parsing should be successful.");
            let maybe_last_session_record = save_data.session_records.pop();
            if let Some(last_session_record) = maybe_last_session_record {
                (
                    SessionRecord { 
                        all_time_best_speed: last_session_record.all_time_best_speed, 
                        weighted_average_speed: last_session_record.weighted_average_speed, 
                        total_time_today: {
                            if last_session_record.date == get_todays_date_in_dhaka_time() {
                                last_session_record.total_time_today
                            } else {
                                StopWatchReadout::new()
                            }
                        }, 
                        date: get_todays_date_in_dhaka_time(),
                    },
                    save_data.text_generation_settings
                )
            } else {
                (SessionRecord::new(), TextGenerationSettings::default()) // Empty file
            }
        } else {
            (SessionRecord::new(), TextGenerationSettings::default()) // No file
        }
}

