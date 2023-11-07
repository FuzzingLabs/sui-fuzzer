all:
	cargo build
	cargo run -- --config-path $(CONFIG_PATH) --target-function fuzzinglabs 2> log
