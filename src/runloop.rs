use std::future::Future;

pub fn spawn<C>(fun: C)
where
    C: Future<Output = ()> + Send + 'static,
{
    #[cfg(not(target_arch = "wasm32"))]
    tokio::spawn(fun);

    #[cfg(target_arch = "wasm32")]
    wasm_bindgen_futures::spawn_local(fun);
}
