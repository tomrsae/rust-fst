use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::parse_macro_input;

use crate::fsm::FiniteStateMachine;

trait VecTokenizeable<T> {
    fn to_tokens<F: FnMut(&T) -> TokenStream2>
        (&self, mapping: F) -> Vec<TokenStream2>;
}

impl<T> VecTokenizeable<T> for Vec<T> {
    fn to_tokens<F: FnMut(&T) -> TokenStream2>(&self, mapping: F) -> Vec<TokenStream2> {
        self.iter().map(mapping).collect()
    }
}

pub fn impl_make_fsm(tokens: TokenStream) -> TokenStream {
    let fsm_tree = parse_macro_input!(tokens as FiniteStateMachine);

    let ident = &fsm_tree.ident;
    let ident_error = format_ident!("{}Error", ident);
    
    let event_idents
        =  fsm_tree.events.to_tokens(|x| {
            let ident = &x.ident;
            quote!(#ident)
        });
    
    let state_enums
        = fsm_tree.states.to_tokens(|x| {
            let ident = &x.ident;
            let states_ident = format_ident!("{}State", ident);
            quote!(#states_ident(#ident))
        });

    let state_resolvables
        = fsm_tree.states.to_tokens(|x| {
            let ident = &x.ident;
            let states_ident = format_ident!("{}State", ident);
            quote!{
                impl ResolvableState for #ident {
                    fn resolve(self) -> InternalStates {
                        InternalStates::#states_ident(self)
                    }
                }
            }
        });

    let state_enum_idents
        = fsm_tree
            .states.to_tokens(|x| {
                let states_ident = &x.ident;
                let ident = format_ident!("{}State", states_ident);
                quote!(#ident)
            });

    let event_methods
        = fsm_tree.events.to_tokens(|x| {
            let ident = &x.ident;
            quote! {
                fn #ident(&mut self) -> Result<(), #ident_error> {
                    self.handle_event(Event::#ident)
                }
            }
        });

    let expanded = quote! {

        #[derive(Debug, Clone)]
        struct #ident_error {
            err: String
        }

        impl #ident_error {
            fn new(error: &str) -> Self {
                #ident_error { err: error.to_owned() }
            }
        }

        impl std::fmt::Display for #ident_error {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(f, "{}", &self.err)
            }
        }

        type EventOutcome = Result<Option<Transition>, #ident_error>;

        trait State {
            fn enter(&mut self);
        
            fn exit(&mut self);

            fn handle_event(&mut self, e: Event)
                -> EventOutcome;
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
                Transition { target: state.resolve() }
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

            fn handle_event(&mut self, e: Event) -> Result<(), #ident_error> {
                let current_state = self.get_current_state();
                let event_result = current_state.handle_event(e)?;
                
                if event_result.is_some() {
                    current_state.exit();

                    let target = event_result.unwrap().target;
                    self.states = target;

                    self.get_current_state().enter();
                }

                Ok(())
            }

            #(#event_methods)*
        }
    };
    
    expanded.into()
}