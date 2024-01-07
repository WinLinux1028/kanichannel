mod create;
mod edit;
mod remove;

pub async fn run(mut args: std::env::Args) {
    match args.next().as_deref() {
        Some("create") => create::run().await,
        Some("edit") => edit::run().await,
        Some("remove") => remove::run().await,
        _ => help(),
    }
}

fn help() {
    println!("board [sub command] [options...]");
    println!("sub commands: create, edit, remove, help");
}
