all:
	cargo build
	cargo run -- --module-path $(MODULE_PATH) 2> log
