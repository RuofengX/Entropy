mod api;
pub mod config;
mod db;
mod entity;
pub mod err;

pub async fn start_server(config: config::Root) -> Result<(), err::RuntimeError> {
    let db = db::prepare_db(config.db).await?;

    if config.http.enable {
        api::http::http_daemon(config.http, &db).await?;
    };
    // if config.socket.enable{
    // api::zmq::socket_daemon(config.socket, &db).await?;
    // };
    Ok(())
}
