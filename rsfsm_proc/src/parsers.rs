use proc_macro2::Span;
use syn::{
    parse::{ Parse, ParseStream, Parser }, punctuated::Punctuated, Error, Ident, Result, Token, Type
};

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

        let ident = match ident_opt {
            Some(n) => n,
            None => return Err(Error::new(Span::call_site(), "No name specified"))
        };

        if states.len() < 1 {
            return Err(Error::new(Span::call_site(), "No states specified"))
        }

        if events.len() < 1 {
            return Err(Error::new(Span::call_site(), "No events specified"))
        }

        Ok(Self {
            ident,
            states,
            events
        })
    }
}