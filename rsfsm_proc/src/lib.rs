/// ------------------------------------------------------------
/// File: lib.rs
/// Author: Tommy Sætre
/// Description: Main interface for rsfsm proc library
/// ------------------------------------------------------------

use proc_macro::TokenStream;

mod make_fsm;
mod parsers;

#[proc_macro]
pub fn make_fsm(tokens: TokenStream) -> TokenStream {
    make_fsm::impl_make_fsm(tokens)
}