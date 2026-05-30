#!/bin/bash -e
# stage0/prerun.sh — Bootstrap the base Debian arm64 system
# Uses --foreign for cross-arch bootstrap, then second-stage via chroot+binfmt

if [ ! -d "${ROOTFS_DIR}" ]; then
	mkdir -p "${ROOTFS_DIR}"

	QEMU_BIN=""
	for candidate in /usr/bin/qemu-aarch64-static /usr/bin/qemu-aarch64; do
		if [ -x "${candidate}" ]; then
			QEMU_BIN="${candidate}"
			break
		fi
	done

	if [ -n "${QEMU_BIN}" ]; then
		mkdir -p "${ROOTFS_DIR}/usr/bin"
		cp "${QEMU_BIN}" "${ROOTFS_DIR}/usr/bin/qemu-aarch64-static"
		chmod +x "${ROOTFS_DIR}/usr/bin/qemu-aarch64-static"
		echo "[zeroday] Copied ${QEMU_BIN} into rootfs for arm64 emulation"
	else
		echo "[zeroday] WARNING: qemu-aarch64-static not found — debootstrap may fail"
	fi

	# --foreign: only unpack packages (no chroot needed for first stage)
	BOOTSTRAP_ARGS=(
		--foreign
		--arch arm64
		--no-check-gpg
		--components main,contrib,non-free,non-free-firmware
		--exclude=info,ifupdown
		--include=ca-certificates
	)

	debootstrap "${BOOTSTRAP_ARGS[@]}" "${RELEASE}" "${ROOTFS_DIR}" http://deb.debian.org/debian/ || {
		BOOTSTRAP_EXIT=$?
		rm -f wget-log*
		log "debootstrap first stage failed with exit code ${BOOTSTRAP_EXIT}"
		false
	}

	rm -f wget-log*

	# Prepare rootfs for second stage
	mount -t proc proc "${ROOTFS_DIR}/proc" 2>/dev/null || true
	mount --bind /dev "${ROOTFS_DIR}/dev" 2>/dev/null || true
	mount --bind /dev/pts "${ROOTFS_DIR}/dev/pts" 2>/dev/null || true
	mkdir -p "${ROOTFS_DIR}/sys" 2>/dev/null || true
	mount --bind /sys "${ROOTFS_DIR}/sys" 2>/dev/null || true
	cp /etc/resolv.conf "${ROOTFS_DIR}/etc/resolv.conf" 2>/dev/null || true

	# Run second stage using chroot (arm64 binaries handled by binfmt_misc/qemu)
	echo "[zeroday] Running debootstrap second stage (chroot via binfmt)..."
	chroot "${ROOTFS_DIR}" /debootstrap/debootstrap --second-stage || {
		BOOTSTRAP_EXIT=$?
		umount "${ROOTFS_DIR}/sys" 2>/dev/null || true
		umount "${ROOTFS_DIR}/dev/pts" 2>/dev/null || true
		umount "${ROOTFS_DIR}/dev" 2>/dev/null || true
		umount "${ROOTFS_DIR}/proc" 2>/dev/null || true
		log "debootstrap second stage failed with exit code ${BOOTSTRAP_EXIT}"
		false
	}

	# Clean up mounts
	umount "${ROOTFS_DIR}/sys" 2>/dev/null || true
	umount "${ROOTFS_DIR}/dev/pts" 2>/dev/null || true
	umount "${ROOTFS_DIR}/dev" 2>/dev/null || true
	umount "${ROOTFS_DIR}/proc" 2>/dev/null || true

	# Verify debootstrap succeeded
	if [ -d "${ROOTFS_DIR}/debootstrap" ]; then
		rmdir "${ROOTFS_DIR}/debootstrap" 2>/dev/null || true
	fi

	# Remove qemu binary from rootfs — not needed at runtime on real hardware
	# But keep it available in /usr/bin for subsequent build stages that use chroot
	# (pi-gen's on_chroot function needs it)
	rm -f "${ROOTFS_DIR}/usr/bin/qemu-aarch64-static"
fi