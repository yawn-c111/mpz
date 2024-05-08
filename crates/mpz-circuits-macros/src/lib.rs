mod evaluate;
mod test;

use proc_macro::TokenStream;

#[proc_macro]
pub fn evaluate(item: TokenStream) -> TokenStream {
    evaluate::evaluate_impl(item)
}

#[proc_macro]
pub fn test_circ(item: TokenStream) -> TokenStream {
    test::test_impl(item)
}
