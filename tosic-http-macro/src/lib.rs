extern crate proc_macro;
use darling::ast::NestedMeta;
use darling::{Error, FromMeta};
use proc_macro::TokenStream;
use proc_macro2::{Ident, TokenStream as TokenStream2};
use quote::quote;
use syn::{parse_str, FnArg, ItemFn, PatType};

/// `RouteArgs` holds the arguments for the `route` attribute macro.
///
/// # Fields
/// - `method`: The HTTP method as a string (e.g., `"GET"`, `"POST"`).
/// - `path`: The URL path as a string (e.g., `"/example"`).
///
/// This struct is parsed from the macro attribute arguments.
#[derive(Debug, FromMeta)]
struct RouteArgs {
    method: String,
    path: String,
}

/// The `route` attribute macro transforms an async function into a handler struct that
/// implements the `Handler` trait, allowing it to be registered as a route in a server.
///
///
/// # Requirements
/// The function annotated with `#[route]` must be asynchronous (`async fn`).
///
/// # Errors
/// - Returns a compile-time error if `RouteArgs` cannot be parsed.
/// - Returns a compile-time error if the function is not async.
///
/// # Examples
/// ```ignore
/// use tosic_http::*;
///
/// #[route(method = "GET", path = "/example")]
/// async fn example_handler() -> ResponseType {
///     // Handler implementation
/// }
/// ```
///
/// This example creates a route for a `GET` request to `"/example"`, which will call
/// `example_handler`.
#[proc_macro_attribute]
pub fn route(args: TokenStream, input: TokenStream) -> TokenStream {
    let attr_args = match NestedMeta::parse_meta_list(args.into()) {
        Ok(v) => v,
        Err(e) => {
            return TokenStream::from(Error::from(e).write_errors());
        }
    };
    let input = syn::parse_macro_input!(input as ItemFn);

    let args = match RouteArgs::from_list(&attr_args) {
        Ok(v) => v,
        Err(e) => {
            return TokenStream::from(e.write_errors());
        }
    };

    let crate_name = Ident::new("tosic_http", proc_macro2::Span::call_site());

    if input.sig.asyncness.is_none() {
        return TokenStream::from(Error::custom("async fn is required").write_errors());
    }

    // Collect parameter identifiers as `Ident` instances.
    let param_idents: Vec<Ident> = input
        .sig
        .inputs
        .iter()
        .filter_map(|arg| {
            if let FnArg::Typed(PatType { pat, .. }) = arg {
                if let syn::Pat::Ident(ref pat_ident) = pat.as_ref() {
                    parse_str::<Ident>(&pat_ident.ident.to_string()).ok()
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect();

    let fn_inputs = &input.sig.inputs;

    // Generate tokens for extracting parameters from `incoming_request`.
    let param_extraction: TokenStream2 = fn_inputs.clone().iter().map(|arg| {
        let (ident, ty) = if let FnArg::Typed(PatType { pat, ty, .. }) = arg {
            (pat, ty)
        } else {
            unreachable!()
        };

        quote! {
            let #ident = match <#ty as #crate_name::traits::from_request::FromRequest>::from_request(&incoming_request).await {
                Ok(value) => value,
                Err(err) => {
                    let response = http::Response::builder()
                        .status(400)
                        .body(err.to_string().into_bytes())
                        .unwrap();
                    return Box::new(response) as ResponseType;
                }
            };
        }
    }).collect();

    // Generate tokens for function parameters in the call.
    let fn_call_params: TokenStream2 = quote! { #(#param_idents),* };

    let fn_name = &input.sig.ident;
    let fn_body = &input.block;
    let fn_visibility = &input.vis;
    let fn_output = &input.sig.output;
    let path = &args.path;
    let method = &args.method;
    let fn_attrs = &input.attrs;

    let expanded = quote! {
        #[allow(non_camel_case_types)]
        #fn_visibility struct #fn_name;

        impl #crate_name::traits::handler::Handler for #fn_name {
            fn call(&self, incoming_request: #crate_name::request::Request) -> #crate_name::traits::handler::ResponseFuture<'_> {
                #(#fn_attrs)*
                async fn #fn_name(#fn_inputs) #fn_output #fn_body

                Box::pin(async move {
                    #param_extraction

                    #fn_name(#fn_call_params).await
                })

            }

            fn method(&self) -> http::Method {
                http::Method::from_bytes(#method.as_bytes()).unwrap()
            }

            fn path(&self) -> String {
                #path.to_string()
            }
        }
    };

    TokenStream::from(expanded)
}
