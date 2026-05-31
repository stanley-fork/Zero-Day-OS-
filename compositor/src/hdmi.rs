use std::os::fd::AsRawFd;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

const DRM_DP_STATUS_PATH: &str = "/sys/class/drm/card0-HDMI-A-1/status";

static HDMI_CONNECTED: AtomicBool = AtomicBool::new(false);

pub fn is_hdmi_connected() -> bool {
    std::fs::read_to_string(DRM_DP_STATUS_PATH)
        .map(|s| s.trim() == "connected")
        .unwrap_or(false)
}

fn wait_for_drm_uevent() {
    let sock = nix::sys::socket::socket(
        nix::sys::socket::AddressFamily::Netlink,
        nix::sys::socket::SockType::Raw,
        nix::sys::socket::SockFlag::empty(),
        nix::sys::socket::SockProtocol::NetlinkKObjectUEvent,
    )
    .expect("failed to create uevent netlink socket");

    let fd = sock.as_raw_fd();
    let addr = nix::sys::socket::NetlinkAddr::new(0, 1);
    nix::sys::socket::bind(fd, &addr).expect("failed to bind uevent socket");

    let sock = sock;
    let mut buf = [0u8; 4096];
    loop {
        match nix::sys::socket::recv(sock.as_raw_fd(), &mut buf, nix::sys::socket::MsgFlags::empty()) {
            Ok(len) => {
                let data = &buf[..len];
                let msg = String::from_utf8_lossy(data);
                if msg.contains("drm") {
                    let connected = is_hdmi_connected();
                    let prev = HDMI_CONNECTED.load(Ordering::SeqCst);
                    if connected != prev {
                        if connected {
                            log::info!("HDMI hotplug: monitor connected");
                        } else {
                            log::info!("HDMI hotplug: monitor disconnected");
                        }
                        HDMI_CONNECTED.store(connected, Ordering::SeqCst);
                    }
                }
            }
            Err(_) => {
                std::thread::sleep(Duration::from_millis(100));
            }
        }
    }
}

pub fn hdmi_hotplug_thread() -> std::thread::JoinHandle<()> {
    let connected = is_hdmi_connected();
    HDMI_CONNECTED.store(connected, Ordering::SeqCst);
    if connected {
        log::info!("HDMI monitor detected at startup");
    } else {
        log::info!("No HDMI monitor detected at startup — will auto-detect on hotplug");
    }

    std::thread::Builder::new()
        .name("hdmi-hotplug".into())
        .stack_size(8192)
        .spawn(move || {
            wait_for_drm_uevent();
        })
        .expect("failed to spawn HDMI hotplug thread")
}

pub fn hdmi_is_connected() -> bool {
    HDMI_CONNECTED.load(Ordering::SeqCst)
}