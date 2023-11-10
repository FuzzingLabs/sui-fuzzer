
sudo docker build -t sui-fuzzer .
sudo docker run -it sui-fuzzer /bin/bash -c 'cd sui-fuzzer && make CONFIG_PATH="./config.json" TARGET_FUNCTION="fuzzinglabs"'