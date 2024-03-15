use crate::F_LOC;
use itertools::{iproduct, Itertools};
use rand::seq::SliceRandom;
use rand::thread_rng;
use regex::Regex;
use statig::{state_machine, Response, Response::Transition};
use std::collections::{HashMap, HashSet};
use std::{
    fs,
    io::{self, Write},
};

/// Contains the state of the game
#[derive(Debug, Default)]
pub struct LetterBoxed {
    /// The word list
    pub dict: Vec<String>,
    /// The 12 letters of the game
    pub word: String,
    /// A list of words that solve the game
    pub solution: Vec<String>,
}

pub enum Event {
    /// Proceed to the next State
    Next,
}

/// The states and state methods for [LetterBoxed]
#[state_machine(initial = "State::load_file()", state(derive(Debug)))]
impl LetterBoxed {
    #[state(entry_action = "read_dict")]
    fn load_file(event: &Event) -> Response<State> {
        match event {
            Event::Next => Transition(State::input()),
        }
    }

    #[action]
    fn read_dict(&mut self) {
        self.dict = fs::read_to_string(F_LOC)
            .unwrap() // panic on possible file-reading errors
            .to_lowercase()
            .lines() // split the string into an iterator of string slices
            .map(String::from) // make each slice into a string
            .collect(); // gather them together into a vector
    }

    #[state(entry_action = "enter_input")]
    fn input(event: &Event) -> Response<State> {
        match event {
            Event::Next => Transition(State::processing()),
        }
    }

    #[action]
    fn enter_input(&mut self) {
        print!("Enter input: ");
        io::stdout().flush().expect("Could not flush output");
        io::stdin()
            .read_line(&mut self.word)
            .expect("can not read user input");
        self.word = String::from(self.word.to_lowercase().strip_suffix("\n").unwrap());
        if self.word.len() != 12 {
            println! {"wrong length!"}
            self.word.clear();
            self.enter_input();
        }
    }

    #[state(entry_action = "process_input")]
    fn processing(event: &Event) -> Response<State> {
        match event {
            Event::Next => Transition(State::output()),
        }
    }

    #[action]
    fn process_input(&mut self) {
        let m = self.get_word_map();
        self.solution = self.solve(&m).unwrap_or(Vec::new());
    }

    #[state(exit_action = "cleanup")]
    fn output(event: &Event) -> Response<State> {
        match event {
            Event::Next => Transition(State::input()),
        }
    }

    #[action]
    fn cleanup(&mut self) {
        println!("{:?}", self.solution);
        self.word.clear();
        self.solution.clear();
    }
}

/// The word logic for [LetterBoxed]
/// TODO: maybe these shouldn't be class methods and instead should take self.word and self.dict as parameters
impl LetterBoxed {
    // Get a map of "{first character}{last character" : [all legal words]
    // Example
    // self.words = "rdea"
    // m = {"rr": ["rear"], "rd": ["red", "rad", "read"], "re":..., "aa": ["area"]}
    fn get_word_map(&self) -> HashMap<String, Vec<String>> {
        let permutations: Vec<Vec<char>> = product(self.word.as_bytes(), 2);
        let _word_vec: Vec<Vec<String>> = Vec::new();
        let l: &String = &self.word.to_lowercase();
        let re: &Regex = &self.get_bad_word_re();
        let mut m: HashMap<String, Vec<String>> = HashMap::new();
        permutations.iter().for_each(|v| {
            m.insert(
                v.iter().collect(),
                find_words(v[0], v[1], l, &self.dict, re),
            );
        });
        m
    }
    /// Return a [Regex] that is true if any illegal
    /// (characters on the same side of the square being next to each other)
    /// substrings are present in a word
    /// For example, if the 4 sides are ["abc", "def", "ghi", "jkl"]
    /// the regex would match if any of ["aa", "ab", "ac", "ba", ..., "ll"]
    /// are in a word.
    fn get_bad_word_re(&self) -> Regex {
        let mut s: String = String::new();
        let l: &String = &self.word.to_lowercase();
        let splits: Vec<String> = vec![
            String::from(&l[0..3]),
            String::from(&l[3..6]),
            String::from(&l[6..9]),
            String::from(&l[9..12]),
        ];
        splits.iter().for_each(|f| {
            f.chars()
                .cartesian_product(f.chars())
                .for_each(|g| s.push_str(&format!("[{}{}]{{2}}|", g.0, g.1)))
        });
        let mut c: std::str::Chars<'_> = s.chars();
        c.next_back();
        s = String::from(c.as_str());
        Regex::new(&s).unwrap()
    }

