use std::sync::atomic::{AtomicUsize, Ordering};

static SHUTDOWN_COUNT: AtomicUsize = AtomicUsize::new(0);

pub fn install() {
    unsafe {
        let action_term = libc::sigaction {
            sa_sigaction: handle_signal as *const () as usize,
            sa_flags: libc::SA_RESTART,
            sa_mask: std::mem::zeroed(),
            sa_restorer: None,
        };
        libc::sigaction(libc::SIGTERM, &action_term, std::ptr::null_mut());
        libc::sigaction(libc::SIGHUP, &action_term, std::ptr::null_mut());

        let action_usr = libc::sigaction {
            sa_sigaction: handle_usr as *const () as usize,
            sa_flags: libc::SA_RESTART,
            sa_mask: std::mem::zeroed(),
            sa_restorer: None,
        };
        libc::sigaction(libc::SIGUSR2, &action_usr, std::ptr::null_mut());
    }
    log::info!("Signal handlers installed (SIGTERM, SIGHUP, SIGUSR2=hotplug)");
}

extern "C" fn handle_signal(sig: libc::c_int, _info: *mut libc::siginfo_t, _ctx: *mut libc::c_void) {
    let count = SHUTDOWN_COUNT.fetch_add(1, Ordering::SeqCst);
    if count > 0 {
        unsafe { libc::_exit(sig); }
    }

    log::info!("Received signal {}, killing child processes", sig);

    let _ = std::process::Command::new("killall")
        .args(["-q", "cyber_launcher"])
        .spawn();
    let _ = std::process::Command::new("killall")
        .args(["-q", "st"])
        .spawn();

    std::process::exit(0);
}

extern "C" fn handle_usr(_sig: libc::c_int, _info: *mut libc::siginfo_t, _ctx: *mut libc::c_void) {
    log::info!("SIGUSR2: input/output hotplug event — rescan devices");
}