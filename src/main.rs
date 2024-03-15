#![warn(missing_docs)]
//! Solve the NYT game Letterboxed

use letter_boxed::{Event, LetterBoxed};
use statig::blocking::IntoStateMachineExt;
mod letter_boxed;

/// The file path of the word list
static F_LOC: &str = "/Users/cheineme/Downloads/words_easy.txt";

/// Create a [LetterBoxed] state machine and have it run forever
fn main() {
    let mut state_machine = LetterBoxed::default().state_machine();
    loop {
        state_machine.handle(&Event::Next);
    }
}
