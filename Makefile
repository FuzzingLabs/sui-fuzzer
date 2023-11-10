all:
	cargo run --release -- --config-path $(CONFIG_PATH) --target-function $(TARGET_FUNCTION)
