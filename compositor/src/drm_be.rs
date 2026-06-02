use std::collections::HashMap;
use std::os::fd::AsFd;
use std::os::unix::io::OwnedFd;
use std::time::Duration;

use smithay::backend::allocator::gbm::{GbmAllocator, GbmBufferFlags};
use smithay::backend::drm::{
    DrmDevice, DrmDeviceFd, DrmDeviceNotifier, DrmEvent, DrmNode,
};
use smithay::backend::drm::compositor::{DrmCompositor, FrameFlags, RenderFrameResult};
use smithay::backend::drm::exporter::gbm::GbmFramebufferExporter;
use smithay::backend::egl::{EGLContext, EGLDisplay};
use smithay::backend::renderer::element::surface::{
    render_elements_from_surface_tree, WaylandSurfaceRenderElement,
};
use smithay::backend::renderer::element::Kind;
use smithay::backend::renderer::gles::GlesRenderer;
use smithay::backend::session::{Event as SessionEvent, Session};
use smithay::backend::session::libseat::LibSeatSession;
use smithay::backend::udev::{UdevBackend, UdevEvent};
use smithay::output::{Output, OutputModeSource};
use smithay::reexports::calloop;
use smithay::reexports::drm::control::Device as DrmControlDevice;
use smithay::reexports::drm::control::connector;
use smithay::reexports::gbm;
use smithay::reexports::rustix::fs::OFlags;
use smithay::utils::{Logical, Point, Size};
use smithay::wayland::seat::WaylandFocus;
use drm_fourcc::{DrmFourcc, DrmFormat};

use smithay::desktop::Window;

type GbmDrmCompositor = DrmCompositor<
    GbmAllocator<DrmDeviceFd>,
    GbmFramebufferExporter<DrmDeviceFd>,
    (),
    DrmDeviceFd,
>;

pub struct DrmBackend {
    pub drm_device: DrmDevice,
    pub drm_fd: DrmDeviceFd,
    pub gbm_device: gbm::Device<DrmDeviceFd>,
    pub allocator: GbmAllocator<DrmDeviceFd>,
    pub egl_display: EGLDisplay,
    pub renderer: GlesRenderer,
    pub compositor: GbmDrmCompositor,
    pub output: Output,
    pub render_formats: Vec<DrmFormat>,
    pub lcd_connector: connector::Handle,
    pub hdmi_connector: Option<connector::Handle>,
}

pub struct Backends {
    pub session: LibSeatSession,
    pub backends: HashMap<libc::dev_t, DrmBackend>,
    pub pending_render: bool,
}

impl DrmBackend {
    pub fn handle_connector_change(&mut self) {
        let resources = match self.drm_device.resource_handles() {
            Ok(r) => r,
            Err(e) => {
                log::warn!("Failed to get DRM resources: {:?}", e);
                return;
            }
        };

        let mut hdmi_connected = None;
        for &conn_handle in resources.connectors() {
            if let Ok(info) = self.drm_device.get_connector(conn_handle, false) {
                if info.interface() == connector::Interface::HDMIA && info.state() == connector::State::Connected {
                    hdmi_connected = Some(conn_handle);
                    break;
                }
            }
        }

        match (self.hdmi_connector, hdmi_connected) {
            (None, Some(conn)) => {
                log::info!("HDMI monitor hotplugged");
                if let Err(e) = self.compositor.add_connector(conn) {
                    log::warn!("Failed to add HDMI connector: {:?}", e);
                } else {
                    self.hdmi_connector = Some(conn);
                }
            }
            (Some(_), None) => {
                log::info!("HDMI monitor unplugged");
                let old = self.hdmi_connector.take();
                if let Some(old_conn) = old {
                    if let Err(e) = self.compositor.remove_connector(old_conn) {
                        log::warn!("Failed to remove HDMI connector: {:?}", e);
                    }
                }
            }
            (Some(old_conn), Some(new_conn)) if old_conn != new_conn => {
                let _ = self.compositor.remove_connector(old_conn);
                let _ = self.compositor.add_connector(new_conn);
                self.hdmi_connector = Some(new_conn);
            }
            _ => {}
        }
    }
}

