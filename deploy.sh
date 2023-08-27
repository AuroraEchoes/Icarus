echo "Building!"
cargo build --release
echo "Sending!"
sshpass -p "maker" scp ~/Projects/CompanyLame/icarus/target/armv5te-unknown-linux-musleabi/release/icarus robot@10.0.0.214:/home/robot/
echo "Connecting!"
sshpass -p "maker" ssh -o StrictHostKeyChecking=no robot@10.0.0.214 "export RUST_BACKTRACE=full && ./icarus"