/// ---------------------------------------------------------------------
/// File: blinky_example/main.rs
/// Author: Tommy Sætre
/// Description:
///     Uses the make_fsm proc macro to implement a blinking
///     machine that blinks on an (imaginary) timer and can
///     be disabled and enabled.
///     
///     This demonstrates the macro FSM's capability of simulating
///     hierarchical state machines (without actually supporting it),
///     as well as being able to store system state between FSM states.
///     
///     The concept for this particular demonstration was taken from
///     the `statig` crate's example: https://github.com/mdeloof/statig  
/// 
///     Concept that is being emulated in this example:
///     ┌───────────────────────────┐                   
///     │         Blinking          │◀─────────┐        
///     │     ┌───────────────┐     │           │        
///     │ ┌─▶│     LedOn     │───┐ │  ┌───────────────┐
///     │ │   └───────────────┘   │ │  │  NotBlinking  │
///     │ │   ┌───────────────┐   │ │  └───────────────┘
///     │ └───│     LedOff    │◀─┘ │           ▲        
///     │     └───────────────┘     │───────────┘        
///     └───────────────────────────┘                   
///     CREDIT: mdeloof @ https://github.com/mdeloof/statig
/// 
/// ---------------------------------------------------------------------

use rsfsm_proc::make_fsm;

make_fsm!(
    name: Blinky,
    events: [
        timer_elapsed(),
        button_pressed()
    ],
    states: [
        Blinking,
        NotBlinking
    ]
);

fn run_example() -> Result<(), BlinkyError> {
    // Start machine in blinking state
    let init = Transition::to(Blinking { led_on: false });
    let mut fsm = Blinky::new(init);

    // LED will toggle 3 times
    for _ in 0..3 {
        fsm.timer_elapsed()?;
    }

    // Machine will be disabled and stop blinking
    fsm.button_pressed()?;
    
    // Nothing will happen - machine is disabled
    for _ in 0..3 {
        fsm.timer_elapsed()?;
    }

    // Machine will enable and pick up where it
    // left off (LED state was stored before disabling)
    fsm.button_pressed()?;

    // LED will toggle 3 times
    for _ in 0..3 {
        fsm.timer_elapsed()?;
    }

    Ok(())
}

fn main() {
    run_example().unwrap();
}

struct Blinking {
    led_on: bool
}
impl State for Blinking {
    fn enter(&mut self) {
    }

    fn exit(&mut self) {
    }

    fn handle_event(&mut self, e:Event) -> EventOutcome {
        match &e {
            Event::timer_elapsed => {
                // Toggle LED
                self.led_on = !self.led_on;

                // Print the state that was toggled
                let state_str = if self.led_on { "ON" } else { "OFF" };
                println!("💡 {}", state_str);

                Ok(None)
            }
            Event::button_pressed => {
                println!("Turning off");
                // Store LED state between fsm states
                Ok(Some(Transition::to(NotBlinking { stored_led_state: self.led_on })))
            }
        }
    }
}

struct NotBlinking {
    stored_led_state: bool // Keeps system state from Blinking state stored
}
impl State for NotBlinking {
    fn enter(&mut self) {
    }

    fn exit(&mut self) {
    }

    fn handle_event(&mut self, e:Event) -> EventOutcome {
        match &e {
            Event::timer_elapsed => {
                println!("    Ignored timer - machine is disabled");
                Ok(None)
            }
            Event::button_pressed => {
                println!("Turning on");
                Ok(Some(Transition::to(Blinking { led_on: self.stored_led_state })))
            }
        }
    }
}