
sudo docker build -t sui-fuzzer .
sudo docker run -it -v $(pwd):/app/ sui-fuzzer /bin/bash -c "cd /app && make  $*"