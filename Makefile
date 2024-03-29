all:
	cargo build --release
	if [ -z "$(DETECTORS)" ]; then \
		if [ -z "$(TARGET_FUNCTIONS)" ]; then \
            	cargo run  --release -- --config-path $(CONFIG_PATH) --target-module $(TARGET_MODULE) --target-function $(TARGET_FUNCTION) 2> log; \
				else \
				cargo run  --release -- --config-path $(CONFIG_PATH) --target-module $(TARGET_MODULE) --functions $(TARGET_FUNCTIONS) 2> log; \
		fi \
        else \
            	cargo run  --release -- --config-path $(CONFIG_PATH) --target-module $(TARGET_MODULE) --target-function $(TARGET_FUNCTION) --detectors $(DETECTORS) 2> log; \
        fi


list_functions:
	cargo run --release -- --config-path $(CONFIG_PATH) --target-module $(TARGET_MODULE) -l


replay:
	cargo run --release -- --config-path $(CONFIG_PATH) --replay $(CRASHFILE)

