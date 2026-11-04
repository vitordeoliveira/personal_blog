//! The database: the posts' view counts, the site's visits and the admin's sessions.

use toasty::Db;
use topcoat::context::{Cx, app_context};

/// Opens the database with every model in the application.
///
/// The tables come from migrations. After changing a model, generate a migration with
/// `cargo run -- toasty migration generate --name <what_changed>` and apply it with
/// `cargo run -- toasty migration apply`.
pub async fn connect() -> toasty::Result<Db> {
    Db::builder()
        .models(toasty::models!(crate::*))
        .connect(&url())
        .await
}

/// `DATABASE_URL`, or else `data.db` in the storage folder oxploy gives the site, which
/// its deploys keep, or else `data.db` where the site runs.
fn url() -> String {
    if let Ok(url) = std::env::var("DATABASE_URL") {
        return url;
    }
    match std::env::var("OXPLOY_STORAGE_DIR") {
        Ok(storage) => format!("sqlite:{storage}/data.db"),
        Err(_) => "sqlite:data.db".to_owned(),
    }
}

/// Returns the database handle from the router's app context. Cloning it is cheap
/// because it refers to a shared connection pool.
pub fn db(cx: &Cx) -> Db {
    app_context::<Db>(cx).clone()
}
