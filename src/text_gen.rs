use std::cmp::Ordering;

use crate::{prng::Lcrng, persist::TextGenerationSettings};

mod data;
use data::{DATASET_TOTAL_CF, WORD_SET};


pub(crate) fn get_word_at(random_frequency_threshold: u64) -> &'static str {
    let word_index = WORD_SET
        .binary_search_by(|probe| {
            let upper_limit = probe.1;
            let lower_limit = probe.2;
            if upper_limit < random_frequency_threshold { 
                Ordering::Greater
            } else if lower_limit > random_frequency_threshold {
                Ordering::Less   
            } else if upper_limit >= random_frequency_threshold && lower_limit < random_frequency_threshold { 
                Ordering::Equal
            } else {
                panic!("Unexpected ordering.");
            }
        })
        .unwrap_or_else(|_| panic!("Expected random number within 1 and {}, but was given {}.", DATASET_TOTAL_CF, random_frequency_threshold));
    WORD_SET[word_index].0
}

const MAX_WORDS_PER_SCREEN: usize = 30;
const MAX_DIGITS_TO_GENERATE: u64 = 8;
const MAX_WORDS_PER_LINE: usize = 10;

const SPECIAL_SUFFIX_CHARS: [&str; 5] = [".", ",", ";", "?", "!"];

const OPENING_SPECIAL_CHARS: [&str; 5] = ["(", "[", "{", "\"", "<"];
const CLOSING_SPECIAL_CHARS: [&str; 5] = [")", "]", "}", "\"", ">"];

const WORD_OCCURENCE_PERCENT: u8 = 70;
const SUFFIX_OCCURENCE_PERCENT: u8 = 10;
const ENCLOSER_OCCURENCE_PERCENT: u8 = 15;
const NUMBER_OCCURENCE_PERCENT: u8 = 5;
const MIN_INTERVAL_BETWEEN_SPECIAL_CHARS: u64 = 5;

