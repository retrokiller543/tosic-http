use std::future::Future;

pub trait Handler<Args>: 'static {
    type Output;
    type Future: Future<Output = Self::Output>;

    fn call(&self, args: Args) -> Self::Future;
}

macro_rules! handler_tuple ({ $($param:ident)* } => {
    impl<Func, Fut, $($param,)*> Handler<($($param,)*)> for Func
    where
        Func: Fn($($param),*) -> Fut + 'static,
        Fut: Future,
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
