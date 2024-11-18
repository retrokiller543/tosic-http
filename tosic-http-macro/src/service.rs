use proc_macro::TokenStream;
use proc_macro2::{Ident, TokenStream as TokenStream2};
use quote::quote;
use syn::{parse_macro_input, FnArg, ItemFn, PatType, ReturnType};

pub(crate) fn service(
    args: TokenStream,
    input: TokenStream,
    method: &str,
) -> proc_macro::TokenStream {
    // Parse the input TokenStream into a syn::ItemFn
    let input = parse_macro_input!(input as ItemFn);

    // Check if the input function is async
    if input.sig.asyncness.is_none() {
        return syn::Error::new_spanned(input.sig.fn_token, "async fn is required")
            .to_compile_error()
            .into();
    }

    // Generate identifiers
    let crate_name = Ident::new("tosic_http", proc_macro2::Span::call_site());
    let fn_name = input.sig.ident.clone();
    let method_ident = Ident::new(
        method.to_uppercase().as_str(),
        proc_macro2::Span::call_site(),
    );

    // Generate parameter destructuring for `call` function
    let mut idents = Vec::new();
    let mut types = Vec::new();
    let fn_inputs = &input.sig.inputs;

    fn_inputs.clone().into_iter().for_each(|arg| {
        if let FnArg::Typed(PatType { pat, ty, .. }) = arg {
            idents.push(pat);
            types.push(ty);
        }
    });

    let args: TokenStream2 = args.into();

    let return_type = match input.sig.output {
        ReturnType::Default => quote!(()),
        ReturnType::Type(_, ref ty) => quote!(#ty),
    };

    let doc_attrs: Vec<_> = input
        .attrs
        .iter()
        .filter(|attr| attr.path().is_ident("doc"))
        .cloned() // Clone since we will use the attribute in a new place
        .collect();

    let vis = &input.vis;

    // Expand the macro to generate the desired output code
    let expanded: TokenStream2 = quote! {
        #[allow(non_camel_case_types)]
        #(#doc_attrs)*
        #vis struct #fn_name;

        impl #crate_name::traits::handler::Handler<(#(#types,)*)> for #fn_name
        {
            type Output = #return_type;
            type Future = impl std::future::Future<Output = Self::Output>;

            #[inline]
            fn call(&self, (#(#idents,)*): (#(#types,)*)) -> Self::Future {
                #input

                #fn_name(#(#idents),*)
            }
        }

        impl #crate_name::services::HttpService<(#(#types,)*)> for #fn_name {
            const METHOD: #crate_name::prelude::Method = #crate_name::prelude::Method::#method_ident;
            const PATH: &'static str = #args;
        }
    };

    proc_macro::TokenStream::from(expanded)
}
