echo "Building!"
cargo build --release
echo "Sending!"
sshpass -p "maker" scp ~/Projects/Icarus/icarus/target/armv5te-unknown-linux-musleabi/release/icarus robot@172.20.10.7:/home/robot/
echo "Connecting!"
sshpass -p "maker" ssh -o StrictHostKeyChecking=no robot@172.20.10.7 "export RUST_BACKTRACE=full && ./icarus"