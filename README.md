# actix-web-middleware-requestid

[![crates.io](https://img.shields.io/crates/v/actix-web-middleware-requestid.svg)](https://crates.io/crates/actix-web-middleware-requestid)
[![Documentation](https://docs.rs/actix-web-middleware-requestid/badge.svg)](https://docs.rs/actix-web-middleware-requestid)
[![License](https://img.shields.io/crates/l/actix-web-middleware-requestid.svg)](https://github.com/tonkonogov/actix-web-middleware-requestid#license)

Request ID middleware for the actix-web framework v1.x

# Usage

Add the package to Cargo.toml:

```toml
[dependencies]
actix-web-middleware-requestid = "1.0"
```

Import and add middleware to your server definition:

```rust
use actix_web_middleware_requestid::RequestIDWrapper;

...

App::new()
    ...
    .wrap(RequestIDWrapper)
```

# Minimal example

```rust
use actix_web::{middleware, web, App, HttpResponse, HttpServer};
use actix_web_middleware_requestid::{RequestID, RequestIDWrapper};

// actix web application state
pub struct AppState {
    pub logger: slog::Logger,
}

fn index((state, id): (web::Data<AppState>, RequestID)) -> HttpResponse {
    let logger = state.logger.new(slog::o!("request_id" => id.0));

    slog::info!(logger, "i am request");

    HttpResponse::Ok()
        .content_type("application/json")
        .body("{}")
}

const LOG_TPLT: &str = "[Code: %s] [Payload: %b] [TTS: %T], request_id: %{x-request-id}i";

fn init_logger() -> slog::Logger {
    use slog::Drain;

    let decorator = slog_term::TermDecorator::new().build();
    let drain = slog_term::FullFormat::new(decorator).build().fuse();
    let drain = slog_async::Async::new(drain).chan_size(512).build().fuse();
    slog::Logger::root(drain, slog::o!())
}

fn main() -> std::io::Result<()> {
    // define env vars if missed
    dotenv::dotenv().ok();

    // initialise the logger
    let root_log = init_logger();
    // define scope logger (for middleware logging)
    let _scope_guard = slog_scope::set_global_logger(root_log.new(slog::o!()));
    slog_stdlog::init().unwrap();

    // slog wrapper to catch log-based logs
    slog_scope::scope(&root_log.new(slog::o!()), || {
        HttpServer::new(move || {
            App::new()
                .data(AppState {
                    logger: root_log.new(slog::o!()),
                })
                .wrap(middleware::Logger::new(LOG_TPLT))
                .wrap(RequestIDWrapper)
                .service(web::resource("/").to(index))
        })
        .bind("0.0.0.0:8080")?
        .run()
    })
}
```

# For actix-web < 1.0

Consider using a similar crate [actix-web-requestid](https://crates.io/crates/actix-web-requestid)

# License

This project is licensed under either of

* Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or [http://www.apache.org/licenses/LICENSE-2.0](http://www.apache.org/licenses/LICENSE-2.0))
* MIT license ([LICENSE-MIT](LICENSE-MIT) or [http://opensource.org/licenses/MIT](http://opensource.org/licenses/MIT))

at your option.
