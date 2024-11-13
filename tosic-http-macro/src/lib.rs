mod service;

extern crate proc_macro;
use proc_macro::TokenStream;

macro_rules! service_method {
    ($ident:ident, $str:literal) => {
        #[proc_macro_attribute]
        /// Generates a `HttpService` implemementation for the given HTTP method
        pub fn $ident(args: TokenStream, input: TokenStream) -> TokenStream {
            $crate::service::service(args, input, $str)
        }
    };
}

service_method!(get, "get");
service_method!(post, "post");
service_method!(put, "put");
service_method!(delete, "delete");
