export $(grep -v '^#' .env | xargs)
cargo watch -x run --ignore src/html.rs