all:
	cargo build
	cargo run -- --package-path $(PACKAGE_PATH) 2> log
