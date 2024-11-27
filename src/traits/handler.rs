//! Handler trait defines the signature of a handler function
//! that can be used to implement a service handler

use crate::prelude::{BoxBody, Responder};
use std::future::Future;

#[diagnostic::on_unimplemented(
    message = "Check the function signature and make sure it matches the format `async fn(..) -> impl Responder<Body = tosic_http::body::BoxBody>`",
    label = "Used here",
    note = "The above format is valid with up to 26 parameters where each parameter implements the `FromRequest` trait"
)]
/// # Handler trait
///
/// The Handler trait defines the signature of a handler function
/// that can be used to implement a service handler
pub trait Handler<Args>: Send + Sync + 'static {
    /// The output type of the handler
    type Output: Responder<Body = BoxBody>;
    /// The future type of the handler
    type Future: Future<Output = Self::Output> + Send;

    /// Calls the handler function with the given arguments
    fn call(&self, args: Args) -> Self::Future;
}

macro_rules! handler_tuple ({ $($param:ident)* } => {
    impl<Func, Fut, $($param,)*> Handler<($($param,)*)> for Func
    where
        Func: Fn($($param),*) -> Fut + Send + Sync + 'static,
        Fut: Future + Send,
        Fut::Output: Responder<Body = BoxBody>
    {
        type Output = Fut::Output;
        type Future = Fut;

        #[inline]
        #[allow(non_snake_case)]
        fn call(&self, ($($param,)*): ($($param,)*)) -> Self::Future {
            (self)($($param,)*)
        }
    }
});

handler_tuple! {}
handler_tuple! { A }
handler_tuple! { A B }
handler_tuple! { A B C }
handler_tuple! { A B C D }
handler_tuple! { A B C D E }
handler_tuple! { A B C D E F }
handler_tuple! { A B C D E F G }
handler_tuple! { A B C D E F G H }
handler_tuple! { A B C D E F G H I }
handler_tuple! { A B C D E F G H I J }
handler_tuple! { A B C D E F G H I J K }
handler_tuple! { A B C D E F G H I J K L }
handler_tuple! { A B C D E F G H I J K L M }
handler_tuple! { A B C D E F G H I J K L M N }
handler_tuple! { A B C D E F G H I J K L M N O }
handler_tuple! { A B C D E F G H I J K L M N O P }
handler_tuple! { A B C D E F G H I J K L M N O P Q }
handler_tuple! { A B C D E F G H I J K L M N O P Q R }
handler_tuple! { A B C D E F G H I J K L M N O P Q R S }
handler_tuple! { A B C D E F G H I J K L M N O P Q R S T }
handler_tuple! { A B C D E F G H I J K L M N O P Q R S T U }
handler_tuple! { A B C D E F G H I J K L M N O P Q R S T U V }
handler_tuple! { A B C D E F G H I J K L M N O P Q R S T U V W }
handler_tuple! { A B C D E F G H I J K L M N O P Q R S T U V W X }
handler_tuple! { A B C D E F G H I J K L M N O P Q R S T U V W X Y }
handler_tuple! { A B C D E F G H I J K L M N O P Q R S T U V W X Y Z }
