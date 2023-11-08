all:
	cargo build
	cargo run -- --config-path $(CONFIG_PATH) --target-function $(TARGET_FUNCTION) 2> log