pub(crate) fn generate_typing_prompt(prng: &mut Lcrng, mut maybe_closing_char: Option<String>, text_gen_settings: &TextGenerationSettings) -> (String, Option<String>, Vec<usize>) {

    let suffix_occurence_percentage = if text_gen_settings.contains_suffixes { SUFFIX_OCCURENCE_PERCENT } else { 0 };
    let encloser_occurence_percentage = if text_gen_settings.contains_enclosers { ENCLOSER_OCCURENCE_PERCENT } else { 0 }; 
    let number_occurence_percentage = if text_gen_settings.contains_numbers { NUMBER_OCCURENCE_PERCENT } else { 0 };

    let word_cf = WORD_OCCURENCE_PERCENT;
    let suffix_cf = word_cf + suffix_occurence_percentage;
    let encloser_cf = suffix_cf + encloser_occurence_percentage;
    let total_cf = encloser_cf + number_occurence_percentage;

    let mut output_text = "".to_string();
    
    let random_frequency_threshold = prng.number_in_range_inclusive(1, DATASET_TOTAL_CF);
    let new_word = String::from(get_word_at(random_frequency_threshold));
    if text_gen_settings.contains_capital_letters {
        output_text += capitalised_word(&new_word).as_str();
    } else {
        output_text += new_word.as_str();
    }
    let mut should_capitalise = false;
    let mut special_char_interval = 0;
    // used to avoid duplicate words
    let mut last_word = new_word;
    
    let mut line_end_indices = Vec::new();
    for word_count in 2..=MAX_WORDS_PER_SCREEN {
        let char_kind_roll = prng.number_in_range_inclusive(1, total_cf as u64) as u8;
        special_char_interval += 1;
        // Generate a word (capitalised depending on context)
        if 1 <= char_kind_roll && char_kind_roll <= word_cf {
            let mut random_frequency_threshold = prng.number_in_range_inclusive(1, DATASET_TOTAL_CF);
            let last_char = output_text.chars().last().expect("There should be a last character").to_string();
            let is_the_last_character_not_an_opening_char = !OPENING_SPECIAL_CHARS.contains(&last_char.as_str());
            let is_the_last_character_a_closing_char = CLOSING_SPECIAL_CHARS.contains(&last_char.as_str()) && maybe_closing_char.is_none();
            if is_the_last_character_not_an_opening_char || is_the_last_character_a_closing_char {
                output_text += "_";
            }
            let mut new_word = String::from(get_word_at(random_frequency_threshold));
            loop {
                if new_word == last_word {
                    random_frequency_threshold = prng.number_in_range_inclusive(1, DATASET_TOTAL_CF);
                    new_word = String::from(get_word_at(random_frequency_threshold));
                } else {
                    break;
                }
            }
            last_word = new_word.clone();
            if should_capitalise && text_gen_settings.contains_capital_letters { 
                output_text += capitalised_word(&new_word).as_str();
                should_capitalise = false;
            } else {
                output_text += new_word.as_str();
            }
        // Generate a special suffix character
        } else if word_cf < char_kind_roll && char_kind_roll <= suffix_cf {
            if special_char_interval > MIN_INTERVAL_BETWEEN_SPECIAL_CHARS {
                let special_char = SPECIAL_SUFFIX_CHARS[prng.number_in_range_inclusive(0, 4) as usize];
                output_text += special_char;
                if special_char == "." && text_gen_settings.contains_capital_letters { should_capitalise = true }
                special_char_interval = 0;
            }
        // Generate an opening and closing special character
        } else if suffix_cf < char_kind_roll && char_kind_roll <= encloser_cf {
            match maybe_closing_char {
                Some(ref cc) => {
                    if special_char_interval > MIN_INTERVAL_BETWEEN_SPECIAL_CHARS {
                        output_text += cc.as_str();
                        maybe_closing_char = None;
                        special_char_interval = 0;
                    }
                },
                None => {
                    output_text += "_";
                    let index = prng.number_in_range_inclusive(0, 4) as usize;
                    output_text += OPENING_SPECIAL_CHARS[index];
                    maybe_closing_char = Some(String::from(CLOSING_SPECIAL_CHARS[index]));
                    special_char_interval = 0;
                }
            }
        // Generate a number
        } else if encloser_cf < char_kind_roll && char_kind_roll <= total_cf {
            output_text += "_";
            let digits_to_generate = prng.number_in_range_inclusive(1, MAX_DIGITS_TO_GENERATE) as usize;
            for _ in 0..digits_to_generate {
                let number = format!["{}", prng.number_in_range_inclusive(0, 9)];
                output_text += number.as_str();
            }
        } else {
            panic!("This branch should be unreachable.")
        }
        if word_count % MAX_WORDS_PER_LINE == 0 {
            // The index corresponds to the first character on the next line
            line_end_indices.push(output_text.len());
        }
    }
    (output_text, maybe_closing_char, line_end_indices)
}

fn capitalised_word(word: &str) -> String {
    let mut capitalised_word = String::from(
        word
            .chars()
            .next()
            .expect("We only have non-empty strings.")
            .to_ascii_uppercase()
    );
    capitalised_word += &word[1..];
    capitalised_word
}

#[test]
#[should_panic]
fn access_word_list_with_invalid_frequency() {
    use crate::prng;
    let mut prng = prng::Lcrng::new(prng::seed_from_time_now());
    for _ in 1..=10 {
        get_word_at(DATASET_TOTAL_CF + prng.number_in_range_inclusive(1, 10));
    }
}

#[test]
fn access_word_list_with_max_valid_frequency() {
    assert_eq!(get_word_at(DATASET_TOTAL_CF), "the");
    assert_eq!(get_word_at(15000), "bits"); 
    assert_eq!(get_word_at(1), "bits"); 
    assert_eq!(get_word_at(952_626), "gentle"); //"gentle", 961416, 945551
    assert_eq!(get_word_at(113_940_121), "football"); // "football", 113_998_149, 113_930_514),
}