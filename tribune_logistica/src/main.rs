
mod test;
use tribune_logistica::server;

#[tokio::main]
async fn main(){
    server().await
}