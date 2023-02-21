export $(cat .env | xargs)
cargo watch -x run --ignore src/html.rs