    /// Get the first solution to the puzzle.
    /// Randomize the order so that on multiple iterations you can get different solutions
    fn solve(&mut self, m: &HashMap<String, Vec<String>>) -> Result<Vec<String>, &str> {
        for i in 3..6 {
            let mut combos: Vec<Vec<char>> = product(self.word.as_bytes(), i);
            let mut rng = thread_rng();
            combos.shuffle(&mut rng);
            for x in combos.iter() {
                match self.solve_combo(&x, &m) {
                    Ok(result) => return Ok(result),
                    _ => {}
                }
            }
        }
        Err("No solutions found")
    }

    /// Get the firt solution to the puzzle given the first and last letters of each word in the solution.
    /// Example
    /// c = ['a', 'r', 'p', "l"]
    /// A possible solution could be ["air", "rap", "pal"]
    fn solve_combo(
        &mut self,
        c: &Vec<char>,
        m: &HashMap<String, Vec<String>>,
    ) -> Result<Vec<String>, &str> {
        let keys: Vec<String> = c
            .windows(2)
            .map(|window| format!("{}{}", window[0], window[1]))
            .collect();

        let out: Vec<Vec<String>> = keys.iter().filter_map(|key| m.get(key)).cloned().collect();
        match self.has_solution(out) {
            Ok(result) => Ok(result),
            Err(_) => Err(""),
        }
    }

    /// Check to see if a solution is found in the combination of all words and return the first
    /// solution that is found.
    /// Example
    /// v = [["art", "apt"], ["trip"], ["party", "pantry"]
    /// This function would check all [2 * 1 * 2] combos of words to see if they are a solution
    fn has_solution(&self, v: Vec<Vec<String>>) -> Result<Vec<String>, &str> {
        for x in v.into_iter().multi_cartesian_product() {
            if self.is_solution(&x) {
                return Ok(x);
            }
        }
        return Err("");
    }

    /// Return true if the words in v are a solution
    /// Because only valid words are entered into this function,
    /// We just need to see if 12 unique letters are entered
    fn is_solution(&self, v: &Vec<String>) -> bool {
        let mut s: HashSet<char> = HashSet::new();
        v.iter().for_each(|word: &String| {
            word.chars().for_each(|c: char| {
                s.insert(c);
            })
        });
        return s.len() == 12;
    }
}

/// Find all words that are legal and have the first and last characters given
fn find_words(
    first_char: char,
    last_char: char,
    all_letters: &String,
    dict: &Vec<String>,
    bad_word_re: &Regex,
) -> Vec<String> {
    let mut res_v: Vec<String> = Vec::new();
    let reg_str: String = format!("^{}[{}]+{}$", first_char, all_letters, last_char);
    let re: Regex = Regex::new(&reg_str).unwrap();

    dict.iter().for_each(|f| {
        if is_valid_word(&f, &re, bad_word_re) {
            res_v.push(f.to_string())
        }
    });
    res_v
}

/// Return if a word is legal
/// A word is legal if it only contains letters from the sides of the puzzle
/// and if no letters from the same side are adjacent to each other in the word
fn is_valid_word(word: &String, only_good_letters_re: &Regex, bad_word_re: &Regex) -> bool {
    only_good_letters_re.is_match(word) & !bad_word_re.is_match(word)
}

/// Get the cartesian product of a vector of bytes with itself n times as characters
/// TODO: learn how to make this generic so it can have signature product<T>(vector: &[T], n: i32) -> Vec<Vec<T>>
fn product(vector: &[u8], n: i32) -> Vec<Vec<char>> {
    let mut result: Vec<Vec<char>> = vec![vec![]];
    for _ in 0..n {
        result = iproduct!(result.iter(), vector.iter())
            .map(|(v, x)| {
                let mut v1 = v.clone();
                v1.push(*x as char);
                v1
            })
            .collect();
    }
    result
}
