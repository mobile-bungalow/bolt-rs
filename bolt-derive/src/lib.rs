use proc_macro::TokenStream;

#[proc_macro_derive(BoltObject)]
pub fn derive_bolt_object(_input: TokenStream) -> TokenStream {
    todo!()
}

#[proc_macro_derive(BoltMethods)]
pub fn derive_bolt_object_impl(_input: TokenStream) -> TokenStream {
    todo!();
}

#[proc_macro_derive(BoltModule)]
pub fn derive_bolt_object_module(_input: TokenStream) -> TokenStream {
    todo!();
}
