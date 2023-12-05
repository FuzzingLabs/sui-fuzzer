all:
	cargo run --release -- --config-path $(CONFIG_PATH) --target-module $(TARGET_MODULE) --target-function $(TARGET_FUNCTION)
