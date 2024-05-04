use rsfsm_proc::make_fsm;

make_fsm!{
    // FSM identifier,
    // used to create an instance later
    // on which to perform events etc..
    name: CoinMachine,

    // Methods `name::event_name` are created
    // and may be called to invoke events
    // (e.g. CoinMachine::push(&mut self) in this case).
    events: [
        push,
        insert_coin
    ],

    // The state names listed here must exist as structs
    // and implement the `State` trait, making them capable
    // of handling events, as well as having enter/exit handlers
    states: [
        Locked, // first state in array is the initial state
        Unlocked
    ]
}

fn main() {
    // Passing in an initial transition to set parameterized initial state
    let init = Transition::to(Locked{ coins: 0 });
    let mut fsm = CoinMachine::new(init);
    
    fsm.push(); // no-op
    fsm.push(); // no-op
    fsm.insert_coin(); // no-op, currently 1 coin
    fsm.insert_coin(); // no-op, currently 2 coin
    fsm.insert_coin(); // transition to unlocked
    fsm.insert_coin(); // no-op (wasted coin)
    fsm.push(); // transition to locked
    fsm.push(); // no-op
}

struct Locked {
    coins: u8
}
impl State for Locked {
    fn enter(&mut self) {
        // e.g. play lock sound
        println!("=> 🔒(L)");
    }

    fn exit(&mut self) {
        println!("🔒(L) =>");
    }

    fn handle_event(&mut self, e: Event) -> Option<Transition> {
        let mut result = None;
        match e {
            Event::insert_coin => {
                println!("Received & accepted coin!");
                self.coins += 1;

                if self.coins >= 3 {
                    result = Some(Transition::to(Unlocked { }))
                }
            }
            _ => println!("Pushed while locked..")
        }

        result
    }
}

struct Unlocked;
impl State for Unlocked {
    fn enter(&mut self) {
        // e.g. play unlock sound
        println!("=> 🔓(U)");
    }

    fn exit(&mut self) {
        println!("🔓(U) =>");
    }

    fn handle_event(&mut self, e: Event) -> Option<Transition> {
        match e {
            Event::push => {
                println!("Pushed, locking!");
                Some(Transition::to(Locked { coins: 0 }))
            }
            Event::insert_coin => {
                println!("Wasted money :(");
                None
            }
        }
    }
}