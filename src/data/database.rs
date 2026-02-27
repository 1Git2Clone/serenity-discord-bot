use crate::prelude::*;

pub static DATABASE_FILENAME: LazyLock<String> = LazyLock::new(|| {
    #[allow(
        clippy::expect_used,
        reason = "If anything fails here, it should fail."
    )]
    std::env::var("DATABASE_URL").expect("Failed to get `DATABASE_URL` from the environment.")
});
