all:
	cargo build
	cargo run -- --config-path $(CONFIG_PATH) 2> log
