use letter_boxed::{Event, LetterBoxed};
use statig::blocking::IntoStateMachineExt;
mod letter_boxed;

static F_LOC: &str = "/Users/cheineme/Downloads/words_easy.txt";

fn main() {
    let mut state_machine = LetterBoxed::default().state_machine();
    loop {
        state_machine.handle(&Event::Next);
    }
}
