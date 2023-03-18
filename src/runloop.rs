use std::future::Future;

fn spawn<C>(fun: C)
where
    C: Future + Send + 'static,
    C::Output: Future + Send + 'static,
{
    #[cfg(not(target_arch = "wasm32"))]
    tokio::spawn(fun);

    #[cfg(target_arch = "wasm32")]
    wasm_bindgen_futures::spawn_local(fun);
}
