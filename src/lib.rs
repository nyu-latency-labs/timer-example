#![feature(
    arbitrary_self_types,
    proc_macro_hygiene
)]

tarpc::service! {
    rpc hello() -> i32;
}
