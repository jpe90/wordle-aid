use crate::wordle_words::word_list;
use std::collections::HashSet;
use std::fmt;
use std::io;

mod wordle_words;

#[derive(Debug, Clone)]
enum GuessedLetterResult {
    NotUsed,
    WrongSpot,
    CorrectSpot,
}

impl fmt::Display for GuessedLetterResult {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self {
            Self::NotUsed => write!(f, "black"),
            Self::WrongSpot => write!(f, "yellow"),
            Self::CorrectSpot => write!(f, "green"),
        }
    }
}

#[derive(Debug, Clone)]
struct GuessedLetter {
    letter: char,
    result: GuessedLetterResult,
}

impl fmt::Display for GuessedLetter {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[{} - {}] ", self.letter.to_uppercase(), self.result)
    }
}

#[derive(Debug)]
struct GuessedWord {
    letters: Vec<GuessedLetter>,
}

impl fmt::Display for GuessedWord {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for v in &self.letters {
            write!(f, "{}", v).expect("Couldn't write to display.");
        }
        Ok(())
    }
}

#[derive(Debug)]
struct AggregateLetterResult {
    wrong_spot: HashSet<char>,
    correct_spot: Option<char>,
}

#[derive(Debug)]
struct AggregateWordResult {
    not_used: HashSet<char>,
    used_somewhere: HashSet<char>,
    aggregate_letter_results: Vec<AggregateLetterResult>,
}

impl AggregateWordResult {
    fn from(guessed_words: &[GuessedWord]) -> AggregateWordResult {
        let mut not_used: HashSet<char> = HashSet::new();
        let mut used_somewhere: HashSet<char> = HashSet::new();
        let mut letter_index = 0;
        let mut aggregate_letter_results = Vec::new();
        while letter_index < 5 {
            let mut correct_spot: Option<char> = None;
            let mut wrong_spot = HashSet::new();
            for word in guessed_words {
                let current_letter = &word.letters[letter_index];
                correct_spot = match current_letter.result {
                    GuessedLetterResult::CorrectSpot => {
                        correct_spot.or(Some(current_letter.letter))
                    }
                    _ => None,
                };
                match current_letter.result {
                    GuessedLetterResult::WrongSpot => {
                        wrong_spot.insert(Some(current_letter.letter).unwrap());
                        used_somewhere.insert(Some(current_letter.letter).unwrap())
                    }
                    GuessedLetterResult::NotUsed => {
                        not_used.insert(Some(current_letter.letter).unwrap())
                    }
                    _ => false,
                };
            }
            aggregate_letter_results.push(AggregateLetterResult {
                // TODO: figure out how to accomplish this without a clone
                wrong_spot: wrong_spot.clone(),
                correct_spot,
            });
            letter_index += 1;
        }
        AggregateWordResult {
            not_used,
            used_somewhere,
            aggregate_letter_results,
        }
    }

    fn letter_matches(&self, index: usize, chr: char) -> bool {
        let mut successful_match = true;
        let aggregate_letter_result = &self.aggregate_letter_results[index];
        successful_match = successful_match
            && match aggregate_letter_result.correct_spot {
                Some(i) => (i == chr),
                _ => true,
            };
        for letter in &aggregate_letter_result.wrong_spot {
            if chr == *letter {
                return false;
            }
        }
        successful_match
    }

    fn word_matches(&self, strng: &str) -> bool {
        let mut successful_match = true;
        successful_match = successful_match && set_not_in_str(&self.not_used, strng);
        successful_match = successful_match && used_at_least_once(&self.used_somewhere, strng);
        for (position, letter) in strng.chars().enumerate() {
            successful_match = successful_match && self.letter_matches(position, letter);
        }
        successful_match
    }
}

#[derive(Debug)]
struct Guesses {
    vec: Vec<GuessedWord>,
}

impl Guesses {
    fn new() -> Guesses {
        Guesses { vec: Vec::new() }
    }

    fn len(&self) -> usize {
        self.vec.len()
    }

    fn add_guess(&mut self, guessed_word: GuessedWord) {
        if self.len() < 6 {
            self.vec.push(guessed_word);
        } else {
            eprintln!("You can't enter more than 5 words");
        }
    }
}

fn set_not_in_str(hash: &HashSet<char>, strng: &str) -> bool {
    let mut successful_match = true;
    for item in hash {
        successful_match = successful_match && strng.chars().all(|x| x != *item);
    }
    successful_match
}

fn used_at_least_once(hash: &HashSet<char>, strng: &str) -> bool {
    let mut successful_match = true;
    for item in hash {
        successful_match = successful_match && strng.chars().any(|x| x == *item);
    }
    successful_match
}

