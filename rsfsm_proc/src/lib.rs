use proc_macro::TokenStream;
use syn::DeriveInput;

mod make_fsm;

#[proc_macro]
pub fn make_fsm(tokens : TokenStream) -> TokenStream {
    let ast = parse_tokens(tokens);

    make_fsm::impl_make_fsm(&ast)
}

#[proc_macro_attribute]
pub fn fsm_trigger(attr: TokenStream, item: TokenStream) -> TokenStream {
    panic!("oopsie")
}

fn parse_tokens(tokens: TokenStream) -> DeriveInput {
    syn::parse(tokens).unwrap()
}