impl Backends {
    pub fn new(
        loop_handle: &calloop::LoopHandle<'_, crate::comp::State>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let (session, session_notifier) = LibSeatSession::new()?;
        let udev = UdevBackend::new(session.seat())?;

        let initial_devices: Vec<(libc::dev_t, std::path::PathBuf)> = udev
            .device_list()
            .map(|(id, path)| (id, path.to_path_buf()))
            .collect();

        for (dev_id, path) in &initial_devices {
            log::info!("Initial GPU device: {:?} at {:?}", dev_id, path);
        }

        loop_handle.insert_source(session_notifier, |event, _, state| {
            match event {
                SessionEvent::PauseSession => {
                    log::info!("Session paused");
                    if let Some(ref mut backends) = state.backends {
                        backends.pending_render = false;
                        for (_, backend) in backends.backends.iter_mut() {
                            backend.drm_device.pause();
                        }
                    }
                }
                SessionEvent::ActivateSession => {
                    log::info!("Session activated");
                    if let Some(ref mut backends) = state.backends {
                        for (_, backend) in backends.backends.iter_mut() {
                            if let Err(e) = backend.drm_device.activate(false) {
                                log::warn!("DRM activate failed: {}", e);
                            }
                        }
                        backends.pending_render = true;
                    }
                }
            }
        })?;

        let mut backends = Backends {
            session,
            backends: HashMap::new(),
            pending_render: false,
        };

        loop_handle.insert_source(udev, |event, _, state| {
            match event {
                UdevEvent::Added { device_id, path } => {
                    log::info!("GPU added: {:?} at {:?}", device_id, path);
                }
                UdevEvent::Changed { device_id } => {
                    log::info!("GPU changed: {:?}", device_id);
                    if let Some(ref mut backends) = state.backends {
                        if let Some(backend) = backends.backends.get_mut(&device_id) {
                            backend.handle_connector_change();
                        }
                        backends.pending_render = true;
                    }
                }
                UdevEvent::Removed { device_id } => {
                    log::info!("GPU removed: {:?}", device_id);
                    if let Some(ref mut backends) = state.backends {
                        backends.backends.remove(&device_id);
                    }
                }
            }
        })?;

        for (dev_id, path) in initial_devices {
            match backends.add_device(dev_id, &path) {
                Ok(drm_notifier) => {
                    loop_handle.insert_source(drm_notifier, |event, _metadata, state| {
                        match event {
                            DrmEvent::VBlank(_crtc) => {
                                if let Some(ref mut backends) = state.backends {
                                    backends.frame_submitted();
                                    backends.pending_render = true;
                                }
                            }
                            DrmEvent::Error(err) => {
                                log::warn!("DRM error: {:?}", err);
                            }
                        }
                    })?;
                }
                Err(e) => log::warn!("Failed to init DRM device {:?}: {}", path, e),
            }
        }

        Ok(backends)
    }