fn main() {
    let words = word_list();
    let guesses = Guesses::new();
    println!("Guess any five letter word- \"arose\" is a good choice!");
    run_guess_loop(guesses, words)
}

fn run_guess_loop(mut guesses: Guesses, words: Vec<&str>) {
    loop {
        get_guess(&mut guesses);

        // I wanted to refactor the below two lines into a function call that
        // returns flines so that I could write some tests for filter behavior,
        // but I couldn't figure out a type signature that lets me
        // return a filtered iterator from a function
        let agg = AggregateWordResult::from(&guesses.vec);
        let mut filtered_words = words.iter().filter(|x| agg.word_matches(x));
        let mut display_words = Vec::new();
        let mut item_index = 0;
        while item_index < 10 {
            let item = filtered_words.next();
            item_index += 1;
            if let Some(val) = item {
                display_words.push(val)
            }
        }
        if item_index == 0 {
            println!("There are no words that match the results you entered. Did you make a mistake entering them?");
            break;
        } else {
            println!("If the word was correct, press CTRL+C to quit. Otherwise, make a guess with one of the following:");
            for item in display_words {
                println!("{}", item);
            }
        }
    }
}

fn get_guess(guesses: &mut Guesses) {
    let guess_string = prompt_for_guess();
    let guess = prompt_for_results(guess_string);
    guesses.add_guess(guess);
}

fn prompt_for_guess() -> String {
    println!("Please enter the word you guessed:");

    let mut guess = String::new();

    loop {
        io::stdin()
            .read_line(&mut guess)
            .expect("Failed to read line");
        guess = guess.trim().to_string(); // remove trailing newline
        if guess.chars().count() == 5 {
            break;
        }
        println!("Please enter a five letter word:");
        guess.clear();
    }
    guess
}

fn prompt_for_results(guess: String) -> GuessedWord {
    let mut letters: Vec<GuessedLetter> = Vec::new();
    'outer: loop {
        for (_position, letter) in guess.chars().enumerate() {
            println!(
                "Enter the result for the letter \"{}\": [G]reen, [Y]ellow, or [B]lack ",
                letter
            );
            let result = prompt_for_color();
            let guessed_letter = GuessedLetter { letter, result };
            // TODO: figure out how to do this without a clone
            letters.push(guessed_letter.clone());
        }
        loop {
            // TODO: figure out how to do this without a clone
            let cloned_letters = letters.clone();
            let guessed_word = GuessedWord {
                letters: cloned_letters,
            };
            println!(
                "You entered {}\nIs this correct? Please enter y or n",
                guessed_word
            );
            let mut input = String::new();
            std::io::stdin()
                .read_line(&mut input)
                .expect("Failed to read line");
            let response = input.chars().take(1).last().unwrap();

            if let 'y' = response {
                break 'outer guessed_word;
            }
            println!("Please only enter characters 'y' or 'n': ");
        }
    }
}

fn prompt_for_color() -> GuessedLetterResult {
    loop {
        let mut input = String::new();
        std::io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line");
        let letter = input.chars().take(1).last().unwrap();
        match letter {
            'g' => break GuessedLetterResult::CorrectSpot,
            'b' => break GuessedLetterResult::NotUsed,
            'y' => break GuessedLetterResult::WrongSpot,
            _ => (),
        }
        println!("Please enter [G]reen, [Y]ellow, or [B]lack: ");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn filtered_words_contains_entry(guesses: &Guesses, entry: &str) -> bool {
        let words = word_list();
        let agg = AggregateWordResult::from(&guesses.vec);
        let mut filtered_words = words.iter().filter(|x| agg.word_matches(x));
        filtered_words.any(|&word| word == entry)
    }

    // quick sanity check
    #[test]
    fn word_is_racer() {
        let c = GuessedLetter {
            letter: 'c',
            result: GuessedLetterResult::WrongSpot,
        };
        let a = GuessedLetter {
            letter: 'a',
            result: GuessedLetterResult::CorrectSpot,
        };
        let r = GuessedLetter {
            letter: 'r',
            result: GuessedLetterResult::WrongSpot,
        };
        let g = GuessedLetter {
            letter: 'g',
            result: GuessedLetterResult::NotUsed,
        };
        let o = GuessedLetter {
            letter: 'o',
            result: GuessedLetterResult::NotUsed,
        };
        let guess_cargo = vec![c, a, r, g, o];
        let guessed_word = GuessedWord {
            letters: guess_cargo,
        };
        let mut guess = Guesses::new();
        guess.add_guess(guessed_word);

        assert!(filtered_words_contains_entry(&guess, "racer"));
        assert!(!filtered_words_contains_entry(&guess, "zebra"));
    }
}
