STAGING_DIR=/home/jianingy/src/openwrt_widora/staging_dir
bbb:
	cargo build --target arm-unknown-linux-gnueabihf 
	rsync -a target/arm-unknown-linux-gnueabihf/debug/airstation debian@192.168.7.2:~/
	rsync -a static debian@192.168.7.2:~/airstation/
bbb-release:
	cargo build --target arm-unknown-linux-gnueabihf --release
	rsync -a target/arm-unknown-linux-gnueabihf/release/airstation debian@192.168.7.2:~/airstation/
	rsync -a static debian@192.168.7.2:~/airstation/
widora:
	cargo build --target mips-unknown-linux-gnu
	rsync -a target/mips-unknown-linux-gnu/debug/airstation root@192.168.78.137:~/
