/*
 *  Hermes - Discord bot for integrating UVa's Tablón into Discord servers.
 *  Copyright (C) 2025  Manuel de Castro
 *
 *  This program is free software: you can redistribute it and/or modify
 *  it under the terms of the GNU General Public License as published by
 *  the Free Software Foundation, either version 3 of the License, or
 *  (at your option) any later version.
 *
 *  This program is distributed in the hope that it will be useful,
 *  but WITHOUT ANY WARRANTY; without even the implied warranty of
 *  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 *  GNU General Public License for more details.
 *
 *  You should have received a copy of the GNU General Public License
 *  along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */
/*
 * We have to make the project a library to define a new procedural macro for some reason (seems to
 * be related with how Rust compilation works).
 */
extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use quote::ToTokens as _; // To use function.into_token_stream().
use syn::spanned::Spanned as _; // To use span() on language items.
use syn::{parse_macro_input, ItemFn};

/*
 * Reference:
 * https://users.rust-lang.org/t/using-macros-to-modify-ast-to-modify-and-add-line-of-codes-in-function/56805/5
 */
#[proc_macro_attribute]
pub fn log_cmd(_macro_attrs: TokenStream, function: TokenStream) -> TokenStream {
    // Parse the function's tokens using syn:
    let mut function = parse_macro_input!(function as ItemFn);
    // Extract the first argument of the function:
    let Some(first_arg) = function.sig.inputs.first() else {
        return darling::Error::from(syn::Error::new(
            function.sig.span(),
            "[log_cmd] function must have at least one argument",
        ))
        .write_errors()
        .into();
    };
    // Cast the first argument to a typed argument
    // (i.e. `ctx: Context<'_>`):
    let ctx_arg = if let syn::FnArg::Typed(arg) = first_arg {
        arg
    } else {
        // syn::FnArg::Receiver(_)
        return darling::Error::from(syn::Error::new(
            first_arg.span(),
            "[log_cmd] `self` argument is not allowed",
        ))
        .write_errors()
        .into();
    };
    // Extract the identifier of the first argument:
    let syn::Pat::Ident(ident) = &*ctx_arg.pat else {
        return darling::Error::from(syn::Error::new(
            ctx_arg.pat.span(),
            "[log_cmd] expected an identifier",
        ))
        .write_errors()
        .into();
    };
    let ctx_ident = ident.ident.clone();

    // Insert a new statement at the beginning of the function,
    // logging the usage of the command to stderr using elog_cmd! and the provided context:
    function.block.stmts.insert(
        0,
        syn::parse(
            quote! {
            crate::utils::elog_cmd!(#ctx_ident);
            }
            .into(),
        )
        .unwrap(),
    );

    // Return the modified function as a TokenStream:
    function.into_token_stream().into()
}
