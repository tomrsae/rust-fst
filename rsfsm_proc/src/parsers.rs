/// ------------------------------------------------------------
/// File: parsers.rs
/// Author: Tommy Sætre
/// Description:
///     Implements parsers for the proc macros.
///     Enables an enforced struct/JSON-like
///     syntax tree for state machines.
/// AI notice:
///     Group parsing (prase_group) was heavily inspired by 
///     the crate sfsm (https://docs.rs/sfsm/0.4.3/sfsm/index.html)'s
///     group parsing implementation.
/// ------------------------------------------------------------

use proc_macro2::Span;
use syn::{
    parse::{ Parse, ParseStream, Parser }, punctuated::Punctuated, Error, Ident, Result, Token, Type
};

// Parse comma-separated groups delimited by [], () or {} and receive elements as a vector
// AI notice: Heavily inspired by sfsm's implementation (https://docs.rs/sfsm/0.4.3/sfsm/index.html)
fn parse_group<T: Parse>(stream: ParseStream) -> Result<Vec<T>> {
    let group: proc_macro2::Group = stream.parse()?;
    let parser = Punctuated::<T, Token![,]>::parse_terminated;
    let stream = group.stream().into();

    let items_punctuated = parser.parse(stream)?;
    Ok(items_punctuated
        .into_iter()
        .collect()
    )
}

enum FsmExpr {
    Name(Ident),
    Events(Vec<Event>),
    States(Vec<State>)
}

impl Parse for FsmExpr {
    fn parse(input: ParseStream) -> Result<Self> {
        let ident: Ident = input.parse()?;
        input.parse::<Token![:]>()?;

        match ident.to_string().as_str() {
            "name" => {
                let ident: Ident = input.parse()?;
                Ok(FsmExpr::Name(ident))
            },
            "events" => {
                let events: Vec<Event> = parse_group::<Event>(input)?;
                Ok(FsmExpr::Events(events))
            }, "states" => {
                let states: Vec<State> = parse_group::<State>(input)?;
                Ok(FsmExpr::States(states))
            },
            _ => Err(Error::new(ident.span(), "Undefined identifier"))
        }
    }
}

#[derive(Clone)]
pub struct State {
    pub ident: Ident
}

impl Parse for State {
    fn parse(input: ParseStream) -> Result<Self> {
        let ident: Ident = input.parse()?;
        
        Ok(Self {
            ident
        })
    }
}

#[derive(Clone)]
pub struct Event {
    pub ident: Ident,
    pub parameters: Vec<Type>
}

impl Parse for Event {
    fn parse(input: ParseStream) -> Result<Self> {
        let ident: Ident = input.parse()?;
        let parameters: Vec<Type> = parse_group(input)?;

        Ok(Self {
            ident,
            parameters
        })
    }
}

pub struct FiniteStateMachine {
    pub ident: Ident,
    pub states: Vec<State>,
    pub events: Vec<Event>
}

impl Parse for FiniteStateMachine {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut ident_opt: Option<Ident> = None;
        let mut states: Vec<State> = Vec::new();
        let mut events: Vec<Event> = Vec::new();

        let fsm_expressions: Punctuated<FsmExpr, Token![,]>
            = input.parse_terminated(FsmExpr::parse)?;
        
        for fsm_expr in fsm_expressions.iter() {
            match fsm_expr {
                FsmExpr::Name(n) => ident_opt = Some(n.to_owned()),
                FsmExpr::States(s) => states = s.to_owned(),
                FsmExpr::Events(e) => events = e.to_owned()
            }
        }

        let err_opt: Option<&str> = {
            if ident_opt.is_none() {
                Some("No name specified")
            } else if states.len() < 1 {
                Some("No states specified")
            } else if events.len() < 1 {
                Some("No events specified")
            } else {
                None
            }
        };        

        if let Some(err) = err_opt {
            Err(Error::new(Span::call_site(), err))
        } else {
            Ok(Self {
                ident: ident_opt.unwrap(),
                states,
                events
            })
        }
    }
}