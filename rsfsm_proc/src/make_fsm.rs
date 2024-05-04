use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::parse_macro_input;

use crate::fsm::FiniteStateMachine;

pub fn impl_make_fsm(tokens: TokenStream) -> TokenStream {
    let fsm_tree = parse_macro_input!(tokens as FiniteStateMachine);

    let event_idents: Vec<_> = fsm_tree
        .events
        .iter()
        .map(|x| {
            let ident = &x.ident;
            quote!(#ident)
        })
        .collect();
    
    let state_enums: Vec<_> = fsm_tree
        .states
        .iter()
        .map(|x| {
            let state_ident = &x.ident;
            let ident = format_ident!("{}State", state_ident);
            quote!(#ident(#state_ident))
        })
        .collect();

    let state_resolvables: Vec<_> = fsm_tree
        .states
        .iter()
        .map(|x| {
            let ident = &x.ident;
            let states_ident = format_ident!("{}State", ident);
            quote!{
                impl ResolvableState for #ident {
                    fn resolve(self) -> InternalStates {
                        InternalStates::#states_ident(self)
                    }
                }
            }
        })
        .collect();

    let state_enum_idents: Vec<_> = fsm_tree
        .states
        .iter()
        .map(|x| {
            let state_ident = &x.ident;
            let ident = format_ident!("{}State", state_ident);
            quote!(#ident)
        })
        .collect();

    let event_methods: Vec<_> = fsm_tree
        .events
        .iter()
        .map(|x| {
            let ident = &x.ident;
            quote! {
                fn #ident(&mut self) {
                    self.handle_event(Event::#ident)
                }
            }
        })
        .collect();

    let ident = &fsm_tree.ident;

    let expanded = quote! {
        trait State {
            fn enter(&mut self);
        
            fn exit(&mut self);

            fn handle_event(&mut self, e: Event)
                -> Option<Transition>;
        }

        #[allow(non_camel_case_types)]
        enum Event {
            #(#event_idents,)*
        }

        struct Transition {
            target: InternalStates
        }

        impl Transition {
            fn to<T: ResolvableState>(state: T) -> Transition {
                Transition {
                    target: state.resolve()
                }
            }
        }

        // Used to "hide" `InternalStates` from user and
        // hence enable a clean Transition::to(State) API 
        trait ResolvableState {
            fn resolve(self) -> InternalStates;
        }

        #(#state_resolvables)*

        enum InternalStates {
            #(#state_enums,)*
        }

        struct #ident {
            states: InternalStates
        }

        impl #ident {
            fn new(init: Transition) -> Self {
                let mut result = Self { states: init.target };
                result.get_current_state().enter();
                result
            }

            fn get_current_state(&mut self) -> &mut dyn State {
                use InternalStates::*;
                match &mut self.states {
                    #(#state_enum_idents(state) => state,)*
                }
            }

            fn handle_event(&mut self, e: Event) {
                let current_state = self.get_current_state();
                let event_result = current_state.handle_event(e);
                
                if event_result.is_some() {
                    current_state.exit();

                    let target = event_result.unwrap().target;
                    self.states = target;
                    
                    self.get_current_state().enter();
                }
            }

            #(#event_methods)*
        }
    };
    
    expanded.into()
}