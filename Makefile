all:
	if [ -z "$(DETECTORS)" ]; then \
            	cargo run --release -- --config-path $(CONFIG_PATH) --target-module $(TARGET_MODULE) --target-function $(TARGET_FUNCTION) 2> log; \
        else \
            	cargo run --release -- --config-path $(CONFIG_PATH) --target-module $(TARGET_MODULE) --target-function $(TARGET_FUNCTION) --detectors $(DETECTORS) 2> log; \
        fi


list_functions:
	cargo run --release -- --config-path $(CONFIG_PATH) -l

