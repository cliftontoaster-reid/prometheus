echo 'Building...'
cargo build
echo 'Sending debug file...'
ssh alarm@192.168.1.21 "killall test-prometheus"
scp -r ./target/debug/prometheus alarm@192.168.1.21:/home/alarm/test-prometheus
echo 'Starting the boot...'
ssh alarm@192.168.1.21 ./test-prometheus --dev--
