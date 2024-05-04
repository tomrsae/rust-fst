/// ------------------------------------------------------------
/// File: coin_machine_example/main.rs
/// Author: Tommy Sætre
/// Description:
///     Uses the make_fsm proc macro to implement a very
///     simple coin machine inspired by a coin-based turnstile
///     machine as desrbired by Wikipedia's FSM page:
///     (https://en.wikipedia.org/wiki/Finite-state_machine#/media/File:Turnstile_state_machine_colored.svg)
/// 
///     However, expands upon the diagram to demonstrate use of
///     more advanced features of the library such as parameterized
///     events and error handling.
/// ------------------------------------------------------------

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
        push(),
        insert_coins(u8), // events may be parametric (but must take ownership!)
        see_balance()
    ],

    // The state names listed here must exist as structs
    // and implement the `State` trait, making them capable
    // of handling events, as well as having enter/exit handlers
    states: [
        Locked,
        Unlocked
    ]
}

fn run_example() -> Result<(), CoinMachineError> {
    // Passing in an initial transition to set parameterized initial state
    let init = Transition::to(Locked{ coins: 0 });
    let mut fsm = CoinMachine::new(init);
    
    fsm.push()?; // no-op
    fsm.push()?; // no-op
    fsm.insert_coins(2)?; // no-op, currently 2/3 coins
    fsm.insert_coins(1)?; // transition to unlocked
    fsm.insert_coins(5)?; // no-op (wasted coins)
    fsm.push()?; // transition to locked
    fsm.push()?; // no-op

    fsm.insert_coins(1)?; // no-op, currently 1/3 coins
    fsm.see_balance()?; // no-op (prints balance)
    fsm.insert_coins(2)?; // transition to unlocked

    fsm.see_balance()?; // error

    fsm.push()?; // won't execute because previous error was propogated

    Ok(())
}

fn main() {
    if let Some(err) = run_example().err() {
        println!("Error occured: {}", err);
    }
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

    fn handle_event(&mut self, e: Event) -> EventOutcome {
        let mut result = None;
        match e {
            Event::insert_coins(num) => {
                println!("Received & accepted {} coins!", num);
                self.coins += num;

                if self.coins >= 3 {
                    result = Some(Transition::to(Unlocked { }))
                }
            }
            Event::see_balance => println!("Current balance: {}", &self.coins),
            _ => println!("Pushed while locked..")
        }

        Ok(result)
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

    fn handle_event(&mut self, e: Event) -> EventOutcome {
        match e {
            Event::push => {
                println!("Pushed, locking!");
                Ok(Some(Transition::to(Locked { coins: 0 })))
            }
            Event::insert_coins(num) => {
                println!("Wasted {} coins!! :(", num);
                Ok(None)
            }
            Event::see_balance => Err(CoinMachineError::new("No balance available"))
        }
    }
}