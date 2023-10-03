echo "Building!"
cargo build --release
echo "Sending!"
sshpass -p "maker" scp ~/Projects/Icarus/icarus/target/armv5te-unknown-linux-musleabi/release/icarus robot@10.0.0.214:/home/robot/