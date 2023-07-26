export $(cat .env.prod | xargs)
cargo watch -x run --ignore src/html.rs