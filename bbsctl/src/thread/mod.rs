mod remove;

pub async fn run(mut args: std::env::Args) {
    match args.next().as_deref() {
        Some("remove") => remove::run().await,
        _ => help(),
    }
}

fn help() {
    println!("thread [sub command] [options...]");
    println!("sub commands: remove, help");
}
