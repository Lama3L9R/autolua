///
/// This crate is for functions that are imported from
/// an existing project due to visibility issues.
///

use proc_macro2::Span;
use syn::parse::ParseStream;

///
/// Liberate parsing::keyword from private domain
///
pub fn keyword(input: ParseStream, token: &str) -> syn::Result<Span> {
    input.step(|cursor| {
        if let Some((ident, rest)) = cursor.ident() {
            if ident == token {
                return Ok((ident.span(), rest));
            }
        }
        Err(cursor.error(format!("expected `{}`", token)))
    })
}


