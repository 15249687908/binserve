use crate::cli::interface;

use super::{
    files, server, templates, watcher,
    routes::RouteHandle,
    config::BinserveConfig,
};

use crate::cli::messages::{Type, push_message};

pub fn init() -> anyhow::Result<()> {
    let start_time = std::time::Instant::now();

    // generate the boilerplate starter public directory
    files::generate_starter_boilerplate()?;

    // generate the boilerplate configuration file
    BinserveConfig::generate_default_config()?;

    // read the configuration file
    let mut config = BinserveConfig::read()?;

    // override with cli configurations if any
    let cli_args = interface::args();
    cli_args.value_of("host").map(|host| config.server.host = host.into());
    cli_args.value_of("tls_key").map(|tls_key| config.server.tls.key = tls_key.into());
    cli_args.value_of("tls_cert").map(|tls_cert| config.server.tls.key = tls_cert.into());

    // prepare template partials
    let handlebars_handle = templates::render_templates(&config)?;

    // prepare routes table
    RouteHandle::add_routes(&config.routes, &handlebars_handle)?;

    let end_time = start_time.elapsed();

    if end_time.as_millis() == 0 {
        push_message(
            Type::Info,
            &format!("Build finished in {} μs ⚡", end_time.as_micros())
        )
    } else {
        push_message(
            Type::Info,
            &format!("Build finished in {} ms ⚡", end_time.as_millis())
        )
    }

    if config.server.tls.enable {
        push_message(
            Type::Info,
            "Enabled TLS (HTTPS) 🔒"
        )
    }

    if config.config.enable_logging {
        push_message(
            Type::Info,
            "Enabled logging 📜"
        )
    }

    // start the hot reloader (file wacther)
    std::thread::spawn(|| {
        watcher::hot_reload_files()
    });

    // and finally server take off!
    server::run_server(config)?;

    Ok(())
}