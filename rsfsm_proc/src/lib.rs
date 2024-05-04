use proc_macro::TokenStream;

mod make_fsm;
mod fsm;

#[proc_macro]
pub fn make_fsm(tokens: TokenStream) -> TokenStream {
    make_fsm::impl_make_fsm(tokens)
}