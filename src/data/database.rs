use crate::prelude::*;

lazy_static! {
    pub static ref DATABASE_FILENAME: String = {
        #[allow(
            clippy::expect_used,
            reason = "If anything fails here, it should fail."
        )]
        std::env::var("DATABASE_URL").expect("Failed to get `DATABASE_URL` from the environment.")
    };
}