    fn add_device(
        &mut self,
        device_id: libc::dev_t,
        path: &std::path::Path,
    ) -> Result<DrmDeviceNotifier, Box<dyn std::error::Error>> {
        let fd: OwnedFd = self.session.open(path, OFlags::RDWR | OFlags::NONBLOCK)?;
        let drm_fd = DrmDeviceFd::new(fd.into());
        let (mut drm_device, drm_notifier) = DrmDevice::new(drm_fd.clone(), true)?;

        drm_device.activate(false)?;

        let gbm_device = gbm::Device::new(drm_fd.clone())?;
        let allocator = GbmAllocator::new(
            gbm_device.clone(),
            GbmBufferFlags::RENDERING | GbmBufferFlags::SCANOUT,
        );

        let egl_display = unsafe { EGLDisplay::new(gbm_device.clone()) }?;
        let egl_context = EGLContext::new(&egl_display)?;
        let renderer = unsafe { GlesRenderer::new(egl_context) }?;

        let render_formats: Vec<DrmFormat> = egl_display.dmabuf_render_formats().iter().cloned().collect();

        let resources = drm_device.resource_handles()?;
        let connector_handles = resources.connectors();

        let mut lcd_connector = None;
        let mut hdmi_connector = None;
        let mut best_mode = None;

        for &conn_handle in connector_handles {
            let info = drm_device.get_connector(conn_handle, true)?;
            if info.state() != connector::State::Connected {
                continue;
            }
            let conn_type = info.interface();
            if lcd_connector.is_none() {
                lcd_connector = Some(conn_handle);
                if let Some(mode) = info.modes().first() {
                    best_mode = Some(*mode);
                }
            } else if conn_type == connector::Interface::HDMIA && hdmi_connector.is_none() {
                hdmi_connector = Some(conn_handle);
            }
        }

        let (connector, mode) = match (lcd_connector, best_mode) {
            (Some(c), Some(m)) => (c, m),
            _ => {
                log::warn!("No connected connector found on {:?}", path);
                return Err("No connected connector".into());
            }
        };

        let crtc = drm_device.crtcs()[0];
        let surface = drm_device.create_surface(crtc, mode, &[connector])?;

        let output = Output::new(
            "LCD".to_string(),
            smithay::output::PhysicalProperties {
                make: "M5Stack".into(),
                model: "Cardputer Zero".into(),
                size: Size::from((36, 19)),
                subpixel: smithay::output::Subpixel::Unknown,
            },
        );

        let output_mode = smithay::output::Mode {
            size: (mode.size().0 as i32, mode.size().1 as i32).into(),
            refresh: mode.vrefresh() as i32 * 1000,
        };
        output.set_preferred(output_mode);
        output.change_current_state(
            Some(output_mode),
            Some(smithay::utils::Transform::Normal),
            None,
            None,
        );

        let drm_node = DrmNode::from_file(gbm_device.as_fd()).ok();
        let framebuffer_exporter = GbmFramebufferExporter::new(gbm_device.clone(), drm_node);

        let compositor = DrmCompositor::new(
            OutputModeSource::Auto(output.clone()),
            surface,
            None,
            allocator.clone(),
            framebuffer_exporter,
            [DrmFourcc::Argb8888],
            render_formats.iter().cloned(),
            drm_device.cursor_size(),
            Some(gbm_device.clone()),
        )?;

        if let Some(hdmi_conn) = hdmi_connector {
            if let Err(e) = compositor.add_connector(hdmi_conn) {
                log::info!("HDMI connector add result: {:?}", e);
            } else {
                log::info!("HDMI connector added to compositor");
            }
        }

        let backend = DrmBackend {
            drm_device,
            drm_fd: drm_fd.clone(),
            gbm_device,
            allocator,
            egl_display,
            renderer,
            compositor,
            output,
            render_formats,
            lcd_connector: connector,
            hdmi_connector,
        };

        self.backends.insert(device_id, backend);
        self.pending_render = true;

        log::info!("DRM backend initialized on {:?}", path);
        Ok(drm_notifier)
    }

    pub fn schedule_repaint(&mut self, _output: &Output) {
        self.pending_render = true;
    }

    pub fn render(
        &mut self,
        windows: &[Window],
        output: &Output,
        cursor_surface: Option<&smithay::reexports::wayland_server::protocol::wl_surface::WlSurface>,
        pointer_location: Point<f64, Logical>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let backend = match self.backends.values_mut().next() {
            Some(b) => b,
            None => return Ok(()),
        };

        let mut elements: Vec<WaylandSurfaceRenderElement<GlesRenderer>> = windows
            .iter()
            .filter_map(|w| w.wl_surface())
            .flat_map(|surface| {
                render_elements_from_surface_tree(
                    &mut backend.renderer,
                    &surface,
                    (0, 0),
                    1.0,
                    1.0,
                    Kind::Unspecified,
                )
            })
            .collect();

        if let Some(cursor_surface) = cursor_surface {
            let cursor_elements = render_elements_from_surface_tree(
                &mut backend.renderer,
                cursor_surface,
                (pointer_location.x as i32, pointer_location.y as i32),
                1.0,
                1.0,
                Kind::Cursor,
            );
            elements.extend(cursor_elements);
        }

        let result: RenderFrameResult<_, _, _> = backend.compositor.render_frame(
            &mut backend.renderer,
            &elements,
            [0.1, 0.1, 0.1, 1.0],
            FrameFlags::DEFAULT,
        )?;

        if !result.is_empty {
            backend.compositor.queue_frame(())?;
        }

        let now = Duration::from_millis(0);
        for window in windows {
            window.send_frame(output, now, None, |_, _| Some(output.clone()));
        }

        self.pending_render = false;

        Ok(())
    }

    pub fn frame_submitted(&mut self) {
        if let Some(backend) = self.backends.values_mut().next() {
            if let Err(e) = backend.compositor.frame_submitted() {
                log::warn!("frame_submitted error: {:?}", e);
            }
        }
    }

    pub fn should_render(&self) -> bool {
        self.pending_render
    }
}