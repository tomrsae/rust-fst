use proc_macro::TokenStream;
use syn::DeriveInput;

#[proc_macro]
pub fn make_fsm(tokens : TokenStream) -> TokenStream {
    let ast = syn::parse(tokens).unwrap();

    impl_make_fsm(&ast) 
}

fn impl_make_fsm(ast: &DeriveInput) -> TokenStream {
    panic!("Not implemented!")
}