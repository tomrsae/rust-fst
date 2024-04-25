use rsfsm_base::state::State;
use rsfsm_proc::{ make_fsm, fsm_trigger };

fn main() {
    println!("Hello, world!");
    make_fsm!(
        name: "Coin Machine",
        states: [
            Locked,
            Unlocked
        ],
        triggers: [
            push,
            insert_coin
        ]
    );
}

#[fsm_trigger { Unlocked => Locked }]
fn push() {

}

#[fsm_trigger { Locked => Unlocked }]
fn insert_coin() {

}

struct Locked {

}

impl State for Locked {

}

struct Unlocked {

}

impl State for Unlocked {

}