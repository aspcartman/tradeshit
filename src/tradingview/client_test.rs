use super::client::{lol, lolreq};

#[test]
fn it_works() {
    let result = lol();
    assert_eq!(result.unwrap(), 10);
}

#[tokio::test]
async fn it_works2() {
    let result = lolreq();
    assert_eq!(result.await.unwrap(), "".to_string());
}
