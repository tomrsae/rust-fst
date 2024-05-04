/// ------------------------------------------------------------
/// /// File: make_fsm.rs
/// Author: Tommy Sætre
/// Description:
///     Implements the make_fsm proc macro, defining
///     tokenizers and implementing the solution.
/// ------------------------------------------------------------

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::{parse_macro_input, spanned::Spanned, Ident};

use crate::parsers::FiniteStateMachine;

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

    let fsm_ident = &fsm_tree.ident;
    let error_ident = format_ident!("{}Error", fsm_ident);
    
    let event_idents
        =  fsm_tree.events.to_tokens(|x| {
            let ident = &x.ident;
            let params = &x.parameters;
            if params.len() > 0 {
                quote!(#ident(#(#params,)*))
            } else {
                quote!(#ident)
            }
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
        = fsm_tree.states.to_tokens(|x| {
            let states_ident = &x.ident;
            let ident = format_ident!("{}State", states_ident);
            quote!(#ident)
        });

    let event_methods: Vec<TokenStream2>
        = fsm_tree.events.to_tokens(|x| {
            let ident = &x.ident;
            let param_types = &x.parameters;
            // Generate valid parameters for event invocation method
            let param_idents: Vec<Ident>
                = x.parameters
                    .iter()
                    .enumerate()
                    .map(|(i, t)| {
                        Ident::new(&format!("p{}", i), t.span())
                    })
                    .collect();
            
            let event = if param_idents.len() > 0 {
                quote!(Event::#ident(#(#param_idents,)*))
            } else {
                quote!(Event::#ident)
            };

            quote! {
                fn #ident(&mut self, #(#param_idents: #param_types,)*) -> Result<(), #error_ident> {
                    self.handle_event(#event)
                }
            }
        });

    let expanded = quote! {

        #[derive(Debug, Clone)]
        struct #error_ident {
            err: String
        }

        impl #error_ident {
            fn new(error: &str) -> Self {
                #error_ident { err: error.to_owned() }
            }
        }

        impl std::fmt::Display for #error_ident {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(f, "{}", &self.err)
            }
        }

        type EventOutcome = Result<Option<Transition>, #error_ident>;

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
        trait ResolvableState : State {
            fn resolve(self) -> InternalStates;
        }

        #(#state_resolvables)*

        enum InternalStates {
            #(#state_enums,)*
        }

        struct #fsm_ident {
            states: InternalStates
        }

        impl #fsm_ident {
            fn new(init: Transition) -> Self {
                let mut result = Self { states: init.target };
                result.get_current_state().enter();
                result
            }

            fn get_current_state(&mut self) -> &mut dyn State {
                use InternalStates::*;
                // No idiomatic way to equally "extract" all enum values
                match &mut self.states {
                    #(#state_enum_idents(state) => state,)*
                }
            }
        
            fn handle_event(&mut self, e: Event) -> Result<(), #error_ident> {
                let current_state = self.get_current_state();
                let event_result = current_state.handle_event(e)?;
                
                // Invoke enter/exit handles before/after (respectively) changing states
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