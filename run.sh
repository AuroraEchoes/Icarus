echo "Connecting!"
sshpass -p "maker" ssh -o StrictHostKeyChecking=no robot@172.20.10.7 "export RUST_BACKTRACE=full && ./icarus"