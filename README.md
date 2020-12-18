# kdmapi-rs

very basic kdmapi bindings for rust

example code:
```Rust
fn test_midi() {
    let res = kdmapi::init();
    match res {
        Ok(()) => (),
        Err(x) => {
            println!("{}", x);
            unreachable!();
        }
    }

    // play a C4 for 1 second
    kdmapi::send_direct_data(0x007F3090);
    std::thread::sleep(std::time::Duration::from_millis(1000));

    kdmapi::terminate();
}
```