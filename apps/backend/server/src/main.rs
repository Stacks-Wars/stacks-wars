// Stacks Wars Entry Point

#[tokio::main]
async fn main() {
    stacks_wars_server::start_server().await;
}
